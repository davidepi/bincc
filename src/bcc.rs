use bcc::analysis::{CFSComparator, StructureBlock, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use clap::Parser;
use fnv::FnvHashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::Reverse;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
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
#[derive(Parser)]
#[clap(author, version, about, verbatim_doc_comment)]
struct Args {
    /// Files that will be compared against eachother for function clones.
    #[clap(required = true)]
    input: Vec<String>,
    /// Minimum threshold to consider a structural clone, measured in amount of nested structures.
    #[clap(short, long, default_value = "3")]
    min_depth: u32,
    /// Prints also the basic blocks offets composing each clone.
    #[clap(short, long, default_value = "false")]
    basic_blocks: bool,
    /// Outputs the result as Comma Separated Value content.
    ///
    /// The CSV will have the following structure:
    /// binary, function, clone_class_id, class_depth
    #[clap(short, long, default_value = "false")]
    csv: bool,
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
    let mut comps = CFSComparator::new(args.min_depth);
    let cfss = calc_cfs(args.input, args.limit_concurrent, args.timeout).await;
    cfss.into_iter()
        .for_each(|(bin, func, res)| comps.insert(res, bin, func));
    print_results(comps, args.sort, args.basic_blocks, args.csv);
}

fn print_results(comps: CFSComparator, sort: SortResult, bbs: bool, csv: bool) {
    let mut clones = 0;
    let mut classes = comps.clones();
    match sort {
        SortResult::None => (),
        SortResult::DepthAsc => classes.sort_unstable_by_key(|a| a.depth()),
        SortResult::DepthDesc => classes.sort_unstable_by_key(|a| Reverse(a.depth())),
        SortResult::SizeAsc => classes.sort_unstable_by_key(|a| a.len()),
        SortResult::SizeDesc => classes.sort_unstable_by_key(|a| Reverse(a.len())),
    }
    if csv {
        print!("binary,function,clone_class_id,class_depth");
        if bbs {
            println!(",basic_blocks");
        } else {
            println!();
        }
        for (class_id, class) in classes.iter().enumerate() {
            for (bin, func, cfs) in class.iter() {
                print!("{},{},{},{}", bin, func, class_id, class.depth());
                if bbs {
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
            }
        }
    } else {
        for class in &classes {
            println!("----- CLONE CLASS ({}) -----", class.depth());
            for (bin, func, cfs) in class.iter() {
                clones += 1;
                if !bbs {
                    println!("{} :: {}", bin, func);
                } else {
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
        println!("Classes: {} Clones: {}", classes.len(), clones);
    }
}

async fn calc_cfs(
    jobs: Vec<String>,
    max_jobs: usize,
    timeout_secs: u64,
) -> Vec<(String, String, StructureBlock)> {
    let style = ProgressStyle::default_bar()
        .template("{msg} {pos:>7}/{len:7} [{bar:40.cyan/blue}] [{elapsed_precise}]")
        .unwrap()
        .progress_chars("#>-");
    let pb = Arc::new(
        ProgressBar::new(jobs.len() as u64)
            .with_style(style)
            .with_message("Disassembling..."),
    );
    let mut tasks = FuturesUnordered::new();
    let mut cfss = Vec::new();
    for job in jobs {
        let fut = tokio::spawn(cfs_job(job, Arc::clone(&pb), timeout_secs));
        tasks.push(fut);
        if tasks.len() == max_jobs {
            if let Some(Ok(result)) = tasks.next().await {
                cfss.extend(result);
            }
        }
    }
    // wait until completion or everything is killed
    while !tasks.is_empty() {
        if let Some(Ok(result)) = tasks.next().await {
            cfss.extend(result);
        }
    }
    pb.finish();
    cfss
}

async fn cfs_job(
    job: String,
    pb: Arc<ProgressBar>,
    timeout_secs: u64,
) -> Vec<(String, String, StructureBlock)> {
    let job_path = Path::new(&job);
    let bin = job_path.to_str().unwrap().to_string();
    let mut cfss = Vec::new();
    if let Ok(mut disassembler) = R2Disasm::new(job_path.to_str().unwrap()).await {
        let analysis_res = timeout(Duration::from_secs(timeout_secs), disassembler.analyse()).await;
        if analysis_res.is_ok() {
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
                        let cfs = CFS::new(&cfg);
                        if let Some(tree) = cfs.get_tree() {
                            cfss.push((bin.clone(), func_name.clone(), tree));
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
    cfss
}
