use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::lexer::Token;
use crate::parser::ast::{Decl, Expr, Stmt, TopLevelStmt, TypeSpec, Param};


pub struct TypeChecker<'a> {
    ast: &'a Vec<TopLevelStmt<'a>>,
    env: HashMap<Token<'a>, TypeSpec>,
    funcs: HashMap<Token<'a>, Vec<TypeSpec>>,
    errors: Vec<String>,
    last_token: Option<Token<'a>>
}

impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Vec<TopLevelStmt<'a>>) -> Self {
        Self {
            ast,
            env: HashMap::new(),
            errors: Vec::new(),
            funcs: HashMap::new(),
            last_token: None
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

    fn convert_to_types_params(&self, params: &Vec<Param<'a>>) -> Vec<TypeSpec> {
        let mut types = Vec::new();
        for param in params {
            types.push(param.type_spec.clone())
        }
        types
    }

    fn convert_to_types_args(&mut self, args: &Vec<Expr<'a>>) -> Vec<TypeSpec> {
        let mut types = Vec::new();
        for arg in args {
            types.push(self.expression(arg))
        }
        types
    }

    fn add_var(&mut self, token: Token<'a>, type_spec: &TypeSpec) -> () {
        if self.env.contains_key(&token) {
            self.error(format!("Key named {} is presented already!", token));
            return;
        }
        self.env.insert(token, type_spec.clone());
    }

    fn get_type(&mut self, token: &Token<'a>) -> TypeSpec {
        let Some(type_spec) = self.env.get(token) else {
            self.error(format!("Token not in type system! That token: {}", token));
            panic!("")
        };

        type_spec.clone()
    }

    fn add_func(&mut self, token: Token<'a>, params: Vec<TypeSpec>) -> () {
        if self.funcs.contains_key(&token) {
            self.error(format!("Function redefinition at {}", token));
        }
        self.funcs.insert(token, params);
    }

    fn validate_func(&mut self, token: Token<'a>, args: Vec<TypeSpec>) -> () {
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
                let token = self.declaration(decl).expect("Impossible situation");
                self.add_var(token.clone(), type_spec);
                self.add_func(token, self.convert_to_types_params(params));

                if let Some(body) = body {
                    self.statement(body);
                }
            },
            TopLevelStmt::GlobalVariable { type_spec, decl, init } => {
                let token = self.declaration(decl).expect("Impossible situation");
                self.add_var(token.clone(), type_spec);

                if let Some(expr) = init {
                    let type_spec_init = self.expression(expr);
                    if *type_spec != type_spec_init {
                        self.error_token(&token, &type_spec, &type_spec_init);
                    }
                }
            }
        }
    }

    fn statement(&mut self, stmt: &Stmt<'a>) -> () {
        match stmt {
            Stmt::If { cond, stmt, otherwise } => {
                let cond_type = self.expression(cond);
                if !matches!(cond_type, TypeSpec::Bool) {
                    self.error(format!("Expected bool got {:?} at if-statement", cond_type));
                };

                self.statement(stmt);
                if let Some(otherwise) = otherwise {
                    self.statement(otherwise);
                }
            },
            Stmt::While {..} => {},
            Stmt::For {..} => {},
            Stmt::Return(..) => { },
            Stmt::Compound(stmts) => {
                for stmt in stmts {
                    self.statement(stmt);
                }
            },
            Stmt::Expression(..) => { },
            Stmt::VarDecl{type_spec, decl, init} => { 
                let token = self.declaration(decl).expect("Impossible case");
                self.add_var(token.clone(), type_spec);

                if let Some(expr) = init {
                    let type_spec_init = self.expression(expr);
                    if *type_spec != type_spec_init {
                        self.error_token(&token, &type_spec, &type_spec_init);
                    }
                }
            }
        }
    }

    fn expression(&mut self, expr: &Expr<'a>) -> TypeSpec {
        match expr {
            Expr::IntLiteral(_) => TypeSpec::Int,
            Expr::FloatLiteral(_) => TypeSpec::Float,
            Expr::BoolLiteral(_) => TypeSpec::Bool,
            Expr::StringLiteral(_) => TypeSpec::String,
            Expr::Identifier(token) => {
                self.last_token = Some(token.clone());
                self.get_type(token)
            },

            Expr::Call{identifier, arguments} => {
                let ident_type = self.expression(identifier);
                let last_token = self.last_token.clone().expect("Impossible case");
                let types = self.convert_to_types_args(&arguments);
                self.validate_func(
                    last_token,
                    types
                );

                ident_type
            },
            Expr::Index{identifier, argument} => {
                let type_spec = self.expression(argument);
                if !matches!(type_spec, TypeSpec::Int) {
                    self.error(format!("Expected Int got {:?}", type_spec));
                }
                self.expression(identifier)
            },

            Expr::UnaryOp{op, val} => {
                self.expression(val)
            },
            Expr::BinaryOp{l, op, r} => {
                let type_spec_l = self.expression(l);
                let type_spec_r = self.expression(r);

                if type_spec_l != type_spec_r {
                    self.error_token(
                        &op,
                        &type_spec_l,
                        &type_spec_r
                    );
                }

                type_spec_l
            },
            Expr::Assignment {l, op, r} => {
                let l_type = self.expression(l);
                let r_type = self.expression(r);
                if l_type != r_type {
                    self.error_token(
                        &op,
                        &l_type,
                        &r_type
                    );
                }
                l_type
            },
            Expr::LogicalOp {..} => TypeSpec::Bool,

            Expr::Group(expr) => self.expression(expr),
        }
    }

    fn print_errors(&self) -> () {
        for err in &self.errors {
            println!("{}", err);
        }
    }
}
