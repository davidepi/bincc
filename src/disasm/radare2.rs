use crate::disasm::architectures::{ArchARM, ArchX86, Architecture};
use crate::disasm::{Disassembler, Statement};
use fnv::FnvHashMap;
use r2pipe::R2Pipe;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::{fs, io};

/// Disassembler using the radare2 backend.
///
/// Using this struct requires having installed radare2, with the `r2` binary on the path.
pub struct R2Disasm {
    // pipe to the external r2 command.
    // no need for a mutex as it is not possible to invoke commands to the same external process
    // at the same time (this struct does not implement copy or clone)
    pipe: RefCell<R2Pipe>,
}

impl R2Disasm {
    /// Creates a new radare2 disassembling interface.
    ///
    /// This method assumes a `radare2` or `r2` process in the `PATH`.
    /// In case of errors [io::Error] is returned with the following ErrorKind:
    /// - [io::ErrorKind::BrokenPipe] : if the radare2 process can not be found
    /// - [io::ErrorKind::NotFound] : if the binary file can not be found or read
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::radare2::R2Disasm;
    ///
    /// let disassembler = R2Disasm::new("/bin/ls");
    ///
    /// assert!(disassembler.is_ok())
    /// ```
    pub fn new(binary: &str) -> Result<R2Disasm, io::Error> {
        //R2Pipe.rs error handling is garbage and will panic if file does not exist
        if fs::metadata(binary).is_ok() {
            let maybe_pipe = R2Pipe::spawn(binary, None);
            match maybe_pipe {
                Ok(pipe) => Ok(R2Disasm {
                    pipe: RefCell::new(pipe),
                }),
                Err(err) => Err(io::Error::new(ErrorKind::BrokenPipe, err)),
            }
        } else {
            Err(io::Error::new(ErrorKind::NotFound, "Could not open file"))
        }
    }
}

impl Drop for R2Disasm {
    fn drop(&mut self) {
        self.pipe.get_mut().close()
    }
}

impl Disassembler for R2Disasm {
    fn analyse(&mut self) {
        self.pipe
            .get_mut()
            .cmd("aaa")
            .expect("Failed to analyse the binary");
    }

    fn get_arch(&self) -> Option<Box<dyn Architecture>> {
        let res = self.pipe.borrow_mut().cmdj("ij");
        match res {
            Ok(json) => match json["bin"]["arch"].as_str() {
                Some("x86") => match json["bin"]["bits"].as_u64() {
                    Some(32) => Some(Box::new(ArchX86::new_i386())),
                    Some(64) => Some(Box::new(ArchX86::new_amd64())),
                    _ => None,
                },
                Some("arm") => match json["bin"]["bits"].as_u64() {
                    Some(16) | Some(32) => Some(Box::new(ArchARM::new_arm32())),
                    Some(64) => Some(Box::new(ArchARM::new_aarch64())),
                    _ => None,
                },
                _ => None,
            },
            Err(_) => None,
        }
    }

    fn get_function_names(&self) -> HashMap<String, u64> {
        let mut retval = HashMap::new();
        if let Ok(json) = self.pipe.borrow_mut().cmdj("aflj") {
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
        retval
    }

    fn get_function_bodies(&self) -> FnvHashMap<u64, Vec<Statement>> {
        let mut retval = FnvHashMap::default();
        let maybe_json = self.pipe.borrow_mut().cmdj("aflqj");
        if let Ok(json) = maybe_json {
            if let Some(offsets) = json.as_array() {
                retval = offsets
                    .iter()
                    .filter_map(|x| x.as_u64())
                    .filter_map(|offset| match self.get_function_body(offset) {
                        None => None,
                        Some(value) => Some((offset, value)),
                    })
                    .collect();
            }
        }
        retval
    }

    fn get_function_body(&self, function: u64) -> Option<Vec<Statement>> {
        let mut retval = None;
        let cmd_change_offset = format!("s {}", function);
        self.pipe
            .borrow_mut()
            .cmd(&cmd_change_offset)
            .expect("Failed to set address");
        if let Ok(json) = self.pipe.borrow_mut().cmdj("pdrj") {
            if let Some(stmts) = json.as_array() {
                let mut list = Vec::new();
                for stmt in stmts {
                    let maybe_offset = stmt["offset"].as_u64();
                    let maybe_opcode = stmt["opcode"].as_str();
                    if let (Some(offset), Some(opcode)) = (maybe_offset, maybe_opcode) {
                        let stmt = Statement::new(offset, opcode);
                        list.push(stmt);
                    }
                }
                retval = Some(list);
            }
        }
        retval
    }
}

#[cfg(test)]
mod tests {
    use crate::disasm::architectures::{ArchARM, ArchX86};
    use crate::disasm::radare2::R2Disasm;
    use crate::disasm::{Architecture, Disassembler};
    use serial_test::serial;
    use std::io::ErrorKind;
    use std::{fs, io};

