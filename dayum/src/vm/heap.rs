use std::collections::HashMap;

use crate::compiler::CompiledFunction;


#[derive(Debug)]
pub(super) enum Unit {
    Array(usize),  // size of bytes
    Function(CompiledFunction)
}

impl Unit {
    pub(super) fn as_function(self) -> CompiledFunction {
        use Unit::*;
        match self {
            Function(f) => f,
            _ => panic!("Non-function")
        }
    }
}

pub(super) struct Heap {
    allocated: Vec<Option<Unit>>,
    functions: HashMap<String, u16>
}

impl Heap {
    pub(super) fn new() -> Self {
        Self {
            allocated: Vec::new(),
            functions: HashMap::new()
        }
    }

    pub(super) fn alloc(&mut self, unit: Unit) -> usize {
        self.allocated.push(Some(unit));
        self.allocated.len() - 1
    }

    pub(super) fn move_val(&mut self, ptr: usize) -> Option<Unit> {
        self.allocated.remove(ptr)
    }
}
