use std::iter::Peekable;
use log::info;
use anyhow::{Result, bail};

use crate::{
    lexer::{Token, TokenType},
    parser::ast::{
        Decl,
        DeclKind,
        Expr,
        ExprKind,
        Stmt,
        StmtKind,
        TopLevelStmt,
        TopLevelStmtKind
    }, 
    type_checker::annotation::TypeID
};

pub mod ast;
mod expression;
mod statement;


pub struct Parser<'a, I: Iterator<Item = Token<'a>>> {
    tokens: Peekable<I>,
    offset: usize
}

impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(tokens: Peekable<I>) -> Self {
        Self { tokens, offset: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<TopLevelStmt<'a>>> {
        self.external_declarations()
    }

    fn advance(&mut self) -> Result<Token<'a>> {
        match self.tokens.next() {
            Some(t) => Ok(t),
            None => bail!("Trying get past eof!")
        }
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        match self.tokens.peek() {
            Some(t) => Some(t),
            _ => None
        }
    }

    fn same(&mut self, t: &[TokenType]) -> bool {
        let Some(token) = self.peek() else {
            return false
        };

        info!("Comparing {:?} and {:?}", token, t);
        t.contains(&token.token_type)
    }

    fn eat(&mut self, t: TokenType) -> Result<Token<'a>> {
        let Some(token) = self.peek() else {
            bail!("Not found any token. EOF")
        };

        if token.token_type == t {
            info!("Ate {:?}", t);
            return Ok(self.advance()?)
        }

        bail!("Not found {:?} at {:?}", t, token)
    }

    fn eat_if(&mut self, t: TokenType) -> bool {
        if let Some(token) = self.peek() && t == token.token_type {
            self.advance().unwrap();
            return true;
        }
        false
    }

    fn type_id(&mut self) -> TypeID {
        let id = TypeID(self.offset);
        self.offset += 1;
        id
    }

    fn tlstmt(&mut self, kind: TopLevelStmtKind<'a>) -> TopLevelStmt<'a> {
        TopLevelStmt {
            kind,
            id: self.type_id()
        }
    }

    fn stmt(&mut self, kind: StmtKind<'a>) -> Stmt<'a> {
        Stmt {
            kind,
            id: self.type_id()
        }
    }

    fn expr(&mut self, kind: ExprKind<'a>) -> Expr<'a> {
        Expr {
            kind,
            id: self.type_id()
        }
    }

    fn decl(&mut self, kind: DeclKind<'a>) -> Decl<'a> {
        Decl {
            kind,
            id: self.type_id()
        }
    }
}
