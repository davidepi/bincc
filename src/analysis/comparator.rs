use crate::analysis::blocks::StructureBlock;
use fnv::FnvHashMap;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CloneClass<'a> {
    binaries: Vec<&'a str>,
    functions: Vec<&'a str>,
    structures: Vec<StructureBlock>,
}

impl<'a> CloneClass<'a> {
    pub fn len(&self) -> usize {
        self.structures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn nth(&self, index: usize) -> Option<(&'a str, &'a str, StructureBlock)> {
        if let (Some(bin), Some(fun), Some(cfs)) = (
            self.binaries.get(index),
            self.functions.get(index),
            self.structures.get(index),
        ) {
            Some((bin, fun, cfs.clone()))
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'a str, &'a str, StructureBlock)> + '_ {
        self.binaries
            .iter()
            .zip(self.functions.iter())
            .zip(self.structures.iter().cloned())
            .map(|((&a, &b), c)| (a, b, c))
    }

    pub fn depth(&self) -> u32 {
        if let Some(first) = self.structures.first() {
            first.depth()
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
struct CloneCandidate {
    binary_id: u32,
    function_id: u32,
    structure: StructureBlock,
}

/// Compares several CFS and discovers binary clones.
pub struct CFSComparator {
    /// Contains the CFS hashes and the possible clone with that hash
    hashes: FnvHashMap<u64, Vec<CloneCandidate>>,
    /// Assigns an ID to binary/function names
    names: HashMap<String, u32>,
    /// Discard CFSs smaller than this length
    mindepth: u32,
}

impl CFSComparator {
    /// Creates a new comparator with the given threshold.
    ///
    ///  The threshold `mindepth` (called `θ` in the paper) is the minimum number of nested nodes
    ///  contained in the CFS.
    pub fn new(mindepth: u32) -> Self {
        CFSComparator {
            hashes: FnvHashMap::default(),
            names: HashMap::new(),
            mindepth,
        }
    }

    /// Inserts a new function in the comparator.
    ///
    /// The actual comparison is done by calling the [`CFSComparator::clones`] function.
    pub fn insert(&mut self, structure: StructureBlock, bin_name: String, func_name: String) {
        let next_id = self.names.len() as u32;
        let binary_id = *self.names.entry(bin_name).or_insert(next_id);
        let next_id = self.names.len() as u32;
        let function_id = *self.names.entry(func_name).or_insert(next_id);
        let mut stack = vec![structure];
        while let Some(node) = stack.pop() {
            if node.depth() >= self.mindepth {
                let candidate = CloneCandidate {
                    binary_id,
                    function_id,
                    structure: node.clone(),
                };
                let mut hasher = DefaultHasher::new();
                node.structural_hash(&mut hasher);
                let hash = hasher.finish();
                self.hashes
                    .entry(hash)
                    .and_modify(|e| e.push(candidate.clone()))
                    .or_insert_with(|| vec![candidate.clone()]);
                stack.extend(node.children().iter().cloned());
            }
        }
    }

    /// Retrieves the clone class from this comparator.
    ///
    /// The various functions to be checcked for clones should be inserted by calling
    /// [`CFSComparator::insert`] prior to this function.
    pub fn clones(&self) -> Vec<CloneClass> {
        let mut retval = Vec::new();
        let mut reverse_tmp = self
            .names
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<Vec<_>>();
        reverse_tmp.sort_unstable();
        let reverse_names = reverse_tmp
            .into_iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>();
        for class_candidate in self.hashes.values() {
            let class_len = class_candidate.len();
            if class_len > 1 {
                let mut binaries = Vec::with_capacity(class_len);
                let mut functions = Vec::with_capacity(class_len);
                let mut structures = Vec::with_capacity(class_len);
                for clone in class_candidate {
                    binaries.push(reverse_names[clone.binary_id as usize]);
                    functions.push(reverse_names[clone.function_id as usize]);
                    structures.push(clone.structure.clone());
                }
                retval.push(CloneClass {
                    binaries,
                    functions,
                    structures,
                });
            }
        }
        retval
    }
}
#[cfg(test)]
mod tests {
    use crate::analysis::{CFSComparator, CFG, CFS};
    use crate::disasm::{ArchX86, Statement, StatementType};

    fn create_function() -> Vec<Statement> {
        vec![
            Statement::new(0x00, StatementType::CMP, "test eax, eax"),
            Statement::new(0x04, StatementType::CJMP, "jg 0x38"),
            Statement::new(0x08, StatementType::ADD, "add ebx, 5"),
            Statement::new(0x0C, StatementType::JMP, "jmp 0x10"),
            Statement::new(0x10, StatementType::CMP, "cmp eax, ebx"),
            Statement::new(0x14, StatementType::CJMP, "jne 0x20"),
            Statement::new(0x18, StatementType::CMP, "cmp ebx, 5"),
            Statement::new(0x1C, StatementType::CJMP, "jne 0x18"),
            Statement::new(0x20, StatementType::MOV, "mov ecx, [ebp+8]"),
            Statement::new(0x24, StatementType::JMP, "jmp 0x28"),
            Statement::new(0x28, StatementType::CMP, "cmp ecx, eax"),
            Statement::new(0x2C, StatementType::MOV, "mov eax, -1"),
            Statement::new(0x30, StatementType::CJMP, "jne 0x08"),
            Statement::new(0x34, StatementType::RET, "ret"),
            Statement::new(0x38, StatementType::ADD, "incl eax"),
            Statement::new(0x3C, StatementType::MOV, "mov ebx, [ebp+20]"),
            Statement::new(0x40, StatementType::CMP, "cmp eax, ebx"),
            Statement::new(0x44, StatementType::CJMP, "je 0x58"),
            Statement::new(0x48, StatementType::MOV, "mov ecx, [ebp+20]"),
            Statement::new(0x4C, StatementType::SUB, "decl ecx"),
            Statement::new(0x50, StatementType::MOV, "mov [ebp+20], ecx"),
            Statement::new(0x54, StatementType::JMP, "jmp 0x38"),
            Statement::new(0x58, StatementType::CMP, "test eax, eax"),
            Statement::new(0x5C, StatementType::MOV, "mov eax, 0"),
            Statement::new(0x60, StatementType::CJMP, "je 0x68"),
            Statement::new(0x64, StatementType::MOV, "mov eax, 1"),
            Statement::new(0x68, StatementType::RET, "ret"),
        ]
    }

    #[test]
    fn cloned_full() {
        let stmts = create_function();
        let cfg = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs = CFS::new(&cfg);
        let mut diff = CFSComparator::new(7);
        diff.insert(cfs.get_tree().unwrap(), "ab".to_string(), "af".to_string());
        diff.insert(cfs.get_tree().unwrap(), "bb".to_string(), "bf".to_string());
        diff.insert(cfs.get_tree().unwrap(), "cb".to_string(), "cf".to_string());
        diff.insert(cfs.get_tree().unwrap(), "db".to_string(), "df".to_string());
        diff.insert(cfs.get_tree().unwrap(), "eb".to_string(), "ef".to_string());
        let clones = diff.clones();
        assert_eq!(clones.len(), 1);
        let class = &clones[0];
        assert_eq!(class.len(), 5);
    }

    #[test]
    fn cloned_partial() {
        let mut stmts = create_function();
        let cfg0 = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs0 = CFS::new(&cfg0);
        let mut diff = CFSComparator::new(2);
        diff.insert(cfs0.get_tree().unwrap(), "ab".to_string(), "af".to_string());
        stmts = create_function();
        stmts[2] = Statement::new(0x08, StatementType::NOP, "nop");
        stmts[3] = Statement::new(0x0C, StatementType::NOP, "nop");
        stmts[10] = Statement::new(0x28, StatementType::NOP, "nop");
        stmts[11] = Statement::new(0x2C, StatementType::NOP, "nop");
        stmts[12] = Statement::new(0x30, StatementType::NOP, "nop");
        let cfg1 = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs1 = CFS::new(&cfg1);
        diff.insert(cfs1.get_tree().unwrap(), "bb".to_string(), "bf".to_string());
        assert!(!cfs0
            .get_tree()
            .unwrap()
            .structural_equality(&cfs1.get_tree().unwrap()));
        let clones = diff.clones();
        assert_eq!(clones.len(), 2);
        assert_eq!(clones[0].depth(), 2);
        assert_eq!(clones[1].depth(), 2);
    }
}
