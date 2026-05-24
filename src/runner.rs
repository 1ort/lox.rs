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

    pub fn run(&mut self, source: &str) -> Option<Box<Interruption>> {
        let tokens = scanner::scan_tokens(source);
        match tokens {
            Ok(tokens) => {
                // println!("{:#?}", tokens);
                match parse_program(tokens) {
                    Ok(program) => {
                        //println!("{:#?}", program);
                        let mut resolver = Resolver::new();
                        if let Err(error) = resolver.resolve_program(&program) {
                            self.report(&error);
                            return Some(Box::new(error));
                        }

                        if let Err(error) = self.interpreter.exec(&program) {
                            self.report(&error);
                            Some(Box::new(error))
                        } else {
                            None
                        }
                    }
                    Err(error) => {
                        self.report(&error);
                        Some(Box::new(error))
                    }
                }
            }
            Err(error) => {
                self.report(&error);
                Some(Box::new(error))
            }
        }
    }

    fn report(&mut self, error: impl Error) {
        println!("{}", error);
    }
}
