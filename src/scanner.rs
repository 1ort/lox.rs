use std::iter::Peekable;
use std::str::Chars;

use crate::interruption::{Interruption, lexer_error};
use crate::token::{Token, TokenType};

type Source<'a> = Peekable<Chars<'a>>;

struct Lexer<'a> {
    source: Source<'a>,
    current_line: usize,
    current_position: usize,
}

pub fn scan_tokens(source: &str) -> Result<Vec<Token>, Interruption> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer {
        source: source.chars().peekable(),
        current_line: 1,
        current_position: 1,
    };

    while let Some(token) = lexer.lex()? {
        tokens.push(token);
    }
    tokens.push(Token {
        token_type: TokenType::Eof,
        lexeme: "".to_string(),
        line: lexer.current_line,
        position: lexer.current_position,
    });

    Ok(tokens)
}

impl<'a> Lexer<'a> {
    fn lex(self: &mut Lexer<'a>) -> Result<Option<Token>, Interruption> {
        let c = self.peek();
        if c.is_none() {
            return Ok(None);
        }
        let c = c.unwrap();

        if c.is_whitespace() {
            self.skip_spaces();
            self.lex()
        } else if c.is_ascii_digit() {
            Ok(Some(self.lex_number()?))
        } else if c == &'"' {
            Ok(Some(self.lex_string()?))
        } else if c.is_ascii_alphanumeric() || matches!(c, '_') {
            Ok(Some(self.lex_keyword_or_identifier()))
        } else {
            self.lex_symbol()
        }
    }

    fn lex_number(self: &mut Lexer<'a>) -> Result<Token, Interruption> {
        let mut buff = self.take_till(|c| c.is_ascii_digit());
        if self.peek() == Some(&'.') {
            buff.push(self.next().unwrap());
            let fract = self.take_till(|c| c.is_ascii_digit());
            if fract.is_empty() {
                return Err(lexer_error(
                    buff,
                    self.current_line,
                    self.current_position,
                    "Invalid number. Fractional part expected.".to_string(),
                ));
            }
            buff.push_str(&fract);
        }

        Ok(Token {
            token_type: TokenType::Number(buff.parse().unwrap()),
            lexeme: buff,
            line: self.current_line,
            position: self.current_position,
        })
    }

    fn lex_string(self: &mut Lexer<'a>) -> Result<Token, Interruption> {
        self.next().unwrap();
        let content = self.take_till(|c| c.ne(&'"'));

        if let Some('"') = self.next() {
            Ok(Token {
                token_type: TokenType::String(content.clone()),
                lexeme: format!("\"{content}\""),
                line: self.current_line,
                position: self.current_position,
            })
        } else {
            Err(lexer_error(
                format!("\"{content}"),
                self.current_line,
                self.current_position,
                "Unterminated string.".to_string(),
            ))
        }
    }

    fn lex_keyword_or_identifier(self: &mut Lexer<'a>) -> Token {
        let buff = self.take_till(|c| c.is_ascii_alphanumeric() || matches!(c, '_'));

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
            line: self.current_line,
            position: self.current_position,
        }
    }

    fn lex_symbol(self: &mut Lexer<'a>) -> Result<Option<Token>, Interruption> {
        let c = self.next().unwrap();

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
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '<' => {
                if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '!' => {
                if self.match_next('=') {
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
            _ => {
                return Err(lexer_error(
                    c.to_string(),
                    self.current_line,
                    self.current_position,
                    "Unexpected token".to_string(),
                ));
            }
        };

        Ok(Some(Token {
            token_type,
            lexeme: c.to_string(),
            line: self.current_line,
            position: self.current_position,
        }))
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
                self.current_line += 1;
                self.current_position = 1;
            }
            self.next();
        }
    }

    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.source.next() {
            self.current_position += 1;
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
