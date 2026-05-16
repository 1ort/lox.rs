use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct LexerError {
    pub line: usize,
    pub lexeme: String,
    pub message: String,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[line {}] Lexer error: \"{}\": {}",
            self.line, self.message, self.lexeme
        )
    }
}

type LexerResult<T> = std::result::Result<T, LexerError>;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Eof,

    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    String(String),
    Number(f64),
    Identifier(String),
    // Keywords.
    Print,
    Var,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    While,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

type Source<'s> = Peekable<Chars<'s>>;

struct Lexer<'s> {
    source: Source<'s>,
    current_line: usize,
}

pub fn scan_tokens(source: String) -> LexerResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer {
        source: source.chars().peekable(),
        current_line: 0,
    };

    while let Some(token) = lexer.lex()? {
        tokens.push(token);
    }
    tokens.push(Token {
        token_type: TokenType::Eof,
        lexeme: "".to_string(),
        line: lexer.current_line,
    });

    Ok(tokens)
}

impl<'a> Lexer<'a> {
    fn lex(self: &mut Lexer<'a>) -> LexerResult<Option<Token>> {
        let c = self.source.peek();
        if c.is_none() {
            return Ok(None);
        }
        let c = c.unwrap();

        if c.is_whitespace() {
            if c == &'\n' {
                self.current_line += 1;
            }
            self.skip_spaces();
            self.lex()
        } else if c.is_ascii_digit() {
            Ok(Some(self.lex_number()?))
        } else if c == &'"' {
            Ok(Some(self.lex_string()?))
        } else if c.is_ascii_alphanumeric() {
            Ok(Some(self.lex_keyword_or_identifier()))
        } else {
            self.lex_symbol()
        }
    }

    fn lex_number(self: &mut Lexer<'a>) -> LexerResult<Token> {
        let mut buff = self.take_till(|c| c.is_ascii_digit());
        if self.source.peek() == Some(&'.') {
            buff.push(self.source.next().unwrap());
            let fract = self.take_till(|c| c.is_ascii_digit());
            if fract.is_empty() {
                return Err(LexerError {
                    lexeme: buff,
                    line: self.current_line,
                    message: "Invalid number. Fractional part expected.".to_string(),
                });
            }
            buff.push_str(&fract);
        }

        Ok(Token {
            token_type: TokenType::Number(buff.parse().unwrap()),
            lexeme: buff,
            line: self.current_line,
        })
    }

    fn lex_string(self: &mut Lexer<'a>) -> LexerResult<Token> {
        self.source.next().unwrap();
        let content = self.take_till(|c| c.ne(&'"'));

        if let Some('"') = self.source.next() {
            Ok(Token {
                token_type: TokenType::String(content.clone()),
                lexeme: format!("\"{content}\""),
                line: self.current_line,
            })
        } else {
            Err(LexerError {
                lexeme: format!("\"{content}"),
                line: self.current_line,
                message: "Unterminated string.".to_string(),
            })
        }
    }

    fn lex_keyword_or_identifier(self: &mut Lexer<'a>) -> Token {
        let buff = self.take_till(|c| c.is_ascii_alphanumeric());

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
                _ => TokenType::Identifier(buff.clone()),
            },
            lexeme: buff,
            line: self.current_line,
        }
    }

    fn lex_symbol(self: &mut Lexer<'a>) -> LexerResult<Option<Token>> {
        let c = self.source.next().unwrap();

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
                return Err(LexerError {
                    lexeme: c.to_string(),
                    line: self.current_line,
                    message: "Unexpected token".to_string(),
                });
            }
        };

        Ok(Some(Token {
            token_type,
            lexeme: c.to_string(),
            line: self.current_line,
        }))
    }

    fn take_till(self: &mut Lexer<'a>, till: impl Fn(char) -> bool) -> String {
        let mut buff = String::new();
        while let Some(c) = self.source.peek() {
            if !till(*c) {
                break;
            }
            buff.push(*c);
            self.source.next();
        }
        buff
    }

    fn skip_spaces(self: &mut Lexer<'a>) {
        self.skip_till(|c| c.is_whitespace());
    }

    fn skip_till(self: &mut Lexer<'a>, till: impl Fn(char) -> bool) {
        while let Some(c) = self.source.peek() {
            if !till(*c) {
                break;
            }
            self.source.next();
        }
    }

    fn match_next(self: &mut Lexer<'a>, expected: char) -> bool {
        if let Some(next) = self.source.peek() {
            if *next == expected {
                self.source.next();
                return true;
            }
            return false;
        }
        false
    }
}
