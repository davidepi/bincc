/// Enum containing a list of possible jump types.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum JumpType {
    /// Not a jump.
    NoJump,
    /// A conditional jump.
    JumpConditional,
    /// A unconditional jump.
    JumpUnconditional,
    /// A conditional return.
    RetConditional,
    /// A unconditional return.
    RetUnconditional,
}

/// Trait representing a CPU architecture.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Architecture {
    ARC(u32),
    AVR,
    Arm(u32),
    I4004,
    I8051(u32),
    I8080,
    LM32,
    Lh5801,
    M6502,
    M68K,
    MSP430,
    Propeller,
    Mips(u32),
    PowerPC(u32),
    Riscv(u32),
    Sparc(u32),
    V850,
    S390(u32),
    X86(u32),
    Z80,
}

impl Architecture {
    /// Returns the name of this [`Architecture`].
    ///
    /// The name is the same as used by radare2, so x86_64 and i386 will both be listed as `"x86"`.
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bcc::disasm::Architecture;
    /// let arch = Architecture::X86(32);
    ///
    /// assert_eq!(arch.name(), "x86");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Architecture::ARC(_) => "arc",
            Architecture::AVR => "avr",
            Architecture::Arm(_) => "arm",
            Architecture::I4004 => "i4004",
            Architecture::I8051(_) => "8051",
            Architecture::I8080 => "i8080",
            Architecture::LM32 => "lm32",
            Architecture::Lh5801 => "LH5801",
            Architecture::M6502 => "6502",
            Architecture::M68K => "m68k",
            Architecture::MSP430 => "msp430",
            Architecture::Propeller => "propeller",
            Architecture::V850 => "v850",
            Architecture::Z80 => "z80",
            Architecture::S390(_) => "s390",
            Architecture::PowerPC(_) => "ppc",
            Architecture::Mips(_) => "mips",
            Architecture::Riscv(_) => "riscv",
            Architecture::Sparc(_) => "sparc",
            Architecture::X86(_) => "x86",
        }
    }

    /// Returns the number of bits of this [`Architecture`].
    pub fn bits(&self) -> u32 {
        match self {
            Architecture::X86(b)
            | Architecture::Arm(b)
            | Architecture::Sparc(b)
            | Architecture::PowerPC(b)
            | Architecture::Riscv(b)
            | Architecture::Mips(b)
            | Architecture::ARC(b)
            | Architecture::S390(b)
            | Architecture::I8051(b) => *b,
            Architecture::I4004 => 4,
            Architecture::I8080
            | Architecture::M6502
            | Architecture::Lh5801
            | Architecture::AVR
            | Architecture::Z80 => 8,
            Architecture::LM32
            | Architecture::M68K
            | Architecture::Propeller
            | Architecture::V850 => 32,
            Architecture::MSP430 => 16,
        }
    }

    /// Returns the type of jump of the input instruction
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bcc::disasm::{Architecture, JumpType};
    /// let arch = Architecture::X86(32);
    /// let jmp_type = arch.jump("jge");
    ///
    /// assert_eq!(jmp_type, JumpType::JumpConditional);
    /// ```
    pub fn jump(&self, mnemonic: &str) -> JumpType {
        match self {
            Architecture::X86(_) => jump_x86(mnemonic),
            Architecture::Arm(_) => jump_arm(mnemonic),
            _ => unimplemented!(),
        }
    }
}

fn jump_x86(mnemonic: &str) -> JumpType {
    if mnemonic == "ret" {
        JumpType::RetUnconditional
    } else if mnemonic.as_bytes()[0] == b'j' {
        if mnemonic == "jmp" {
            JumpType::JumpUnconditional
        } else {
            JumpType::JumpConditional
        }
    } else {
        JumpType::NoJump
    }
}

