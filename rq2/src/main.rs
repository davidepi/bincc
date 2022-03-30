use bcc::analysis::{CFSComparator, StructureBlock, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::Parser;
use fnv::FnvHashMap;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::Path;
use std::sync::Arc;

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
}

fn main() {
    let args = Args::parse();
    let mut comps = CFSComparator::new(args.min_depth);
    let cfss = calc_cfs(args.input, "Disassembling...");
    let clones = cfss
        .into_iter()
        .flat_map(|(bin, func, res)| comps.compare_and_insert(res, bin, func))
        .collect::<Vec<_>>();
    println!("bin_a,func_a,offset_a,bin_b,func_b,offset_b");
    for clone in clones {
        print!("{},", clone.first_bin());
        print!("{},", clone.first_fun());
        print!("{:#x},", clone.first_tree().offset());
        print!("{},", clone.second_bin());
        print!("{},", clone.second_fun());
        print!("{:#x} ", clone.second_tree().offset());
        println!();
    }
}

fn calc_cfs(jobs: Vec<String>, msg: &'static str) -> Vec<(String, String, StructureBlock)> {
    let style = ProgressStyle::default_bar()
        .template("{msg} {pos:>7}/{len:7} [{bar:40.cyan/blue}] [{elapsed_precise}]")
        .progress_chars("#>-");
    let pb = Arc::new(
        ProgressBar::new(jobs.len() as u64)
            .with_style(style)
            .with_message(msg),
    );
    let cfss = jobs
        .par_iter()
        .flat_map(|job| {
            let personal_pb = pb.clone();
            let job_path = Path::new(job);
            let bin = job_path.to_str().unwrap().to_string();
            let mut cfss = Vec::new();
            if let Ok(mut disassembler) = R2Disasm::new(job_path.to_str().unwrap()) {
                disassembler.analyse_functions();
                let funcs = disassembler.get_function_offsets();
                let names = disassembler
                    .get_function_names()
                    .into_iter()
                    .map(|(k, v)| (v, k))
                    .collect::<FnvHashMap<_, _>>();
                for func in funcs {
                    if let Some(bare) = disassembler.get_function_cfg(func) {
                        if let Some(func_name) = names.get(&func) {
                            let cfg = CFG::from(bare);
                            let cfs = CFS::new(&cfg);
                            if let Some(tree) = cfs.get_tree() {
                                cfss.push((bin.clone(), func_name.clone(), tree));
                            }
                        }
                    }
                }
                personal_pb.inc(1);
            } else {
                eprintln!("Disassembler error for {}", bin);
            }
            cfss
        })
        .collect::<Vec<_>>();
    pb.finish_and_clear();
    cfss
}
