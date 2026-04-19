use anyhow::{Result, Context, bail};

use crate::parser::*;

use super::vm::types::*;

mod types;


pub struct VirtualMachine<'a> {
    stack: Vec<Value>,
    chunk: &'a Chunk
}

impl<'a> VirtualMachine<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        VirtualMachine {
            stack: Vec::with_capacity(256),
            chunk
        }
    }

    pub fn exec(&mut self) -> Result<()> {
        let n = self.chunk.instructions.len();

        let mut ip = 0usize;

        loop {
            if ip >= n { return Ok(()) }

            let ins = &self.chunk.instructions[ip];
            let op = ins.operand;
            ip += 1;

            match ins.opcode {
                OpCode::LoadConst => {
                    let v = self.chunk.constants[op as usize].clone();
                    self.push(v);
                },
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.add(a, b);
                    self.push(v);
                },
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.sub(a, b);
                    self.push(v);
                },
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.mul(a, b);
                    self.push(v);
                },
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.div(a, b);
                    self.push(v);
                },
                OpCode::Not => {
                    let a = self.pop()?;

                },
                OpCode::Stop => {
                    println!("{:?}", self.stack);
                    return Ok(())
                },
            }
        }
    }

    fn add(&self, a: Value, b: Value) -> Value {
        use Value::*;

        match (a, b) {
            (Int(v1), Int(v2)) => Int(v1 + v2),
            (Float(v1), Float(v2)) => Float(v1 + v2),
            (Str(v1), Str(v2)) => Str(format!("{}{}", v1, v2)),
            _ => todo!()  // Throw error or make implicit coerce?
        }
    }

    fn sub(&self, a: Value, b: Value) -> Value {
        use Value::*;

        match (a, b) {
            (Int(v1), Int(v2)) => Int(v1 - v2),
            (Float(v1), Float(v2)) => Float(v1 - v2),
            _ => todo!()  // Throw error or make implicit coerce?
        }
    }

    fn mul(&self, a: Value, b: Value) -> Value {
        use Value::*;

        match (a, b) {
            (Int(v1), Int(v2)) => Int(v1 * v2),
            (Float(v1), Float(v2)) => Float(v1 * v2),
            _ => todo!()  // Throw error or make implicit coerce?
        }
    }

    fn div(&self, a: Value, b: Value) -> Value {
        use Value::*;

        match (a, b) {
            (Int(v1), Int(v2)) => Int(v1 / v2),
            (Float(v1), Float(v2)) => Float(v1 / v2),
            _ => todo!()  // Throw error or make implicit coerce?
        }
    }

    #[inline] pub(crate) fn push(&mut self, v: Value) { self.stack.push(v); }

    #[inline] pub(crate) fn pop(&mut self) -> Result<Value> {
        self.stack.pop().context("Stack underflow!")
    }
}
