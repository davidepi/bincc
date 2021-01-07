use std::cmp::Ordering;

/// Struct representing an assembly instruction associated with an offset.
///
/// The Statement struct represents an instruction in binary code.
/// It is composed of:
/// - an offset (ideally from the beginning of the binary)
/// - the actual instruction
///
/// This struct is architecture and assembly syntax agnostic, and using it consistently is duty of
/// the programmer.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Statement {
    offset: u64,
    instruction: String,
}

impl Statement {
    /// Creates a new statement with the following parameters:
    /// - `offset`: an offset (from which point is up to the programmer)
    /// - `instruction`: a string representing the actual instruction, like `"mov eax, eax"`.
    /// A space between the mnemonic and the arguments is expected in order to achieve a correct
    /// behaviour in the methods [Statement::get_mnemonic] and [Statement::get_args].
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Statement;
    ///
    /// let stmt = Statement::new(600, "ret");
    /// ```
    pub fn new(offset: u64, instruction: &str) -> Statement {
        Statement {
            offset,
            instruction: instruction.to_ascii_lowercase().trim().to_string(),
        }
    }

    /// Returns the offset where the instruction corresponding to this statement is located.
    ///
    /// This is the offset originally given by the programmer in the constructor.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Statement;
    ///
    /// let stmt = Statement::new(0x600, "mov r9d, dword [rsp + r10 + 0x20]");
    ///
    /// assert_eq!(stmt.get_offset(), 0x600);
    /// ```
    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    /// Returns the instruction associated with this statement.
    ///
    /// This is the instruction originally given by the programmer in the constructor.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Statement;
    ///
    /// let stmt = Statement::new(0x600, "mov r9d, dword [rsp + r10 + 0x20]");
    ///
    /// assert_eq!(stmt.get_instruction(), "mov r9d, dword [rsp + r10 + 0x20]");
    /// ```
    pub fn get_instruction(&self) -> &str {
        &self.instruction
    }

    /// Returns the mnemonic associated with this statement.
    ///
    /// The mnemonic is a string name for a single CPU instruction in a given architecture.
    /// In this case, the mnemonic corresponds to every letter from the beginning of the
    /// instruction's textual representation until the first space.
    ///
    /// The mnemonic will be **always** lowercase.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Statement;
    ///
    /// let stmt = Statement::new(0x600, "MOV r9d, dword [rsp + r10 + 0x20]");
    ///
    /// assert_eq!(stmt.get_mnemonic(), "mov");
    /// ```
    pub fn get_mnemonic(&self) -> &str {
        match self.instruction.find(' ') {
            Some(args_index) => &self.instruction[0..args_index],
            None => &self.instruction,
        }
    }

    /// Returns the instruction's arguments for the current statement.
    ///
    /// This method is complementar to [Statement::get_mnemonic].
    ///
    /// If no arguments are present, an empty string is returned.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Statement;
    ///
    /// let stmt = Statement::new(0x600, "mov r9d, dword [rsp + r10 + 0x20]");
    ///
    /// assert_eq!(stmt.get_args(), "r9d, dword [rsp + r10 + 0x20]");
    /// ```
    pub fn get_args(&self) -> &str {
        match self.instruction.find(' ') {
            Some(args_at) => &self.instruction[args_at + 1..],
            None => &self.instruction[0..0],
        }
    }
}

impl Ord for Statement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl PartialOrd for Statement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::disasm::Statement;

    #[test]
    fn new_no_args() {
        let stmt = Statement::new(1552, "ret");
        assert_eq!(stmt.get_offset(), 0x610);
        assert_eq!(stmt.get_instruction(), "ret");
        assert_eq!(stmt.get_mnemonic(), "ret");
        assert_eq!(stmt.get_args(), "");
    }

    #[test]
    fn new_no_args_untrimmed() {
        //corner case for getting the arguments
        let stmt = Statement::new(1552, "ret ");
        assert_eq!(stmt.get_offset(), 0x610);
        assert_eq!(stmt.get_instruction(), "ret");
        assert_eq!(stmt.get_mnemonic(), "ret");
        assert_eq!(stmt.get_args(), "");
    }

    #[test]
    fn new_multi_args() {
        let stmt = Statement::new(0x5341A5, "mov r9d, dword [rsp + r10 + 0x20]");
        assert_eq!(stmt.get_offset(), 5456293);
        assert_eq!(stmt.get_instruction(), "mov r9d, dword [rsp + r10 + 0x20]");
        assert_eq!(stmt.get_mnemonic(), "mov");
        assert_eq!(stmt.get_args(), "r9d, dword [rsp + r10 + 0x20]");
    }

    #[test]
    fn new_uppercase() {
        let stmt = Statement::new(0x5667, "CMP RAX, r8");
        assert_eq!(stmt.get_offset(), 0x5667);
        assert_eq!(stmt.get_instruction(), "cmp rax, r8");
        assert_eq!(stmt.get_mnemonic(), "cmp");
        assert_eq!(stmt.get_args(), "rax, r8");
    }

    #[test]
    fn ord() {
        let stmt0 = Statement::new(1552, "push");
        let stmt1 = Statement::new(1553, "ret");
        assert!(stmt0 < stmt1);
    }
}
