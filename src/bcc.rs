use bcc::analysis::{
    CFSComparator, CloneClass, FVec, Graph, SemanticComparator, StructureBlock, CFG, CFS,
};
use bcc::disasm::radare2::R2Disasm;
use clap::Parser;
use fnv::FnvHashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(clap::ValueEnum, Copy, Clone)]
enum SortResult {
    /// Do not sort the results.
    None,
    /// Sorts by clone class structural depth, ascending.
    DepthAsc,
    /// Sorts by clone class structural depth, descending.
    DepthDesc,
    /// Sorts by amount of clones inside a clone class, ascending.
    SizeAsc,
    /// Sorts by amount of clones inside a clone class, descending.
    SizeDesc,
}

/// Detects code clones in the given binary files.
///
/// The report will be printed to stdout and will contains all the clones divided in clone classes.
/// Each clone class has the following format:
///
/// CLONE CLASS (depth)
/// <clone_binary> :: <clone_function> [basic blocks...]
///
/// For example CLONE CLASS (3) means that the clone class contains clones of at least 3 nested
/// structures.
#[derive(Parser, Clone)]
#[clap(author, version, about, verbatim_doc_comment)]
struct Args {
    /// Files that will be compared against eachother for function clones.
    #[clap(required = true)]
    input: Vec<String>,
    /// Minimum threshold to consider a structural clone, measured in amount of nested structures.
    #[clap(short, long, default_value = "3")]
    min_depth: u32,
    /// Minimum threshold to consider a semantic clone, measure in cosine similarity.
    #[clap(long, default_value = "0.85")]
    min_similarity: f32,
    /// Prints also the basic blocks offets composing each clone.
    #[clap(short, long)]
    basic_blocks: bool,
    /// Outputs the result as Comma Separated Value content.
    ///
    /// The CSV will have the following structure:
    /// binary, function, clone_class_id, class_depth
    #[clap(short, long)]
    csv: bool,
    /// Disable the structural comparison step.
    ///
    /// WARNING: without this step the execution time may increase dramatically.
    #[clap(long)]
    disable_structural: bool,
    /// Disable the semantic comparison step.
    #[clap(long)]
    disable_semantic: bool,
    /// Sorts the results.
    #[clap(short, long, default_value = "none")]
    sort: SortResult,
    /// Limits the maximum amount of applications analysed concurrently.
    #[clap(short='l', long="limit", default_value_t = num_cpus::get())]
    limit_concurrent: usize,
    /// Maximum time limit for a single application analysis, in seconds.
    #[clap(short, long, default_value_t = u64::MAX)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cross_arch = same_arch(&args.input).await;
    if cross_arch {
        eprintln!("Cross architecture detection");
    } else {
        eprintln!("Same architecture detection");
    }
    // storing all opcodes for every function will go out of memory really quickly.
    // I will just store the frequency and use an ID to identify them.
    let analysis_result = analyse(args.clone(), cross_arch).await;
    let clones = if args.disable_semantic {
        structural_analysis_only(&analysis_result, args.min_depth)
    } else if args.disable_structural {
        semantic_analysis_only(&analysis_result, args.min_similarity)
    } else {
        structural_semantic_combined(&analysis_result, args.min_depth, args.min_similarity)
    };
    print_results(clones, args.sort, args.basic_blocks, args.csv);
}

fn structural_analysis_only(analysis_res: &AnalysisResult, threshold: u32) -> Vec<CloneClass> {
    eprintln!(
        "Structural analysis: {} candidates",
        analysis_res
            .result
            .iter()
            .filter(|res| res.cfs.is_some())
            .count()
    );
    let mut comps = CFSComparator::new(threshold);
    let iter = analysis_res
        .result
        .iter()
        .filter(|res| res.cfs.is_some())
        .map(|res| (res.cfs.as_ref().unwrap(), res.bin, res.func));
    let start_t = Instant::now();
    for (cfs, bin, func) in iter {
        comps.insert(bin, func, cfs);
    }
    let clones = comps.clones(&analysis_res.string_cache);
    let end_t = Instant::now();
    let sa_time = end_t.checked_duration_since(start_t).unwrap().as_micros() as u64;
    eprintln!("Structural analysis took {} µs", sa_time);
    clones
}

fn semantic_analysis_only(analysis_res: &AnalysisResult, threshold: f32) -> Vec<CloneClass> {
    let mut comps = SemanticComparator::new(threshold);
    eprintln!(
        "Semantic analysis: {} candidates",
        analysis_res
            .result
            .iter()
            .filter(|res| res.fvec.is_some())
            .count()
    );
    let start_t = Instant::now();
    for res in analysis_res.result.iter().filter(|res| res.fvec.is_some()) {
        comps.insert(res.bin, res.func, res.fvec.as_ref().unwrap(), None);
    }
    let clones = comps.clones(&analysis_res.string_cache);
    let end_t = Instant::now();
    let se_time = end_t.checked_duration_since(start_t).unwrap().as_micros() as u64;
    eprintln!("Semantic analysis took {} µs", se_time);
    clones
}

