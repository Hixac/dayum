use anyhow::Result;
use std::collections::HashMap;

use crate::compiler::{Chunk, CompiledFunction, OpCode, Value};
use heap::{Heap, Unit};

mod heap;

struct CallFrame {
    pub func: CompiledFunction,
    pub ip: usize,
    pub stack_base: usize, // where this frame's locals start on the stack
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    heap: Heap
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        let frame = CallFrame {
            func: CompiledFunction {
                name: "".to_string(),
                arity: 0,
                chunk
            },
            ip: 0,
            stack_base: 0
        };

        Self { 
            frames: vec![frame],
            stack: Vec::new(),
            globals: HashMap::new(),
            heap: Heap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let ip = self.current_frame().ip;
            let instr = self.current_chunk().instructions[ip];
            let operand = instr.operand;
            self.current_frame().ip += 1;

            println!("{}. (out of {})", ip, self.current_chunk().instructions.len());
            println!("{:?}", instr);
            println!("{:?}", self.stack);
            println!("{:?}", self.globals);
            println!();

            use OpCode::*;
            match instr.opcode {
                LoadInt | LoadFloat | LoadString | LoadBool | LoadPtr => {
                    let constant = self.current_chunk().constants[operand as usize].clone();
                    self.stack.push(constant);
                },
                LoadNull => {
                    self.stack.push(Value::Null);
                },

                AddInt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.add_numeric(b));
                },
                AddFloat => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.add_numeric(b));
                },
                AddString => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.concatenate(b));
                },

                SubInt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.sub_numeric(b));
                },
                SubFloat => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.sub_numeric(b));
                },

                MulInt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.mul_numeric(b));
                },
                MulFloat => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.mul_numeric(b));
                },

                DivInt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.div_numeric(b));
                },
                DivFloat => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.div_numeric(b));
                },

                AndBool => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.and_logical(b));
                },
                OrBool => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a.or_logical(b));
                },
                NotBool => {
                    let a = self.pop();
                    self.push(a.not());
                },

                Call => {
                    let arity = operand as usize;
                    let ptr = self.pop().as_ptr();
                    let func = self.heap.move_val(ptr).unwrap().as_function();
                    self.frames.push(CallFrame {
                        func,
                        ip: 0,
                        stack_base: self.stack.len() - arity
                    });
                },

                DefineGlobal => {
                    let var = &self.current_chunk().names[operand as usize];
                    self.globals.insert(var.name.clone(), Value::Null);
                },
                LoadGlobal => {
                    let var = &self.current_chunk().names[operand as usize];
                    let global = self.globals.get(&var.name).unwrap();
                    self.push(global.clone());
                },
                StoreGlobal => {
                    let val = self.pop();

                    let var = &self.current_chunk().names[operand as usize];
                    let name = var.name.clone();
                    let global = self.globals.get_mut(&name).unwrap();
                    *global = val;
                },

                LoadName => {
                    let idx = self.current_frame().stack_base + operand as usize;
                    self.push(self.stack[idx].clone());
                },
                StoreName => {
                    let idx = self.current_frame().stack_base + operand as usize;
                    let val = self.pop();
                    self.stack[idx] = val;
                },

                AllocPtr => {
                    let ptr = self.heap.alloc(  // TODO: refactor it to be inside
                        match self.current_chunk().constants[operand as usize].clone() {
                            Value::Func(f) => Unit::Function(f),
                            _ => todo!("")
                        }
                    );
                    self.push(Value::Ptr(ptr));
                }

                Return => {
                    let return_val = self.stack.pop().unwrap_or(Value::Null);
                    let frame = self.frames.pop().unwrap();
                    self.stack.truncate(frame.stack_base);
                    self.stack.push(return_val);
                }

                Stop => { return Ok(()) },
            };

        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, value: Value) -> () {
        self.stack.push(value);
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn current_chunk(&self) -> &Chunk {
        &self.frames.last().unwrap().func.chunk
    }
}
