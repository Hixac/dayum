
const MAX_INSTRUCTIONS: usize = 65_535;

#[derive(Debug)]
pub enum OpCode {
    Add, Sub, Mul, Div, Not,
    LoadConst,
    Stop
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f32),
    Str(String)
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
}
