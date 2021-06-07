use std::fmt::Formatter;

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
pub trait Architecture {
    /// Returns the name of this architecture.
    ///
    /// The name is the same as in GNU multiarch.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::{ArchX86, Architecture};
    ///
    /// let arch = ArchX86::new_amd64();
    ///
    /// assert_eq!(arch.name(), "x86_64");
    /// ```
    fn name(&self) -> String;

    /// Returns the type of jump of the input instruction
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::{ArchX86, Architecture, JumpType};
    ///
    /// let arch = ArchX86::new_amd64();
    /// let jmp_type = arch.jump("jge");
    ///
    /// assert_eq!(jmp_type, JumpType::JumpConditional);
    /// ```
    fn jump(&self, instruction: &str) -> JumpType;
}

/// Intel x86 architecture.
///
/// Represents both the i386 (x86) and AMD64 (x86_64) variants.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArchX86 {
    /// number of bits of this architecture
    bits: u8,
}

impl ArchX86 {
    /// Builds a new architecture of type i386 (x86).
    pub fn new_i386() -> ArchX86 {
        ArchX86 { bits: 32 }
    }
    /// Builds a new architecture of type AMD64 (x86_64)
    pub fn new_amd64() -> ArchX86 {
        ArchX86 { bits: 64 }
    }
}

impl std::fmt::Display for ArchX86 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.bits {
            32 => write!(f, "i386"),
            64 => write!(f, "amd64"),
            _ => panic!(),
        }
    }
}

impl Architecture for ArchX86 {
    fn name(&self) -> String {
        match self.bits {
            32 => "i386".to_string(),
            64 => "x86_64".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn jump(&self, mnemonic: &str) -> JumpType {
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
}

/// ARM architecture.
///
/// Represents both the ARM32 (including Thumb) and AArch64 architectures.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArchARM {
    bits: u8,
}

impl ArchARM {
    /// Builds a new architecture of type ARM32.
    ///
    /// This comprises also the Thumb instruction set.
    pub fn new_arm32() -> ArchARM {
        ArchARM { bits: 32 }
    }

    /// Builds a new architecture of type AArch64.
    pub fn new_aarch64() -> ArchARM {
        ArchARM { bits: 64 }
    }

    /// Removes the conditional part of an opcode.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::ArchARM;
    ///
    /// let insns = "b.eq";
    /// let clean = ArchARM::remove_condition(insns);
    ///
    /// assert_eq!(clean, "b")
    /// ```
    pub fn remove_condition(mnemonic: &str) -> &str {
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
}

impl Architecture for ArchARM {
    fn name(&self) -> String {
        match self.bits {
            32 => "arm".to_string(),
            64 => "aarch64".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn jump(&self, mnemonic: &str) -> JumpType {
        let conditionless_mnemonic = ArchARM::remove_condition(mnemonic);
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
}

#[cfg(test)]
mod tests {
    use crate::disasm::architectures::Architecture;
    use crate::disasm::architectures::{ArchARM, ArchX86, JumpType};

    #[test]
    fn x86_name() {
        let arch32 = ArchX86::new_i386();
        assert_eq!(arch32.name(), "i386");
        let arch64 = ArchX86::new_amd64();
        assert_eq!(arch64.name(), "x86_64");
    }

    #[test]
    fn x86_jump() {
        let archs = vec![ArchX86::new_i386(), ArchX86::new_amd64()];
        let mut mne;
        for arch in archs {
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
    }

    #[test]
    fn arm_name() {
        let arch32 = ArchARM::new_arm32();
        assert_eq!(arch32.name(), "arm");
        let arch64 = ArchARM::new_aarch64();
        assert_eq!(arch64.name(), "aarch64");
    }

    #[test]
    fn arm_jump() {
        let archs = vec![ArchARM::new_arm32(), ArchARM::new_aarch64()];
        let mut mne;
        for arch in archs {
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
            let new_root = ArchARM::remove_condition(&mnemonic);
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
            let new_root = ArchARM::remove_condition(&mnemonic);
            assert_eq!(new_root, root);
        }
    }
}
