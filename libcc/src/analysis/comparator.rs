use crate::analysis::blocks::StructureBlock;
use fnv::FnvHashMap;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClonePair {
    a: StructureBlock,
    a_bin: String,
    a_fun: String,
    b: StructureBlock,
    b_bin: String,
    b_fun: String,
}

impl ClonePair {
    pub fn new(
        a: StructureBlock,
        a_bin: String,
        a_fun: String,
        b: StructureBlock,
        b_bin: String,
        b_fun: String,
    ) -> Self {
        ClonePair {
            a,
            a_bin,
            a_fun,
            b,
            b_bin,
            b_fun,
        }
    }

    pub fn first_bin(&self) -> &str {
        &self.a_bin
    }

    pub fn first_fun(&self) -> &str {
        &self.a_fun
    }

    pub fn first_tree(&self) -> &StructureBlock {
        &self.a
    }

    pub fn second_bin(&self) -> &str {
        &self.b_bin
    }

    pub fn second_fun(&self) -> &str {
        &self.b_fun
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
    names: HashMap<StructureBlock, (String, String)>,
    // used to store the strings for the ClonePairs (that uses references)
    mindepth: u32,
}

impl CFSComparator {
    pub fn new(mindepth: u32) -> Self {
        CFSComparator {
            hashes: FnvHashMap::default(),
            names: HashMap::new(),
            mindepth,
        }
    }

    pub fn compare_and_insert(
        &mut self,
        other: StructureBlock,
        bin_name: String,
        func_name: String,
    ) -> Vec<ClonePair> {
        let mut stack = vec![other];
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
                        let (orig_bin_name, orig_func_name) = self.names.get(original).unwrap();
                        let pair = ClonePair::new(
                            original.clone(),
                            orig_bin_name.clone(),
                            orig_func_name.clone(),
                            node.clone(),
                            bin_name.clone(),
                            func_name.clone(),
                        );
                        ret.push(pair);
                    } else {
                        log::warn!("Same structural hash but different structure.");
                    }
                } else {
                    self.hashes.insert(hash, node.clone());
                    self.names
                        .insert(node.clone(), (bin_name.clone(), func_name.clone()));
                }
                stack.extend(node.children().iter().cloned());
            }
        }
        if !ret.is_empty() {
            ret = remove_overlapping(ret);
        }
        ret
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
                if current.depth() != compare.depth() && overlaps(&current, &compare) {
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

fn overlaps(a: &ClonePair, b: &ClonePair) -> bool {
    let mut retval = false;
    if a.depth() >= b.depth()
        && a.a_bin == b.a_bin
        && a.a_fun == b.a_fun
        && a.b_bin == b.b_bin
        && a.b_fun == b.b_fun
    {
        let mut first_ok = false;
        let mut second_ok = false;
        let mut stack = a.a.children().iter().collect::<Vec<_>>();
        while let Some(child) = stack.pop() {
            if child.offset() == b.a.offset() {
                first_ok = true;
                break;
            } else {
                stack.extend(child.children());
            }
        }
        if first_ok {
            stack.clear();
            stack.extend(a.b.children());
            while let Some(child) = stack.pop() {
                if child.offset() == b.b.offset() {
                    second_ok = true;
                    break;
                }
            }
        }
        retval = first_ok && second_ok;
    }
    retval
}

#[cfg(test)]
mod tests {
    use crate::analysis::{CFSComparator, CFG, CFS};
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
    fn cloned_full() {
        let stmts = create_function();
        let cfg = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs = CFS::new(&cfg);
        let mut diff = CFSComparator::new(3);
        let first = diff.compare_and_insert(
            cfs.get_tree().unwrap(),
            "first_bin".to_string(),
            "first_func".to_string(),
        );
        assert!(first.is_empty());
        let second = diff.compare_and_insert(
            cfs.get_tree().unwrap(),
            "other_bin".to_string(),
            "other_func".to_string(),
        );
        assert!(!second.is_empty());
        assert_eq!(second.len(), 1);
        let pair = second.last().unwrap();
        assert_eq!(pair.first_tree().offset(), 0);
        assert_eq!(second[0].first_bin(), "first_bin");
        assert_eq!(second[0].first_fun(), "first_func");
        assert_eq!(second[0].second_bin(), "other_bin");
        assert_eq!(second[0].second_fun(), "other_func");
    }

    #[test]
    fn cloned_partial() {
        let mut stmts = create_function();
        let cfg0 = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs0 = CFS::new(&cfg0);
        let mut diff = CFSComparator::new(2);
        let first = diff.compare_and_insert(
            cfs0.get_tree().unwrap(),
            "first_bin".to_string(),
            "first_func".to_string(),
        );
        assert!(first.is_empty());
        stmts = create_function();
        stmts[2] = Statement::new(0x08, "nop");
        stmts[3] = Statement::new(0x0C, "nop");
        stmts[10] = Statement::new(0x28, "nop");
        stmts[11] = Statement::new(0x2C, "nop");
        stmts[12] = Statement::new(0x30, "nop");
        let cfg1 = CFG::new(&stmts, 0x6C, &ArchX86::new_amd64()).add_sink();
        let cfs1 = CFS::new(&cfg1);
        let second = diff.compare_and_insert(
            cfs1.get_tree().unwrap(),
            "second_bin".to_string(),
            "second_func".to_string(),
        );
        assert!(!second.is_empty());
        assert_eq!(second.len(), 2);
        assert_eq!(second[0].first_bin(), "first_bin");
        assert_eq!(second[0].first_fun(), "first_func");
        assert_eq!(second[0].second_bin(), "second_bin");
        assert_eq!(second[0].second_fun(), "second_func");
    }
}
