use std::fmt;
use std::fmt::{Display, Formatter};

use crate::token::Token;

#[derive(Debug)]
pub enum Interruption {
    LexerError {
        lexeme: String,
        line: usize,
        message: String,
    },
    ParserError {
        token: Token,
        message: String,
    },
    RuntimeError {
        message: String,
    },
    Break,
}

impl Display for Interruption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError {
                lexeme,
                line,
                message,
            } => f.write_fmt(format_args!(
                "[line {}] Lexer error. \"{}\": {}",
                line, lexeme, message
            )),
            Self::ParserError { token, message } => f.write_fmt(format_args!(
                "[line {}] Parser error. \"{}\": {}",
                token.line, token.lexeme, message
            )),
            Interruption::RuntimeError { message } => {
                f.write_fmt(format_args!("Runtime error. {}", message))
            }
            Interruption::Break => f.write_str("#break"),
        }
    }
}

impl std::error::Error for Interruption {}

pub fn lexer_error(lexeme: String, line: usize, message: String) -> Interruption {
    Interruption::LexerError {
        lexeme,
        line,
        message,
    }
}

pub fn parser_error(token: Token, message: &str) -> Interruption {
    Interruption::ParserError {
        token,
        message: message.to_string(),
    }
}

pub fn runtime_error(message: String) -> Interruption {
    Interruption::RuntimeError { message }
}

pub fn brake_inter() -> Interruption {
    Interruption::Break
}
