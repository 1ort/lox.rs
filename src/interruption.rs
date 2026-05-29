use std::fmt;
use std::fmt::{Display, Formatter};

use crate::object::LoxObject;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Interruption {
    ParserError { token: Token, message: String },
    ResolverError { message: String },
    RuntimeError { message: String },
    Break,
    Return { object: LoxObject },
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
            Interruption::Break => f.write_str("#break"),
            Interruption::Return { object } => f.write_fmt(format_args!("#return {:?}", object,)),
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
            Interruption::Break => 1,
            Interruption::Return { .. } => 1,
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

pub fn brake_inter() -> Interruption {
    Interruption::Break
}

pub fn retun_inter(object: LoxObject) -> Interruption {
    Interruption::Return { object }
}
