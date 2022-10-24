use crate::disasm::architectures::Architecture;
use crate::disasm::{Statement, StatementFamily};
use fnv::{FnvHashMap, FnvHashSet};
use lazy_static::lazy_static;
use r2pipe::{R2PipeAsync, R2PipeSpawnOptions};
use regex::Regex;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::str::FromStr;
use std::{fs, io};

/// A very basic Control Flow Graph.
///
/// This crate provide a more advanced version in [crate::analysis::CFG].
/// This struct, however, is used to store the data retrieved from the underlying disassembler.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct BareCFG {
    /// Address of the function entry point.
    pub root: Option<u64>,
    /// Vector of basic blocks. Each tuple contains a basic block in the form:
    /// - offset of the first instruction.
    /// - length of the basic block.
    pub blocks: Vec<(u64, u64)>,
    /// Vector of CFG edges. Each tuple contains an edge in the form:
    /// - offset of the source basic block.
    /// - offset of the destination basic block.
    ///
    /// The edge corresponding to the "true" condition of a conditional jump in the CFG
    /// should come before the edge corresponding to the "false" condition.
    pub edges: Vec<(u64, u64)>,
}

/// Disassembler using the radare2 backend.
///
/// Using this struct requires having installed radare2, with the `r2` binary on the path.
pub struct R2Disasm {
    // pipe to the external r2 command.
    // no need for a mutex as it is not possible to invoke commands to the same external process
    // at the same time (this struct does not implement copy or clone)
    pipe: R2PipeAsync,
}

impl R2Disasm {
    /// Creates a new radare2 disassembling interface.
    ///
    /// This method assumes a `radare2` or `r2` process in the `PATH`.
    /// In case of errors [io::Error] is returned with the following ErrorKind:
    /// - [io::ErrorKind::BrokenPipe] : if the radare2 process can not be found
    /// - [io::ErrorKind::NotFound] : if the binary file can not be found or read
    pub async fn new(binary: &str) -> Result<Self, io::Error> {
        //R2Pipe.rs error handling is garbage and will panic if file does not exist
        if fs::metadata(binary).is_ok() {
            let flags = R2PipeSpawnOptions {
                exepath: "r2".to_string(),
                args: vec!["-2"],
            };
            let maybe_pipe = R2PipeAsync::spawn(binary, Some(flags)).await;
            match maybe_pipe {
                Ok(pipe) => Ok(Self { pipe }),
                Err(err) => Err(io::Error::new(ErrorKind::BrokenPipe, err)),
            }
        } else {
            Err(io::Error::new(ErrorKind::NotFound, "Could not open file"))
        }
    }

    /// Performs analysis on the underlying binary.
    pub async fn analyse(&mut self) {
        match self.pipe.cmd("aaa").await {
            Ok(_) => {}
            Err(error) => {
                log::error!("{}", error);
            }
        }
    }

    /// Performs analysis on the function bounds only.
    ///
    /// The default implementation calls [R2Disasm::analyse] thus performing a full-binary
    /// analysis.
    pub async fn analyse_functions(&mut self) {
        match self.pipe.cmd("aa").await {
            Ok(_) => match self.pipe.cmd("aac").await {
                Ok(_) => {}
                Err(error) => {
                    log::error!("{}", error);
                }
            },
            Err(error) => {
                log::error!("{}", error);
            }
        }
    }

    /// Returns the architecture of a specific file.
    ///
    /// This operation *DOES NOT* require to run [R2Disasm::analyse] first.
    ///
    /// If the architecture can not be recognized, None is returned.
    pub async fn get_arch(&mut self) -> Option<Architecture> {
        match self.pipe.cmdj("ij").await {
            Ok(json) => {
                let bits = json["bin"]["bits"].as_u64()?;
                let arch = json["bin"]["arch"].as_str()?;
                match arch {
                    "arc" => Some(Architecture::ARC(bits as u32)),
                    "avr" => Some(Architecture::AVR),
                    "arm" => Some(Architecture::Arm(bits as u32)),
                    "i4004" => Some(Architecture::I4004),
                    "8051" => Some(Architecture::I8051(bits as u32)),
                    "i8080" => Some(Architecture::I8080),
                    "lm32" => Some(Architecture::LM32),
                    "LH5801" => Some(Architecture::Lh5801),
                    "6502" => Some(Architecture::M6502),
                    "m68k" => Some(Architecture::M68K),
                    "msp430" => Some(Architecture::MSP430),
                    "propeller" => Some(Architecture::Propeller),
                    "v850" => Some(Architecture::V850),
                    "z80" => Some(Architecture::Z80),
                    "s390" => Some(Architecture::S390(bits as u32)),
                    "ppc" => Some(Architecture::PowerPC(bits as u32)),
                    "mips" => Some(Architecture::Mips(bits as u32)),
                    "riscv" => Some(Architecture::Riscv(bits as u32)),
                    "sparc" => Some(Architecture::Sparc(bits as u32)),
                    "x86" => Some(Architecture::X86(bits as u32)),
                    _ => None,
                }
            }
            Err(error) => {
                log::error!("{}", error);
                None
            }
        }
    }

