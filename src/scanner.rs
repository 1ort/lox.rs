use std::iter::Peekable;
use std::str::Chars;

use crate::span::Span;
use crate::token::{Token, TokenType};

pub fn scan_tokens(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer {
        source: source.chars().peekable(),
        line: 1,
        col: 1,
        pos: 0,
    };

    loop {
        let next_token = lexer.lex();
        if next_token.token_type == TokenType::Eof {
            tokens.push(next_token);
            break;
        } else {
            tokens.push(next_token);
        }
    }

    tokens
}

type Source<'a> = Peekable<Chars<'a>>;

struct Lexer<'a> {
    source: Source<'a>,
    line: usize,
    col: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn span(&self, len: usize) -> Span {
        Span {
            line: self.line,
            col: self.col,
            pos: self.pos,
            len,
        }
    }

    fn lex(self: &mut Lexer<'a>) -> Token {
        if let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.skip_spaces();
                self.lex()
            } else if c.is_ascii_digit() {
                self.lex_number()
            } else if c == &'"' {
                self.lex_string()
            } else if c.is_ascii_alphanumeric() || matches!(c, '_') {
                self.lex_keyword_or_identifier()
            } else {
                self.lex_symbol()
            }
        } else {
            Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                span: self.span(0),
            }
        }
    }

    fn lex_number(self: &mut Lexer<'a>) -> Token {
        let mut span = self.span(0);
        let mut buff = self.take_till(|c| c.is_ascii_digit());
        if self.peek() == Some(&'.') {
            buff.push(self.next().unwrap());
            let fract = self.take_till(|c| c.is_ascii_digit());
            if fract.is_empty() {
                span.len = buff.len();
                return Token {
                    token_type: TokenType::Unexpected(
                        "Invalid number. Fractional part expected.".to_string(),
                    ),
                    lexeme: buff,
                    span,
                };
            }
            buff.push_str(&fract);
        }

        span.len = buff.len();
        Token {
            token_type: TokenType::Number(buff.parse().unwrap()),
            lexeme: buff,
            span,
        }
    }

    fn lex_string(self: &mut Lexer<'a>) -> Token {
        let mut span = self.span(0);
        let mut buff = String::new();
        buff.push(self.next().unwrap());
        let content = &self.take_till(|c| c.ne(&'"'));
        buff.push_str(content);

        let token_type = if self.match_next('"') {
            buff.push('"');
            TokenType::String(content.clone())
        } else {
            TokenType::Unexpected("expect closing '\"' for string.".to_string())
        };
        span.len = buff.len();
        Token {
            token_type,
            lexeme: buff,
            span,
        }
    }

    fn lex_keyword_or_identifier(self: &mut Lexer<'a>) -> Token {
        let mut span = self.span(0);
        let buff = self.take_till(|c| c.is_ascii_alphanumeric() || matches!(c, '_'));
        span.len = buff.len();
        Token {
            token_type: match buff.as_str() {
                "print" => TokenType::Print,
                "var" => TokenType::Var,
                "and" => TokenType::And,
                "class" => TokenType::Class,
                "else" => TokenType::Else,
                "false" => TokenType::False,
                "fun" => TokenType::Fun,
                "for" => TokenType::For,
                "if" => TokenType::If,
                "nil" => TokenType::Nil,
                "or" => TokenType::Or,
                "return" => TokenType::Return,
                "super" => TokenType::Super,
                "this" => TokenType::This,
                "true" => TokenType::True,
                "while" => TokenType::While,
                "break" => TokenType::Break,
                _ => TokenType::Identifier(buff.clone()),
            },
            lexeme: buff,
            span,
        }
    }

    fn lex_symbol(self: &mut Lexer<'a>) -> Token {
        let mut span = self.span(1);
        let c = self.next().unwrap();

        let mut lexeme = None;

        let token_type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,
            '=' => {
                if self.match_next('=') {
                    lexeme = Some("==");
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '<' => {
                if self.match_next('=') {
                    lexeme = Some("<=");
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_next('=') {
                    lexeme = Some(">=");
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '!' => {
                if self.match_next('=') {
                    lexeme = Some("!=");
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '/' => {
                if self.match_next('/') {
                    self.skip_till(|c| !c.eq(&'\n'));
                    return self.lex();
                } else {
                    TokenType::Slash
                }
            }
            _ => TokenType::Unknown,
        };

        let lexeme = match lexeme {
            Some(lexeme) => lexeme.to_owned(),
            None => c.to_string(),
        };
        span.len = lexeme.len();

        Token {
            token_type,
            lexeme,
            span,
        }
    }

    fn take_till(self: &mut Lexer<'a>, till: impl Fn(char) -> bool) -> String {
        let mut buff = String::new();
        while let Some(c) = self.peek() {
            if !till(*c) {
                break;
            }
            buff.push(*c);
            self.next();
        }
        buff
    }

    fn skip_spaces(self: &mut Lexer<'a>) {
        self.skip_till(|c| c.is_whitespace());
    }

    fn skip_till(self: &mut Lexer<'a>, till: impl Fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if !till(*c) {
                break;
            }
            if c == &'\n' {
                self.line += 1;
                self.col = 1;
                self.pos += 1;
            }
            self.next();
        }
    }

    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.source.next() {
            self.pos += 1;
            self.col += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn match_next(self: &mut Lexer<'a>, expected: char) -> bool {
        if let Some(next) = self.peek() {
            if *next == expected {
                self.next();
                return true;
            }
            return false;
        }
        false
    }
}
