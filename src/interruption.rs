use std::fmt;
use std::fmt::{Display, Formatter};

use crate::span::Span;
use crate::token::Token;

#[derive(Debug)]
pub enum LoxError {
    Syntax { message: String, token: Token },
    Resolver { message: String, span: Option<Span> },
    Runtime { message: String, span: Option<Span> },
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoxError::Runtime { message, span } => {
                write!(f, "{}", message)?;
                if let Some(span) = span {
                    write!(f, "[line {}]", span.line)
                } else {
                    Ok(())
                }
            }
            LoxError::Resolver { message, span } => {
                write!(f, "{}", message)?;
                if let Some(span) = span {
                    write!(f, "[line {}]", span.line)
                } else {
                    Ok(())
                }
            }
            LoxError::Syntax {
                message,
                token:
                    Token {
                        token_type: _token_type,
                        lexeme,
                        span,
                    },
            } => {
                write!(f, "[line {}] Error at {}: {}", span.line, lexeme, message)
            }
        }
    }
}

impl std::error::Error for LoxError {}

pub fn new_runtime_error(message: String) -> LoxError {
    LoxError::Runtime {
        message,
        span: None,
    }
}

pub fn new_syntax_error(message: String, token: Token) -> LoxError {
    LoxError::Syntax { message, token }
}

pub fn new_resolver_error(message: String) -> LoxError {
    LoxError::Resolver {
        message,
        span: None,
    }
}
