use anyhow::{Result, bail};

use super::ast::{Stmt, Decl, TypeSpec, Expr};
use super::Parser;
use crate::{lexer::{Token, TokenType}};


impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub(crate) fn external_declarations(&mut self) -> Result<Vec<Stmt<'a>>> {
        let mut decls = Vec::new();
        loop {
            let Some(decl) = self.declaration()? else { break; };
            decls.push(decl);
        }
        Ok(decls)
    }

    fn declarator(&mut self) -> Result<Decl<'a>> {
        self.direct_declarator(0)
    }

    fn direct_declarator(&mut self, min_bp: u8) -> Result<Decl<'a>> {
        let Some(token) = self.peek() else {
            bail!("No tokens left!")
        };

        use TokenType::*;
        let mut lhs = match token.token_type {
            Identifier => Decl::Identifier(self.advance().unwrap()),
            OpStar => {
                self.eat(OpStar)?;
                let decl = self.declarator()?;
                Decl::Pointer(Box::new(decl))
            }
            Lparen => {
                self.eat(Lparen)?;
                let decl = self.declarator()?;
                self.eat(Rparen)?;
                Decl::Group(Box::new(decl))
            },
            _ => panic!("No identifier")
        };

        loop {
            let Some(symbol) = self.peek() else { break; };
            let toktype = symbol.token_type;

            let Some((l_bp, _)) = self.postfix_binding(toktype) else { break; };
            if l_bp < min_bp { break; }

            match toktype {
                Lbracket => {
                    self.eat(TokenType::Lbracket)?;
                    let mut constant = None;
                    if !self.same(&[TokenType::Rbracket]) { constant = Some(self.expression()?); }
                    self.eat(TokenType::Rbracket)?;

                    lhs = Decl::Array { decl: Box::new(lhs), constant }
                },
                Lparen => {
                    self.eat(TokenType::Lparen)?;
                    let mut params = None;
                    if !self.same(&[TokenType::Rparen]) { params = self.parameters()?; }
                    self.eat(TokenType::Rparen)?;

                    lhs = Decl::Function { decl: Box::new(lhs), params }
                },
                _ => unreachable!()
            }
        }

        Ok(lhs)
    }

    fn postfix_binding(&self, symbol: TokenType) -> Option<(u8, ())> {
        use TokenType::*;
        Some(match symbol {
            Lbracket => (15, ()),
            Lparen => (17, ()),
            _ => return None
        })
    }

    fn parameters(&mut self) -> Result<Option<Vec<Stmt<'a>>>> {
        let mut params: Vec<Stmt<'a>> = Vec::new();

        loop {
            let Some(decl) = self.parameter_declaration()? else { break; };
            params.push(decl);
            if self.same(&[TokenType::Rparen]) { break; }
            self.eat(TokenType::Comma)?;
        }
        if params.len() == 0 {
            return Ok(None)
        }

        Ok(Some(params))
    }

    fn parameter_declaration(&mut self) -> Result<Option<Stmt<'a>>> {
        let Some(type_spec) = self.declaration_specifier() else {
            return Ok(None)
        };
        if self.same(&[TokenType::Comma, TokenType::Rparen]) {
            return Ok(Some(
                    Stmt::Declarator(TypeSpec::from_token(&type_spec), None, None)
            ))
        }
        let decl = Some(self.declarator()?);

        let mut stmt = None;
        if self.same(&[TokenType::OpEqual]) {
            self.eat(TokenType::OpEqual)?;
            stmt = Some(Box::new(self.expression()?));
        }

        Ok(Some(
                Stmt::Declarator(TypeSpec::from_token(&type_spec), decl, stmt)
        ))
    }

    fn declaration_specifier(&mut self) -> Option<Token<'a>> {
        use TokenType::*;
        if self.same(&[KwInt, KwFloat, KwChar, KwString, KwBool, KwVoid]) {
            return Some(self.advance().unwrap())
        }
        None
    }

    fn declaration(&mut self) -> Result<Option<Stmt<'a>>> {
        let Some(type_spec) = self.declaration_specifier() else {
            return Ok(None)
        };
        let decl = self.declarator()?;
        if matches!(&decl, Decl::Function { .. }) {
            return Ok(Some(self.function_definition(TypeSpec::from_token(&type_spec), decl)?))
        }

        let mut stmt = None;
        if self.same(&[TokenType::OpEqual]) {
            self.eat(TokenType::OpEqual)?;
            stmt = Some(Box::new(self.expression()?));
        }
        self.eat(TokenType::Semicolon)?;

        Ok(Some(
                Stmt::Declarator(TypeSpec::from_token(&type_spec), Some(decl), stmt)
        ))
    }

    fn function_definition(&mut self, type_spec: TypeSpec, decl: Decl<'a>) -> Result<Stmt<'a>> {
        let stmt = self.compound_statement()?;
        Ok(Stmt::Declarator(
                type_spec, 
                Some(decl),
                Some(Box::new(Expr::Statement(Box::new(stmt))))
        ))
    }

    fn statement(&mut self) -> Result<Stmt<'a>> {
        use TokenType::*;

        let Some(token) = self.peek() else {
            bail!("No tokens left!")
        };

        match token.token_type {
            KwIf => self.selection_statement(),
            Lbrace => self.compound_statement(),
            KwReturn => {
                self.eat(KwReturn)?;
                let expr = self.expression_statement()?;
                match expr {  // is it okay to do that, huh?
                    Stmt::Expression(expr) => Ok(Stmt::Return(expr)),
                    _ => unreachable!()
                }
            }
            _ => self.expression_statement()
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt<'a>> {
        let expr = self.expression()?;
        self.eat(TokenType::Semicolon)?;

        Ok(Stmt::Expression(expr))
    }

    fn compound_statement(&mut self) -> Result<Stmt<'a>> {
        self.eat(TokenType::Lbrace)?;

        let mut stmts: Vec<Stmt<'a>> = Vec::new();
        while let Some(token) = self.peek() {
            if token.token_type == TokenType::Rbrace {
                break;
            }

            let Some(stmt) = self.declaration()? else {
                stmts.push(self.statement()?);
                continue;
            };
            stmts.push(stmt);
        }
        self.eat(TokenType::Rbrace)?;

        Ok(Stmt::Compound(stmts))
    }

    fn selection_statement(&mut self) -> Result<Stmt<'a>> {
        self.eat(TokenType::KwIf)?;
        self.eat(TokenType::Lparen)?;

        let cond = self.expression()?;

        self.eat(TokenType::Rparen)?;

        let stmt = self.statement()?;

        let mut otherwise: Option<Box<Stmt<'a>>> = None;
        if self.eat_if(TokenType::KwElse) {
            otherwise = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If { cond, stmt: Box::new(stmt), otherwise })
    }
}
