use std::iter::Peekable;
use anyhow::{Context, Result, bail};
use crate::lexer::{Token, TokenType};
pub use types::*;

mod expression;
mod types;

pub struct Parser<'a, I: Iterator<Item = Token<'a>>> {
    tokens: Peekable<I>,
    pub chunk: Chunk,
}

impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(tokens: Peekable<I>) -> Self {
        Self { tokens, chunk: Chunk::default() }
    }

    fn advance(&mut self) -> Result<Token<'a>> {
        match self.tokens.next() {
            Some(t) => Ok(t),
            None => bail!("Trying get past eof!")
        }
    }

    fn peek(&mut self) -> Option<TokenType> {
        match self.tokens.peek() {
            Some(t) => Some(t.token_type),
            _ => None
        }
    }

    fn eat(&mut self, t: TokenType) -> Result<()> {
        if t == self.peek().with_context(|| format!("Not found any token"))? {
            self.advance()?;
            return Ok(())
        }

        bail!("Not found {:?}", t)
    }

    fn emit_const(&mut self, value: Value) -> () {
        let idx = self.chunk.push_constant(value);
        self.chunk.emit(OpCode::LoadConst, idx);
    }
}