fn structural_semantic_combined(
    analysis_res: &AnalysisResult,
    threshold_structural: u32,
    threshold_semantic: f32,
) -> Vec<CloneClass> {
    let reverse_map = analysis_res
        .string_cache
        .iter()
        .map(|(k, v)| (v.as_str(), k))
        .collect::<HashMap<_, _>>();
    let fvec_map = analysis_res
        .result
        .iter()
        .map(|res| ((res.bin, res.func), res.fvec.as_ref().unwrap()))
        .collect::<FnvHashMap<_, _>>();
    let mut comps = CFSComparator::new(threshold_structural);
    let iter = analysis_res
        .result
        .iter()
        .filter(|res| res.cfs.is_some())
        .map(|res| (res.cfs.as_ref().unwrap(), res.bin, res.func));
    let start_t = Instant::now();
    for (cfs, bin, func) in iter {
        comps.insert(bin, func, cfs);
    }
    let structural_clones = comps.clones(&analysis_res.string_cache);
    let end_t = Instant::now();
    let sa_time = end_t.checked_duration_since(start_t).unwrap().as_micros() as u64;
    let mut comparison_done = 0;
    let mut retval = Vec::new();
    let start_t = Instant::now();
    for clone_class in structural_clones {
        let mut comps = SemanticComparator::new(threshold_semantic);
        for (bin, func, structure) in clone_class {
            let bin_id = **reverse_map.get(bin).unwrap();
            let fun_id = **reverse_map.get(func).unwrap();
            let structure = structure.unwrap();
            let fvec = *fvec_map.get(&(bin_id, fun_id)).unwrap();
            comps.insert(bin_id, fun_id, fvec, Some(structure));
            comparison_done += 1;
        }
        retval.extend(comps.clones(&analysis_res.string_cache));
    }
    let end_t = Instant::now();
    let se_time = end_t.checked_duration_since(start_t).unwrap().as_micros() as u64;
    eprintln!(
        "Structural+Semantic analysis: {} comparisons",
        comparison_done
    );
    eprintln!("Structural analysis took {} µs", sa_time);
    eprintln!("Semantic analysis took {} µs", se_time);
    retval
}

fn print_results(mut classes: Vec<CloneClass>, sort: SortResult, bbs: bool, csv: bool) {
    match sort {
        SortResult::None => (),
        SortResult::DepthAsc => classes.sort_unstable_by_key(|a| a.depth()),
        SortResult::DepthDesc => classes.sort_unstable_by_key(|a| Reverse(a.depth())),
        SortResult::SizeAsc => classes.sort_unstable_by_key(|a| a.len()),
        SortResult::SizeDesc => classes.sort_unstable_by_key(|a| Reverse(a.len())),
    }
    if csv {
        print_csv(classes, bbs)
    } else {
        print_stdout(classes, bbs)
    }
}

fn print_stdout(classes: Vec<CloneClass>, bbs: bool) {
    let mut clones = 0;
    let classes_no = classes.len();
    for class in classes {
        println!("----- CLONE CLASS ({}) -----", class.depth());
        for (bin, func, maybe_cfs) in class {
            clones += 1;
            if !bbs {
                println!("{} :: {}", bin, func);
            } else if let Some(cfs) = maybe_cfs {
                let bbs = cfs
                    .basic_blocks()
                    .into_iter()
                    .filter(|bb| !bb.is_sink())
                    .map(|bb| format!("0x{:x}", bb.offset))
                    .collect::<Vec<_>>()
                    .join(",");
                println!("{} :: {} [{}]", bin, func, bbs);
            }
        }
    }
    println!("----------------------------");
    println!("Classes: {} Clones: {}", classes_no, clones);
}

fn print_csv(classes: Vec<CloneClass>, bbs: bool) {
    print!("arch,bits,binary,function,clone_class_id,class_depth");
    if bbs {
        println!(",basic_blocks");
    } else {
        println!();
    }
    for (class_id, class) in classes.into_iter().enumerate() {
        let class_depth = class.depth();
        for (bin, func, maybe_cfs) in class {
            // arch substring is appended by this program, it's always ASCII so this call is safe
            let archbits_substring_end = bin.find(']').unwrap();
            let archbits_substring = &bin[1..archbits_substring_end];
            let arch_substring_end = archbits_substring.find('_').unwrap();
            let arch_substring = &bin[1..arch_substring_end + 1];
            let bits_substring = &bin[arch_substring_end + 2..archbits_substring_end];
            let bin_substring = &bin[archbits_substring_end + 1..];
            print!(
                "{},{},{},{},{},{}",
                arch_substring, bits_substring, bin_substring, func, class_id, class_depth
            );
            if bbs {
                if let Some(cfs) = maybe_cfs {
                    let bbs = cfs
                        .basic_blocks()
                        .into_iter()
                        .filter(|bb| !bb.is_sink())
                        .map(|bb| format!("0x{:x}", bb.offset))
                        .collect::<Vec<_>>()
                        .join(",");
                    println!(",\"{}\"", bbs);
                } else {
                    println!();
                }
            } else {
                println!();
            }
        }
    }
}

