use std::fmt;
use std::fmt::{Display, Formatter};

use crate::object::LoxObject;
use crate::token::{self, Token};

#[derive(Debug)]
pub enum Interruption {
    LexerError {
        lexeme: String,
        line: usize,
        position: usize,
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
    Return {
        object: LoxObject,
    },
}

impl Display for Interruption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError {
                lexeme,
                line,
                position,
                message,
            } => f.write_fmt(format_args!(
                "[{}:{}] Lexer error. \"{}\": {}",
                line, position, lexeme, message
            )),
            Self::ParserError { token, message } => f.write_fmt(format_args!(
                "[{}:{}] Parser error. \"{}\": {}",
                token.line, token.position, token.lexeme, message
            )),
            Interruption::RuntimeError { message } => {
                f.write_fmt(format_args!("Runtime error. {}", message))
            }
            Interruption::Break => f.write_str("#break"),
            Interruption::Return { object } => f.write_fmt(format_args!("#return {:?}", object,)),
        }
    }
}

impl std::error::Error for Interruption {}

pub fn lexer_error(lexeme: String, line: usize, position: usize, message: String) -> Interruption {
    Interruption::LexerError {
        lexeme,
        line,
        position,
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

pub fn retun_inter(object: LoxObject) -> Interruption {
    Interruption::Return { object }
}
