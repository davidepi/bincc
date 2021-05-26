use bcc::analysis::CFG;
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use log::LevelFilter;
use std::error::Error;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    let matches = App::new("bcc")
        .version("0.1")
        .author("Davide Pizzolotto <davide.pizzolotto@gmail.com>")
        .about("Structural comparison of source and binary files")
        .arg(
            Arg::with_name("extract")
                .long("extract")
                .help("Perform the CFG extraction only")
                .takes_value(false),
        )
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
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .long("jobs")
                .help("Number of working jobs")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log")
                .short("l")
                .long("log")
                .help("Log file")
                .required(false)
                .takes_value(true)
                .value_name("FILE"),
        )
        .get_matches();
    if let Some(logfile) = matches.value_of("log") {
        simple_logging::log_to_file(logfile, LevelFilter::Debug).expect("Could not setup log");
    }
    let threads_no = if cfg!(debug_assertions) {
        log::warn!("Debug build, forcing 1 working thread");

        1
    } else if let Some(jobs) = matches.value_of("jobs") {
        match jobs.parse::<usize>() {
            Ok(val) => val,
            Err(_) => {
                let default_threads = num_cpus::get();
                log::warn!(
                    "Failed to parse jobs number, defaulting to {}",
                    default_threads
                );
                default_threads
            }
        }
    } else {
        let default_threads = num_cpus::get();
        log::info!("Using {} threads", default_threads);
        default_threads
    };
    let output = Path::new(matches.value_of("output").unwrap());
    if !output.exists() {
        log::error!("Output folder {} does not exist!", output.to_str().unwrap());
        exit(1);
    }
    let inputs = matches
        .values_of("input")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    log::debug!("Total jobs: {}", inputs.len());
    let pb = Arc::new(ProgressBar::new(inputs.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7}")
            .progress_chars("##-"),
    );
    let jobs = Arc::new(Mutex::new(inputs));
    let mut threads = vec![];
    for t in 0..threads_no {
        let personal_jobs = jobs.clone();
        let out = output.to_str().unwrap().to_string();
        let personal_pb = pb.clone();
        threads.push(thread::spawn({
            move || loop {
                if !personal_jobs.lock().unwrap().is_empty() {
                    let maybe_job = personal_jobs.lock().unwrap().pop();
                    if let Some(job) = maybe_job {
                        log::trace!("[{}] starting job on {}", t, job);
                        match get_and_save_cfg(&job, &out, t) {
                            Ok(_) => {
                                log::trace!("[{}] finished job on {}", t, job);
                            }
                            Err(err) => {
                                log::warn!("[{}] could not process file {}: {}", t, job, err)
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

fn get_and_save_cfg(
    relative: &str,
    output_dir: &str,
    tid: usize,
) -> Result<Vec<CFG>, Box<dyn Error>> {
    let relative_path = Path::new(relative);
    let filename = relative_path.file_name().unwrap();
    let out_dir = Path::new(output_dir).join(Path::new(filename));
    let metadata = std::fs::metadata(relative_path)?;
    std::fs::create_dir(out_dir.clone())?;
    log::info!(
        "Created folder {}",
        out_dir.as_os_str().to_str().unwrap_or("ERR")
    );
    let mut disassembler = R2Disasm::new(relative)?;
    log::trace!("[{}] starting disassembling", tid);
    let start_t = Instant::now();
    disassembler.analyse();
    let end_t = Instant::now();
    log::trace!("[{}] finished disassembling", tid);
    log::info!(
        "[{}] disassembling {} ({} bytes) took {} ms",
        tid,
        relative,
        metadata.len(),
        end_t.checked_duration_since(start_t).unwrap().as_millis()
    );
    let mut ret = Vec::new();
    if let Some(arch) = disassembler.get_arch() {
        let fnames = disassembler.get_function_names();
        let bodies = disassembler.get_function_bodies();
        log::debug!(
            "[{}] found {} function bodies for {}",
            tid,
            bodies.len(),
            relative
        );
        for (function, offset) in fnames {
            let graph_filename = format!("{}{}", function, ".dot");
            let outfile = out_dir.clone().join(Path::new(&graph_filename));
            if let Some(body) = bodies.get(&offset) {
                let cfg = CFG::new(&body[..], &*arch);
                log::trace!("[{}] extracted CFG of {}::{}", tid, relative, function);
                cfg.to_file(outfile).unwrap_or_else(|_| {
                    log::warn!("[{}] could not save CFG of {}::{}", tid, relative, function)
                });
                ret.push(cfg)
            }
        }
        Ok(ret)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown architecture".to_string(),
        )))
    }
}