    /// Returns the starting offset of each function contained in the disassembled executable
    ///
    /// This operation requires calling [R2Disasm::analyse] first.
    pub async fn get_function_offsets(&mut self) -> FnvHashSet<u64> {
        match self.pipe.cmdj("aflqj").await {
            Ok(json) => {
                if let Some(offsets) = json.as_array() {
                    offsets
                        .iter()
                        .filter_map(|offset| offset.as_u64())
                        .collect::<FnvHashSet<_>>()
                } else {
                    FnvHashSet::default()
                }
            }
            Err(error) => {
                log::error!("{}", error);
                FnvHashSet::default()
            }
        }
    }

    /// Returns names and offsets of every function in the current executable.
    ///
    /// This operation requires calling [R2Disasm::analyse] first.
    ///
    /// The returned map contains pairs `(function name, offset in the binary)`.
    pub async fn get_function_names(&mut self) -> HashMap<String, u64> {
        let mut retval = HashMap::new();
        match self.pipe.cmdj("aflj").await {
            Ok(json) => {
                if let Some(funcs) = json.as_array() {
                    for func in funcs {
                        let maybe_offset = func["offset"].as_u64();
                        let maybe_name = func["name"].as_str();
                        if let (Some(offset), Some(name)) = (maybe_offset, maybe_name) {
                            retval.insert(name.to_string(), offset);
                        }
                    }
                }
            }
            Err(error) => {
                log::error!("{}", error)
            }
        }
        retval
    }

    /// Returns the statements composing a single basic block.
    ///
    /// This operation requires calling [R2Disasm::analyse] first.
    pub async fn get_basic_block_body(&mut self, offset: u64) -> Option<Vec<Statement>> {
        let mut retval = None;
        let cmd_change_offset = format!("s {}", offset);
        match self.pipe.cmd(&cmd_change_offset).await {
            Ok(_) => {
                if let Ok(json) = self.pipe.cmdj("pdbj").await {
                    if let Some(stmts) = json.as_array() {
                        let mut list = Vec::new();
                        for stmt in stmts {
                            let maybe_offset = stmt["offset"].as_u64();
                            let maybe_type = stmt["type"].as_str();
                            let maybe_opcode = stmt["opcode"].as_str();
                            if let (Some(offset), Some(stype), Some(opcode)) =
                                (maybe_offset, maybe_type, maybe_opcode)
                            {
                                let stype_enum = StatementFamily::try_from(stype)
                                    .unwrap_or(StatementFamily::UNK);
                                let stmt = Statement::new(offset, stype_enum, opcode);
                                list.push(stmt);
                            }
                        }
                        retval = Some(list);
                    }
                }
            }
            Err(error) => {
                log::error!("{}", error);
            }
        }
        retval
    }

    /// Return a map containing all the statements for every function in the binary.
    ///
    /// The returned map contains pairs `(function offset, vector of statements)`
    ///
    /// This operation requires calling [R2Disasm::analyse] first.
    pub async fn get_function_bodies(&mut self) -> FnvHashMap<u64, Vec<Statement>> {
        let mut retval = FnvHashMap::default();
        let maybe_json = self.pipe.cmdj("aflqj").await;
        match maybe_json {
            Ok(json) => {
                if let Some(offsets) = json.as_array() {
                    for value in offsets {
                        if let Some(offset) = value.as_u64() {
                            let body = self.get_function_body(offset).await;
                            if let Some(body) = body {
                                retval.insert(offset, body);
                            }
                        }
                    }
                }
            }
            Err(error) => {
                log::error!("{}", error)
            }
        }
        retval
    }

    /// Returns a list of statements for a given function.
    ///
    /// This method takes as input the function offset in the binary and returns a vector containing
    /// the list of statements. None if the function can not be found.
    ///
    /// This operation requires calling [R2Disasm::analyse] first.
    pub async fn get_function_body(&mut self, function: u64) -> Option<Vec<Statement>> {
        let mut retval = None;
        let cmd_change_offset = format!("s {}", function);
        match self.pipe.cmd(&cmd_change_offset).await {
            Ok(_) => {
                if let Ok(json) = self.pipe.cmdj("pdfj").await {
                    let ops = &json["ops"];
                    if let Some(stmts) = ops.as_array() {
                        let mut list = Vec::new();
                        for stmt in stmts {
                            let maybe_offset = stmt["offset"].as_u64();
                            let maybe_type = stmt["type"].as_str();
                            let maybe_opcode = stmt["opcode"].as_str();
                            if let (Some(offset), Some(stype), Some(opcode)) =
                                (maybe_offset, maybe_type, maybe_opcode)
                            {
                                let stype_enum = StatementFamily::try_from(stype)
                                    .unwrap_or(StatementFamily::UNK);
                                let stmt = Statement::new(offset, stype_enum, opcode);
                                list.push(stmt);
                            }
                        }
                        retval = Some(list);
                    }
                }
            }
            Err(error) => {
                log::error!("{}", error);
            }
        }
        retval
    }

