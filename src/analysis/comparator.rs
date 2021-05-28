use crate::analysis::blocks::StructureBlock;
use crate::analysis::CFS;
use fnv::FnvHashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub struct CFSComparator {
    hashes: FnvHashMap<u64, StructureBlock>,
    mindepth: u32,
}

impl CFSComparator {
    pub fn new(baseline: &CFS, mindepth: u32) -> Option<CFSComparator> {
        if let Some(root) = baseline.get_tree() {
            let mut stack = vec![root];
            let mut map = FnvHashMap::default();
            while let Some(node) = stack.pop() {
                if node.get_depth() >= mindepth {
                    let mut hasher = DefaultHasher::new();
                    node.structural_hash(&mut hasher);
                    let hash = hasher.finish();
                    map.insert(hash, node.clone());
                    let mut children = node.children().iter().cloned().collect::<Vec<_>>();
                    stack.append(&mut children)
                }
            }
            Some(CFSComparator {
                hashes: map,
                mindepth,
            })
        } else {
            None
        }
    }

    pub fn compare(&self, other: &CFS) -> Option<Vec<(StructureBlock, StructureBlock)>> {
        if let Some(root) = other.get_tree() {
            let mut stack = vec![root];
            let mut ret = Vec::new();
            while let Some(node) = stack.pop() {
                if node.get_depth() >= self.mindepth {
                    let mut hasher = DefaultHasher::new();
                    node.structural_hash(&mut hasher);
                    let hash = hasher.finish();
                    if let Some(original) = self.hashes.get(&hash) {
                        if original.structural_equality(&node) {
                            ret.push((original.clone(), node.clone()));
                        }
                    }
                    let mut children = node.children().iter().cloned().collect::<Vec<_>>();
                    stack.append(&mut children)
                }
            }
            Some(ret)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

}
