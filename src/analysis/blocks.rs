use crate::analysis::BasicBlock;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Hash)]
pub enum BlockType {
    Basic,
    SelfLooping,
    Sequence,
    IfThen,
    IfThenElse,
    While,
    DoWhile,
}

pub trait AbstractBlock<H: Hasher> {
    fn get_type(&self) -> BlockType;
    fn get_depth(&self) -> u32;
    fn children(&self) -> &[Box<dyn AbstractBlock<H>>];

    fn structural_hash(&self, state: &mut H) {
        self.children()
            .iter()
            .for_each(|x| x.structural_hash(state));
        self.get_type().hash(state);
    }

    fn len(&self) -> usize {
        self.children().len()
    }

    fn is_empty(&self) -> bool {
        self.children().is_empty()
    }
}

impl<H: Hasher> AbstractBlock<H> for BasicBlock {
    fn get_type(&self) -> BlockType {
        BlockType::Basic
    }

    fn get_depth(&self) -> u32 {
        0
    }

    fn children(&self) -> &[Box<dyn AbstractBlock<H>>] {
        &[]
    }
}

pub struct StructureBlock<H: Hasher> {
    block_type: BlockType,
    content: Vec<Box<dyn AbstractBlock<H>>>,
    depth: u32,
}

impl<H: Hasher> StructureBlock<H> {
    pub fn new_self_loop(child: Box<dyn AbstractBlock<H>>) -> Box<StructureBlock<H>> {
        let old_depth = child.get_depth();
        Box::new(StructureBlock {
            block_type: BlockType::SelfLooping,
            content: vec![child],
            depth: old_depth + 1,
        })
    }

    pub fn new_sequence(children: Vec<Box<dyn AbstractBlock<H>>>) -> Box<StructureBlock<H>> {
        let old_depth = children.iter().fold(0, |max, val| max.max(val.get_depth()));
        Box::new(StructureBlock {
            block_type: BlockType::Sequence,
            content: children,
            depth: old_depth + 1,
        })
    }

    pub fn new_if_then(
        ifb: Box<dyn AbstractBlock<H>>,
        thenb: Box<dyn AbstractBlock<H>>,
    ) -> Box<StructureBlock<H>> {
        let children = vec![ifb, thenb];
        let mut block = Self::new_sequence(children);
        block.block_type = BlockType::IfThenElse;
        block
    }

    pub fn new_if_then_else(
        ifb: Box<dyn AbstractBlock<H>>,
        thenb: Box<dyn AbstractBlock<H>>,
        elseb: Box<dyn AbstractBlock<H>>,
    ) -> Box<StructureBlock<H>> {
        let children = vec![ifb, thenb, elseb];
        let mut block = Self::new_sequence(children);
        block.block_type = BlockType::IfThenElse;
        block
    }
}

impl<H: Hasher> AbstractBlock<H> for StructureBlock<H> {
    fn get_type(&self) -> BlockType {
        self.block_type
    }

    fn get_depth(&self) -> u32 {
        self.depth
    }

    fn children(&self) -> &[Box<dyn AbstractBlock<H>>] {
        &self.content.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{AbstractBlock, BasicBlock, StructureBlock};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    fn calculate_hashes<S: AbstractBlock<DefaultHasher>>(a: S, b: S) -> (u64, u64) {
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
        let bb0 = BasicBlock {
            id: 0,
            first: 0,
            last: 0xA,
        };
        let bb1 = BasicBlock {
            id: 1,
            first: 0xA,
            last: 0xC,
        };
        let hashes = calculate_hashes(bb0, bb1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_same_order() {
        let bb = Box::new(BasicBlock {
            id: 0,
            first: 0,
            last: 0,
        });
        let self_loop = StructureBlock::new_self_loop(bb.clone());
        let sequence0 = StructureBlock::new_sequence(vec![self_loop, bb.clone()]);
        let self_loop = StructureBlock::new_self_loop(bb.clone());
        let sequence1 = StructureBlock::new_sequence(vec![self_loop, bb]);
        let hashes = calculate_hashes(*sequence0, *sequence1);
        assert_eq!(hashes.0, hashes.1)
    }

    #[test]
    fn structural_hash_different_order() {
        let bb = Box::new(BasicBlock {
            id: 0,
            first: 0,
            last: 0,
        });
        let self_loop = StructureBlock::new_self_loop(bb.clone());
        let sequence0 = StructureBlock::new_sequence(vec![self_loop, bb.clone()]);
        let self_loop = StructureBlock::new_self_loop(bb.clone());
        let sequence1 = StructureBlock::new_sequence(vec![bb, self_loop]);
        let hashes = calculate_hashes(*sequence0, *sequence1);
        assert_ne!(hashes.0, hashes.1)
    }
}
