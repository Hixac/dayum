use crate::lexer::{Token, TokenType};
pub use types::*;

use crate::parser::ast::{
    TopLevelStmt,
    Stmt,
    Expr,
    Decl
};

mod types;


pub struct Compiler {
    pub chunk: Chunk,
}

impl<'a> Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::default(),
        }
    }

    pub fn compile(&mut self, stmts: Vec<TopLevelStmt<'a>>) -> () {
        todo!()
    }
}
