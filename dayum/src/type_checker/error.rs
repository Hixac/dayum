use crate::{lexer::Token, type_checker::annotation::{Type, TypeID}};


enum Error<'a> {
    FunctionRedefenition{at: TypeID, ctx: TypeID},
    WrongNumberOfArgs{at: TypeID, ctx: TypeID},
    ExpectedType{at: TypeID, ctx: TypeID},
    ExpectedConcreteType{kind: Type, ctx: TypeID},
    VariableShadowing{token: Token<'a>, ctx: TypeID},
    WrongVariableDefine{token: Token<'a>, ctx: TypeID},
    WrongReturnType{kind: Type, ctx: TypeID},
}
