use bcc::analysis::{CFSComparator, StructureBlock, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use clap::Parser;
use fnv::FnvHashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Research Question #2
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Files that will be compared against eachother for function clones.
    #[clap(required = true)]
    input: Vec<String>,
    /// Minimum threshold to consider a structural clone, measured in amount of nested structures.
    #[clap(short = 's', long, default_value = "5")]
    min_depth: u32,
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
    let mut clones = 0;
    let classes = comps.clones();
    for class in &classes {
        println!("----- CLONE CLASS ({}) -----", class.depth());
        for (bin, func, _) in class.iter() {
            clones += 1;
            println!("{} :: {}", bin, func);
        }
    }
    println!("----------------------------");
    println!("Classes: {} Clones: {}", classes.len(), clones);
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
