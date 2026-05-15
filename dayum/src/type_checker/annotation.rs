use crate::lexer::{Token, TokenType};


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TypeID(pub usize);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimaryEnum {
    Int, Float, Char, String, Void, Bool
}

#[derive(Debug, Clone)]
pub enum Type {
    Primary(PrimaryEnum),
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Function(Box<Type>, Vec<Type>),
}

impl Type {
    pub fn primitive<'a>(token: &Token<'a>) -> Self {
        use TokenType::*; use PrimaryEnum::*; use Type::*;

        match token.token_type {
            KwInt => Primary(Int),
            KwFloat => Primary(Float),
            KwChar => Primary(Char),
            KwString => Primary(String),
            KwVoid => Primary(Void),
            KwBool => Primary(Bool),
            _ => panic!("Wrong type")
        }
    }

    pub fn equal(&self, other: &Type) -> bool {
        use Type::*;
        match (self, other) {
            (Primary(one), Primary(two)) => one == two,
            _ => todo!()
        }
    }

    pub fn is_primary(&self) -> bool {
        use Type::*;
        match self {
            Primary(_) => true,
            _ => false
        }
    }

    pub fn as_primary(&self) -> &PrimaryEnum {
        use Type::*;
        match self {
            Primary(v) => v,
            _ => panic!("Bro yo did wrong")
        }
    }
}
