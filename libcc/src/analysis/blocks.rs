use crate::analysis::BasicBlock;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum BlockType {
    Basic,
    SelfLooping,
    Sequence,
    IfThen,
    IfThenElse,
    While,
    DoWhile,
    Switch,
    ProperInterval,
    ImproperInterval,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::Basic => write!(f, "Basic Block"),
            BlockType::SelfLooping => write!(f, "Self Loop"),
            BlockType::Sequence => write!(f, "Sequence"),
            BlockType::IfThen => write!(f, "If-Then"),
            BlockType::IfThenElse => write!(f, "If-Then-Else"),
            BlockType::While => write!(f, "While"),
            BlockType::DoWhile => write!(f, "Do-While"),
            BlockType::Switch => write!(f, "Switch"),
            BlockType::ProperInterval => write!(f, "Proper Interval"),
            BlockType::ImproperInterval => write!(f, "Improper Interval"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NestedBlock {
    pub(crate) offset: u64,
    pub(crate) block_type: BlockType,
    pub(crate) content: Vec<StructureBlock>,
    pub(crate) depth: u32,
}

impl NestedBlock {
    pub fn new(bt: BlockType, children: Vec<StructureBlock>) -> NestedBlock {
        let old_depth = children.iter().fold(0, |max, val| max.max(val.depth()));
        let offset = children
            .iter()
            .fold(u64::MAX, |min, val| min.min(val.offset()));
        NestedBlock {
            offset,
            block_type: bt,
            content: children,
            depth: old_depth + 1,
        }
    }
}

impl Display for NestedBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.block_type, self.offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructureBlock {
    Basic(Arc<BasicBlock>),
    Nested(Arc<NestedBlock>),
}

impl Display for StructureBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StructureBlock::Basic(bb) => write!(f, "{}", bb),
            StructureBlock::Nested(n) => write!(f, "{}", n),
        }
    }
}

impl StructureBlock {
    pub fn block_type(&self) -> BlockType {
        match self {
            StructureBlock::Basic(_) => BlockType::Basic,
            StructureBlock::Nested(nb) => nb.block_type,
        }
    }

    pub fn depth(&self) -> u32 {
        match self {
            StructureBlock::Basic(_) => 0,
            StructureBlock::Nested(nb) => nb.depth,
        }
    }

    pub fn children(&self) -> &[StructureBlock] {
        match self {
            StructureBlock::Basic(_) => &[],
            StructureBlock::Nested(nb) => nb.content.as_slice(),
        }
    }

    pub fn structural_hash(&self, state: &mut DefaultHasher) {
        self.children()
            .iter()
            .for_each(|x| x.structural_hash(state));
        self.block_type().hash(state);
    }

