use bcc::analysis::{Graph, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Research Question #1
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file(s) to be analyzed
    #[clap(required = true)]
    input: Vec<String>,
    /// Output directory
    output: String,
    /// Output prefix
    #[clap(long)]
    prefix: Option<String>,
}

fn main() {
    let args = Args::parse();
    let mut inputs = args.input;
    inputs.shuffle(&mut thread_rng());
    let prefix = args.prefix.unwrap_or_else(|| "".to_string());
    calc_cfs(inputs, args.output, &prefix);
}

fn calc_cfs(input_jobs: Vec<String>, output_dir: String, output_prefix: &str) {
    let pb = Arc::new(ProgressBar::new(input_jobs.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7}")
            .progress_chars("#>-"),
    );
    let jobs = input_jobs;
    let mut out_times_name = PathBuf::from(output_dir.clone());
    out_times_name.push(Path::new(&format!("{}times.csv", output_prefix)));
    let mut out_funcs_name = PathBuf::from(output_dir);
    out_funcs_name.push(Path::new(&format!("{}funcs.csv", output_prefix)));
    let out_times = Arc::new(Mutex::new(File::create(out_times_name).unwrap()));
    let out_funcs = Arc::new(Mutex::new(File::create(out_funcs_name).unwrap()));
    out_times
        .lock()
        .unwrap()
        .write(b"config,exec,size,disasm,cfs\n")
        .ok();
    out_funcs
        .lock()
        .unwrap()
        .write(b"config,exec,func,original,reduced\n")
        .ok();
    jobs.par_iter().for_each(|job| {
        let personal_pb = pb.clone();
        let personal_out_times = out_times.clone();
        let personal_out_funcs = out_funcs.clone();
        const BUFFER_SIZE: usize = 10000;
        let mut func_buffer = Vec::with_capacity(BUFFER_SIZE);
        let job_path = Path::new(job);
        let bin = job_path.file_name().unwrap().to_str().unwrap();
        let config = job_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let fsize = match std::fs::metadata(job_path) {
            Ok(val) => format!("{}", val.len()),
            Err(_) => "".to_string(),
        };
        if let Ok(mut disassembler) = R2Disasm::new(job_path.to_str().unwrap()) {
            let start_t = Instant::now();
            disassembler.analyse_functions();
            let end_t = Instant::now();
            let disasm_time = end_t.checked_duration_since(start_t).unwrap().as_millis();
            let mut cfs_time_micros = 0_u128;
            let funcs = disassembler.get_function_offsets();
            for func in funcs {
                if let Some(bare) = disassembler.get_function_cfg(func) {
                    let start_t = Instant::now();
                    let cfg = CFG::from(bare);
                    let cfs = CFS::new(&cfg);
                    let end_t = Instant::now();
                    cfs_time_micros += end_t.checked_duration_since(start_t).unwrap().as_micros();
                    let cfs_len = if cfs.get_tree().is_some() {
                        1
                    } else {
                        cfs.get_graph().len()
                    };
                    let func_str =
                        format!("{},{},{},{},{}\n", config, bin, func, cfg.len(), cfs_len);
                    func_buffer.push(func_str);
                    if func_buffer.len() >= BUFFER_SIZE {
                        let mut locked_file = personal_out_funcs.lock().unwrap();
                        while let Some(str) = func_buffer.pop() {
                            locked_file.write(str.as_bytes()).ok();
                        }
                    }
                }
            }
            let time_string = format!(
                "{},{},{},{},{}\n",
                config,
                bin,
                fsize,
                disasm_time,
                cfs_time_micros / 1000
            );
            personal_out_times
                .lock()
                .unwrap()
                .write(time_string.as_bytes())
                .ok();
            if !func_buffer.is_empty() {
                let mut locked_file = personal_out_funcs.lock().unwrap();
                while let Some(str) = func_buffer.pop() {
                    locked_file.write(str.as_bytes()).ok();
                }
            }
            personal_pb.inc(1);
        } else {
            eprintln!("Disassembler error!");
        }
    });
    pb.finish();
}