struct AnalysisResult {
    // reversed cache containing all the bin/fun names
    string_cache: FnvHashMap<u32, String>,
    // result of the analysis
    result: Vec<AnalysisStepResult>,
}

async fn analyse(args: Args, cross_arch: bool) -> AnalysisResult {
    let style = ProgressStyle::default_bar()
        .template("{msg} {pos:>7}/{len:7} [{bar:40.cyan/blue}] [{elapsed_precise}]")
        .unwrap()
        .progress_chars("#>-");
    let pb = Arc::new(
        ProgressBar::new(args.input.len() as u64)
            .with_style(style)
            .with_message("Disassembling..."),
    );
    let mut tasks = FuturesUnordered::new();
    let string_cache = Arc::new(Mutex::new(HashMap::new()));
    let opcode_cache = Arc::new(Mutex::new(HashMap::new()));
    let mut analysis_all_res = Vec::new();
    for job in args.input {
        let fut = tokio::spawn(gather_analysis_data_job(
            job,
            Arc::clone(&pb),
            Arc::clone(&string_cache),
            Arc::clone(&opcode_cache),
            args.disable_structural,
            args.disable_semantic,
            args.timeout,
            cross_arch,
        ));
        tasks.push(fut);
        if tasks.len() == args.limit_concurrent {
            if let Some(Ok(result)) = tasks.next().await {
                analysis_all_res.extend(result);
            }
        }
    }
    // wait until completion or everything is killed
    while !tasks.is_empty() {
        if let Some(Ok(result)) = tasks.next().await {
            analysis_all_res.extend(result);
        }
    }
    pb.finish();
    let string_cache = Arc::try_unwrap(string_cache)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (v, k))
        .collect::<FnvHashMap<_, _>>();
    AnalysisResult {
        string_cache,
        result: analysis_all_res,
    }
}

async fn same_arch(jobs: &[String]) -> bool {
    let mut archs = Vec::with_capacity(jobs.len());
    for job in jobs {
        let job_path = Path::new(&job);
        if let Ok(mut disassembler) = R2Disasm::new(job_path.to_str().unwrap()).await {
            if let Some(arch) = disassembler.get_arch().await {
                archs.push(arch);
            } else {
                eprintln!(
                    "Failed to recognize architecture of file {}. Exiting.",
                    job_path.display()
                );
                std::process::exit(1);
            }
        } else {
            eprintln!("Disassembler error for {}", job_path.display());
            std::process::exit(1)
        }
    }
    archs.dedup();
    archs.len() > 1
}

struct AnalysisStepResult {
    bin: u32,
    func: u32,
    cfs: Option<StructureBlock>,
    fvec: Option<FVec>,
}

#[allow(clippy::too_many_arguments)]
async fn gather_analysis_data_job(
    job: String,
    pb: Arc<ProgressBar>,
    string_cache: Arc<Mutex<HashMap<String, u32>>>,
    opcode_cache: Arc<Mutex<HashMap<String, u16>>>,
    disable_structural: bool,
    disable_semantic: bool,
    timeout_secs: u64,
    cross_arch: bool,
) -> Vec<AnalysisStepResult> {
    let job_path = Path::new(&job);
    let bin = job_path.to_str().unwrap().to_string();
    let mut result = Vec::new();
    if let Ok(mut disassembler) = R2Disasm::new(job_path.to_str().unwrap()).await {
        let analysis_res = timeout(Duration::from_secs(timeout_secs), disassembler.analyse()).await;
        if analysis_res.is_ok() {
            let arch = disassembler
                .get_arch()
                .await
                .expect("Unsupported architecture");
            let bin_with_arch = format!("[{}_{}]{}", arch.name(), arch.bits(), bin);
            let funcs = disassembler.get_function_offsets().await;
            let names = disassembler
                .get_function_names()
                .await
                .into_iter()
                .map(|(k, v)| (v, k))
                .collect::<FnvHashMap<_, _>>();
            for func in funcs {
                if let Some(bare) = disassembler.get_function_cfg(func).await {
                    if let Some(func_name) = names.get(&func) {
                        let cfg = CFG::from(bare);
                        if cfg.len() > 1 {
                            let cfs = if !disable_structural {
                                CFS::new(&cfg).get_tree()
                            } else {
                                None
                            };
                            let fvec = if !disable_semantic {
                                disassembler.get_function_body(func).await.map(|stmts| {
                                    FVec::new(stmts, &mut opcode_cache.lock().unwrap(), cross_arch)
                                })
                            } else {
                                None
                            };
                            if let Ok(mut cache) = string_cache.lock() {
                                let next_id = cache.len() as u32;
                                let bin_id = *cache.entry(bin_with_arch.clone()).or_insert(next_id);
                                let next_id = cache.len() as u32;
                                let func_id =
                                    *cache.entry(func_name.to_string()).or_insert(next_id);
                                result.push(AnalysisStepResult {
                                    bin: bin_id,
                                    func: func_id,
                                    cfs,
                                    fvec,
                                })
                            } else {
                                panic!("Mutex poisoned")
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("Killed {} (timeout)", job);
        }
    } else {
        eprintln!("Disassembler error for {}", bin);
    }
    pb.inc(1);
    result
}
