use bcc::analysis::{
    CFSComparator, CloneClass, FVec, Graph, SemanticComparator, StructureBlock, CFG, CFS,
};
use bcc::disasm::radare2::R2Disasm;
use clap::Parser;
use fnv::FnvHashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Research Question #4
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file(s) to be analyzed
    #[clap(required = true)]
    inputs: Vec<String>,
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
    let paired = pair_inputs(args.inputs);
    analyse(paired, args.timeout, args.limit_concurrent).await;
}

fn pair_inputs(inputs: Vec<String>) -> Vec<(String, String)> {
    // no need to make this efficient and deadline is tomorrow
    let mut retval = Vec::new();
    let mut set = HashSet::new();
    for bin_a in &inputs {
        let a_name = Path::new(&bin_a)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        for bin_b in &inputs {
            let b_name = Path::new(&bin_b)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            if a_name == b_name && bin_a != bin_b && !set.contains(&b_name) {
                set.insert(b_name);
                retval.push((bin_a.clone(), bin_b.clone()));
            }
        }
    }
    retval.sort_unstable();
    retval
}

fn structural_analysis_only(analysis_res: &AnalysisResult, threshold: u32) -> Vec<CloneClass> {
    let mut comps = CFSComparator::new(threshold);
    let iter = analysis_res
        .result
        .iter()
        .filter(|res| res.cfs.is_some())
        .map(|res| (res.cfs.as_ref().unwrap(), res.bin, res.func));
    for (cfs, bin, func) in iter {
        comps.insert(bin, func, cfs);
    }
    comps.clones(&analysis_res.string_cache)
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
    for (cfs, bin, func) in iter {
        comps.insert(bin, func, cfs);
    }
    let structural_clones = comps.clones(&analysis_res.string_cache);
    let mut retval = Vec::new();
    for clone_class in structural_clones {
        let mut comps = SemanticComparator::new(threshold_semantic);
        for (bin, func, structure) in clone_class {
            let bin_id = **reverse_map.get(bin).unwrap();
            let fun_id = **reverse_map.get(func).unwrap();
            let structure = structure.unwrap();
            let fvec = *fvec_map.get(&(bin_id, fun_id)).unwrap();
            comps.insert(bin_id, fun_id, fvec, Some(structure));
        }
        retval.extend(comps.clones(&analysis_res.string_cache));
    }
    retval
}

async fn analyse(jobs: Vec<(String, String)>, timeout: u64, limit_concurrent: usize) {
    let style = ProgressStyle::default_bar()
        .template("{msg} {pos:>7}/{len:7} [{bar:40.cyan/blue}] [{elapsed_precise}]")
        .unwrap()
        .progress_chars("#>-");
    let pb = Arc::new(
        ProgressBar::new(jobs.len() as u64)
            .with_style(style)
            .with_message("Analyzing..."),
    );
    let mut tasks = FuturesUnordered::new();
    println!("bin,total_size,disasm_time,cfs_time,fvec_time,structural_time,combined_time");
    for job in jobs {
        let fut = tokio::spawn(gather_analysis_data_job(job, Arc::clone(&pb), timeout));
        tasks.push(fut);
        if tasks.len() == limit_concurrent {
            tasks.next().await;
        }
    }
    // wait until completion or everything is killed
    while !tasks.is_empty() {
        tasks.next().await;
    }
    pb.finish();
}

struct AnalysisResult {
    // reversed cache containing all the bin/fun names
    string_cache: FnvHashMap<u32, String>,
    // result of the analysis
    result: Vec<AnalysisStepResult>,
}

struct AnalysisStepResult {
    bin: u32,
    func: u32,
    cfs: Option<StructureBlock>,
    fvec: Option<FVec>,
}

