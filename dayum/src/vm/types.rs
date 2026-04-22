use anyhow::Result;

use std::fmt::Debug;
use std::collections::HashMap;

const QNAN: u64 = 0x7FFC_0000_0000_0000;
const SIGN: u64 = 0x8000_0000_0000_0000;
const TAG_TRUE: u64 = QNAN | 2;
const TAG_FALSE: u64 = QNAN | 3;
const TAG_INT: u64 = QNAN | SIGN;
const TAG_HEAP: u64 = QNAN | 4;

#[derive(Clone, Copy)]
pub struct Val(pub(crate) u64);

impl PartialEq for Val {
    #[inline] fn eq(&self, o: &Self) -> bool { self.0 == o.0 }
}
impl Eq for Val {}

impl core::hash::Hash for Val {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Val {
    #[inline(always)] pub fn float(f: f64) -> Self {
        let bits = f.to_bits();
        if (bits & QNAN) == QNAN { Self(QNAN) } else { Self(bits) }
    }
    #[inline(always)]
    pub fn is_numeric(&self) -> bool {
        self.is_int() || self.is_float()
    }
    pub const INT_MAX: i64 =  0x0000_7FFF_FFFF_FFFF;
    pub const INT_MIN: i64 = -0x0000_8000_0000_0000;
    #[inline(always)] pub fn int(i: i64) -> Self {
        Self(TAG_INT | (i as u64 & 0x0000_FFFF_FFFF_FFFF))
    }
    #[inline(always)] pub fn int_checked(i: i64) -> Option<Self> {
        if i > Self::INT_MAX || i < Self::INT_MIN { None } else { Some(Self::int(i)) }
    }
    #[inline(always)] pub fn bool(b: bool) -> Self { Self(if b { TAG_TRUE } else { TAG_FALSE }) }
    #[inline(always)] pub fn heap(idx: u32) -> Self { Self(TAG_HEAP | ((idx as u64) << 4)) }

    #[inline(always)] pub fn is_float(&self) -> bool { (self.0 & QNAN) != QNAN }
    #[inline(always)] pub fn is_int(&self) -> bool { (self.0 & (QNAN | SIGN)) == TAG_INT }
    #[inline(always)] pub fn is_true(&self) -> bool { self.0 == TAG_TRUE }
    #[inline(always)] pub fn is_false(&self) -> bool { self.0 == TAG_FALSE }
    #[inline(always)] pub fn is_bool(&self) -> bool { self.0 == TAG_TRUE || self.0 == TAG_FALSE }
    #[inline(always)] pub fn is_heap(&self) -> bool {
        (self.0 & QNAN) == QNAN && (self.0 & SIGN) == 0 && (self.0 & 0xF) >= 4
    }

    #[inline(always)] pub fn as_float(&self) -> f64  { f64::from_bits(self.0) }
    #[inline(always)] pub fn as_int(&self) -> i64  {
        let raw = (self.0 & 0x0000_FFFF_FFFF_FFFF) as i64;
        (raw << 16) >> 16
    }
    #[inline(always)] pub fn as_bool(&self) -> bool { self.0 == TAG_TRUE }
    #[inline(always)] pub fn as_heap(&self) -> u32 { ((self.0 >> 4) & 0x0FFF_FFFF) as u32 }
}

impl Debug for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_int() {
            return write!(f, "{} (int)", self.as_int())
        }
        if self.is_float() {
            return write!(f, "{} (float)", self.as_float())
        }
        if self.is_bool() {
            return write!(f, "{} (bool)", self.as_bool())
        }
        if self.is_heap() {
            return write!(f, "{} (heap)", self.as_heap())
        }
        todo!()
    }
}

#[derive(Debug)]
pub enum Obj {
    Str(String),
}

struct HeapSlot {
    obj: Obj
}

pub struct HeapPool {
    slots: Vec<HeapSlot>,
    strings: HashMap<String, u32>,
}

impl HeapPool {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            strings: HashMap::new(),
        }
    }

    pub fn alloc(&mut self, obj: Obj) -> Result<Val> {
        if let Obj::Str(ref s) = obj &&
            s.len() <= 128 &&
           let Some(&idx) = self.strings.get(s) {
            return Ok(Val::heap(idx))
        }

        let idx = self.slots.len() as u32;
        self.slots.push(HeapSlot { obj });

        if let Obj::Str(s) = &self.slots[idx as usize].obj &&
            s.len() <= 128 { self.strings.insert(s.clone(), idx); }

        Ok(Val::heap(idx))
    }

    #[inline(always)] pub fn get(&self, v: Val) -> &Obj {
        &self.slots[v.as_heap() as usize].obj
    }
}
