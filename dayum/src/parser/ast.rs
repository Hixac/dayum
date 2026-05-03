use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone, Copy, PartialEq)]
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
pub struct Param<'a> {
    pub type_spec: TypeSpec,
    pub decl: Option<Decl<'a>>,
    pub init: Option<Expr<'a>>
}

#[derive(Debug)]
pub enum Decl<'a> {
    Group(Box<Decl<'a>>),
    Pointer(Box<Decl<'a>>),

    Identifier(Token<'a>),

    Function{decl: Box<Decl<'a>>, params: Vec<Param<'a>>},
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
    Assignment{l: Box<Expr<'a>>, op: Token<'a>, r: Box<Expr<'a>> },

    Group(Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum Stmt<'a> {
    If { 
        cond: Expr<'a>,
        stmt: Box<Stmt<'a>>,
        otherwise: Option<Box<Stmt<'a>>>
    },
    Compound(Vec<Stmt<'a>>),
    Return(Option<Expr<'a>>),

    Expression(Expr<'a>),
    VarDecl {
        type_spec: TypeSpec,
        decl: Decl<'a>,
        init: Option<Expr<'a>>
    }
}

#[derive(Debug)]
pub enum TopLevelStmt<'a> {
    FunctionDefinition {
        type_spec: TypeSpec,
        decl: Decl<'a>,
        params: Vec<Param<'a>>,
        body: Option<Stmt<'a>>
    },

    GlobalVariable {
        type_spec: TypeSpec,
        decl: Decl<'a>,
        init: Option<Expr<'a>>
    }
}
