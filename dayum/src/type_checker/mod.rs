use anyhow::{Result, bail};
use std::collections::{HashMap, VecDeque};

use crate::lexer::Token;
use crate::parser::ast::{Decl, Expr, Stmt, TopLevelStmt, TypeSpec};


pub struct TypeChecker<'a> {
    ast: &'a Vec<TopLevelStmt<'a>>,
    env: HashMap<Token<'a>, TypeSpec>,
    funcs: HashMap<Token<'a>, Option<Vec<TypeSpec>>>,
    checklist: VecDeque<Token<'a>>,
    errors: Vec<String>
}

impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Vec<TopLevelStmt<'a>>) -> Self {
        Self {
            ast,
            env: HashMap::new(),
            errors: Vec::new(),
            checklist: VecDeque::new(),
            funcs: HashMap::new(),
        }
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
            self.top_level_statement(stmt);
        }
    }

    fn error(&mut self, msg: String) -> () {
        self.errors.push(msg);
    }

    fn error_token(&mut self, at: &Token<'a>, expected: &TypeSpec, got: &TypeSpec) -> () {
        self.errors.push(format!("Expected {:?} got {:?} at {}", expected, got, at))
    }

    fn add_to_checklist(&mut self, token: Token<'a>) -> () {
        self.checklist.push_back(token);
    }

    fn add_var(&mut self, token: &Token<'a>, type_spec: &TypeSpec) -> () {
        if self.env.contains_key(&token) {
            self.error(format!("Key named {} is presented already!", token));
            return;
        }
        self.env.insert(token.clone(), type_spec.clone());
    }

    fn get_type(&mut self, token: &Token<'a>) -> Option<TypeSpec> {
        let Some(type_spec) = self.env.get(token) else {
            self.error(format!("Token not in type system! That token: {}", token));
            return None
        };

        Some(type_spec.clone())
    }

    fn add_func(&mut self, token: Token<'a>, params: Option<Vec<TypeSpec>>) -> () {
        if self.funcs.contains_key(&token) {
            self.error(format!("Function redefinition at {}", token));
        }
        self.funcs.insert(token, params);
    }

    fn validate_func(&mut self, token: Token<'a>, args: Option<Vec<TypeSpec>>) -> () {
        let Some(params) = self.funcs.get(&token) else {
            self.error(format!("Trying to call non-existing function at {}!", token));
            return;
        };

        if *params != args {
            self.error(format!("Wrong argument type at {}", token));
        }
    }

    fn declaration(&mut self, decl: &Decl<'a>) -> Option<Token<'a>> {
        match decl {
            Decl::Group(nested_decl) => self.declaration(nested_decl),
            Decl::Function { decl, params } =>  {
                let Some(token) = self.declaration(decl) else {
                    return None
                };
                Some(token)
            }
            Decl::Array { decl, constant } => self.declaration(decl),
            Decl::Pointer(pointer_decl)  => self.declaration(pointer_decl),
            Decl::Identifier(token) => {
                Some(token.clone())
            }
        }
    }

    fn top_level_statement(&mut self, stmt: &TopLevelStmt<'a>) -> () {
        match stmt {
            TopLevelStmt::FunctionDefinition { type_spec, decl, params, body } => {

            },
            TopLevelStmt::GlobalVariable { type_spec, decl, init } => {

            }
        }
    }

    fn statement(&mut self, stmt: &Stmt<'a>) -> () {
        match stmt {
            Stmt::If { cond, stmt, otherwise } => {
                if let Some(type_spec) = self.expression(cond) && 
                    !matches!(type_spec, TypeSpec::Bool) {
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
            Stmt::VarDecl{..} => {
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

            Expr::Call{identifier, arguments} => {

                self.expression(identifier)
            },
            Expr::Index{identifier, argument} => {
                if let Some(arg) = self.expression(argument) &&
                    !matches!(arg, TypeSpec::Int) {
                    self.error(format!("Expected Int got {:?}", arg));
                }
                self.expression(identifier)
            },

            Expr::UnaryOp{op, val} => {
                self.expression(val)
            },
            Expr::BinaryOp{l, op, r} => {
                let Some(type_spec_l) = self.expression(l) else { return None };
                let Some(type_spec_r) = self.expression(r) else { return None };

                if type_spec_l != type_spec_r {
                    self.error_token(
                        &op,
                        &type_spec_l,
                        &type_spec_r
                        );
                    return None
                }

                Some(type_spec_l)
            },
            Expr::Assignment {..} => todo!(),

            Expr::Group(expr) => self.expression(expr),
        }
    }

    fn print_errors(&self) -> () {
        for err in &self.errors {
            println!("{}", err);
        }
    }
}
