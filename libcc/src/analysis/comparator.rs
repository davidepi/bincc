use crate::analysis::blocks::StructureBlock;
use crate::analysis::CFS;
use fnv::FnvHashMap;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClonePair {
    a: StructureBlock,
    aname: String,
    b: StructureBlock,
    bname: String,
}

impl ClonePair {
    pub fn new(a: StructureBlock, a_name: &str, b: StructureBlock, b_name: &str) -> ClonePair {
        ClonePair {
            a,
            b,
            aname: a_name.to_string(),
            bname: b_name.to_string(),
        }
    }

    pub fn first(&self) -> (&str, &StructureBlock) {
        (&self.aname, &self.a)
    }

    pub fn first_name(&self) -> &str {
        &self.aname
    }

    pub fn first_tree(&self) -> &StructureBlock {
        &self.a
    }

    pub fn second(&self) -> (&str, &StructureBlock) {
        (&self.bname, &self.b)
    }

    pub fn second_name(&self) -> &str {
        &self.bname
    }

    pub fn second_tree(&self) -> &StructureBlock {
        &self.b
    }

    pub fn depth(&self) -> u32 {
        self.a.depth() // b should be same depth
    }
}

pub struct CFSComparator {
    hashes: FnvHashMap<u64, StructureBlock>,
    names: HashMap<StructureBlock, String>,
    mindepth: u32,
}

impl CFSComparator {
    pub fn new(mindepth: u32) -> CFSComparator {
        CFSComparator {
            hashes: FnvHashMap::default(),
            names: HashMap::new(),
            mindepth,
        }
    }

    pub fn insert(&mut self, other: &CFS, identifier: &str) -> Option<Vec<ClonePair>> {
        if let Some(root) = other.get_tree() {
            let mut stack = vec![root];
            let mut ret = Vec::new();
            while let Some(node) = stack.pop() {
                if node.depth() >= self.mindepth {
                    let mut hasher = DefaultHasher::new();
                    node.structural_hash(&mut hasher);
                    let hash = hasher.finish();
                    if let Some(original) = self.hashes.get(&hash) {
                        if original.structural_equality(&node) {
                            // pick only a_name: b is never inserted in the hashmap because it's the
                            // clone! so its name is used only in this pair and never recorded
                            let a_name = self.names.get(original).unwrap().as_str();
                            let pair =
                                ClonePair::new(original.clone(), a_name, node.clone(), identifier);
                            ret.push(pair);
                        } else {
                            log::warn!("Same structural hash but different structure.");
                        }
                    } else {
                        self.hashes.insert(hash, node.clone());
                        self.names.insert(node.clone(), identifier.to_string());
                    }
                    let mut children = node.children().to_vec();
                    stack.append(&mut children)
                }
            }
            if !ret.is_empty() {
                ret = remove_overlapping(ret);
            }
            Some(ret)
        } else {
            None
        }
    }
}

fn remove_overlapping(mut clone_list: Vec<ClonePair>) -> Vec<ClonePair> {
    // drop intervals that are contained inside each other
    // partially overlapping intervals can not exists (can't think of an example)
    // probably not efficient O(n^2)? but I don't expect a big list here and deadline
    // is close
    let mut todo = clone_list.clone();
    // this minimizes the number of comparisons (sorting is nlogn, the removal is n^2)
    todo.sort_unstable_by_key(|a| std::cmp::Reverse(a.depth()));
    let mut removed = HashSet::new();
    while !todo.is_empty() {
        let current = todo.pop().unwrap();
        let mut keep = Vec::with_capacity(clone_list.len());
        while let Some(compare) = clone_list.pop() {
            if !removed.contains(&compare) {
                if current.depth() != compare.depth()
                    && current.a.starting_offset() <= compare.a.starting_offset()
                    && current.b.starting_offset() <= compare.b.starting_offset()
                    && current.a.ending_offset() >= compare.a.ending_offset()
                    && current.b.ending_offset() >= compare.b.ending_offset()
                {
                    // not same depth (otherwise I remove myself),
                    // and one is contained inside the other one
                    removed.insert(compare);
                } else {
                    keep.push(compare);
                }
            }
        }
        clone_list = keep;
    }
    clone_list
}

