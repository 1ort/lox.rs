use std::fmt;
use std::fmt::{Display, Formatter};

use crate::span::Span;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum LoxError {
    Syntax { message: String, token: Token },
    Resolver { message: String, span: Option<Span> },
    Runtime { message: String, span: Option<Span> },
}

impl LoxError {
    pub fn with_span(mut self, new_span: Span) -> Self {
        match self {
            LoxError::Syntax {
                token: Token { ref mut span, .. },
                ..
            } => {
                *span = new_span;
            }
            LoxError::Resolver { ref mut span, .. } => {
                if span.is_none() {
                    *span = Some(new_span);
                }
            }
            LoxError::Runtime { ref mut span, .. } => {
                if span.is_none() {
                    *span = Some(new_span);
                }
            }
        }
        self
    }
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoxError::Runtime { message, span } => {
                write!(f, "{}", message)?;
                if let Some(span) = span {
                    write!(f, "[{}]", span.line)
                } else {
                    write!(f, "\n[line 0]")
                }
            }
            LoxError::Resolver { message, span } => {
                write!(f, "{}", message)?;
                if let Some(span) = span {
                    write!(f, "\n[line {}]", span.line)
                } else {
                    write!(f, "\n[line 0]")
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
                write!(f, "[{}] Error at '{}': {}", span.line, lexeme, message)
            }
        }
    }
}

impl std::error::Error for LoxError {}

pub fn new_runtime_error(message: String, span: Option<&Span>) -> LoxError {
    LoxError::Runtime {
        message,
        span: span.cloned(),
    }
}

pub fn new_syntax_error(message: String, token: Token) -> LoxError {
    LoxError::Syntax { message, token }
}

pub fn new_resolver_error(message: String, span: Option<&Span>) -> LoxError {
    LoxError::Resolver {
        message,
        span: span.cloned(),
    }
}
