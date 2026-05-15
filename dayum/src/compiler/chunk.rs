use std::collections::HashMap;


const MAX_INSTRUCTIONS: usize = 65_535;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    AddInt, SubInt, MulInt, DivInt,
    AddFloat, SubFloat, MulFloat, DivFloat,
    AddString,
    AndBool, OrBool, NotBool,
    LoadInt, LoadFloat, LoadString, LoadBool, LoadPtr, LoadNull,
    DefineGlobal, StoreGlobal, LoadGlobal,
    Call,
    AllocPtr,
    StoreName, LoadName,
    Return,
    Stop,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i32),
    Float(f32),
    Str(String),
    Bool(bool),
    Func(CompiledFunction),
    Ptr(usize)
}

impl Value {
    pub fn add_numeric(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Int(v1), Int(v2)) => Value::Int(v1 + v2),
            (Float(v1), Float(v2)) => Value::Float(v1 + v2),
            _ => panic!("Non-numeric")
        }
    }

    pub fn sub_numeric(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Int(v1), Int(v2)) => Value::Int(v1 - v2),
            (Float(v1), Float(v2)) => Value::Float(v1 - v2),
            _ => panic!("Non-numeric")
        }
    }

    pub fn mul_numeric(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Int(v1), Int(v2)) => Value::Int(v1 * v2),
            (Float(v1), Float(v2)) => Value::Float(v1 * v2),
            _ => panic!("Non-numeric")
        }
    }

    pub fn div_numeric(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Int(v1), Int(v2)) => Value::Int(v1 / v2),
            (Float(v1), Float(v2)) => Value::Float(v1 / v2),
            _ => panic!("Non-numeric")
        }
    }

    pub fn and_logical(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 && v2),
            _ => panic!("Non-boolean")
        }
    }

    pub fn or_logical(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 || v2),
            _ => panic!("Non-boolean")
        }
    }

    pub fn concatenate(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Str(v1), Str(v2)) => Value::Str(format!("{}{}", v1, v2)),
            _ => panic!("Non-string")
        }
    }

    pub fn not(self) -> Value {
        use Value::*;
        match self {
            Bool(v) => Value::Bool(!v),
            _ => panic!("Non-boolean")
        }
    }

    pub fn eq(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 == *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn neq(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 != *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn lt(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 < *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn le(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 <= *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn gt(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 > *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn ge(self, other: &Value) -> Value {
        use Value::*;
        match (self, other) {
            (Bool(v1), Bool(v2)) => Value::Bool(v1 >= *v2),
            _ => panic!("Non-string")
        }
    }

    pub fn as_func(self) -> CompiledFunction {
        use Value::*;
        match self {
            Func(v) => v,
            _ => panic!("Non-func")
        }
    }

    pub fn as_ptr(self) -> usize {
        use Value::*;
        match self {
            Ptr(v) => v,
            _ => panic!("Non-func")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: u16
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub arity: u16,
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub depth: u16,
}

#[derive(Debug)]
pub struct BuiltinCall {
    pub name: String,
    pub arity: u16,
}

#[derive(Default, Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub names: Vec<Variable>,
    pub name_indices: HashMap<String, u16>,
    pub overflow: bool,
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

    pub fn push_name(&mut self, name: String, depth: u16) -> u16 {
        if let Some(&idx) = self.name_indices.get(&name) { return idx; }

        self.name_indices.insert(name.clone(), self.names.len() as u16);
        self.names.push(Variable { name, depth });

        (self.names.len() - 1) as u16
    }
}
