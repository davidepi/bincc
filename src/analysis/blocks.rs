use crate::analysis::BasicBlock;
use std::hash::Hash;
use std::rc::Rc;
use std::collections::hash_map::DefaultHasher;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub enum BlockType {
    Basic,
    SelfLooping,
    Sequence,
    IfThen,
    IfThenElse,
    While,
    DoWhile,
}

#[derive(PartialEq, Eq, Hash)]
pub struct NestedBlock {
    block_type: BlockType,
    content: Vec<Rc<StructureBlock>>,
    depth: u32,
}

impl NestedBlock {
    pub fn new_self_loop(child: Rc<StructureBlock>) -> NestedBlock {
        let old_depth = child.get_depth();
        NestedBlock {
            block_type: BlockType::SelfLooping,
            content: vec![child],
            depth: old_depth + 1,
        }
    }

    pub fn new_sequence(children: Vec<Rc<StructureBlock>>) -> NestedBlock {
        let old_depth = children.iter().fold(0, |max, val| max.max(val.get_depth()));
        NestedBlock {
            block_type: BlockType::Sequence,
            content: children,
            depth: old_depth + 1,
        }
    }

    pub fn new_if_then(
        ifb: Rc<StructureBlock>,
        thenb: Rc<StructureBlock>,
    ) -> NestedBlock {
        let children = vec![ifb, thenb];
        let mut block = Self::new_sequence(children);
        block.block_type = BlockType::IfThenElse;
        block
    }

    pub fn new_if_then_else(
        ifb: Rc<StructureBlock>,
        thenb: Rc<StructureBlock>,
        elseb: Rc<StructureBlock>,
    ) -> NestedBlock {
        let children = vec![ifb, thenb, elseb];
        let mut block = Self::new_sequence(children);
        block.block_type = BlockType::IfThenElse;
        block
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum StructureBlock {
    Basic(BasicBlock),
    Nested(NestedBlock),
}

impl StructureBlock {
    pub fn get_type(&self) -> BlockType {
        match self {
            StructureBlock::Basic(_) => BlockType::Basic,
            StructureBlock::Nested(nb) => nb.block_type
        }
    }

    pub fn get_depth(&self) -> u32 {
        match self {
            StructureBlock::Basic(_) => 0,
            StructureBlock::Nested(nb) => nb.depth,
        }
    }

    pub fn children(&self) -> &[Rc<StructureBlock>] {
        match self {
            StructureBlock::Basic(_) => &[],
            StructureBlock::Nested(nb) => nb.content.as_slice(),
        }
    }

    pub fn structural_hash(&self, state: &mut DefaultHasher) {
        self.children()
            .iter()
            .for_each(|x| x.structural_hash(state));
        self.get_type().hash(state);
    }

    pub fn len(&self) -> usize {
        self.children().len()
    }

    pub fn is_empty(&self) -> bool {
        self.children().is_empty()
    }
}

impl From<BasicBlock> for StructureBlock {
    fn from(bb: BasicBlock) -> Self {
        StructureBlock::Basic(bb)
    }
}

impl From<NestedBlock> for StructureBlock {
    fn from(nb: NestedBlock) -> Self {
        StructureBlock::Nested(nb)
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{BasicBlock, NestedBlock};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use std::rc::Rc;
    use crate::analysis::blocks::StructureBlock;

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
    fn structural_hash_different_id() {
        let bb0 = StructureBlock::from(BasicBlock {
            id: 0,
            first: 0,
            last: 0xA,
        });
        let bb1 = StructureBlock::from(BasicBlock {
            id: 1,
            first: 0xA,
            last: 0xC,
        });
        let hashes = calculate_hashes(bb0, bb1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_same_order() {
        let bb = Rc::new(StructureBlock::from(BasicBlock {
            id: 0,
            first: 0,
            last: 0,
        }));
        let self_loop = Rc::new(StructureBlock::from(NestedBlock::new_self_loop(bb.clone())));
        let sequence0 = StructureBlock::from(NestedBlock::new_sequence(vec![self_loop.clone(), bb.clone()]));
        let self_loop = Rc::new(StructureBlock::from(NestedBlock::new_self_loop(bb.clone())));
        let sequence1 = StructureBlock::from(NestedBlock::new_sequence(vec![self_loop.clone(), bb]));
        let hashes = calculate_hashes(sequence0, sequence1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_different_order() {
        let bb = Rc::new(StructureBlock::from(BasicBlock {
            id: 0,
            first: 0,
            last: 0,
        }));
        let self_loop = Rc::new(StructureBlock::from(NestedBlock::new_self_loop(bb.clone())));
        let sequence0 = StructureBlock::from(NestedBlock::new_sequence(vec![self_loop, bb.clone()]));
        let self_loop = Rc::new(StructureBlock::from(NestedBlock::new_self_loop(bb.clone())));
        let sequence1 = StructureBlock::from(NestedBlock::new_sequence(vec![bb, self_loop]));
        let hashes = calculate_hashes(sequence0, sequence1);
        assert_ne!(hashes.0, hashes.1)
    }
}
