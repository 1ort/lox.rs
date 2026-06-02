use crate::{
    compile::token::{Token, TokenType},
    error::LoxError,
};

pub struct ErrorReporter<'a> {
    source: &'a str,
    index: Vec<usize>,
}

impl<'a> ErrorReporter<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            index: Self::build_index(source),
        }
    }

    fn build_index(source: &str) -> Vec<usize> {
        source
            .char_indices()
            .filter(|&(_, char)| char == '\n')
            .map(|(index, _)| index)
            .collect()
    }

    fn position_to_line(&self, position: usize) -> usize {
        self.index
            .iter()
            .enumerate()
            .skip_while(|(_, newline_pos)| **newline_pos < position)
            .map(|(line_num, _)| line_num)
            .next()
            .unwrap_or(self.index.len())
            + 1
    }
}

impl<'a> crate::compile::error_reporter::ErrorReporter for ErrorReporter<'a> {
    fn report(&self, err: &LoxError) {
        match err {
            LoxError::Runtime { message, span } => {
                eprintln!("{}", message);
                if let Some(span) = span {
                    let line = self.position_to_line(span.pos);
                    eprintln!("[line {}]", line)
                }
            }
            LoxError::Resolver { message, span } => {
                if let Some(span) = span {
                    let line = self.position_to_line(span.pos);
                    eprint!("[line {}] ", line);
                }
                eprintln!("{}", message)
            }
            LoxError::Syntax {
                message,
                token:
                    Token {
                        token_type,
                        lexeme,
                        span,
                    },
            } => {
                let line = self.position_to_line(span.pos);
                eprint!("[line {}]", line);
                match token_type {
                    TokenType::Eof => eprint!(" Error at end:"),
                    _ => {
                        if !lexeme.is_empty() {
                            eprint!(" Error at '{}':", lexeme,);
                        } else {
                            eprint!(" Error:");
                        }
                    }
                }
                eprintln!(" {}", message)
            }
        }
    }
}
