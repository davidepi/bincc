use std::{cmp::Ordering, io::ErrorKind};

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
    stype: StatementFamily,
    instruction: String,
}

impl Statement {
    /// Creates a new statement with the following parameters:
    /// - `offset`: an offset (from which point is up to the programmer)
    /// - `stype`: the type of this statement. This represent a more abstract categorization for
    /// the statement, independent of its underlying architecture.
    /// - `instruction`: a string representing the actual instruction, like `"mov eax, eax"`.
    /// A space between the mnemonic and the arguments is expected in order to achieve a correct
    /// behaviour in the methods [Statement::get_mnemonic] and [Statement::get_args].
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(600, StatementFamily::RET, "ret");
    /// ```
    pub fn new(offset: u64, stype: StatementFamily, instruction: &str) -> Statement {
        Statement {
            offset,
            stype,
            instruction: instruction.to_ascii_lowercase().trim().to_string(),
        }
    }

    /// Returns the offset where the instruction corresponding to this statement is located.
    ///
    /// This is the offset originally given by the programmer in the constructor.
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(
    ///     0x600,
    ///     StatementFamily::MOV,
    ///     "mov r9d, dword [rsp + r10 + 0x20]",
    /// );
    ///
    /// assert_eq!(stmt.get_offset(), 0x600);
    /// ```
    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    /// Returns the group this statement belongs to.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(
    ///     0x600,
    ///     StatementFamily::MOV,
    ///     "mov r9d, dword [rsp + r10 + 0x20]",
    /// );
    ///
    /// assert_eq!(stmt.get_family(), StatementFamily::MOV);
    /// ```
    pub fn get_family(&self) -> StatementFamily {
        self.stype
    }

    /// Returns the instruction associated with this statement.
    ///
    /// This is the instruction originally given by the programmer in the constructor.
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(
    ///     0x600,
    ///     StatementFamily::MOV,
    ///     "mov r9d, dword [rsp + r10 + 0x20]",
    /// );
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
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(
    ///     0x600,
    ///     StatementFamily::MOV,
    ///     "MOV r9d, dword [rsp + r10 + 0x20]",
    /// );
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
    /// # use bincc::disasm::{Statement, StatementFamily};
    /// let stmt = Statement::new(
    ///     0x600,
    ///     StatementFamily::MOV,
    ///     "mov r9d, dword [rsp + r10 + 0x20]",
    /// );
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

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// Statements categorization that does not depend on the statement particular ISA.
/// This categorization is similar to the one used in `radare2` by the variable `R_ANAL_OP_TYPE_###`
pub enum StatementFamily {
    ABS,
    ADD,
    AND,
    CALL,
    CAST,
    CJMP,
    CMOV,
    CMP,
    CPL,
    CRYPTO,
    DEBUG,
    DIV,
    FPU,
    ILL,
    IO,
    JMP,
    LEA,
    LEAVE,
    LENGTH,
    LOAD,
    MASK,
    MOD,
    MOV,
    MUL,
    NEW,
    NOP,
    NOR,
    NOT,
    NULL,
    OR,
    POP,
    PRIV,
    PUSH,
    RET,
    ROL,
    ROR,
    SAL,
    SAR,
    SHL,
    SHR,
    STORE,
    SUB,
    SWI,
    SYNC,
    TRAP,
    UNK,
    XCHG,
    XOR,
}

impl StatementFamily {
    /// Converts the statement type into a string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ABS => "abs",
            Self::ADD => "add",
            Self::AND => "and",
            Self::CALL => "call",
            Self::CAST => "cast",
            Self::CJMP => "cjmp",
            Self::CMOV => "cmov",
            Self::CMP => "cmp",
            Self::CPL => "cpl",
            Self::CRYPTO => "crypto",
            Self::DEBUG => "debug",
            Self::DIV => "div",
            Self::FPU => "fpu",
            Self::ILL => "ill",
            Self::IO => "io",
            Self::JMP => "jmp",
            Self::LEA => "lea",
            Self::LEAVE => "leave",
            Self::LENGTH => "length",
            Self::LOAD => "load",
            Self::MASK => "mask",
            Self::MOD => "mod",
            Self::MOV => "mov",
            Self::MUL => "mul",
            Self::NEW => "new",
            Self::NOP => "nop",
            Self::NOR => "nor",
            Self::NOT => "not",
            Self::NULL => "null",
            Self::OR => "or",
            Self::POP => "pop",
            Self::PRIV => "priv",
            Self::PUSH => "push",
            Self::RET => "ret",
            Self::ROL => "rol",
            Self::ROR => "ror",
            Self::SAL => "sal",
            Self::SAR => "sar",
            Self::SHL => "shl",
            Self::SHR => "shr",
            Self::STORE => "store",
            Self::SUB => "sub",
            Self::SWI => "swi",
            Self::SYNC => "sync",
            Self::TRAP => "trap",
            Self::XCHG => "xchg",
            Self::XOR => "xor",
            Self::UNK => "unk",
        }
    }
}

impl TryFrom<&str> for StatementFamily {
    type Error = std::io::Error;

    fn try_from(s: &str) -> Result<StatementFamily, Self::Error> {
        match s {
            "abs" => Ok(Self::ABS),
            "add" => Ok(Self::ADD),
            "and" => Ok(Self::AND),
            "call" | "icall" | "ircall" | "ucall" | "rcall" | "ccall" | "uccall" => Ok(Self::CALL),
            "cast" => Ok(Self::CAST),
            "cjmp" | "rcjmp" | "ucjmp" | "mcjmp" => Ok(Self::CJMP),
            "cmov" => Ok(Self::CMOV),
            "acmp" | "cmp" => Ok(Self::CMP),
            "cpl" => Ok(Self::CPL),
            "crypto" => Ok(Self::CRYPTO),
            "debug" => Ok(Self::DEBUG),
            "div" => Ok(Self::DIV),
            "fpu" => Ok(Self::FPU),
            "ill" => Ok(Self::ILL),
            "io" => Ok(Self::IO),
            "jmp" | "ijmp" | "irjmp" | "ujmp" | "mjmp" | "rjmp" => Ok(Self::JMP),
            "lea" | "ulea" => Ok(Self::LEA),
            "leave" => Ok(Self::LEAVE),
            "length" => Ok(Self::LENGTH),
            "load" => Ok(Self::LOAD),
            "mask" => Ok(Self::MASK),
            "mod" => Ok(Self::MOD),
            "mov" => Ok(Self::MOV),
            "mul" => Ok(Self::MUL),
            "new" => Ok(Self::NEW),
            "nop" => Ok(Self::NOP),
            "nor" => Ok(Self::NOR),
            "not" => Ok(Self::NOT),
            "null" => Ok(Self::NULL),
            "or" => Ok(Self::OR),
            "pop" => Ok(Self::POP),
            "priv" => Ok(Self::PRIV),
            "push" | "rpush" | "upush" => Ok(Self::PUSH),
            "ret" | "cret" => Ok(Self::RET),
            "rol" => Ok(Self::ROL),
            "ror" => Ok(Self::ROR),
            "sal" => Ok(Self::SAL),
            "sar" => Ok(Self::SAR),
            "shl" => Ok(Self::SHL),
            "shr" => Ok(Self::SHR),
            "store" => Ok(Self::STORE),
            "sub" => Ok(Self::SUB),
            "swi" | "cswi" => Ok(Self::SWI),
            "sync" => Ok(Self::SYNC),
            "trap" => Ok(Self::TRAP),
            "xchg" => Ok(Self::XCHG),
            "xor" => Ok(Self::XOR),
            "unk" => Ok(Self::UNK),
            _ => Err(Self::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid type {}", s),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::disasm::{statement::StatementFamily, Statement};

    #[test]
    fn new_no_args() {
        let stmt = Statement::new(1552, StatementFamily::RET, "ret");
        assert_eq!(stmt.get_offset(), 0x610);
        assert_eq!(stmt.get_family(), StatementFamily::RET);
        assert_eq!(stmt.get_instruction(), "ret");
        assert_eq!(stmt.get_mnemonic(), "ret");
        assert_eq!(stmt.get_args(), "");
    }

    #[test]
    fn new_no_args_untrimmed() {
        //corner case for getting the arguments
        let stmt = Statement::new(1552, StatementFamily::RET, "ret ");
        assert_eq!(stmt.get_offset(), 0x610);
        assert_eq!(stmt.get_family(), StatementFamily::RET);
        assert_eq!(stmt.get_instruction(), "ret");
        assert_eq!(stmt.get_mnemonic(), "ret");
        assert_eq!(stmt.get_args(), "");
    }

    #[test]
    fn new_multi_args() {
        let stmt = Statement::new(
            0x5341A5,
            StatementFamily::MOV,
            "mov r9d, dword [rsp + r10 + 0x20]",
        );
        assert_eq!(stmt.get_offset(), 5456293);
        assert_eq!(stmt.get_family(), StatementFamily::MOV);
        assert_eq!(stmt.get_instruction(), "mov r9d, dword [rsp + r10 + 0x20]");
        assert_eq!(stmt.get_mnemonic(), "mov");
        assert_eq!(stmt.get_args(), "r9d, dword [rsp + r10 + 0x20]");
    }

    #[test]
    fn new_uppercase() {
        let stmt = Statement::new(0x5667, StatementFamily::CMP, "CMP RAX, r8");
        assert_eq!(stmt.get_offset(), 0x5667);
        assert_eq!(stmt.get_family(), StatementFamily::CMP);
        assert_eq!(stmt.get_instruction(), "cmp rax, r8");
        assert_eq!(stmt.get_mnemonic(), "cmp");
        assert_eq!(stmt.get_args(), "rax, r8");
    }

    #[test]
    fn ord() {
        let stmt0 = Statement::new(1552, StatementFamily::PUSH, "push");
        let stmt1 = Statement::new(1553, StatementFamily::RET, "ret");
        assert!(stmt0 < stmt1);
    }
}
