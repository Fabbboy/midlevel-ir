use crate::instruction::InstId;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Instruction(InstId),
    ConstantInt(i64),
    ConstantFloat(f64),
}