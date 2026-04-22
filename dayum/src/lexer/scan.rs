use crate::lexer::{Token, TokenType};
use tracing::{error, info};


pub struct Scanner<'a> {
    source: &'a str,
    pos: usize,
    line: u16,
    is_eofed: bool
}


impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0, line: 0, is_eofed: false }
    }

    fn next_char(&mut self) -> Option<char> {
        self.pos += 1;
        self.source.chars().nth(self.pos - 1)
    }

    fn peek(&mut self) -> Option<char> {
        self.source.chars().nth(self.pos)
    }

    fn skip_whitespaces(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\r' | '\t' => {},
                '\n' => self.line += 1,
                _ => return
            }
            self.next_char().unwrap();
        }
    }

    fn make_token(&self, token_type: TokenType, start: usize) -> Token<'a> {
        let token = Token { pos: start, line: self.line, token_type, lexeme: &self.source[start..self.pos] };
        info!("{}", token);
        token
    }

    fn scan_number(&mut self) -> Option<TokenType> {
        let mut dot = 0;
        let mut toktype = TokenType::IntegerLiteral;
        while let Some(c) = self.peek() {
            if c == '.' {
                dot += 1;
                toktype = TokenType::FloatLiteral;
            }
            if dot > 1 {
                error!("Syntax error while lexing float literal");
                return None
            }
            if !c.is_numeric() && c != '.' {
                return Some(toktype)
            }

            self.next_char().unwrap();
        }

        Some(toktype)
    }

    fn scan_identifier(&mut self, start: usize) -> Option<TokenType> {
        while let Some(c) = self.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            self.next_char().unwrap();
        }

        match &self.source[start..self.pos] {
            "int" => Some(TokenType::KwInt),
            "char" => Some(TokenType::KwChar),
            "float" => Some(TokenType::KwFloat),
            "void" => Some(TokenType::KwVoid),
            "struct" => Some(TokenType::KwStruct),
            "if" => Some(TokenType::KwIf),
            "else" => Some(TokenType::KwElse),
            "while" => Some(TokenType::KwWhile),
            "return" => Some(TokenType::KwReturn),
            "break" => Some(TokenType::KwBreak),
            "continue" => Some(TokenType::KwContinue),
            "true" => Some(TokenType::KwTrue),
            "false" => Some(TokenType::KwFalse),
            _  => Some(TokenType::Identifier)
        }

    }

    fn scan_string(&mut self) -> Option<TokenType> {
        while let Some(c) = self.peek() {
            self.next_char().unwrap();
            if c == '"' {
                return Some(TokenType::StringLiteral);
            }
        }

        error!("String literal is not closed");
        None
    }

    fn match_and_move(&mut self, desired_type: TokenType, c: char) -> Option<TokenType> {
        if self.peek()? == c {
            self.next_char();
            return Some(desired_type);
        }
        None
    }

    fn scan_symbol(&mut self, c: char) -> Option<TokenType> {
        match c {
            '+' => Some(TokenType::OpPlus),
            '-' => Some(TokenType::OpMinus),
            '*' => Some(TokenType::OpStar),
            '%' => Some(TokenType::OpPercent),

            ';' => Some(TokenType::Semicolon),
            ',' => Some(TokenType::Comma),

            '(' => Some(TokenType::Lparen),
            ')' => Some(TokenType::Rparen),
            '{' => Some(TokenType::Lbrace),
            '}' => Some(TokenType::Rbrace),
            '[' => Some(TokenType::Lbracket),
            ']' => Some(TokenType::Rbracket),

            '=' => {
                self.match_and_move(TokenType::OpEqualEqual, '=').or(
                    Some(TokenType::OpEqual)
                )
            },
            '&' => {
                self.match_and_move(TokenType::OpLogAnd, '&').or(
                    Some(TokenType::OpAmp)
                )
            },
            '!' => {
                self.match_and_move(TokenType::OpNotEqual, '=').or(
                    Some(TokenType::OpExclaim)
                )
            },
            '<' => {
                self.match_and_move(TokenType::OpLessEqual, '=').or(
                    Some(TokenType::OpLess)
                )
            },
            '>' => {
                self.match_and_move(TokenType::OpGreaterEqual, '=').or(
                    Some(TokenType::OpGreater)
                )
            },
            '/' => {
                self.match_and_move(TokenType::SlashSlash, '/').or(
                    Some(TokenType::OpSlash)
                )
            },
            '|' => { self.match_and_move(TokenType::OpLogOr, '|') },

            _ => None
        }
    }

    fn eof(&mut self) -> Option<Token<'a>> {
        if self.is_eofed {
            return None
        }
        self.is_eofed = true;
        Some(self.make_token(TokenType::EndOfFile, self.pos))
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespaces();

        if self.pos == self.source.len() {
            return self.eof()
        }

        let start = self.pos;
        let c = self.next_char()?;

        if c.is_numeric() {
            let token_type = self.scan_number()?;
            return Some(self.make_token(token_type, start));
        }

        if c.is_alphabetic() {
            let token_type = self.scan_identifier(start)?;
            return Some(self.make_token(token_type, start));
        }

        if c == '"' {
            let token_type = self.scan_string()?;
            return Some(self.make_token(token_type, start));
        }

        let token_type = self.scan_symbol(c)?;
        return Some(self.make_token(token_type, start));
    }
}
