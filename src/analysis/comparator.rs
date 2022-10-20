use crate::analysis::blocks::StructureBlock;
use crate::disasm::Statement;
use fnv::FnvHashMap;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CloneClass<'a> {
    binaries: Vec<&'a str>,
    functions: Vec<&'a str>,
    structures: Option<Vec<&'a StructureBlock>>,
    iterator_index: usize,
}

impl<'a> CloneClass<'a> {
    pub fn len(&self) -> usize {
        self.binaries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_names(&self) -> impl Iterator<Item = (&'a str, &'a str)> + '_ {
        self.binaries
            .iter()
            .copied()
            .zip(self.functions.iter().copied())
    }

    pub fn depth(&self) -> u32 {
        if let Some(structures) = &self.structures {
            structures.first().map(|s| s.depth()).unwrap_or(0)
        } else {
            0
        }
    }
}

impl<'a> Iterator for CloneClass<'a> {
    type Item = (&'a str, &'a str, Option<&'a StructureBlock>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.iterator_index < self.binaries.len() {
            let bin = self.binaries[self.iterator_index];
            let fun = self.functions[self.iterator_index];
            let structure = self.structures.as_ref().map(|v| v[self.iterator_index]);
            self.iterator_index += 1;
            Some((bin, fun, structure))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct CloneCandidate<'a> {
    bin_id: u32,
    func_id: u32,
    structure: &'a StructureBlock,
}

/// Compares several CFS and discovers binary clones.
pub struct CFSComparator<'a> {
    /// Contains the CFS hashes and the possible clone with that hash
    hashes: FnvHashMap<u64, Vec<CloneCandidate<'a>>>,
    /// Discard CFSs smaller than this length
    mindepth: u32,
}

impl<'a> CFSComparator<'a> {
    /// Creates a new comparator with the given threshold.
    ///
    ///  The threshold `mindepth` (called `θ` in the paper) is the minimum number of nested nodes
    ///  contained in the CFS.
    pub fn new(mindepth: u32) -> Self {
        CFSComparator {
            hashes: FnvHashMap::default(),
            mindepth,
        }
    }

    /// Inserts a new function in the comparator.
    ///
    /// The actual comparison is done by calling the [`CFSComparator::clones`] function.
    ///
    /// binary_id and function_id are unique identifiers for a binary or a function.
    pub fn insert(&mut self, binary_id: u32, function_id: u32, structure: &'a StructureBlock) {
        let mut stack = vec![structure];
        while let Some(node) = stack.pop() {
            if node.depth() >= self.mindepth {
                let candidate = CloneCandidate {
                    bin_id: binary_id,
                    func_id: function_id,
                    structure: node,
                };
                let mut hasher = DefaultHasher::new();
                node.structural_hash(&mut hasher);
                let hash = hasher.finish();
                self.hashes
                    .entry(hash)
                    .and_modify(|e| e.push(candidate.clone()))
                    .or_insert_with(|| vec![candidate.clone()]);
                stack.extend(node.children().iter());
            }
        }
    }

    /// Retrieves the clone class from this comparator.
    ///
    /// The various functions to be checcked for clones should be inserted by calling
    /// [`CFSComparator::insert`] prior to this function.
    pub fn clones<'b: 'a>(&self, string_cache: &'b FnvHashMap<u32, String>) -> Vec<CloneClass<'a>> {
        let mut retval = HashSet::new();
        for class_candidate in self.hashes.values() {
            let class_len = class_candidate.len();
            if class_len > 1 {
                let mut binaries = Vec::with_capacity(class_len);
                let mut functions = Vec::with_capacity(class_len);
                let mut structures = Vec::with_capacity(class_len);
                for clone in class_candidate {
                    binaries.push(string_cache.get(&clone.bin_id).unwrap().as_str());
                    functions.push(string_cache.get(&clone.func_id).unwrap().as_str());
                    structures.push(clone.structure);
                }
                retval.insert(CloneClass {
                    binaries,
                    functions,
                    structures: Some(structures),
                    iterator_index: 0,
                });
            }
        }
        retval.into_iter().collect()
    }
}

pub struct SemanticComparator<'a> {
    bin_id: Vec<u32>,
    fun_id: Vec<u32>,
    fvec: Vec<&'a FVec>,
    structures: Vec<&'a StructureBlock>,
    min_similarity: f32,
}

impl<'a> SemanticComparator<'a> {
    pub fn new(min_similarity: f32) -> Self {
        SemanticComparator {
            bin_id: Vec::new(),
            fun_id: Vec::new(),
            fvec: Vec::new(),
            structures: Vec::new(),
            min_similarity,
        }
    }

    pub fn insert(
        &mut self,
        binary_id: u32,
        function_id: u32,
        fvec: &'a FVec,
        structure: Option<&'a StructureBlock>,
    ) {
        self.bin_id.push(binary_id);
        self.fun_id.push(function_id);
        self.fvec.push(fvec);
        if let Some(structure) = structure {
            self.structures.push(structure);
        }
    }

