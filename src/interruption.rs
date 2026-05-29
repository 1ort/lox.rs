use std::fmt;
use std::fmt::{Display, Formatter};

use crate::token::Token;

#[derive(Debug, Clone)]
pub enum LoxError {
    ParserError { token: Token, message: String },
    ResolverError { message: String },
    RuntimeError { message: String },
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParserError {
                token: Token { lexeme, span, .. },
                message,
            } => f.write_fmt(format_args!(
                "[{}:{}] Parser error. \"{}\": {}",
                span.line, span.col, lexeme, message
            )),
            LoxError::ResolverError { message } => {
                f.write_fmt(format_args!("Resolver error. {}", message))
            }
            LoxError::RuntimeError { message } => {
                f.write_fmt(format_args!("Runtime error. {}", message))
            }
        }
    }
}

impl std::error::Error for LoxError {}

impl LoxError {
    pub fn exit_code(&self) -> i32 {
        match self {
            LoxError::ParserError { .. } => 65,
            LoxError::ResolverError { .. } => 65,
            LoxError::RuntimeError { .. } => 70,
        }
    }
}

pub fn parser_error(token: Token, message: &str) -> LoxError {
    LoxError::ParserError {
        token,
        message: message.to_string(),
    }
}

pub fn resolver_error(message: String) -> LoxError {
    LoxError::ResolverError { message }
}

pub fn runtime_error(message: String) -> LoxError {
    LoxError::RuntimeError { message }
}
