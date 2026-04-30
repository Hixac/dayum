use std::iter::Peekable;
use log::info;
use anyhow::{Result, bail};

use crate::{lexer::{Token, TokenType}};
use ast::Decl;

mod ast;
mod expression;
mod statement;


pub struct Parser<'a, I: Iterator<Item = Token<'a>>> {
    tokens: Peekable<I>,
    pub ast: Vec<Decl<'a>>
}

impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(tokens: Peekable<I>) -> Self {
        Self { tokens, ast: Vec::new()  }
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

    fn eat(&mut self, t: TokenType) -> Result<()> {
        let Some(token) = self.peek() else {
            bail!("Not found any token. EOF")
        };

        if token.token_type == t {
            info!("Ate {:?}", t);
            self.advance()?;
            return Ok(())
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
}
