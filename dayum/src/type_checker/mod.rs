use anyhow::{Result, bail};
use std::collections::{HashMap, VecDeque};

use crate::lexer::Token;
use crate::parser::ast::{Stmt, Decl, TypeSpec, Expr};


pub struct TypeChecker<'a> {
    ast: &'a Vec<Stmt<'a>>,
    env: HashMap<Token<'a>, TypeSpec>,
    checklist: VecDeque<Token<'a>>,
    errors: Vec<String>
}

impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Vec<Stmt<'a>>) -> Self {
        Self { ast, env: HashMap::new(), errors: Vec::new(), checklist: VecDeque::new() }
    }

    pub fn check(&mut self) -> Result<()> {
        self.walk();

        if self.errors.len() > 0 {
            println!("List of errors:");
            self.print_errors();
            bail!("Got errors!");
        }

        Ok(())
    }

    fn walk(&mut self) -> () {
        for stmt in self.ast {
            self.statement(stmt);
        }
    }

    fn error(&mut self, msg: String) -> () {
        self.errors.push(msg);
    }

    fn error_token(&mut self, at: Token<'a>, expected: TypeSpec, got: TypeSpec) -> () {
        self.errors.push(format!("Expected {:?} got {:?} at {}", expected, got, at))
    }

    fn add_to_checklist(&mut self, token: Token<'a>) -> () {
        self.checklist.push_back(token);
    }

    fn add(&mut self, token: &Token<'a>, type_spec: &TypeSpec) -> () {
        if self.env.contains_key(&token) {
            self.error(format!("Key named {} is presented already!", token));
            return;
        }
        self.env.insert(token.clone(), type_spec.clone());
    }

    fn pop_same(&mut self, type_spec: TypeSpec) -> Option<Token<'a>> {
        let Some(token) = self.checklist.pop_back() else {
            self.error("Trying to pop_back empty checklist!".to_string());
            return None;
        };

        if let Some(other_type_spec) = self.env.get(&token) {
            if type_spec != *other_type_spec {
                self.error_token(
                    token.clone(),
                    other_type_spec.clone(),
                    type_spec.clone()
                );
            }
        }

        Some(token)
    }

    fn get_type(&mut self, token: &Token<'a>) -> Option<TypeSpec> {
        let Some(type_spec) = self.env.get(token) else {
            self.error(format!("Token not in type system! That token: {}", token));
            return None
        };

        Some(type_spec.clone())
    }

    fn declaration(&mut self, decl: &Decl<'a>, type_spec: &TypeSpec) -> bool {
        match decl {
            Decl::Group(nested_decl) => self.declaration(nested_decl, type_spec),
            Decl::Function { decl, params } => self.declaration(decl, type_spec),
            Decl::Array { decl, constant } => self.declaration(decl, type_spec),
            Decl::Pointer(pointer_decl)  => self.declaration(pointer_decl, type_spec),
            Decl::Parameter(param_decl) => {
                if let Some(decl) = param_decl {
                    self.declaration(decl, type_spec);
                }
                false
            }
            Decl::Identifier(token) => {
                self.add(token, type_spec);
                self.add_to_checklist(token.clone());
                true
            }
        }
    }

    fn statement(&mut self, stmt: &Stmt<'a>) -> () {
        match stmt {
            Stmt::If { cond, stmt, otherwise } => {
                if let Some(type_spec) = self.expression(cond) {
                    self.error(format!("Expected bool got {:?} at if-statement", type_spec));
                };

                self.statement(stmt);
                if let Some(otherwise) = otherwise {
                    self.statement(otherwise);
                }
            },
            Stmt::Return(expr) => {

            },
            Stmt::Compound(stmts) => {
                for stmt in stmts {
                    self.statement(stmt);
                }
            },
            Stmt::Expression(expr) => {
                let _ = self.expression(expr);
            },
            Stmt::Declarator(type_spec, decl, val) => {
                if let Some(decl) = decl && self.declaration(decl, type_spec) {
                    let Some(token) = self.pop_same(type_spec.clone()) else { return; };
                    if let Some(val) = val {
                        let Some(type_spec) = self.expression(val) else {
                            return
                        };

                        self.add_to_checklist(token);
                        let _ = self.pop_same(type_spec);
                    }
                }

            }
        }
    }

    fn expression(&mut self, expr: &Expr<'a>) -> Option<TypeSpec> {
        match expr {
            Expr::IntLiteral(_) => Some(TypeSpec::Int),
            Expr::FloatLiteral(_) => Some(TypeSpec::Float),
            Expr::BoolLiteral(_) => Some(TypeSpec::Bool),
            Expr::StringLiteral(_) => Some(TypeSpec::String),
            Expr::Identifier(token) => self.get_type(token),

            Expr::Call{..} => { todo!() },
            Expr::Index{..} => { todo!() },

            Expr::UnaryOp{..} => { todo!() },
            Expr::BinaryOp{l, op, r} => {
                let Some(type_spec_l) = self.expression(l) else { return None };
                let Some(type_spec_r) = self.expression(r) else { return None };

                if type_spec_l != type_spec_r {
                    self.error_token(
                        op.clone(),
                        type_spec_l,
                        type_spec_r
                        );
                    return None
                }

                Some(type_spec_l)
            },

            Expr::Group(expr) => self.expression(expr),

            Expr::Statement(stmt) => {
                self.statement(stmt);
                None
            }
        }
    }

    fn print_errors(&self) -> () {
        for err in &self.errors {
            println!("{}", err);
        }
    }
}
