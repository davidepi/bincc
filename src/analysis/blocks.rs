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
    fn get_id(&self) -> usize;
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
    fn get_id(&self) -> usize {
        self.id
    }

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
    id: usize,
    block_type: BlockType,
    content: Vec<Box<dyn AbstractBlock<H>>>,
    depth: u32,
}

impl<H: Hasher> AbstractBlock<H> for StructureBlock<H> {
    fn get_id(&self) -> usize {
        self.id
    }

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
