use crate::disasm::Statement;
use fnv::FnvHashMap;
use std::collections::HashMap;

/// Enum containing a list of CPU architectures.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Architecture {
    UNKNOWN,
    X86_64,
    AARCH64,
    X86,
    ARM32,
}

/// Trait providing disassembler services.
pub trait Disassembler {
    /// Perform analysis on the underlying binary.
    fn analyse(&mut self);

    /// Returns the architecture of a specific file.
    ///
    /// This operation *DOES NOT* require to run [Disassembler::analyse] first.
    ///
    /// This implementation currently supports only the architectures defined in the enum
    /// [Architecture].
    fn get_arch(&self) -> Architecture;

    /// Returns names and offsets of every function in the current executable.
    ///
    /// This operation requires calling [Disassembler::analyse] first.
    ///
    /// The returned map contains pairs `(function name, offset in the binary)`.
    fn get_function_names(&self) -> HashMap<String, u64>;

    /// Return a map containing all the statements for every function in the binary.
    ///
    /// The returned map contains pairs `(function offset, vector of statements)`
    ///
    /// This operation requires calling [Disassembler::analyse] first.
    fn get_function_bodies(&self) -> FnvHashMap<u64, Vec<Statement>>;

    /// Returns a list of statements for a given function.
    ///
    /// This method takes as input the function offset in the binary and returns a vector containing
    /// the list of statements. None if the function can not be found.
    ///
    /// This operation requires calling [Disassembler::analyse] first.
    fn get_function_body(&self, function: u64) -> Option<Vec<Statement>>;
}
