use dayum::lexer::scan::Scanner;
use dayum::lexer::TokenType;

#[cfg(test)]
fn it_works() {
    let mut scanner = Scanner::new("1 2.32 3 - == != \n>= // string \"string\" struct");

    let integer_literal = scanner.next().unwrap();
    assert_eq!(integer_literal.token_type, TokenType::IntegerLiteral);
    assert_eq!(integer_literal.lexeme, "1");

    let float_literal = scanner.next().unwrap();
    assert_eq!(float_literal.token_type, TokenType::FloatLiteral);
    assert_eq!(float_literal.lexeme, "2.32");

    let integer_literal = scanner.next().unwrap();
    assert_eq!(integer_literal.token_type, TokenType::IntegerLiteral);
    assert_eq!(integer_literal.lexeme, "3");

    let minus = scanner.next().unwrap();
    assert_eq!(minus.token_type, TokenType::OpMinus);
    assert_eq!(minus.lexeme, "-");

    let equal_equal = scanner.next().unwrap();
    assert_eq!(equal_equal.token_type, TokenType::OpEqualEqual);
    assert_eq!(equal_equal.lexeme, "==");

    let op_not_eq = scanner.next().unwrap();
    assert_eq!(op_not_eq.token_type, TokenType::OpNotEqual);
    assert_eq!(op_not_eq.lexeme, "!=");
    assert_eq!(op_not_eq.line, 0);

    let greater_equal = scanner.next().unwrap();
    assert_eq!(greater_equal.token_type, TokenType::OpGreaterEqual);
    assert_eq!(greater_equal.lexeme, ">=");
    assert_eq!(greater_equal.line, 1);

    let slash_slash = scanner.next().unwrap();
    assert_eq!(slash_slash.token_type, TokenType::SlashSlash);
    assert_eq!(slash_slash.lexeme, "//");

    let identifier = scanner.next().unwrap();
    assert_eq!(identifier.token_type, TokenType::Identifier);
    assert_eq!(identifier.lexeme, "string");

    let string = scanner.next().unwrap();
    assert_eq!(string.token_type, TokenType::StringLiteral);
    assert_eq!(string.lexeme, "\"string\"");

    let kwstruct = scanner.next().unwrap();
    assert_eq!(kwstruct.token_type, TokenType::KwStruct);
    assert_eq!(kwstruct.lexeme, "struct");

    let eof = scanner.next().unwrap();
    assert_eq!(eof.token_type, TokenType::EndOfFile);

    let end = scanner.next();
    assert_eq!(end.is_none(), true);
}
