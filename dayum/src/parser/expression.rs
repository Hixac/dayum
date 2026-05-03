use anyhow::{Result, bail};

use super::ast::Expr;
use super::Parser;
use crate::lexer::{Token, TokenType};


impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub(crate) fn expression(&mut self) -> Result<Expr<'a>> { 
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr<'a>> {
        use TokenType::*;

        let mut lhs = self.unary()?;

        loop {
            let Some(op) = self.peek() else { break; };
            let toktype = op.token_type;

            if let Some((l_bp, ())) = self.postfix_binding_power(toktype) {
                if l_bp < min_bp {
                    break;
                }

                match toktype {
                    TokenType::Lparen => {
                        self.eat(Lparen)?;
                        let mut args: Vec<Expr<'a>> = Vec::new();
                        if self.same(&[Rparen]) {
                            self.eat(Rparen)?;
                            lhs = Expr::Call { identifier: Box::new(lhs), arguments: args };
                            continue;
                        }
                        loop {
                            args.push(self.expression()?);
                            if self.same(&[Rparen]) { break; }
                            self.eat(Comma)?;
                        }
                        self.eat(Rparen)?;
                        lhs = Expr::Call { identifier: Box::new(lhs), arguments: args }
                    },
                    Lbracket => {
                        self.eat(Lbracket)?;
                        let argument = Box::new(self.expression()?);
                        self.eat(Rbracket)?;
                        lhs = Expr::Index { identifier: Box::new(lhs), argument }
                    }
                    _ => panic!("Wrong op")
                }
                continue;
            }

            let Some((l_bp, r_bp)) = self.infix_binding_power(toktype) else { break; };

            if l_bp < min_bp {
                break;
            }

            let op = self.advance()?;
            let rhs = self.expr_bp(r_bp)?;

            if matches!(op.token_type, OpEqual) {
                lhs = Expr::Assignment { l: Box::new(lhs), op, r: Box::new(rhs) };
                continue;
            }
            if matches!(op.token_type, OpLess | OpLessEqual | OpGreater | OpGreaterEqual) {
                lhs = Expr::LogicalOp { l: Box::new(lhs), op, r: Box::new(rhs) };
                continue;
            }
            lhs = Expr::BinaryOp { l: Box::new(lhs), op, r: Box::new(rhs) };
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr<'a>> {
        let Some(token) = self.peek() else {
            bail!("No tokens left!");
        };

        match token.token_type {
            TokenType::OpExclaim => {
                let token = self.advance()?;
                let val = self.expr_bp(15)?;
                Ok(Expr::UnaryOp { op: token, val: Box::new(val) })
            },
            _ => Ok(self.primary()?),
        }
    }

    fn primary(&mut self) -> Result<Expr<'a>> {
        let t = self.advance()?;
        match t.token_type {
            TokenType::StringLiteral => {
                Ok(Expr::StringLiteral(t.lexeme.trim_matches('"')))
            }
            TokenType::IntegerLiteral => {
                Ok(Expr::IntLiteral(t.lexeme.parse().unwrap()))
            }
            TokenType::FloatLiteral => {
                Ok(Expr::FloatLiteral(t.lexeme.parse().unwrap()))
            }
            TokenType::KwTrue | TokenType::KwFalse => {
                Ok(Expr::BoolLiteral(t.lexeme.parse().unwrap()))
            }
            TokenType::Lparen => {
                let group = Ok(Expr::Group(Box::new(self.expression()?)));
                self.eat(TokenType::Rparen)?;
                group
            }
            TokenType::Identifier => {
                Ok(Expr::Identifier(t))
            }
            _ => bail!("Token fail on primary grammar. Caused token: {:?}", t)
        }
    }

    fn postfix_binding_power(&self, tok: TokenType) -> Option<(u8, ())> {
        match tok {
            TokenType::Lparen => Some((15, ())),
            TokenType::Lbracket => Some((16, ())),
            _ => None,
        }
    }

    fn infix_binding_power(&self, tok: TokenType) -> Option<(u8, u8)> {
        match tok {
            TokenType::OpEqual => Some((2, 1)),

            TokenType::OpLess | TokenType::OpLessEqual => Some((3, 4)),
            TokenType::OpGreater | TokenType::OpGreaterEqual => Some((5, 6)),

            TokenType::OpLogAnd => Some((7, 8)),
            TokenType::OpLogOr => Some((9, 10)),

            TokenType::OpPlus => Some((20, 21)),
            TokenType::OpMinus => Some((20, 21)),
            TokenType::OpStar => Some((22, 23)),
            TokenType::OpSlash => Some((22, 23)),
            _ => None,
        }
    }
}
