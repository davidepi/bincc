use bcc::analysis::CFG;
use bcc::disasm::radare2::R2Disasm;
use bcc::disasm::Disassembler;
use clap::{App, Arg};
use std::error::Error;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

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
        .get_matches();

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
    let jobs = Arc::new(Mutex::new(inputs));
    let mut threads = vec![];
    for _ in 1..num_cpus::get() {
        threads.push(thread::spawn({
            let personal_jobs = jobs.clone();
            let out = output.clone().to_str().unwrap();
            move || loop {
                if !personal_jobs.lock().unwrap().is_empty() {
                    let maybe_job = personal_jobs.lock().unwrap().pop();
                    if let Some(job) = maybe_job {
                        match get_and_save_cfg(&job, out) {
                            Ok(_) => {}
                            Err(err) => log::warn!("Could not process file {}: {}", job, err),
                        }
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
}

fn get_and_save_cfg(relative: &str, output_dir: &str) -> Result<Vec<CFG>, Box<dyn Error>> {
    let relative_path = Path::new(relative);
    let filename = relative_path.file_name().unwrap();
    let out_dir = Path::new(output_dir).join(Path::new(filename));
    std::fs::create_dir(out_dir)?;
    let disassembler = R2Disasm::new(relative)?;
    let mut ret = Vec::new();
    if let Some(arch) = disassembler.get_arch() {
        let fnames = disassembler.get_function_names();
        let bodies = disassembler.get_function_bodies();
        for (function, offset) in fnames {
            let graph_filename = format!("{}{}", function, ".dot");
            let outfile = out_dir.join(Path::new(&graph_filename));
            let cfg = CFG::new(&bodies.get(&offset).unwrap()[..], &arch);
            cfg.to_file(outfile);
            ret.push(cfg)
        }
        Ok(ret)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown architecture".to_string(),
        )))
    }
}
