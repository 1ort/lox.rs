use std::error::Error;

use crate::{
    interpreter::Interpreter, interruption::Interruption, parser::parse_program,
    resolver::Resolver, scanner,
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

    pub fn run(&mut self, source: &str) -> Option<Interruption> {
        let tokens = scanner::scan_tokens(source);
        match tokens {
            Ok(tokens) => {
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
                    Err(error) => {
                        self.report(&error);
                        Some(error)
                    }
                }
            }
            Err(error) => {
                self.report(&error);
                Some(error)
            }
        }
    }

    fn report(&mut self, error: impl Error) {
        eprintln!("{}", error);
    }
}
