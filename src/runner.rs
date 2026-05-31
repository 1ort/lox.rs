use crate::{
    compile::{
        error_reporter::ErrorReporter, parser::parse_program, resolver::Resolver,
        scanner::scan_tokens,
    },
    runtime::interpreter::Interpreter,
};
use std::error::Error;

pub struct Lox {
    interpreter: Interpreter,
}

pub enum ErrorKind {
    Input,
    Runtime,
}

struct BaseErrorReporter;

impl ErrorReporter for BaseErrorReporter {}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), ErrorKind> {
        let tokens = scan_tokens(source);
        let error_reporter = BaseErrorReporter;

        let mut program = parse_program(tokens, &error_reporter).map_err(|_| ErrorKind::Input)?;

        let mut resolver = Resolver::new();
        if let Err(error) = resolver.resolve_program(&mut program) {
            self.report(&error);
            return Err(ErrorKind::Input);
        }
        if let Err(error) = self.interpreter.exec(&program) {
            self.report(&error);
            Err(ErrorKind::Runtime)
        } else {
            Ok(())
        }
    }

    fn report(&mut self, error: impl Error) {
        eprintln!("{}", error);
    }
}
