use crate::{compile::token::Token, span::Span};
use std::fmt::{self, Display, Formatter};

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
        write!(f, "{:?}", self)
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