    /// Returns a simple CFG for the given function.
    ///
    /// This method takes as input the function offset in the binary and returns its CFG generated
    /// by the underlying disassembler.
    ///
    /// If the disassembler is incapable of generating a CFG or the function address is wrong,
    /// [Option::None] is returned.
    pub async fn get_function_cfg(&mut self, function: u64) -> Option<BareCFG> {
        let mut retval = None;
        let cmd_change_offset = format!("s {}", function);
        match self.pipe.cmd(&cmd_change_offset).await {
            Ok(_) => {
                if let (Ok(bbs), Ok(dot)) =
                    (self.pipe.cmd("afb").await, self.pipe.cmd("agfdm").await)
                {
                    if !bbs.is_empty() && !dot.is_empty() {
                        let blocks = radare_dot_to_bare_cfg_nodes(&bbs);
                        let edges = radare_dot_to_bare_cfg_edges(&dot);
                        retval = Some(BareCFG {
                            root: Some(function),
                            blocks,
                            edges,
                        })
                    }
                }
            }
            Err(error) => {
                log::error!("{}", error);
            }
        }
        retval
    }
}

fn radare_dot_to_bare_cfg_edges(dot: &str) -> Vec<(u64, u64)> {
    let mut edges = Vec::new();
    lazy_static! {
        static ref RE_EDGES: Regex =
            Regex::new(r#""0[xX]([0-9a-fA-F]+)"\s*->\s*"0[xX]([0-9a-fA-F]+)".*"#).unwrap();
    }
    for line in dot.lines() {
        if let Some(cap) = RE_EDGES.captures(line) {
            let src = u64::from_str_radix(cap.get(1).unwrap().as_str(), 16);
            let dst = u64::from_str_radix(cap.get(2).unwrap().as_str(), 16);
            if let (Ok(src), Ok(dst)) = (src, dst) {
                edges.push((src, dst));
            } else {
                log::error!(
                    "While reading the radare2 CFG, failed to parse the offsets in \
                     line \"{}\". Continuing, but the CFG may be wrong",
                    line
                );
            }
        }
    }
    edges
}

fn radare_dot_to_bare_cfg_nodes(bbs: &str) -> Vec<(u64, u64)> {
    let mut blocks = Vec::new();
    lazy_static! {
        static ref RE_BLOCKS: Regex =
            Regex::new(r#"0[xX]([0-9a-fA-F]+)\s+[^\s]+\s+[^\s]+\s+(\d+).*"#).unwrap();
    }
    for line in bbs.lines() {
        if let Some(cap) = RE_BLOCKS.captures(line) {
            let off = u64::from_str_radix(cap.get(1).unwrap().as_str(), 16);
            let length = u64::from_str(cap.get(2).unwrap().as_str());
            if let (Ok(offset), Ok(length)) = (off, length) {
                blocks.push((offset, length));
            } else {
                log::error!(
                    "While reading the radare2 basic blocks, failed to parse the block offsets in \
                     line \"{}\". Continuing, but the CFG may be wrong",
                    line
                );
            }
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use crate::disasm::radare2::{BareCFG, R2Disasm};
    use crate::disasm::Architecture;
    use serial_test::serial;
    use std::io::ErrorKind;
    use std::{fs, io};

    #[tokio::test]
    #[serial]
    async fn new_radare2_process_not_existing() {
        let old_path = std::env::var("PATH").unwrap_or_else(|_| "".to_string());
        std::env::set_var("PATH", "");
        let disassembler = R2Disasm::new("/bin/ls").await;
        std::env::set_var("PATH", old_path);
        assert!(disassembler.is_err());
        assert_eq!(disassembler.err().unwrap().kind(), ErrorKind::BrokenPipe);
    }

    #[tokio::test]
    async fn new_radare2_file_not_existing() {
        let file = "/bin/0BXVnvGMp1OehPlTvbf7";
        assert!(fs::metadata(file).is_err());
        let disassembler = R2Disasm::new(file).await;
        assert!(disassembler.is_err());
        assert_eq!(disassembler.err().unwrap().kind(), ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn new_radare2() {
        let disassembler = R2Disasm::new("/bin/ls").await;
        assert!(disassembler.is_ok());
    }

    #[tokio::test]
    async fn architecture_non_binary() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let plaintext = format!("{}/{}", project_root, "resources/tests/plaintext");
        let mut disassembler = R2Disasm::new(&plaintext).await?;
        assert!(disassembler.get_arch().await.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_unsupported_arch() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let text = format!("{}/{}", project_root, "resources/tests/plaintext");
        let mut disassembler = R2Disasm::new(&text).await?;
        assert!(disassembler.get_arch().await.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_nes() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86 = format!("{}/{}", project_root, "resources/tests/nes");
        let mut disassembler = R2Disasm::new(&x86).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::M6502.name());
        assert_eq!(arch.bits(), Architecture::M6502.bits());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_riscv() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86 = format!("{}/{}", project_root, "resources/tests/riscv");
        let mut disassembler = R2Disasm::new(&x86).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::Riscv(64).name());
        assert_eq!(arch.bits(), Architecture::Riscv(64).bits());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_x86() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86 = format!("{}/{}", project_root, "resources/tests/x86");
        let mut disassembler = R2Disasm::new(&x86).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::X86(32).name());
        assert_eq!(arch.bits(), Architecture::X86(32).bits());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_x86_64() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::X86(64).name());
        assert_eq!(arch.bits(), Architecture::X86(64).bits());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_arm() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let armhf = format!("{}/{}", project_root, "resources/tests/armhf");
        let mut disassembler = R2Disasm::new(&armhf).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::Arm(16).name());
        assert_eq!(arch.bits(), Architecture::Arm(16).bits());
        Ok(())
    }

    #[tokio::test]
    async fn architecture_aarch64() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let aarch64 = format!("{}/{}", project_root, "resources/tests/aarch64");
        let mut disassembler = R2Disasm::new(&aarch64).await?;
        let arch = disassembler.get_arch().await.unwrap();
        assert_eq!(arch.name(), Architecture::Arm(64).name());
        assert_eq!(arch.bits(), Architecture::X86(64).bits());
        Ok(())
    }

    #[tokio::test]
    async fn function_offsets() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let offsets = disassembler.get_function_offsets().await;
        assert!(offsets.contains(&0x1149));
        Ok(())
    }

    #[tokio::test]
    async fn function_names() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let funcs = disassembler.get_function_names().await;
        assert_eq!(*funcs.get("main").unwrap(), 0x1149);
        Ok(())
    }

    #[tokio::test]
    async fn function_names_no_analysis() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let funcs = disassembler.get_function_names().await;
        assert!(funcs.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn function_body_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let body = disassembler.get_function_body(0x1000).await;
        assert!(body.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn function_body_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let maybe_body = disassembler.get_function_body(0x1000).await;
        assert!(maybe_body.is_some());
        let body = maybe_body.unwrap();
        assert_eq!(body.len(), 8);
        assert_eq!(body[1].get_offset(), 0x1004);
        assert_eq!(body[1].get_instruction(), "sub rsp, 8");
        assert_eq!(body[7].get_offset(), 0x101A);
        assert_eq!(body[7].get_instruction(), "ret");
        Ok(())
    }

    #[tokio::test]
    async fn function_bodies_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let bodies = disassembler.get_function_bodies().await;
        assert_eq!(bodies.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn function_bodies_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let bodies = disassembler.get_function_bodies().await;
        let last_body = disassembler.get_function_body(0x1000).await;
        assert_eq!(bodies.get(&0x1000).unwrap(), &last_body.unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn basic_block_body_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let body = disassembler.get_basic_block_body(0x1016).await;
        assert!(body.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn basic_block_body_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let maybe_body = disassembler.get_basic_block_body(0x1016).await;
        assert!(maybe_body.is_some());
        let body = maybe_body.unwrap();
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].get_mnemonic(), "add");
        assert_eq!(body[1].get_mnemonic(), "ret");
        Ok(())
    }

    #[tokio::test]
    async fn function_cfg_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        let cfg = disassembler.get_function_cfg(0x1000).await;
        assert!(cfg.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn function_cfg_exits() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64).await?;
        disassembler.analyse().await;
        let cfg = disassembler.get_function_cfg(0x1000).await;
        assert!(cfg.is_some());
        let cfg = cfg.unwrap();
        let expected = BareCFG {
            root: Some(0x1000),
            blocks: vec![(0x1000, 20), (0x1014, 2), (0x1016, 5)],
            edges: vec![(0x1000, 0x1016), (0x1000, 0x1014), (0x1014, 0x1016)],
        };
        assert_eq!(cfg, expected);
        Ok(())
    }
}
