use std::fmt;
use std::fmt::{Display, Formatter};

use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Interruption {
    ParserError { token: Token, message: String },
    ResolverError { message: String },
    RuntimeError { message: String },
}

impl Display for Interruption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParserError {
                token: Token { lexeme, span, .. },
                message,
            } => f.write_fmt(format_args!(
                "[{}:{}] Parser error. \"{}\": {}",
                span.line, span.col, lexeme, message
            )),
            Interruption::ResolverError { message } => {
                f.write_fmt(format_args!("Resolver error. {}", message))
            }
            Interruption::RuntimeError { message } => {
                f.write_fmt(format_args!("Runtime error. {}", message))
            }
        }
    }
}

impl std::error::Error for Interruption {}

impl Interruption {
    pub fn exit_code(&self) -> i32 {
        match self {
            Interruption::ParserError { .. } => 65,
            Interruption::ResolverError { .. } => 65,
            Interruption::RuntimeError { .. } => 70,
        }
    }
}

pub fn parser_error(token: Token, message: &str) -> Interruption {
    Interruption::ParserError {
        token,
        message: message.to_string(),
    }
}

pub fn resolver_error(message: String) -> Interruption {
    Interruption::ResolverError { message }
}

pub fn runtime_error(message: String) -> Interruption {
    Interruption::RuntimeError { message }
}
