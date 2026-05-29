use std::error::Error;

use crate::{
    interpreter::Interpreter, parser::parse_program, resolver::Resolver, scanner::scan_tokens,
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

    pub fn run(&mut self, source: &str) -> i32 {
        let tokens = scan_tokens(source);
        // println!("{:#?}", tokens);
        match parse_program(tokens) {
            Ok(mut program) => {
                //println!("{:#?}", program);
                let mut resolver = Resolver::new();
                if let Err(error) = resolver.resolve_program(&mut program) {
                    self.report(&error);
                    return 65;
                }

                if let Err(error) = self.interpreter.exec(&program) {
                    self.report(&error);
                    return 70;
                } else {
                    return 0;
                }
            }
            Err(errors) => {
                for error in &errors {
                    self.report(error);
                }
                65
            }
        }
    }

    fn report(&mut self, error: impl Error) {
        eprintln!("{}", error);
    }
}