    #[test]
    #[serial]
    fn new_radare2_process_not_existing() {
        let old_path = std::env::var("PATH").unwrap_or("".to_string());
        std::env::set_var("PATH", "");
        let disassembler = R2Disasm::new("/bin/ls");
        std::env::set_var("PATH", old_path);
        assert!(disassembler.is_err());
        assert_eq!(disassembler.err().unwrap().kind(), ErrorKind::BrokenPipe);
    }

    #[test]
    fn new_radare2_file_not_existing() {
        let file = "/bin/0BXVnvGMp1OehPlTvbf7";
        assert!(fs::metadata(file).is_err());
        let disassembler = R2Disasm::new(file);
        assert!(disassembler.is_err());
        assert_eq!(disassembler.err().unwrap().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn new_radare2() {
        let disassembler = R2Disasm::new("/bin/ls");
        assert!(disassembler.is_ok());
    }

    #[test]
    fn architecture_non_binary() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let plaintext = format!("{}/{}", project_root, "resources/tests/plaintext");
        disassembler = R2Disasm::new(&plaintext)?;
        assert!(disassembler.get_arch().is_none());
        Ok(())
    }

    #[test]
    fn architecture_unsupported_arch() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let riscv = format!("{}/{}", project_root, "resources/tests/riscv");
        disassembler = R2Disasm::new(&riscv)?;
        assert!(disassembler.get_arch().is_none());
        Ok(())
    }

    #[test]
    fn architecture_x86() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86 = format!("{}/{}", project_root, "resources/tests/x86");
        disassembler = R2Disasm::new(&x86)?;
        assert_eq!(
            disassembler.get_arch().unwrap().name(),
            ArchX86::new_i386().name()
        );
        Ok(())
    }

    #[test]
    fn architecture_x86_64() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        disassembler = R2Disasm::new(&x86_64)?;
        assert!(disassembler.get_arch().is_some());
        assert_eq!(
            disassembler.get_arch().unwrap().name(),
            ArchX86::new_amd64().name()
        );
        Ok(())
    }

    #[test]
    fn architecture_arm() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let armhf = format!("{}/{}", project_root, "resources/tests/armhf");
        disassembler = R2Disasm::new(&armhf)?;
        assert!(disassembler.get_arch().is_some());
        assert_eq!(
            disassembler.get_arch().unwrap().name(),
            ArchARM::new_arm32().name()
        );
        Ok(())
    }

    #[test]
    fn architecture_aarch64() -> Result<(), io::Error> {
        let disassembler;
        let project_root = env!("CARGO_MANIFEST_DIR");
        let aarch64 = format!("{}/{}", project_root, "resources/tests/aarch64");
        disassembler = R2Disasm::new(&aarch64)?;
        assert!(disassembler.get_arch().is_some());
        assert_eq!(
            disassembler.get_arch().unwrap().name(),
            ArchARM::new_aarch64().name()
        );
        Ok(())
    }

    #[test]
    fn function_names() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64)?;
        disassembler.analyse();
        let funcs = disassembler.get_function_names();
        assert_eq!(*funcs.get("main").unwrap(), 0x1149);
        Ok(())
    }

    #[test]
    fn function_names_no_analysis() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let disassembler = R2Disasm::new(&x86_64)?;
        let funcs = disassembler.get_function_names();
        assert!(funcs.is_empty());
        Ok(())
    }

    #[test]
    fn function_body_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let disassembler = R2Disasm::new(&x86_64)?;
        let body = disassembler.get_function_body(0x1000);
        assert!(body.is_none());
        Ok(())
    }

    #[test]
    fn function_body_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64)?;
        disassembler.analyse();
        let maybe_body = disassembler.get_function_body(0x1000);
        assert!(maybe_body.is_some());
        let body = maybe_body.unwrap();
        assert_eq!(body.len(), 8);
        assert_eq!(body[1].get_offset(), 0x1004);
        assert_eq!(body[1].get_instruction(), "sub rsp, 8");
        assert_eq!(body[7].get_offset(), 0x101A);
        assert_eq!(body[7].get_instruction(), "ret");
        Ok(())
    }

    #[test]
    fn function_bodies_not_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let disassembler = R2Disasm::new(&x86_64)?;
        let bodies = disassembler.get_function_bodies();
        assert_eq!(bodies.len(), 0);
        Ok(())
    }

    #[test]
    fn function_bodies_exist() -> Result<(), io::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let x86_64 = format!("{}/{}", project_root, "resources/tests/x86_64");
        let mut disassembler = R2Disasm::new(&x86_64)?;
        disassembler.analyse();
        let bodies = disassembler.get_function_bodies();
        let last_body = disassembler.get_function_body(0x1000);
        assert_eq!(bodies.get(&0x1000).unwrap(), &last_body.unwrap());
        Ok(())
    }
}