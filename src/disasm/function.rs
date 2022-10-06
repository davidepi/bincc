use std::cmp::Ordering;

/// Struct providing information about a function.
///
/// The Function struct represents the position of a function inside a binary file.
/// It is composed of an offset and the actual function name.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Function {
    offset: u64,
    name: String,
}

impl Function {
    /// Creates a new Function with the following parameters:
    /// - `offset`: offset of the function in the binary, ideally from the beginning of the file.
    /// This number will be used to order the various functions.
    /// - `name`: name of the function.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Function;
    ///
    /// let func = Function::new(0x1060, "entry0");
    /// ```
    pub fn new(offset: u64, name: &str) -> Function {
        Function {
            offset,
            name: name.to_string(),
        }
    }

    /// Returns the name of a function.
    ///
    /// This is the same name provided in [Function::new].
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Function;
    ///
    /// let func = Function::new(0x1060, "entry0");
    /// let name = func.get_name();
    ///
    /// assert_eq!(name, "entry0")
    /// ```
    pub fn get_name(&self) -> &str {
        &self.name[..]
    }

    /// Returns the offset of a function.
    ///
    /// This is the same offset provided in [Function::new].
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::disasm::Function;
    ///
    /// let func = Function::new(0x1060, "entry0");
    /// let offset = func.get_offset();
    ///
    /// assert_eq!(offset, 0x1060)
    /// ```
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
}

impl Ord for Function {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::disasm::Function;

    #[test]
    fn ord() {
        let func0 = Function::new(0x441c, "sym.imp.atoi");
        let func1 = Function::new(0x4536, "sym.imp.strlen");
        assert!(func0 < func1)
    }
}
