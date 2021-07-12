use bcc::analysis::{Graph, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    let matches = App::new("bcc extractor (Research Question #1)")
        .version("0.1")
        .author("Davide Pizzolotto <davide.pizzolotto@gmail.com>")
        .arg(
            Arg::with_name("input")
                .help("Input file(s)")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help("Output directory")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let threads_no = num_cpus::get();
    let mut inputs = matches
        .values_of("input")
        .unwrap()
        .map(|path| path.to_string())
        .collect::<Vec<_>>();
    inputs.shuffle(&mut thread_rng());
    let output = matches.value_of("output").unwrap().to_string();
    calc_cfs(inputs, output, threads_no);
}

fn calc_cfs(input_jobs: Vec<String>, output_dir: String, threads_no: usize) {
    let pb = Arc::new(ProgressBar::new(input_jobs.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7}")
            .progress_chars("#>-"),
    );
    let jobs = Arc::new(Mutex::new(input_jobs));
    let mut out_times_name = PathBuf::from(output_dir.clone());
    out_times_name.push(Path::new("times.csv"));
    let mut out_funcs_name = PathBuf::from(output_dir);
    out_funcs_name.push(Path::new("funcs.csv"));
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
    let mut threads = vec![];
    for _ in 0..threads_no {
        let personal_jobs = jobs.clone();
        let personal_pb = pb.clone();
        let personal_out_times = out_times.clone();
        let personal_out_funcs = out_funcs.clone();
        const BUFFER_SIZE: usize = 10000;
        let mut func_buffer = Vec::with_capacity(BUFFER_SIZE);
        threads.push(thread::spawn({
            move || loop {
                if !personal_jobs.lock().unwrap().is_empty() {
                    let maybe_job = personal_jobs.lock().unwrap().pop();
                    if let Some(job) = maybe_job {
                        let job_path = Path::new(&job);
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
                        let mut disassembler = match R2Disasm::new(job_path.to_str().unwrap()) {
                            Ok(disasm) => disasm,
                            Err(_) => {
                                eprintln!("Disassembler error!");
                                continue;
                            }
                        };
                        let start_t = Instant::now();
                        disassembler.analyse_functions();
                        let end_t = Instant::now();
                        let disasm_time =
                            end_t.checked_duration_since(start_t).unwrap().as_millis();
                        let mut cfs_time_micros = 0_u128;
                        let funcs = disassembler.get_function_offsets();
                        for func in funcs {
                            if let Some(bare) = disassembler.get_function_cfg(func) {
                                let start_t = Instant::now();
                                let cfg = CFG::from(bare);
                                let cfs = CFS::new(&cfg);
                                let end_t = Instant::now();
                                cfs_time_micros +=
                                    end_t.checked_duration_since(start_t).unwrap().as_micros();
                                let cfs_len = if cfs.get_tree().is_some() {
                                    1
                                } else {
                                    cfs.get_graph().len()
                                };
                                let func_str = format!(
                                    "{},{},{},{},{}\n",
                                    config,
                                    bin,
                                    func,
                                    cfg.len(),
                                    cfs_len
                                );
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
                    }
                } else {
                    break;
                }
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    pb.finish();
}

// fn extract_cfg_to_dot(input: &str, output: Option<&str>, tid: usize) {
//     let relative_path = Path::new(input);
//     let filename = relative_path.file_name().unwrap();
//     let out_dir = Path::new(output.unwrap()).join(Path::new(filename));
//     let metadata = match std::fs::metadata(relative_path) {
//         Ok(val) => val,
//         Err(err) => {
//             log::error!("[{}] {}", tid, err);
//             return;
//         }
//     };
//     if std::fs::create_dir(out_dir.clone()).is_err() {
//         log::error!(
//             "[{}] Could not create output directory {}",
//             tid,
//             output.unwrap()
//         );
//         return;
//     }
//     log::info!(
//         "Created folder {}",
//         out_dir.as_os_str().to_str().unwrap_or("ERR")
//     );
//     let mut disassembler = match R2Disasm::new(input) {
//         Ok(disasm) => disasm,
//         Err(err) => {
//             log::error!("[{}] Disassembler error: {}", tid, err);
//             return;
//         }
//     };
//     log::trace!("[{}] starting disassembling", tid);
//     let start_t = Instant::now();
//     disassembler.analyse_functions();
//     let end_t = Instant::now();
//     log::trace!("[{}] finished disassembling", tid);
//     log::info!(
//         "[{}] disassembling {} ({} bytes) took {} ms",
//         tid,
//         input,
//         metadata.len(),
//         end_t.checked_duration_since(start_t).unwrap().as_millis()
//     );
//     let fnames = disassembler.get_function_names();
//     let functions_no = fnames.len();
//     log::debug!(
//         "[{}] found {} function bodies for {}",
//         tid,
//         functions_no,
//         input
//     );
//     let start_t = Instant::now();
//     for (function, offset) in fnames {
//         let graph_filename = format!("{}{}", function, ".dot");
//         let outfile = out_dir.clone().join(Path::new(&graph_filename));
//         if let Some(barecfg) = disassembler.get_function_cfg(offset) {
//             let cfg = CFG::from(barecfg);
//             log::trace!("[{}] extracted CFG of {}::{}", tid, input, function);
//             cfg.to_file(outfile).unwrap_or_else(|_| {
//                 log::error!("[{}] could not save CFG of {}::{}", tid, input, function)
//             });
//         }
//     }
//     let end_t = Instant::now();
//     log::info!(
//         "[{}] Building the CFG for {} functions took {} ms",
//         tid,
//         functions_no,
//         end_t.checked_duration_since(start_t).unwrap().as_millis()
//     );
// }
//
// fn calculate_comparison(input: &str, _: Option<&str>, tid: usize) {
//     if let Ok(cfg) = CFG::from_file(input) {
//         let start_t = Instant::now();
//         let cfs = CFS::new(&cfg);
//         let end_t = Instant::now();
//         if cfs.get_tree().is_some() {
//             let duration = end_t.checked_duration_since(start_t).unwrap().as_millis();
//             log::debug!(
//                 "[{}] CFS successful for {} (took {} ms)",
//                 tid,
//                 input,
//                 duration
//             );
//         } else {
//             log::debug!(
//                 "[{}] CFS failed for {}. Reduced from {} nodes to {} nodes",
//                 tid,
//                 input,
//                 cfg.len(),
//                 cfs.get_graph().len()
//             );
//         }
//     } else {
//         log::error!("[{}] Failed to read CFG {}", tid, input);
//     }
// }
//
// fn read_dot_files(path: String) -> Vec<String> {
//     let path_p = Path::new(&path);
//     if !path_p.is_dir() {
//         log::warn!("Ignoring input path {}.", path);
//         Vec::new()
//     } else {
//         match std::fs::read_dir(&path) {
//             Ok(read) => read
//                 .into_iter()
//                 .flatten()
//                 .filter(|x| x.path().extension().unwrap_or_default() == "dot")
//                 .map(|x| (path_p.join(x.file_name()).to_str().unwrap().to_string()))
//                 .collect::<Vec<_>>(),
//             Err(err) => {
//                 log::error!(
//                     "Error while reading folder {}. {}. Path ignored.",
//                     path,
//                     err
//                 );
//                 Vec::new()
//             }
//         }
//     }
// }
