use crate::{types::MIRType, value::Value};

#[derive(Debug)]
pub struct DefineInst {
    type_: MIRType,
    value: Value,
}

impl DefineInst {
    pub fn new(type_: MIRType, value: Value) -> Self {
        DefineInst { type_, value }
    }

    pub fn get_type(&self) -> MIRType {
        self.type_
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug)]
pub struct AssignInst {
    dest: Value,
    src: Value,
}

impl AssignInst {
    pub fn new(dest: Value, src: Value) -> Self {
        AssignInst { dest, src }
    }

    pub fn get_dest(&self) -> &Value {
        &self.dest
    }

    pub fn get_src(&self) -> &Value {
        &self.src
    }
}

#[derive(Debug)]
pub enum Instruction {
    Define(DefineInst),
    Assign(AssignInst),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstId(pub usize);
