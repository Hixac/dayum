use std::collections::HashMap;

use crate::{
    lexer::{TokenType, Token},
    type_checker::annotation::{Type, TypeID, PrimaryEnum}
};

pub use chunk::*;
use crate::parser::ast::{
    Expr,
    ExprKind,
    Decl,
    DeclKind,
    Stmt,
    StmtKind,
    TopLevelStmt,
    TopLevelStmtKind
};

mod chunk;


pub struct Compiler {
    pub chunk: Chunk,
    depth: u16,
    type_map: HashMap<TypeID, Type>,
    var_map: HashMap<String, (TypeID, u16)>
}

impl<'a> Compiler {
    pub fn new(type_map: HashMap<TypeID, Type>, var_map: HashMap<String, (TypeID, u16)>) -> Self {
        Self {
            chunk: Chunk::default(),
            depth: 0,
            type_map,
            var_map
        }
    }

    pub fn compile(&mut self, stmts: Vec<TopLevelStmt<'a>>) {
        for stmt in stmts {
            self.walk(stmt);
        }
        self.emit_main_execution();
        self.chunk.emit(OpCode::Stop, 0);
    }

    fn walk(&mut self, stmt: TopLevelStmt<'a>) -> () {
        use TopLevelStmtKind::*;
        match stmt.kind {
            FunctionDefinition { type_spec: _, decl, params, body } => {
                let (tok, _) = self.declaration(decl);

                let mut fn_compiler = Compiler::new(self.type_map.clone(), self.var_map.clone());
                fn_compiler.depth = 1; // locals live at depth >= 1

                let param_len = params.len() as u16;
                for param in params {
                    if let Some(decl) = param.decl {
                        let (tok, _) = fn_compiler.declaration(decl);
                        fn_compiler.chunk.push_name(tok.lexeme.to_string(), 1);
                    }
                }

                fn_compiler.statement(body.unwrap());
                fn_compiler.chunk.emit(OpCode::Return, 0);

                let func = CompiledFunction {
                    name: tok.lexeme.to_string(),
                    arity: param_len,
                    chunk: fn_compiler.chunk,
                };

                let oper = self.chunk.push_name(func.name.clone(), 0);
                self.chunk.emit(OpCode::DefineGlobal, oper);
                let oper_const = self.chunk.push_constant(Value::Func(func));
                self.chunk.emit(OpCode::AllocPtr, oper_const);
                self.chunk.emit(OpCode::StoreGlobal, oper);
            },
            GlobalVariable { type_spec: _, decl, init } => {
                let (tok, _) = self.declaration(decl);
                let oper = self.chunk.push_name(tok.lexeme.to_string(), 0);
                self.chunk.emit(OpCode::DefineGlobal, oper);

                if let Some(init) = init {
                    self.expression(init);
                    self.chunk.emit(OpCode::StoreGlobal, oper);
                }
            }
        }
    }

    fn statement(&mut self, stmt: Stmt<'a>) -> () {
        use StmtKind::*;
        match stmt.kind {
            If {..} => {},
            While {..} => {},
            For {..} => {},

            Compound(body) => {
                self.depth += 1;
                for stmt in body {
                    self.statement(stmt);
                }
                self.depth -= 1;
            },
            Return{..} => {},

            Expression(expr) => { self.expression(expr); },
            VarDecl {..} => {},
        }
    }

    fn declaration(&mut self, decl: Decl<'a>) -> (Token<'a>, bool) {
        use DeclKind::*;
        match decl.kind {
            Group(decl) => {
                self.declaration(*decl)
            },
            Pointer(decl) => {
                (self.declaration(*decl).0, true)
            },

            Identifier(ident) => {
                (ident, false)
            },

            Function{decl, params} => {
                self.declaration(*decl)
            },
            Array{decl, constant} => {
                self.declaration(*decl)
            },
        }
    }

    fn expression(&mut self, expr: Expr<'a>) -> () {
        use ExprKind::*;
        match expr.kind {
            IntLiteral{debug: _, val} => {
                let oper = self.chunk.push_constant(Value::Int(val));
                self.chunk.emit(OpCode::LoadInt, oper);
            },
            FloatLiteral{debug: _, val} => {
                let oper = self.chunk.push_constant(Value::Float(val));
                self.chunk.emit(OpCode::LoadFloat, oper);
            },
            BoolLiteral{debug: _, val} => {
                let oper = self.chunk.push_constant(Value::Bool(val));
                self.chunk.emit(OpCode::LoadBool, oper);
            },
            StringLiteral{debug: _, val} => {
                let oper = self.chunk.push_constant(Value::Str(val.to_string()));
                self.chunk.emit(OpCode::LoadString, oper);
            },
            Identifier(tok) => {
                let oper = self.chunk.push_name(tok.lexeme.to_string(), self.depth);
                if let Some(var) = self.chunk.names.get(oper as usize) {
                    if var.depth == 0 {
                        self.chunk.emit(OpCode::LoadGlobal, oper);
                    }
                }
                self.chunk.emit(OpCode::LoadName, oper);
            },

            Call{identifier, arguments} => {
                let args_len = arguments.len() as u16;
                for arg in arguments {
                    self.expression(arg);
                }

                self.expression(*identifier);
                self.chunk.emit(OpCode::Call, args_len);
            },
            Index{identifier, argument} => {

            },

            UnaryOp{op, val} => {
                self.expression(*val);

                match op.token_type {
                    TokenType::OpExclaim => {
                        self.chunk.emit(OpCode::NotBool, 0);
                    },
                    _ => unreachable!()
                }
            },
            BinaryOp{l, op, r} => {
                self.expression(*l);
                self.expression(*r);

                let spec = self.type_map.get(&expr.id).unwrap();

                match (op.token_type, spec.as_primary()) {
                    (TokenType::OpPlus, PrimaryEnum::Int) => self.chunk.emit(OpCode::AddInt, 0),
                    (TokenType::OpPlus, PrimaryEnum::Float) => self.chunk.emit(OpCode::AddFloat, 0),
                    (TokenType::OpPlus, PrimaryEnum::String) => self.chunk.emit(OpCode::AddString, 0),

                    (TokenType::OpMinus, PrimaryEnum::Int) => self.chunk.emit(OpCode::SubInt, 0),
                    (TokenType::OpMinus, PrimaryEnum::Float) => self.chunk.emit(OpCode::SubFloat, 0),

                    (TokenType::OpStar, PrimaryEnum::Int) => self.chunk.emit(OpCode::MulInt, 0),
                    (TokenType::OpStar, PrimaryEnum::Float) => self.chunk.emit(OpCode::MulFloat, 0),

                    (TokenType::OpSlash, PrimaryEnum::Int) => self.chunk.emit(OpCode::DivInt, 0),
                    (TokenType::OpSlash, PrimaryEnum::Float) => self.chunk.emit(OpCode::DivFloat, 0),
                    _ => unreachable!()
                }

            },
            LogicalOp{l, op, r} => {
                self.expression(*l);
                self.expression(*r);

                match op.token_type {
                    TokenType::OpLogAnd => self.chunk.emit(OpCode::AndBool, 0),
                    TokenType::OpLogOr => self.chunk.emit(OpCode::OrBool, 0),
                    _ => unreachable!()
                }
            },
            Assignment{l, op, r} => {
                let oper = self.lvalue_expression(*l);
                self.expression(*r);


                match op.token_type {
                    TokenType::OpEqual => {
                        if self.is_global(oper) {
                            self.chunk.emit(OpCode::StoreGlobal, oper);
                            return;
                        }
                        self.chunk.emit(OpCode::StoreName, oper)
                    },
                    _ => unreachable!()
                }
            },

            Group(grp) => {
                self.expression(*grp);
            },
        }
    }

    fn lvalue_expression(&mut self, expr: Expr<'a>) -> u16 {
        use ExprKind::*;
        match expr.kind {
            Identifier(tok) => {
                let (_, depth) = self.var_map.get(tok.lexeme).unwrap();
                let oper = self.chunk.push_name(tok.lexeme.to_string(), depth.clone());
                oper
            }
            _ => panic!("Invalid kind")
        }
    }

    fn is_global(&self, idx: u16) -> bool {
        self.chunk.names[idx as usize].depth == 0
    }

    fn emit_main_execution(&mut self) -> () {
        let oper = self.chunk.push_name("main".to_string(), 0);
        self.chunk.emit(OpCode::LoadGlobal, oper);
        self.chunk.emit(OpCode::Call, 0);
    }
}
