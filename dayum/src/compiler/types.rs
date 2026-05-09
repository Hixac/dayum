use std::collections::HashMap;

const MAX_INSTRUCTIONS: usize = 65_535;

#[derive(Debug)]
pub enum OpCode {
    AddInt, SubInt, MulInt, DivInt,
    AddFloat, SubFloat, MulFloat, DivFloat,
    AddString,
    NotBool,
    LoadInt, LoatFloat, LoadString, LoadBool, LoadPtr, LoatNull,
    StoreName, LoadName,
    Stop
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i32),
    Float(f32),
    Str(String),
    Bool(bool),
    Ptr(usize)
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: u16
}

#[derive(Default, Debug)]
pub struct Chunk {
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<Value>,
    pub(crate) overflow: bool,

    name_count: u16,
    pub(crate) name_indices: HashMap<String, u16>,
}

impl Chunk {
    pub fn emit(&mut self, opcode: OpCode, operand: u16) -> () {
        if self.instructions.len() > MAX_INSTRUCTIONS {
            self.overflow = true;
            return
        }
        self.instructions.push(Instruction { opcode, operand })
    }

    pub fn push_constant(&mut self, value: Value) -> u16 {
        if self.constants.len() >= u16::MAX as usize {
            return 0
        }
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn push_name(&mut self, name: String) -> u16 {
        if let Some(&idx) = self.name_indices.get(&name) { return idx; }

        self.name_indices.insert(name, self.name_count);
        self.name_count += 1;

        self.name_count - 1
    }
}