#[cfg(test)]
mod tests {
    use crate::analysis::{CFSComparator, CFG, CFS, SINK_ADDR};
    use crate::disasm::{ArchX86, Statement};

    fn create_function() -> Vec<Statement> {
        vec![
            Statement::new(0x00, "test eax, eax"),
            Statement::new(0x04, "jg 0x38"),
            Statement::new(0x08, "add ebx, 5"),
            Statement::new(0x0C, "jmp 0x10"),
            Statement::new(0x10, "cmp eax, ebx"),
            Statement::new(0x14, "jne 0x20"),
            Statement::new(0x18, "cmp ebx, 5"),
            Statement::new(0x1C, "jne 0x18"),
            Statement::new(0x20, "mov ecx, [ebp+8]"),
            Statement::new(0x24, "jmp 0x28"),
            Statement::new(0x28, "cmp ecx, eax"),
            Statement::new(0x2C, "mov eax, -1"),
            Statement::new(0x30, "jne 0x08"),
            Statement::new(0x34, "ret"),
            Statement::new(0x38, "incl eax"),
            Statement::new(0x3C, "mov ebx, [ebp+20]"),
            Statement::new(0x40, "cmp eax, ebx"),
            Statement::new(0x44, "je 0x58"),
            Statement::new(0x48, "mov ecx, [ebp+20]"),
            Statement::new(0x4C, "decl ecx"),
            Statement::new(0x50, "mov [ebp+20], ecx"),
            Statement::new(0x54, "jmp 0x38"),
            Statement::new(0x58, "test eax, eax"),
            Statement::new(0x5C, "mov eax, 0"),
            Statement::new(0x60, "je 0x68"),
            Statement::new(0x64, "mov eax, 1"),
            Statement::new(0x68, "ret"),
        ]
    }

    #[test]
    fn cloned_failed_cfs() {
        let stmts = vec![
            Statement::new(0x0, "jge 0x2"),
            Statement::new(0x1, "xor eax, eax"),
            Statement::new(0x2, "jmp 0x1"),
        ];
        let cfg = CFG::new(&stmts, &ArchX86::new_amd64()).add_sink();
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
        let mut diff = CFSComparator::new(5);
        let first = diff.insert(&cfs, "failed");
        assert!(first.is_none());
    }

    #[test]
    fn cloned_full() {
        let stmts = create_function();
        let cfg = CFG::new(&stmts, &ArchX86::new_amd64()).add_sink();
        let cfs = CFS::new(&cfg);
        let mut diff = CFSComparator::new(5);
        let first = diff.insert(&cfs, "first");
        assert!(first.is_some());
        assert!(first.unwrap().is_empty());
        let second = diff.insert(&cfs, "other");
        assert!(second.is_some());
        let s = second.unwrap();
        assert_eq!(s.len(), 1);
        let pair = s.last().unwrap();
        assert_eq!(pair.first_tree().starting_offset(), 0);
        assert_eq!(pair.first_tree().ending_offset(), SINK_ADDR);
        assert_eq!(s[0].first_name(), "first");
        assert_eq!(s[0].second_name(), "other");
    }

    #[test]
    fn cloned_partial() {
        let mut stmts = create_function();
        let cfg0 = CFG::new(&stmts, &ArchX86::new_amd64()).add_sink();
        let cfs0 = CFS::new(&cfg0);
        let mut diff = CFSComparator::new(2);
        let first = diff.insert(&cfs0, "first");
        assert!(first.is_some());
        assert!(first.unwrap().is_empty());
        stmts = create_function();
        stmts[2] = Statement::new(0x08, "nop");
        stmts[3] = Statement::new(0x0C, "nop");
        stmts[10] = Statement::new(0x28, "nop");
        stmts[11] = Statement::new(0x2C, "nop");
        stmts[12] = Statement::new(0x30, "nop");
        let cfg1 = CFG::new(&stmts, &ArchX86::new_amd64()).add_sink();
        let cfs1 = CFS::new(&cfg1);
        let second = diff.insert(&cfs1, "second");
        assert!(second.is_some());
        let s = second.unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].first_name(), "first");
        assert_eq!(s[0].second_name(), "second");
    }
}