    pub fn structural_equality(&self, b: &StructureBlock) -> bool {
        if self.block_type() == b.block_type() {
            let children_a = self.children();
            let children_b = b.children();
            if children_a.is_empty() && children_b.is_empty() {
                true //basic block
            } else if children_a.len() == children_b.len() {
                let mut retval = true;
                let mut i = 0;
                while retval && i < children_a.len() {
                    let child_a = &children_a[i];
                    let child_b = &children_b[i];
                    retval &= child_a.structural_equality(child_b);
                    i += 1;
                }
                retval
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.children().len()
    }

    pub fn is_empty(&self) -> bool {
        self.children().is_empty()
    }

    pub fn get_type_name(&self) -> &'static str {
        let bt = match self {
            StructureBlock::Basic(_) => BlockType::Basic,
            StructureBlock::Nested(nb) => nb.block_type,
        };
        match bt {
            BlockType::Basic => "Basic Block",
            BlockType::SelfLooping => "Self Loop",
            BlockType::Sequence => "Sequence",
            BlockType::IfThen => "If-Then",
            BlockType::IfThenElse => "If-Then-Else",
            BlockType::While => "While",
            BlockType::DoWhile => "Do-While",
            BlockType::Switch => "Switch",
            BlockType::ProperInterval => "Proper Interval",
            BlockType::ImproperInterval => "Improper Interval",
        }
    }

    pub fn offset(&self) -> u64 {
        match self {
            StructureBlock::Basic(bb) => bb.offset,
            StructureBlock::Nested(nb) => nb.offset,
        }
    }
}

impl From<Arc<BasicBlock>> for StructureBlock {
    fn from(bb: Arc<BasicBlock>) -> Self {
        StructureBlock::Basic(bb)
    }
}

impl From<Arc<NestedBlock>> for StructureBlock {
    fn from(nb: Arc<NestedBlock>) -> Self {
        StructureBlock::Nested(nb)
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::blocks::StructureBlock;
    use crate::analysis::{BasicBlock, BlockType, NestedBlock};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use std::sync::Arc;

    fn calculate_hashes(a: StructureBlock, b: StructureBlock) -> (u64, u64) {
        let mut hasher0 = DefaultHasher::new();
        a.structural_hash(&mut hasher0);
        let hash0 = hasher0.finish();
        let mut hasher1 = DefaultHasher::new();
        b.structural_hash(&mut hasher1);
        let hash1 = hasher1.finish();
        (hash0, hash1)
    }

    #[test]
    fn structure_block_strong_equality() {
        // checks that despite having two different StructureBlock, if their content is the same Rc
        // equality returns true.
        let bb = Arc::new(BasicBlock {
            offset: 1,
            length: 0xA,
        });
        let sb0 = StructureBlock::from(bb.clone());
        let sb1 = StructureBlock::from(bb);
        assert_eq!(sb0, sb1);
    }

    #[test]
    fn structure_block_structural_equality() {
        // two basic blocks with different content, but structural equality should be the same
        let bb0 = Arc::new(BasicBlock {
            offset: 1,
            length: 0xA,
        });
        let bb1 = Arc::new(BasicBlock {
            offset: 1,
            length: 0xB,
        });
        let sb0 = StructureBlock::from(bb0);
        let sb1 = StructureBlock::from(bb1);
        assert_ne!(sb0, sb1);
        assert!(sb0.structural_equality(&sb1));
    }

    #[test]
    fn structural_hash_different_id() {
        let bb0 = StructureBlock::from(Arc::new(BasicBlock {
            offset: 1,
            length: 0xA,
        }));
        let bb1 = StructureBlock::from(Arc::new(BasicBlock {
            offset: 0xA,
            length: 0xC,
        }));
        let hashes = calculate_hashes(bb0, bb1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_same_order() {
        let bb = StructureBlock::from(Arc::new(BasicBlock {
            offset: 1,
            length: 1,
        }));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence0 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb.clone()],
        )));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence1 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb],
        )));
        let hashes = calculate_hashes(sequence0, sequence1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_equality_same_order() {
        let bb = StructureBlock::from(Arc::new(BasicBlock {
            offset: 1,
            length: 1,
        }));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence0 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb.clone()],
        )));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence1 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb],
        )));
        assert!(sequence0.structural_equality(&sequence1));
    }

    #[test]
    fn structural_hash_different_order() {
        let bb = StructureBlock::from(Arc::new(BasicBlock {
            offset: 1,
            length: 1,
        }));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence0 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb.clone()],
        )));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence1 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![bb, self_loop],
        )));
        let hashes = calculate_hashes(sequence0, sequence1);
        assert_ne!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_equality_different_order() {
        let bb = StructureBlock::from(Arc::new(BasicBlock {
            offset: 1,
            length: 1,
        }));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence0 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![self_loop, bb.clone()],
        )));
        let self_loop = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::SelfLooping,
            vec![bb.clone()],
        )));
        let sequence1 = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![bb, self_loop],
        )));
        assert!(!sequence0.structural_equality(&sequence1));
    }
}
