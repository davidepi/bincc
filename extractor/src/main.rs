use bcc::analysis::{Graph, CFG, CFS};
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use log::LevelFilter;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    let matches = App::new("bcc extractor")
        .version("0.1")
        .author("Davide Pizzolotto <davide.pizzolotto@gmail.com>")
        .about("Extracts CFG from binary files")
        .arg(
            Arg::with_name("input")
                .help("Input file(s)")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("extract")
                .long("extract")
                .help("Perform the CFG extraction only")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output")
                .help("Output directory")
                .short("o")
                .long("out")
                .required(false)
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
    } else if cfg!(debug_assertions) {
        simple_logging::log_to_stderr(LevelFilter::Trace);
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
    let inputs = matches
        .values_of("input")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    if matches.is_present("extract") {
        if let Some(out) = matches.value_of("output") {
            if !Path::new(out).exists() {
                log::error!("Output folder {} does not exist!", out);
                exit(1);
            } else {
                log::debug!("Total jobs: {}", inputs.len());
                multithreaded_work(
                    inputs,
                    Some(out.to_string()),
                    extract_cfg_to_dot,
                    threads_no,
                );
            }
        } else {
            let msg = "Output dir is required with the --extract flag!";
            log::error!("{}", msg);
            exit(1);
        }
    } else {
        let extracted = inputs
            .into_iter()
            .flat_map(read_dot_files)
            .collect::<Vec<_>>();
        log::debug!("Total jobs: {}", extracted.len());
        multithreaded_work(extracted, None, calculate_comparison, threads_no);
    }
}

fn multithreaded_work(
    input_jobs: Vec<String>,
    output: Option<String>,
    function: fn(&str, Option<&str>, usize),
    threads_no: usize,
) {
    let pb = Arc::new(ProgressBar::new(input_jobs.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7}")
            .progress_chars("#>-"),
    );
    let jobs = Arc::new(Mutex::new(input_jobs));
    let mut threads = vec![];
    for t in 0..threads_no {
        let personal_jobs = jobs.clone();
        let personal_pb = pb.clone();
        let personal_outdir = output.clone();
        threads.push(thread::spawn({
            move || loop {
                if !personal_jobs.lock().unwrap().is_empty() {
                    let maybe_job = personal_jobs.lock().unwrap().pop();
                    if let Some(job) = maybe_job {
                        log::trace!("[{}] starting job on {}", t, job);
                        function(&job, personal_outdir.as_deref(), threads_no);
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

fn extract_cfg_to_dot(input: &str, output: Option<&str>, tid: usize) {
    let relative_path = Path::new(input);
    let filename = relative_path.file_name().unwrap();
    let out_dir = Path::new(output.unwrap()).join(Path::new(filename));
    let metadata = match std::fs::metadata(relative_path) {
        Ok(val) => val,
        Err(err) => {
            log::error!("[{}] {}", tid, err);
            return;
        }
    };
    if std::fs::create_dir(out_dir.clone()).is_err() {
        log::error!(
            "[{}] Could not create output directory {}",
            tid,
            output.unwrap()
        );
        return;
    }
    log::info!(
        "Created folder {}",
        out_dir.as_os_str().to_str().unwrap_or("ERR")
    );
    let mut disassembler = match R2Disasm::new(input) {
        Ok(disasm) => disasm,
        Err(err) => {
            log::error!("[{}] Disassembler error: {}", tid, err);
            return;
        }
    };
    log::trace!("[{}] starting disassembling", tid);
    let start_t = Instant::now();
    disassembler.analyse_functions();
    let end_t = Instant::now();
    log::trace!("[{}] finished disassembling", tid);
    log::info!(
        "[{}] disassembling {} ({} bytes) took {} ms",
        tid,
        input,
        metadata.len(),
        end_t.checked_duration_since(start_t).unwrap().as_millis()
    );
    let fnames = disassembler.get_function_names();
    log::debug!(
        "[{}] found {} function bodies for {}",
        tid,
        fnames.len(),
        input
    );
    for (function, offset) in fnames {
        let graph_filename = format!("{}{}", function, ".dot");
        let outfile = out_dir.clone().join(Path::new(&graph_filename));
        if let Some(barecfg) = disassembler.get_function_cfg(offset) {
            let cfg = CFG::from(barecfg);
            log::trace!("[{}] extracted CFG of {}::{}", tid, input, function);
            cfg.to_file(outfile).unwrap_or_else(|_| {
                log::error!("[{}] could not save CFG of {}::{}", tid, input, function)
            });
        }
    }
}

fn calculate_comparison(input: &str, _: Option<&str>, tid: usize) {
    if let Ok(cfg) = CFG::from_file(input) {
        let start_t = Instant::now();
        let cfs = CFS::new(&cfg);
        let end_t = Instant::now();
        if cfs.get_tree().is_some() {
            let duration = end_t.checked_duration_since(start_t).unwrap().as_millis();
            log::debug!(
                "[{}] CFS successful for {} (took {} ms)",
                tid,
                input,
                duration
            );
        } else {
            log::debug!(
                "[{}] CFS failed for {}. Reduced from {} nodes to {} nodes",
                tid,
                input,
                cfg.len(),
                cfs.get_graph().len()
            );
        }
    } else {
        log::error!("[{}] Failed to read CFG {}", tid, input);
    }
}

fn read_dot_files(path: String) -> Vec<String> {
    let path_p = Path::new(&path);
    if !path_p.is_dir() {
        log::warn!("Ignoring input path {}.", path);
        Vec::new()
    } else {
        match std::fs::read_dir(&path) {
            Ok(read) => read
                .into_iter()
                .flatten()
                .filter(|x| x.path().extension().unwrap_or_default() == "dot")
                .map(|x| (path_p.join(x.file_name()).to_str().unwrap().to_string()))
                .collect::<Vec<_>>(),
            Err(err) => {
                log::error!(
                    "Error while reading folder {}. {}. Path ignored.",
                    path,
                    err
                );
                Vec::new()
            }
        }
    }
}
