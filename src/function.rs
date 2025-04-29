use std::ops::Range;

use crate::{instruction::{InstId, Instruction}, types::MIRType};

#[derive(Debug, Clone, Copy)]
pub struct BlockId(pub usize);

#[derive(Debug)]
pub struct Block<'ctx> {
    name: &'ctx str,
    range: Range<InstId>,
}

impl<'ctx> Block<'ctx> {
    pub fn new(name: &'ctx str, start: InstId) -> Self {
        Block {
            name,
            range: start..start,
        }
    }

    pub fn adjust_range(&mut self, inst: InstId) {
        if inst.0 > self.range.end.0 {
            self.range.end = InstId(inst.0 + 1);
        }
    }

    pub fn get_name(&self) -> &'ctx str {
        self.name
    }

    pub fn get_range(&self) -> Range<InstId> {
        self.range.clone()
    }

    pub fn get_instructions<'f>(&self, func: &'f Function<'ctx>) -> &'f [Instruction] {
        let start = self.range.start.0;
        let end = self.range.end.0;
        &func.instructions[start..end]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FuncId(pub usize);

#[derive(Debug)]
pub struct Function<'ctx> {
    name: &'ctx str,
    ret_type: MIRType,
    instructions: Vec<Instruction>,
    blocks: Vec<Block<'ctx>>,
    inst_id: usize,
}

impl<'ctx> Function<'ctx> {
    pub fn new(name: &'ctx str, ret_type: MIRType) -> Self {
        Function {
            name,
            ret_type,
            instructions: Vec::new(),
            blocks: Vec::new(),
            inst_id: 0,
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> InstId {
        let inst_id = InstId(self.inst_id);
        self.instructions.push(instruction);
        self.inst_id += 1;
        inst_id
    }

    pub fn add_block(&mut self, block: Block<'ctx>) -> BlockId {
        let block_id = BlockId(self.blocks.len());
        self.blocks.push(block);
        block_id
    }

    pub fn last_block(&self) -> Option<BlockId> {
        if self.blocks.is_empty() {
            None
        } else {
            Some(BlockId(self.blocks.len() - 1))
        }
    }

    pub fn get_name(&self) -> &'ctx str {
        self.name
    }

    pub fn get_instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn get_blocks(&self) -> &[Block<'ctx>] {
        &self.blocks
    }

    pub fn get_block(&self, id: BlockId) -> Option<&Block<'ctx>> {
        self.blocks.get(id.0)
    }

    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut Block<'ctx>> {
        self.blocks.get_mut(id.0)
    }

    pub fn get_ret_type(&self) -> MIRType {
        self.ret_type
    }
}
