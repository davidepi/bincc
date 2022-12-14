use crate::analysis::BasicBlock;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::sync::Arc;

/// High-level structure label assigned to a [`NestedBlock`].
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

/// A group of [`StructureBlock`] with the same [`BlockType`] label.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NestedBlock {
    pub(crate) offset: u64,
    pub(crate) block_type: BlockType,
    pub(crate) content: Vec<StructureBlock>,
    pub(crate) depth: u32,
}

impl NestedBlock {
    /// Creates a new nested block with the given label and children.
    pub fn new(label: BlockType, children: Vec<StructureBlock>) -> NestedBlock {
        let old_depth = children.iter().fold(0, |max, val| max.max(val.depth()));
        let offset = children
            .iter()
            .fold(u64::MAX, |min, val| min.min(val.offset()));
        NestedBlock {
            offset,
            block_type: label,
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

/// Contains either a [`BasicBlock`] or a [`NestedBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructureBlock {
    Basic(BasicBlock),
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
    /// Returns the label of this block.
    pub fn block_type(&self) -> BlockType {
        match self {
            StructureBlock::Basic(_) => BlockType::Basic,
            StructureBlock::Nested(nb) => nb.block_type,
        }
    }

    /// Returns the amount of nested structures in this block.
    ///
    /// If this block contains only basic blocks, 0 is returned.
    pub fn depth(&self) -> u32 {
        match self {
            StructureBlock::Basic(_) => 0,
            StructureBlock::Nested(nb) => nb.depth,
        }
    }

    /// Returns the children of this block.
    pub fn children(&self) -> &[StructureBlock] {
        match self {
            StructureBlock::Basic(_) => &[],
            StructureBlock::Nested(nb) => nb.content.as_slice(),
        }
    }

    /// Calculate a unique hash for this block that does not account for basic block offsets.
    pub fn structural_hash(&self, state: &mut DefaultHasher) {
        self.children()
            .iter()
            .for_each(|x| x.structural_hash(state));
        self.block_type().hash(state);
    }

    /// Checks if two blocks have the same structure (does not check for basic blocks equality).
    pub fn structural_equality(&self, b: &StructureBlock) -> bool {
        if self.block_type() == b.block_type() {
            let children_a = self.children();
            let children_b = b.children();
            if children_a.is_empty() && children_b.is_empty() {
                true //basic block
            } else if children_a.len() == children_b.len() {
                children_a
                    .iter()
                    .zip(children_b.iter())
                    .fold(true, |acc, (a, b)| acc & a.structural_equality(b))
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns the amount of children in this block.
    pub fn len(&self) -> usize {
        self.children().len()
    }

    /// Returns true if this block has no children.
    pub fn is_empty(&self) -> bool {
        self.children().is_empty()
    }

    /// Returns a string representing this block type.
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

    /// Returns the offset of the first basic block contained in this cluster.
    pub fn offset(&self) -> u64 {
        match self {
            StructureBlock::Basic(bb) => bb.offset,
            StructureBlock::Nested(nb) => nb.offset,
        }
    }

    /// Returns the list of basic blocks contained in this cluster, ordered by offset.
    pub fn basic_blocks(&self) -> Vec<BasicBlock> {
        let mut retval = Vec::new();
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if let StructureBlock::Basic(bb) = node {
                retval.push(*bb);
            } else {
                stack.extend(node.children());
            }
        }
        retval.sort_unstable();
        retval
    }
}

impl From<BasicBlock> for StructureBlock {
    fn from(bb: BasicBlock) -> Self {
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
        let bb = BasicBlock {
            offset: 1,
            length: 0xA,
        };
        let sb0 = StructureBlock::from(bb);
        let sb1 = StructureBlock::from(bb);
        assert_eq!(sb0, sb1);
    }

    #[test]
    fn structure_block_structural_equality() {
        // two basic blocks with different content, but structural equality should be the same
        let bb0 = BasicBlock {
            offset: 1,
            length: 0xA,
        };
        let bb1 = BasicBlock {
            offset: 1,
            length: 0xB,
        };
        let sb0 = StructureBlock::from(bb0);
        let sb1 = StructureBlock::from(bb1);
        assert_ne!(sb0, sb1);
        assert!(sb0.structural_equality(&sb1));
    }

    #[test]
    fn structural_hash_different_id() {
        let bb0 = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 0xA,
        });
        let bb1 = StructureBlock::from(BasicBlock {
            offset: 0xA,
            length: 0xC,
        });
        let hashes = calculate_hashes(bb0, bb1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_same_order() {
        let bb = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 1,
        });
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
        let bb = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 1,
        });
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
        let bb = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 1,
        });
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
        let bb = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 1,
        });
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

    #[test]
    fn retrieve_basic_blocks_from_structure_block() {
        let bb0 = StructureBlock::from(BasicBlock {
            offset: 1,
            length: 1,
        });
        let bb1 = StructureBlock::from(BasicBlock {
            offset: 10,
            length: 1,
        });
        let bb2 = StructureBlock::from(BasicBlock {
            offset: 100,
            length: 1,
        });
        let bb3 = StructureBlock::from(BasicBlock {
            offset: 1000,
            length: 1,
        });
        let ifelse = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::IfThenElse,
            vec![bb1, bb2, bb3],
        )));
        let sequence = StructureBlock::from(Arc::new(NestedBlock::new(
            BlockType::Sequence,
            vec![bb0, ifelse],
        )));
        let bbs = sequence.basic_blocks();
        assert_eq!(bbs[0].offset, 1);
        assert_eq!(bbs[1].offset, 10);
        assert_eq!(bbs[2].offset, 100);
        assert_eq!(bbs[3].offset, 1000);
    }
}
