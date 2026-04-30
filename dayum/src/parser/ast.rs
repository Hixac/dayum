use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone, Copy)]
pub enum TypeSpec {
    Int, Float, Char, String, Void, Bool
}

impl TypeSpec {
    pub fn from_token<'a>(token: &Token<'a>) -> Self {
        use TokenType::*; use TypeSpec::*;

        match token.token_type {
            KwInt => Int,
            KwFloat => Float,
            KwChar => Char,
            KwString => String,
            KwVoid => Void,
            KwBool => Bool,
            _ => panic!("Wrong type")
        }
    }
}

#[derive(Debug)]
pub enum Decl<'a> {
    Group(Box<Decl<'a>>),

    Pointer(Box<Decl<'a>>),

    Identifier(Token<'a>),
    Parameter(Option<Box<Decl<'a>>>),

    Function{decl: Box<Decl<'a>>, params: Option<Vec<Stmt<'a>>>},
    Array{decl: Box<Decl<'a>>, constant: Option<Expr<'a>>},
}

#[derive(Debug)]
pub enum Expr<'a> {
    IntLiteral(i32),
    FloatLiteral(f32),
    BoolLiteral(bool),
    StringLiteral(&'a str),
    Identifier(Token<'a>),

    Call{identifier: Box<Expr<'a>>, arguments: Vec<Expr<'a>>},
    Index{identifier: Box<Expr<'a>>, argument: Box<Expr<'a>>},

    UnaryOp{op: Token<'a>, val: Box<Expr<'a>>},
    BinaryOp{l: Box<Expr<'a>>, op: Token<'a>, r: Box<Expr<'a>> },

    Group(Box<Expr<'a>>),

    Statement(Box<Stmt<'a>>)
}

#[derive(Debug)]
pub enum Stmt<'a> {
    If{cond: Expr<'a>, stmt: Box<Stmt<'a>>, otherwise: Option<Box<Stmt<'a>>>},
    Compound(Vec<Stmt<'a>>),
    Expression(Expr<'a>),
    Declarator(TypeSpec, Option<Decl<'a>>, Option<Box<Expr<'a>>>)
}