/// Removes the conditional part of an opcode.
fn remove_condition_arm(mnemonic: &str) -> &str {
    if mnemonic.len() < 3 {
        mnemonic
    } else {
        let cond = &mnemonic[mnemonic.len() - 2..];
        if cond == "eq"
            || cond == "ne"
            || cond == "cs"
            || cond == "hs"
            || cond == "cc"
            || cond == "lo"
            || cond == "mi"
            || cond == "pl"
            || cond == "vs"
            || cond == "vc"
            || cond == "hi"
            || cond == "ls"
            || cond == "ge"
            || cond == "gt"
            || cond == "lt"
            || cond == "le"
        {
            let cut = &mnemonic[..mnemonic.len() - 2];
            if cut.ends_with('.') {
                &mnemonic[..mnemonic.len() - 3]
            } else {
                cut
            }
        } else {
            mnemonic
        }
    }
}

fn jump_arm(mnemonic: &str) -> JumpType {
    let conditionless_mnemonic = remove_condition_arm(mnemonic);
    if conditionless_mnemonic == "b" {
        if conditionless_mnemonic != mnemonic {
            JumpType::JumpConditional
        } else {
            JumpType::JumpUnconditional
        }
    } else if conditionless_mnemonic == "bx" {
        if conditionless_mnemonic != mnemonic {
            JumpType::RetConditional
        } else {
            JumpType::RetUnconditional
        }
    } else {
        JumpType::NoJump
    }
}

#[cfg(test)]
mod tests {
    use crate::disasm::{architectures::remove_condition_arm, Architecture, JumpType};

    #[test]
    fn x86_jump() {
        let arch = Architecture::X86(32);
        let mut mne;
        mne = "jo";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnl";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "mul";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "jnbe";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jl";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jcxz";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnc";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jb";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jno";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jp";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jg";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "div";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "jge";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jng";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jns";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnz";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jpe";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jle";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jmp";
        assert_eq!(arch.jump(mne), JumpType::JumpUnconditional);
        mne = "jna";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jne";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "min";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "jnae";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnp";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "rsqrt";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "je";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "ja";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnle";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jnb";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jc";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jae";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jpo";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "max";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "jnge";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jbe";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jecxz";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "sqrt";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "sub";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "js";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "jz";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "rcp";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "add";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "bx lr";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "ret";
        assert_eq!(arch.jump(mne), JumpType::RetUnconditional);
    }

    #[test]
    fn arm_jump() {
        let arch = Architecture::Arm(32);
        let mut mne;
        mne = "beq";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bne";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bcs";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bhs";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bcc";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "blo";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bmi";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bpl";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bvs";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bvc";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bhi";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bls";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bge";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "bgt";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "blt";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "ble";
        assert_eq!(arch.jump(mne), JumpType::JumpConditional);
        mne = "b";
        assert_eq!(arch.jump(mne), JumpType::JumpUnconditional);
        mne = "bl";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
        mne = "bxle";
        assert_eq!(arch.jump(mne), JumpType::RetConditional);
        mne = "bx";
        assert_eq!(arch.jump(mne), JumpType::RetUnconditional);
        mne = "ret";
        assert_eq!(arch.jump(mne), JumpType::NoJump);
    }

    #[test]
    fn arm_remove_cond_old_syntax() {
        //older versions of radare2 used the syntax `beq`
        let root = "b";
        let suffixes = vec![
            "eq", "ne", "cs", "hs", "cc", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "gt",
            "lt", "le",
        ];
        for suffix in suffixes {
            let mnemonic = format!("{}{}", root, suffix);
            let new_root = remove_condition_arm(&mnemonic);
            assert_eq!(new_root, root);
        }
    }

    #[test]
    fn arm_remove_cond_new_syntax() {
        //newer versions of radare2 used the syntax `b.eq`
        let root = "b";
        let suffixes = vec![
            "eq", "ne", "cs", "hs", "cc", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "gt",
            "lt", "le",
        ];
        for suffix in suffixes {
            let mnemonic = format!("{}.{}", root, suffix);
            let new_root = remove_condition_arm(&mnemonic);
            assert_eq!(new_root, root);
        }
    }
}
