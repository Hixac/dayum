use crate::lexer::Token;
use crate::type_checker::annotation::TypeID;


#[derive(Debug)]
pub enum DeclKind<'a> {
    Group(Box<Decl<'a>>),
    Pointer(Box<Decl<'a>>),

    Identifier(Token<'a>),

    Function{decl: Box<Decl<'a>>, params: Vec<Param<'a>>},
    Array{decl: Box<Decl<'a>>, constant: Option<Token<'a>>},
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    IntLiteral{debug: Token<'a>, val: i32},
    FloatLiteral{debug: Token<'a>, val: f32},
    BoolLiteral{debug: Token<'a>, val: bool},
    StringLiteral{debug: Token<'a>, val: &'a str},
    Identifier(Token<'a>),

    Call{identifier: Box<Expr<'a>>, arguments: Vec<Expr<'a>>},
    Index{identifier: Box<Expr<'a>>, argument: Box<Expr<'a>>},

    UnaryOp{op: Token<'a>, val: Box<Expr<'a>>},
    BinaryOp{l: Box<Expr<'a>>, op: Token<'a>, r: Box<Expr<'a>> },
    LogicalOp{l: Box<Expr<'a>>, op: Token<'a>, r: Box<Expr<'a>> },
    Assignment{l: Box<Expr<'a>>, op: Token<'a>, r: Box<Expr<'a>> },

    Group(Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum StmtKind<'a> {
    If { 
        debug: Token<'a>,
        cond: Expr<'a>,
        stmt: Box<Stmt<'a>>,
        otherwise: Option<Box<Stmt<'a>>>
    },
    While {
        debug: Token<'a>,
        cond: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
    For {
        debug: Token<'a>,
        decl: Box<Stmt<'a>>,
        cond: Expr<'a>,
        incr: Expr<'a>,
        body: Box<Stmt<'a>>,
    },

    Compound(Vec<Stmt<'a>>),
    Return{debug: Token<'a>, expr: Option<Expr<'a>>},

    Expression(Expr<'a>),
    VarDecl {
        type_spec: Token<'a>,
        decl: Decl<'a>,
        init: Option<Expr<'a>>
    }
}

#[derive(Debug)]
pub enum TopLevelStmtKind<'a> {
    FunctionDefinition {
        type_spec: Token<'a>,
        decl: Decl<'a>,
        params: Vec<Param<'a>>,
        body: Option<Stmt<'a>>
    },

    GlobalVariable {
        type_spec: Token<'a>,
        decl: Decl<'a>,
        init: Option<Expr<'a>>
    }
}

#[derive(Debug)]
pub struct TopLevelStmt<'a> {
    pub kind: TopLevelStmtKind<'a>,
    pub id: TypeID
}

#[derive(Debug)]
pub struct Stmt<'a> {
    pub kind: StmtKind<'a>,
    pub id: TypeID
}

#[derive(Debug)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub id: TypeID
}

#[derive(Debug)]
pub struct Decl<'a> {
    pub kind: DeclKind<'a>,
    pub id: TypeID,
}

#[derive(Debug)]
pub struct Param<'a> {
    pub type_spec: Token<'a>,
    pub decl: Option<Decl<'a>>,
    pub init: Option<Expr<'a>>,
    pub id: TypeID,
}
