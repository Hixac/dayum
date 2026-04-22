use anyhow::{Result, bail};

use crate::vm::types::Obj;

use super::types::Val;
use super::VirtualMachine;


impl<'a> VirtualMachine<'a> {

    pub fn add(&mut self, a: Val, b: Val) -> Result<Val> {
        if a.is_int() && b.is_int() {
            return Ok(Val::int(a.as_int() + b.as_int()))
        } 
        if a.is_float() && b.is_float() {
            return Ok(Val::float(a.as_float() + b.as_float()))
        } 
        if a.is_int() && b.is_float() {
            return Ok(Val::float(a.as_int() as f64 + b.as_float()))
        }
        if a.is_float() && b.is_int() {
            return Ok(Val::float(a.as_float() + b.as_int() as f64))
        }
        if a.is_heap() && b.is_heap() {
            match (self.heap.get(a), self.heap.get(b)) {
                (Obj::Str(x), Obj::Str(y)) => {
                    return self.heap.alloc(Obj::Str(format!("{}{}", x, y)));
                }
            }
        }

        bail!("No yet implemented or wrong type")
    }


    pub fn sub(&self, a: Val, b: Val) -> Result<Val> {
        if a.is_int() && b.is_int() {
            return Ok(Val::int(a.as_int() - b.as_int()))
        } 
        if a.is_float() && b.is_float() {
            return Ok(Val::float(a.as_float() - b.as_float()))
        } 
        if a.is_int() && b.is_float() {
            return Ok(Val::float(a.as_int() as f64 - b.as_float()))
        }
        if a.is_float() && b.is_int() {
            return Ok(Val::float(a.as_float() - b.as_int() as f64))
        }

        bail!("No yet implemented or wrong type")
    }

    pub fn mul(&self, a: Val, b: Val) -> Result<Val> {
        if a.is_int() && b.is_int() {
            return Ok(Val::int(a.as_int() * b.as_int()))
        } 
        if a.is_float() && b.is_float() {
            return Ok(Val::float(a.as_float() * b.as_float()))
        } 
        if a.is_int() && b.is_float() {
            return Ok(Val::float(a.as_int() as f64 * b.as_float()))
        }
        if a.is_float() && b.is_int() {
            return Ok(Val::float(a.as_float() * b.as_int() as f64))
        }

        bail!("No yet implemented or wrong type")
    }

    pub fn div(&self, a: Val, b: Val) -> Result<Val> {
        if a.is_int() && b.is_int() {
            return Ok(Val::int(a.as_int() / b.as_int()))
        } 
        if a.is_float() && b.is_float() {
            return Ok(Val::float(a.as_float() / b.as_float()))
        } 
        if a.is_int() && b.is_float() {
            return Ok(Val::float(a.as_int() as f64 / b.as_float()))
        }
        if a.is_float() && b.is_int() {
            return Ok(Val::float(a.as_float() / b.as_int() as f64))
        }

        bail!("No yet implemented or wrong type")
    }
}
