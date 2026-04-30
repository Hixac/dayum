pub mod scan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // ----- Type keywords -----
    KwInt,
    KwChar,
    KwString,
    KwFloat,
    KwVoid,
    KwBool,
    KwTrue,
    KwFalse,
    KwStruct,     // optional, but common for "with types"

    // ----- Other keywords -----
    KwIf,
    KwElse,
    KwWhile,
    KwReturn,
    KwBreak,
    KwContinue,

    // ----- Literals & names -----
    Identifier,       // e.g., hello
    IntegerLiteral,   // e.g., 42
    FloatLiteral,     // e.g., 42.0
    StringLiteral,    // e.g., "hello"

    // ----- Operators (single char) -----
    OpPlus,      // +
    OpMinus,     // -
    OpStar,      // *
    OpSlash,     // /
    OpPercent,   // %
    OpEqual,     // =
    OpLess,      // <
    OpGreater,   // >
    OpExclaim,   // !
    OpAmp,       // & (address‑of)

    // ----- Two‑character operators -----
    OpEqualEqual,   // ==
    OpNotEqual,     // !=
    OpLessEqual,    // <=
    OpGreaterEqual, // >=
    OpLogAnd,       // &&
    OpLogOr,        // ||

    SlashSlash,     // //

    // ----- Punctuation -----
    Semicolon,   // ;
    Comma,       // ,
    Lparen,      // (
    Rparen,      // )
    Lbrace,      // {
    Rbrace,      // }
    Lbracket,    // [
    Rbracket,    // ]

    // ----- Special -----
    EndOfFile,
    Error,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub pos: usize,
    pub line: u16,
    pub token_type: TokenType,
    pub lexeme: &'a str
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token: (pos: {}, token_type: {:?}, line: {}, lexeme: {})", 
            self.pos, self.token_type, self.line, self.lexeme)
    }
}
