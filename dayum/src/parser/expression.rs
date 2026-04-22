use anyhow::Result;

use super::Parser;
use super::types::OpCode;
use super::types::Value;
use crate::lexer::Token;
use crate::lexer::TokenType;


impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn expression(&mut self) -> Result<()> { 
        self.expr_bp(0)?;

        Ok(())
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<()> {
        match self.peek() {
            Some(TokenType::OpExclaim) => {
                self.advance()?;
                self.expr_bp(5)?;
                self.chunk.emit(OpCode::Not, 0);
            },
            _ => self.unary()?,
        }
        self.infix_bp(min_bp)?;

        Ok(())
    }

    fn infix_bp(&mut self, min_bp: u8) -> Result<()> {
        loop {
            let Some(op) = self.peek() else { break; };

            let Some((l_bp, r_bp, opcode)) = self.infix_binding_power(op) 
                else { break; };

            if l_bp < min_bp {
                break;
            }

            self.advance()?;
            self.expr_bp(r_bp)?;
            self.chunk.emit(opcode, 0);
        }

        Ok(())
    }

    fn unary(&mut self) -> Result<()> {
        self.primary()?;
        Ok(())
    }

    fn primary(&mut self) -> Result<()> {
        let t = self.advance()?;
        match t.token_type {
            TokenType::StringLiteral => {
                let mut chars = t.lexeme.chars();
                chars.next();
                chars.next_back();  // to remove quotes
                let string_literal = chars.as_str().to_string();

                self.emit_const(Value::Str(string_literal));
            }
            TokenType::IntegerLiteral => {
                self.emit_const(Value::Int(t.lexeme.parse().unwrap()));
            }
            TokenType::FloatLiteral => {
                self.emit_const(Value::Float(t.lexeme.parse().unwrap()));
            }
            TokenType::KwTrue | TokenType::KwFalse => {
                self.emit_const(Value::Bool(t.lexeme.parse().unwrap()));
            }
            TokenType::Lparen => {
                self.expression()?;
                self.eat(TokenType::Rparen)?
            }
            _ => ()
        }
        Ok(())
    }

    fn infix_binding_power(&self, tok: TokenType) -> Option<(u8, u8, OpCode)> {
        match tok {
            TokenType::OpPlus => Some((17, 18, OpCode::Add)),
            TokenType::OpMinus => Some((17, 18, OpCode::Sub)),
            TokenType::OpStar => Some((19, 20, OpCode::Mul)),
            TokenType::OpSlash => Some((19, 20, OpCode::Div)),
            _ => None,
        }
    }
}
