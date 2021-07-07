use crate::disasm::architectures::Architecture;
use crate::disasm::Statement;
use fnv::FnvHashMap;
use std::collections::HashMap;

/// A very basic Control Flow Graph.
///
/// This crate provide a more advanced version in [analysis::CFG].
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

/// Trait providing disassembler services.
pub trait Disassembler {
    /// Performs analysis on the underlying binary.
    fn analyse(&mut self);

    /// Performs analysis on the function bounds only.
    ///
    /// The default implementation calls [Disassembler::analyse] thus performing a full-binary
    /// analysis.
    fn analyse_functions(&mut self) {
        self.analyse();
    }

    /// Returns the architecture of a specific file.
    ///
    /// This operation *DOES NOT* require to run [Disassembler::analyse] first.
    ///
    /// This implementation currently supports only the architectures defined in the enum
    /// [Architecture].
    ///
    /// If the architecture can not be recognized, None is returned.
    fn get_arch(&self) -> Option<Box<dyn Architecture>>;

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

    /// Returns a simple CFG for the given function.
    ///
    /// This method takes as input the function offset in the binary and returns its CFG generated
    /// by the underlying disassembler.
    ///
    /// If the disassembler is incapable of generating a CFG or the function address is wrong,
    /// [Option::None] is returned.
    fn get_function_cfg(&self, function: u64) -> Option<BareCFG>;
}
