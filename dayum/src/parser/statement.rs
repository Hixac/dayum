use anyhow::{Result, bail};

use super::ast::{Stmt, Decl, TypeSpec, TopLevelStmt, Param};
use super::Parser;
use crate::{lexer::{Token, TokenType}};


impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub(crate) fn external_declarations(&mut self) -> Result<Vec<TopLevelStmt<'a>>> {
        let mut stmts = Vec::new();
        loop {
            let Some(stmt) = self.top_level_declaration()? else { break; };
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn top_level_declaration(&mut self) -> Result<Option<TopLevelStmt<'a>>> {
        let Some(_) = self.peek() else {
            return Ok(None)
        };

        let type_spec = self.advance()?;
        let identifier = self.advance()?;
        let mut params: Vec<Param<'a>> = Vec::new();
        let mut is_fun = false;

        if self.same(&[TokenType::Lparen]) {
            self.eat(TokenType::Lparen)?;
            params = self.parameters()?;
            self.eat(TokenType::Rparen)?;
            is_fun = true;
        }

        if !is_fun && self.same(&[TokenType::OpEqual]) {
            self.eat(TokenType::OpEqual)?;
            let expr = self.expression()?;
            self.eat(TokenType::Semicolon)?;
            return Ok(Some(TopLevelStmt::GlobalVariable {
                type_spec: TypeSpec::from_token(&type_spec),
                decl: Decl::Identifier(identifier),
                init: Some(expr)
            }))
        }


        if self.same(&[TokenType::Semicolon]) {
            self.eat(TokenType::Semicolon)?;
            if !is_fun {
                return Ok(Some(TopLevelStmt::GlobalVariable {
                    type_spec: TypeSpec::from_token(&type_spec),
                    decl: Decl::Identifier(identifier),
                    init: None
                }))
            } else {
                return Ok(Some(TopLevelStmt::FunctionDefinition {
                    type_spec: TypeSpec::from_token(&type_spec),
                    decl: Decl::Identifier(identifier),
                    params,
                    body: None
                }))
            }
        }

        let body = self.compound_statement()?;

        Ok(Some(TopLevelStmt::FunctionDefinition {
            type_spec: TypeSpec::from_token(&type_spec),
            decl: Decl::Identifier(identifier),
            params,
            body: Some(body)
        }))
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
                    let mut params = Vec::new();
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

    fn parameters(&mut self) -> Result<Vec<Param<'a>>> {
        let mut params: Vec<Param<'a>> = Vec::new();

        loop {
            let Some(decl) = self.parameter_declaration()? else { break; };
            params.push(decl);
            if self.same(&[TokenType::Rparen]) { break; }
            self.eat(TokenType::Comma)?;
        }
        if params.len() == 0 {
            return Ok(vec![])
        }

        Ok(params)
    }

    fn parameter_declaration(&mut self) -> Result<Option<Param<'a>>> {
        let Some(type_spec) = self.declaration_specifier() else {
            return Ok(None)
        };
        if self.same(&[TokenType::Comma, TokenType::Rparen]) {
            return Ok(Some(
                    Param {
                        type_spec: TypeSpec::from_token(&type_spec),
                        decl: None,
                        init: None
                    }
            ))
        }
        let decl = Some(self.declarator()?);

        let mut init = None;
        if self.same(&[TokenType::OpEqual]) {
            self.eat(TokenType::OpEqual)?;
            init = Some(self.expression()?);
        }

        Ok(Some(
                Param {
                    type_spec: TypeSpec::from_token(&type_spec),
                    decl,
                    init
                }
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

        let mut init = None;
        if self.same(&[TokenType::OpEqual]) {
            self.eat(TokenType::OpEqual)?;
            init = Some(self.expression()?);
        }
        self.eat(TokenType::Semicolon)?;

        Ok(Some(
                Stmt::VarDecl{
                    type_spec: TypeSpec::from_token(&type_spec),
                    decl, init
                }
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
                if self.same(&[TokenType::Semicolon]) {
                    self.eat(TokenType::Semicolon)?;
                    return Ok(Stmt::Return(None))
                }
                let expr = self.expression_statement()?;
                match expr {  // is it okay to do that, huh?
                    Stmt::Expression(expr) => Ok(Stmt::Return(Some(expr))),
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
