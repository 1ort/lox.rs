use crate::{
    compile::{
        error_reporter::ErrorReporter, parser::parse_program, resolver::resolve_program,
        scanner::scan_tokens,
    },
    error_reporter,
    runtime::interpreter::Interpreter,
};

pub struct Lox {
    interpreter: Interpreter,
}

pub enum ErrorKind {
    Input,
    Runtime,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), ErrorKind> {
        let error_reporter = error_reporter::ErrorReporter::new(source);

        let tokens = scan_tokens(source);
        let mut program = parse_program(&tokens, &error_reporter).map_err(|_| ErrorKind::Input)?;
        resolve_program(&mut program, &error_reporter).map_err(|_| ErrorKind::Input)?;
        if let Err(err) = self.interpreter.exec(&program) {
            error_reporter.report(&err);
            Err(ErrorKind::Runtime)
        } else {
            Ok(())
        }
    }
}
