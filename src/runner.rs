use std::error::Error;

use crate::{
    interpreter::Interpreter, interruption::LoxError, parser::parse_program, resolver::Resolver,
    scanner::scan_tokens,
};

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Option<LoxError> {
        let tokens = scan_tokens(source);
        // println!("{:#?}", tokens);
        match parse_program(tokens) {
            Ok(mut program) => {
                //println!("{:#?}", program);
                let mut resolver = Resolver::new();
                if let Err(error) = resolver.resolve_program(&mut program) {
                    self.report(&error);
                    return Some(error);
                }

                if let Err(error) = self.interpreter.exec(&program) {
                    self.report(&error);
                    Some(error)
                } else {
                    None
                }
            }
            Err(errors) => {
                for error in &errors {
                    self.report(error);
                }
                Some(errors[0].clone())
            }
        }
    }

    fn report(&mut self, error: impl Error) {
        eprintln!("{}", error);
    }
}
