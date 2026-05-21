use std::error::Error;

use crate::{interpreter::Interpreter, parser::parse_program, scanner};

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Option<Box<String>> {
        let tokens = scanner::scan_tokens(source.to_string());
        match tokens {
            Ok(tokens) => {
                //println!("{:#?}", tokens);
                match parse_program(tokens) {
                    Ok(program) => {
                        //println!("{:#?}", program);
                        if let Err(err) = self.interpreter.exec(&program) {
                            self.error(0, &err);
                            return Some(Box::new(err));
                        } else {
                            return None;
                        }
                    }
                    Err(err) => {
                        self.error(0, &err);

                        return Some(Box::new(err));
                    }
                }
            }
            Err(error) => {
                self.report(&error);
                return Some(Box::new(error.to_string()));
            }
        }
    }

    fn error(&mut self, line: usize, message: &str) {
        println!("[line {}] Error {}", line, message);
    }

    fn report(&mut self, error: impl Error) {
        println!("{}", error);
    }
}
