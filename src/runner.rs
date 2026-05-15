use crate::{interpreter::Interpreter, parser::parse_program, scanner};

pub struct Lox {
    had_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            had_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: &str) {
        self.had_error = false; // reset error flag
        let tokens = scanner::scan_tokens(source.to_string());
        match tokens {
            Ok(tokens) => {
                //println!("{:#?}", tokens);
                match parse_program(tokens) {
                    Ok(program) => {
                        //println!("{:#?}", program);
                        if let Err(err) = self.interpreter.exec(&program) {
                            self.error(0, &err);
                        }
                    }
                    Err(err) => self.error(0, &err),
                }
            }
            Err(lexer_error) => {
                self.report(lexer_error.line, &lexer_error.lexeme, &lexer_error.message);
            }
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, _where: &str, message: &str) {
        println!("[line {}] Error {}: {}", line, _where, message);
        self.had_error = true;
    }
}
