use std::collections::HashMap;

use anyhow::{Result, Context, bail};

use crate::compiler::*;

use super::vm::types::*;

mod types;
mod ops;


pub struct VirtualMachine<'a> {
    stack: Vec<Val>,
    chunk: &'a Chunk,
    heap: HeapPool,
    globals: HashMap<String, Val>
}

impl<'a> VirtualMachine<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        VirtualMachine {
            stack: Vec::with_capacity(256),
            chunk,
            heap: HeapPool::new(),
            globals: HashMap::new()
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
                    let v = self.to_val(&v)?;
                    self.push(v);
                },
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.add(a, b)?;
                    self.push(v);
                },
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.sub(a, b)?;
                    self.push(v);
                },
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.mul(a, b)?;
                    self.push(v);
                },
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let v = self.div(a, b)?;
                    self.push(v);
                },
                OpCode::Not => {
                    let a = self.pop()?;

                },
                OpCode::DefineGlobal => {
                    let name = self.pop()?;
                    if name.is_heap() {
                        let obj = self.heap.get(name);
                        if let Obj::Str(name) = obj {
                            self.globals.insert(name.to_string(), Val::none());
                            continue;
                        }
                        bail!("Name is not string");
                    }
                    bail!("Name is not heap allocated");
                }
                OpCode::GetGlobal => {
                    let name = self.pop()?;
                    if name.is_heap() {
                        let obj = self.heap.get(name);
                        if let Obj::Str(name) = obj {
                            let value = self.globals.get(name).context("Not found")?;
                            self.push(value.clone());
                            continue;
                        }
                        bail!("Name is not string");
                    }
                    bail!("Name is not heap allocated");
                }
                OpCode::SetGlobal => {
                    let name = self.pop()?;
                    let val = self.pop()?;
                    if name.is_heap() {
                        let obj = self.heap.get(name);
                        if let Obj::Str(name) = obj {
                            self.globals.insert(name.to_string(), val);
                            continue;
                        }
                        bail!("Name is not string");
                    }
                    bail!("Name is not heap allocated");
                }
                OpCode::Stop => {
                    return Ok(())
                },
            }
            self.print_stack()?;
        }
    }

    fn to_val(&mut self, v: &Value) -> Result<Val> {
        Ok(match v {
            Value::Int(i) => Val::int(*i as i64),
            Value::Float(f) => Val::float(*f as f64),
            Value::Bool(b) => Val::bool(*b),
            Value::Str(s) => self.heap.alloc(Obj::Str(s.clone()))?,
        })
    }

    #[inline] pub(crate) fn push(&mut self, v: Val) { self.stack.push(v); }

    #[inline] pub(crate) fn pop(&mut self) -> Result<Val> {
        self.stack.pop().context("Stack underflow!")
    }

    pub fn print_stack(&self) -> Result<()> {
        let mut output = "[".to_string();
        for i in &self.stack {
            if i.is_heap() {
                output = format!("{}{:?}, ", output, self.heap.get(*i));
                continue
            }
            output = format!("{}{:?}, ", output, i);
        }
        let mut output = output.chars();
        output.next_back();
        output.next_back();
        let output = output.as_str();  // Is it kinda stupid to do that?

        println!("{}]", output);

        Ok(())
    }
}
