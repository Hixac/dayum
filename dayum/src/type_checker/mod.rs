use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::lexer::Token;
use crate::parser::ast::{
    Expr,
    ExprKind,
    Decl,
    DeclKind,
    Param,
    Stmt,
    StmtKind,
    TopLevelStmt,
    TopLevelStmtKind
};

use annotation::{Type, TypeID, PrimaryEnum};
pub mod annotation;


pub struct TypeChecker {
    pub type_map: HashMap<TypeID, Type>,
    pub var_map: HashMap<String, (TypeID, usize)>,
    debug: HashMap<TypeID, (u16, usize)>,
    last_func_def: Option<Type>,
    errors: Vec<String>,
    depth: usize
}

impl<'a> TypeChecker {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
            var_map: HashMap::new(),
            debug: HashMap::new(),
            last_func_def: None,
            errors: Vec::new(),
            depth: 0
        }
    }

    pub fn check(&mut self, stmts: &Vec<TopLevelStmt<'a>>) -> Result<()> {
        for stmt in stmts {
            self.top_level_stmt(stmt);
        }

        if self.errors.len() > 0 {
            println!("List of errors:");
            for err in &self.errors {
                println!("{}", err);
            }
            bail!("Errors are present");
        }

        Ok(())
    }

    fn debug_info(&mut self, id: TypeID, token: &Token<'a>) {
        self.debug.insert(id, (token.line, token.pos));
    }

    fn error(&mut self, context: &TypeID, msg: &str) {
        let Some(info) = self.debug.get(&context) else {
            println!("No context available!");
            return;
        };
        self.errors.push(
            format!("{} in line {} at {} position", msg, info.0, info.1)
        );
    }

    fn error_two_contexts(&mut self, con1: &TypeID, con2: &TypeID, msg: &str) {
        let Some(con1) = self.debug.get(&con1) else {
            println!("No context available!");
            return;
        };
        let Some(con2) = self.debug.get(&con2) else {
            println!("No context available!");
            return;
        };

        self.errors.push(
            format!(
                "{} in line {} at {} position for more context look at line {} at {} position",
                msg, con1.0, con1.1, con2.0, con2.1
            )
        );
    }

    fn error_token(&mut self, context: &Token<'a>, msg: &str) {
        self.errors.push(
            format!("{} in line {} at {} position", msg, context.line, context.pos)
        );
    }

    fn error_unary(&mut self, op: &Token<'a>, val: &TypeID) {
        let Some(val) = self.type_map.get(val) else {
            println!("No context available!");
            return;
        };

        self.errors.push(
            format!("Expected bool when using not operation ({} {:?}) in line {} at {} position", 
                op.lexeme, val, op.line, op.pos
            )
        );
    }

    fn error_binary(&mut self, l: &TypeID, op: &Token<'a>, r: &TypeID) {
        let Some(l) = self.type_map.get(l) else {
            println!("No context available!");
            return;
        };
        let Some(r) = self.type_map.get(r) else {
            println!("No context available!");
            return;
        };

        self.errors.push(
            format!("Different types on binary operation ({:?} {} {:?}) in line {} at {} position", 
                l, op.lexeme, r, op.line, op.pos
            )
        );
    }

    fn function_definition(&mut self, return_type: Type, params: &Vec<Param<'a>>, type_id: TypeID, identifier: String) {
        if self.var_map.contains_key(&identifier) {
            self.error(&type_id, "Function redefinition is not allowed");
            return;
        }

        let mut types = Vec::new();
        self.depth += 1;  // artificially adding depth 'cause of function scope
        for param in params {  // TODO: add recursive typing
            if let Some(decl) = &param.decl {
                let (id, identifier) = self.declaration(decl, &param.type_spec);
                self.variable_definition(&param.type_spec, &param.init, id, identifier);
            };
            types.push(Type::primitive(&param.type_spec));
        }
        self.depth -= 1;

        self.var_map.insert(identifier, (type_id.clone(), self.depth));
        self.type_map.insert(type_id, Type::Function(Box::new(return_type), types));
    }

    fn parameters_compare(&mut self, identifier_id: TypeID, args: &Vec<Expr<'a>>) {
        let mut ids = Vec::new();
        for arg in args {
            let arg_id = self.expression(arg);
            ids.push(arg_id);
        }

        let params = {
            let Some(func_type) = self.type_map.get(&identifier_id) else { unreachable!() };
            match func_type {
                Type::Function(_, params) => {
                    params.clone()
                },
                _ => unreachable!()
            }
        };

        if args.len() != params.len() {
            self.error(&identifier_id, "Wrong number of arguments!");
        }

        for (idx, arg_id) in ids.iter().enumerate() {
            let Some(arg_type) = self.type_map.get(&arg_id) else { unreachable!() };
            if !params[idx].equal(arg_type) {
                self.error_two_contexts(
                    arg_id,
                    &identifier_id,
                    format!("Expected {:?} parameter type got {:?} argument type", params[idx], arg_type).as_str()
                );
            }
        }
    }

    fn array_definition(&mut self, spec: Type, constant: &Option<Token<'a>>, type_id: TypeID) {
        let mut constant_value = None;

        if let Some(constant) = constant {
            match constant.lexeme.parse::<usize>() {
                Ok(v) => {
                    constant_value = Some(v);
                }
                Err(e) => {
                    self.error_token(
                        &constant, 
                        format!("Got wrong type as constant in array definition while expecting int: {}", e).as_str()
                    );
                    constant_value = None;
                }
            }
        }

        self.type_map.insert(type_id, Type::Array(Box::new(spec), constant_value));
    }

    fn variable_definition(&mut self, type_spec: &Token<'a>, init: &Option<Expr<'a>>, type_id: TypeID, identifier: String) {
        if self.var_map.contains_key(&identifier) {
            self.error(&type_id, "Variable shadowing are not allowed");
            return;
        }

        let var_type = Type::primitive(type_spec);

        if let Some(init) = init {
            let init_id = self.expression(init);
            if let Some(init_type) = self.type_map.get(&init_id) {
                if !var_type.equal(init_type) {
                    self.error_token(
                        type_spec,
                        format!("Expected {:?} got {:?} while variable defining", var_type, init_type).as_str(), 
                    );
                }
            }
        }

        self.var_map.insert(identifier, (type_id.clone(), self.depth));
        self.type_map.insert(type_id, var_type);
    }

    fn top_level_stmt(&mut self, stmt: &TopLevelStmt<'a>) {
        use TopLevelStmtKind::*;
        match &stmt.kind {
            GlobalVariable { type_spec, decl, init } => {
                let (type_id, identifier) = self.declaration(decl, type_spec);
                self.variable_definition(type_spec, init, type_id, identifier);
            },
            FunctionDefinition { type_spec, decl, params, body } => {
                let (type_id, identifier) = self.declaration(decl, type_spec);
                self.function_definition(
                    Type::primitive(type_spec),
                    params,
                    type_id,
                    identifier
                );

                self.last_func_def = Some(Type::primitive(type_spec));

                if let Some(body) = body {
                    self.statement(body);
                }
            }
        }
    }

    fn statement(&mut self, stmt: &Stmt<'a>) {
        use StmtKind::*;
        match &stmt.kind {
            If{
                cond,
                stmt,
                otherwise,
                ..
            } => {
                let cond_id = self.expression(cond);
                if let Some(cond_type) = self.type_map.get(&cond_id) {
                    if !cond_type.equal(&Type::Primary(PrimaryEnum::Bool)) {
                        self.error(&cond_id, "Expected bool type");
                    }
                    self.statement(stmt);
                    if let Some(otherwise) = otherwise {
                        self.statement(otherwise);
                    }
                }
            },
            While{
                cond,
                body,
                ..
            } => {
                let cond_id = self.expression(cond);
                if let Some(cond_type) = self.type_map.get(&cond_id) {
                    if !cond_type.equal(&Type::Primary(PrimaryEnum::Bool)) {
                        self.error(&cond_id, "Expected bool type");
                    }
                    self.statement(body);
                }
            },
            For{
                cond,
                body,
                ..
            } => {
                let cond_id = self.expression(cond);
                if let Some(cond_type) = self.type_map.get(&cond_id) {
                    if !cond_type.equal(&Type::Primary(PrimaryEnum::Bool)) {
                        self.error(&cond_id, "Expected bool type");
                    }
                    self.statement(body);
                }
            },

            Compound(stmts) => {
                self.depth += 1;
                for stmt in stmts {
                    self.statement(stmt);
                }
                self.depth -= 1;
            },
            Return{debug, expr} => {
                let Some(expr) = expr else {
                    let Some(func_type) = &self.last_func_def else {
                        unreachable!()
                    };

                    if !func_type.equal(&Type::Primary(PrimaryEnum::Void)) {
                        self.error_token(
                            debug,
                            format!("Expected {:?} return type got void", func_type).as_str()
                        );
                    }
                    return;
                };

                let expr_id = self.expression(expr);
                let Some(expr_type) = self.type_map.get(&expr_id) else {
                    unreachable!();
                };

                let Some(func_type) = &self.last_func_def else {
                    unreachable!()
                };

                if !func_type.equal(&expr_type) {
                    self.error_token(
                        debug,
                        format!("Expected {:?} return type got {:?}", func_type, expr_type).as_str()
                    );
                }
            },

            Expression(expr) => { self.expression(expr); },
            VarDecl{type_spec, decl, init} => {
                let (type_id, identifier) = self.declaration(decl, type_spec);
                self.variable_definition(type_spec, init, type_id, identifier);
            }
        }
    }

    fn declaration(&mut self, decl: &Decl<'a>, type_spec: &Token<'a>) -> (TypeID, String) {
        use DeclKind::*;

        let type_id = decl.id.clone();
        match &decl.kind {
            Group(decl) => { self.declaration(decl, type_spec) },
            Pointer(decl) => { self.declaration(decl, type_spec) },

            Identifier(tok) => {
                self.debug_info(type_id.clone(), tok);
                (type_id, tok.lexeme.to_string())
            },

            Function{decl, params} => {
                let (id, identifier) = self.declaration(decl, type_spec);
                self.function_definition(
                    Type::primitive(type_spec),  // TODO: make type recursive
                    params,
                    type_id,
                    identifier.clone()
                );
                (id, identifier)
            },
            Array{decl, constant} => {
                let id = self.declaration(decl, type_spec);
                self.array_definition(
                    Type::primitive(type_spec),
                    constant,
                    type_id
                );
                id
            },
        }
    }

    fn expression(&mut self, expr: &Expr<'a>) -> TypeID {
        use ExprKind::*;

        let type_id = expr.id.clone();
        match &expr.kind {
            IntLiteral{debug, ..} => {
                self.debug_info(type_id.clone(), debug);
                self.type_map.insert(type_id.clone(), Type::Primary(PrimaryEnum::Int)); 
                type_id
            },
            FloatLiteral{debug, ..} => {
                self.debug_info(type_id.clone(), debug);
                self.type_map.insert(type_id.clone(), Type::Primary(PrimaryEnum::Float)); 
                type_id
            },
            BoolLiteral{debug, ..} => {
                self.debug_info(type_id.clone(), debug);
                self.type_map.insert(type_id.clone(), Type::Primary(PrimaryEnum::Bool)); 
                type_id
            },
            StringLiteral{debug, ..} => {
                self.debug_info(type_id.clone(), debug);
                self.type_map.insert(type_id.clone(), Type::Primary(PrimaryEnum::String)); 
                type_id
            },
            Identifier(tok) => { 
                self.debug_info(type_id.clone(), tok);
                let Some((var_id, depth)) = self.var_map.get(&tok.lexeme.to_string()) else {
                    self.error_token(
                        tok,
                        format!("Can't lookup non-existing variable named '{}'", tok.lexeme).as_str()
                    );
                    return type_id;
                };
                if *depth > self.depth {
                    self.error_token(
                        tok,
                        format!("Trying lookup inaccessible variable '{}'", tok.lexeme).as_str()
                    );
                    return type_id;
                }
                var_id.clone()
            },

            Call{identifier, arguments} => {
                let identifier_id = self.expression(identifier);
                self.parameters_compare(identifier_id.clone(), arguments);
                identifier_id
            },
            Index{identifier, argument} => {
                let id = self.expression(argument);
                if let Some(spec) = self.type_map.get(&id) {
                    match spec {
                        Type::Primary(PrimaryEnum::Int) => {},
                        _ => self.error(&id, "Wrong type used, expected int"),
                    }
                }
                self.expression(identifier)
            },

            UnaryOp{op, val} => {
                let id = self.expression(val);
                if let Some(spec) = self.type_map.get(&id) {
                    match spec {
                        Type::Primary(PrimaryEnum::Bool) => {},
                        _ => self.error_unary(op, &id),
                    }
                }
                id
            },
            BinaryOp{l, op, r} => {
                let l_id = self.expression(l);
                let r_id = self.expression(r);

                if let Some(l_type) = self.type_map.get(&l_id) &&
                    let Some(r_type) = self.type_map.get(&r_id) && !l_type.equal(r_type) {
                        self.error_binary(&l_id, op, &r_id);
                }

                l_id
            },
            LogicalOp{l, op, r} => {
                let l_id = self.expression(l);
                let r_id = self.expression(r);

                if let Some(l_type) = self.type_map.get(&l_id) &&
                    let Some(r_type) = self.type_map.get(&r_id) && !l_type.equal(r_type) {
                        self.error_binary(&l_id, op, &r_id);
                }

                l_id
            },
            Assignment{l, op, r} => {
                let l_id = self.expression(l);
                let r_id = self.expression(r);

                if let Some(l_type) = self.type_map.get(&l_id) &&
                    let Some(r_type) = self.type_map.get(&r_id) && !l_type.equal(r_type) {
                        self.error_binary(&l_id, op, &r_id);
                }

                l_id
            },

            Group(expr) => {
                self.expression(expr)
            },
        }
    }
}