    pub fn clones<'b>(&self, string_cache: &'b FnvHashMap<u32, String>) -> Vec<CloneClass<'b>>
    where
        'a: 'b,
    {
        let mut retval = HashSet::new();
        let use_structures = self.fvec.len() == self.structures.len();
        for a in self.fvec.iter() {
            let mut binaries = Vec::new();
            let mut functions = Vec::new();
            let mut structures = Vec::new();
            for (index_b, b) in self.fvec.iter().enumerate() {
                if a.cosine_similarity(b) > self.min_similarity {
                    let bin_b = string_cache.get(&self.bin_id[index_b]).unwrap().as_str();
                    let func_b = string_cache.get(&self.fun_id[index_b]).unwrap().as_str();
                    binaries.push(bin_b);
                    functions.push(func_b);
                    if use_structures {
                        structures.push(self.structures[index_b]);
                    }
                }
            }
            if binaries.len() > 1 {
                retval.insert(CloneClass {
                    binaries,
                    functions,
                    structures: if use_structures {
                        Some(structures)
                    } else {
                        None
                    },
                    iterator_index: 0,
                });
            }
        }
        retval.into_iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct FVec {
    sparse: FnvHashMap<u16, f32>,
}

impl FVec {
    pub fn new(
        stmts: Vec<Statement>,
        opcode_map: &mut HashMap<String, u16>,
        cross_arch: bool,
    ) -> Self {
        let mut count = FnvHashMap::default();
        let stmts_no = stmts.len();
        for stmt in stmts {
            let next = opcode_map.len() as u16;
            let stmt_name = if cross_arch {
                stmt.get_family().to_str().to_string()
            } else {
                stmt.get_mnemonic().to_string()
            };
            let id = *opcode_map.entry(stmt_name).or_insert(next);
            count
                .entry(id)
                .and_modify(|occurrences| *occurrences += 1)
                .or_insert(1);
        }
        FVec {
            sparse: count
                .into_iter()
                .map(|(id, count)| (id, count as f32 / stmts_no as f32))
                .collect(),
        }
    }

    pub fn cosine_similarity(&self, other: &FVec) -> f32 {
        let my_max = self.sparse.keys().max().copied().unwrap_or(0);
        let other_max = other.sparse.keys().max().copied().unwrap_or(0);
        let maxmax = u16::max(my_max, other_max);
        // expand sparse vectors
        let mut my_expanded = vec![0.0; maxmax as usize + 1];
        for (opcode_id, frequency) in &self.sparse {
            my_expanded[*opcode_id as usize] = *frequency;
        }
        let mut other_expanded = vec![0.0; maxmax as usize + 1];
        for (opcode_id, frequency) in &other.sparse {
            other_expanded[*opcode_id as usize] = *frequency;
        }
        let adotb = my_expanded
            .iter()
            .zip(other_expanded.iter())
            .fold(0.0, |acc, (a, b)| acc + a * b);
        let norm_a = my_expanded.iter().fold(0.0, |acc, a| acc + a * a).sqrt();
        let norm_b = other_expanded.iter().fold(0.0, |acc, b| acc + b * b).sqrt();
        adotb / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::analysis::{CFSComparator, FVec, SemanticComparator, CFG, CFS};
    use crate::disasm::{Architecture, Statement, StatementFamily};
    use fnv::FnvHashMap;

    fn create_function() -> Vec<Statement> {
        vec![
            Statement::new(0x00, StatementFamily::CMP, "test eax, eax"),
            Statement::new(0x04, StatementFamily::CJMP, "jg 0x38"),
            Statement::new(0x08, StatementFamily::ADD, "add ebx, 5"),
            Statement::new(0x0C, StatementFamily::JMP, "jmp 0x10"),
            Statement::new(0x10, StatementFamily::CMP, "cmp eax, ebx"),
            Statement::new(0x14, StatementFamily::CJMP, "jne 0x20"),
            Statement::new(0x18, StatementFamily::CMP, "cmp ebx, 5"),
            Statement::new(0x1C, StatementFamily::CJMP, "jne 0x18"),
            Statement::new(0x20, StatementFamily::MOV, "mov ecx, [ebp+8]"),
            Statement::new(0x24, StatementFamily::JMP, "jmp 0x28"),
            Statement::new(0x28, StatementFamily::CMP, "cmp ecx, eax"),
            Statement::new(0x2C, StatementFamily::MOV, "mov eax, -1"),
            Statement::new(0x30, StatementFamily::CJMP, "jne 0x08"),
            Statement::new(0x34, StatementFamily::RET, "ret"),
            Statement::new(0x38, StatementFamily::ADD, "incl eax"),
            Statement::new(0x3C, StatementFamily::MOV, "mov ebx, [ebp+20]"),
            Statement::new(0x40, StatementFamily::CMP, "cmp eax, ebx"),
            Statement::new(0x44, StatementFamily::CJMP, "je 0x58"),
            Statement::new(0x48, StatementFamily::MOV, "mov ecx, [ebp+20]"),
            Statement::new(0x4C, StatementFamily::SUB, "decl ecx"),
            Statement::new(0x50, StatementFamily::MOV, "mov [ebp+20], ecx"),
            Statement::new(0x54, StatementFamily::JMP, "jmp 0x38"),
            Statement::new(0x58, StatementFamily::CMP, "test eax, eax"),
            Statement::new(0x5C, StatementFamily::MOV, "mov eax, 0"),
            Statement::new(0x60, StatementFamily::CJMP, "je 0x68"),
            Statement::new(0x64, StatementFamily::MOV, "mov eax, 1"),
            Statement::new(0x68, StatementFamily::RET, "ret"),
        ]
    }

    fn create_string_cache() -> FnvHashMap<u32, String> {
        let mut string_cache = FnvHashMap::default();
        string_cache.insert(0, "bin_a".to_string());
        string_cache.insert(1, "bin_b".to_string());
        string_cache.insert(2, "bin_c".to_string());
        string_cache.insert(3, "bin_d".to_string());
        string_cache.insert(4, "bin_e".to_string());
        string_cache.insert(10, "fun_a".to_string());
        string_cache.insert(11, "fun_b".to_string());
        string_cache.insert(12, "fun_c".to_string());
        string_cache.insert(13, "fun_d".to_string());
        string_cache.insert(14, "fun_e".to_string());
        string_cache
    }

    #[test]
    fn structural_cloned_full() {
        let stmts = create_function();
        let cfg = CFG::new(&stmts, 0x6C, Architecture::X86(64)).add_sink();
        let cfs = CFS::new(&cfg).get_tree().unwrap();
        let mut diff = CFSComparator::new(7);
        diff.insert(0, 10, &cfs);
        diff.insert(1, 11, &cfs);
        diff.insert(2, 12, &cfs);
        diff.insert(3, 13, &cfs);
        diff.insert(4, 14, &cfs);
        let string_cache = create_string_cache();
        let clones = diff.clones(&string_cache);
        assert_eq!(clones.len(), 1);
        let class = &clones[0];
        assert_eq!(class.len(), 5);
    }

    #[test]
    fn structural_cloned_partial() {
        let mut stmts = create_function();
        let cfg0 = CFG::new(&stmts, 0x6C, Architecture::X86(64)).add_sink();
        let cfs0 = CFS::new(&cfg0).get_tree().unwrap();
        let string_cache = create_string_cache();
        let mut diff = CFSComparator::new(2);
        diff.insert(0, 10, &cfs0);
        stmts = create_function();
        stmts[2] = Statement::new(0x08, StatementFamily::NOP, "nop");
        stmts[3] = Statement::new(0x0C, StatementFamily::NOP, "nop");
        stmts[10] = Statement::new(0x28, StatementFamily::NOP, "nop");
        stmts[11] = Statement::new(0x2C, StatementFamily::NOP, "nop");
        stmts[12] = Statement::new(0x30, StatementFamily::NOP, "nop");
        let cfg1 = CFG::new(&stmts, 0x6C, Architecture::X86(64)).add_sink();
        let cfs1 = CFS::new(&cfg1).get_tree().unwrap();
        diff.insert(1, 11, &cfs1);
        let clones = diff.clones(&string_cache);
        assert_eq!(clones.len(), 2);
        assert_eq!(clones[0].depth(), 2);
        assert_eq!(clones[1].depth(), 2);
    }

    #[test]
    fn semantic_clone_full() {
        let stmts = create_function();
        let mut opcode_map = HashMap::new();
        let fvec = FVec::new(stmts, &mut opcode_map, false);
        let mut diff = SemanticComparator::new(0.7);
        diff.insert(0, 10, &fvec, None);
        diff.insert(1, 11, &fvec, None);
        diff.insert(2, 12, &fvec, None);
        diff.insert(3, 13, &fvec, None);
        diff.insert(4, 14, &fvec, None);
        let string_cache = create_string_cache();
        let clones = diff.clones(&string_cache);
        assert_eq!(clones.len(), 1);
        let class = &clones[0];
        assert_eq!(class.len(), 5);
    }

    #[test]
    fn semantic_clone_different() {
        let stmts = create_function();
        let mut opcode_map = HashMap::new();
        let fvec = FVec::new(stmts, &mut opcode_map, false);
        let mut diff = SemanticComparator::new(0.7);
        diff.insert(0, 10, &fvec, None);
        let mut stmts = create_function();
        for stmt in &mut stmts[..15] {
            *stmt = Statement::new(0x2C, StatementFamily::NOP, "nop");
        }
        let fvec = FVec::new(stmts, &mut opcode_map, false);
        diff.insert(1, 11, &fvec, None);
        let string_cache = create_string_cache();
        let clones = diff.clones(&string_cache);
        assert_eq!(clones.len(), 0);
    }
}