#[allow(clippy::too_many_arguments)]
async fn gather_analysis_data_job(job: (String, String), pb: Arc<ProgressBar>, timeout_secs: u64) {
    let mut opcode_cache = HashMap::new();
    let mut string_cache = FnvHashMap::default();
    let (job_path_a, job_path_b) = (Path::new(&job.0), Path::new(&job.1));
    let bin = job_path_a.to_str().unwrap().to_string(); // bin_b is the same
    let fsize_a = std::fs::metadata(job_path_a).unwrap().len();
    let fsize_b = std::fs::metadata(job_path_b).unwrap().len();
    let fsize = fsize_a + fsize_b;
    let mut start_t;
    let mut end_t;
    let mut disasm_time = 0;
    let mut cfs_time = 0;
    let mut fvec_time = 0;
    let mut result = Vec::new();
    if let (Ok(mut dis_a), Ok(mut dis_b)) = (
        R2Disasm::new(job_path_a.to_str().unwrap()).await,
        R2Disasm::new(job_path_b.to_str().unwrap()).await,
    ) {
        start_t = Instant::now();
        let analysis_a = timeout(Duration::from_secs(timeout_secs), dis_a.analyse()).await;
        end_t = Instant::now();
        disasm_time += end_t.checked_duration_since(start_t).unwrap().as_micros();
        start_t = Instant::now();
        let analysis_b = timeout(Duration::from_secs(timeout_secs), dis_b.analyse()).await;
        end_t = Instant::now();
        disasm_time += end_t.checked_duration_since(start_t).unwrap().as_micros();
        if analysis_a.is_ok() && analysis_b.is_ok() {
            for mut disassembler in [dis_a, dis_b] {
                let funcs = disassembler.get_function_offsets().await;
                for func in funcs.into_iter() {
                    if let Some(bare) = disassembler.get_function_cfg(func).await {
                        let cfg = CFG::from(bare);
                        if cfg.len() > 1 {
                            start_t = Instant::now();
                            let cfs = CFS::new(&cfg).get_tree();
                            end_t = Instant::now();
                            cfs_time += end_t.checked_duration_since(start_t).unwrap().as_micros();
                            if cfs.is_some() {
                                start_t = Instant::now();
                                let fvec = disassembler
                                    .get_function_body(func)
                                    .await
                                    .map(|stmts| FVec::new(stmts, &mut opcode_cache, true));
                                end_t = Instant::now();
                                fvec_time +=
                                    end_t.checked_duration_since(start_t).unwrap().as_micros();
                                assert!(fvec.is_some());
                                let next_id = string_cache.len() as u32;
                                let bin_id = *string_cache.entry(bin.clone()).or_insert(next_id);
                                let next_id = string_cache.len() as u32;
                                let func_id =
                                    *string_cache.entry(func.to_string()).or_insert(next_id);

                                result.push(AnalysisStepResult {
                                    bin: bin_id,
                                    func: func_id,
                                    cfs,
                                    fvec,
                                })
                            }
                        }
                    }
                }
            }
        }
    }
    let analysis_result = AnalysisResult {
        string_cache: string_cache.into_iter().map(|(k, v)| (v, k)).collect(),
        result,
    };
    start_t = Instant::now();
    let structural_clones = structural_analysis_only(&analysis_result, 3);
    end_t = Instant::now();
    let structural_time = end_t.checked_duration_since(start_t).unwrap().as_micros();
    start_t = Instant::now();
    let combined_clones = structural_semantic_combined(&analysis_result, 3, 0.99);
    end_t = Instant::now();
    let combined_time = end_t.checked_duration_since(start_t).unwrap().as_micros();
    println!(
        "{},{},{},{},{},{},{}",
        job_path_a.file_name().unwrap().to_str().unwrap(),
        fsize,
        disasm_time,
        cfs_time,
        fvec_time,
        structural_time,
        combined_time
    );
    //write the results so they are not optimized away
    std::fs::write("/dev/null", format!("{:?}", structural_clones)).unwrap();
    std::fs::write("/dev/null", format!("{:?}", combined_clones)).unwrap();
    pb.inc(1);
}
