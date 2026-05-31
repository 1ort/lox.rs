use crate::{
    compile::{parser::parse_program, resolver::Resolver, scanner::scan_tokens},
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

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), ErrorKind> {
        let tokens = scan_tokens(source);
        // println!("{:#?}", tokens);
        match parse_program(tokens) {
            Ok(mut program) => {
                //println!("{:#?}", program);
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
            Err(errors) => {
                for error in &errors {
                    self.report(error);
                }
                Err(ErrorKind::Input)
            }
        }
    }

    fn report(&mut self, error: impl Error) {
        eprintln!("{}", error);
    }
}
