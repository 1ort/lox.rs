use crate::{
    ast::Statement,
    environment::{self, Environment},
    interpreter::Interpreter,
    object::LoxObject,
};

#[derive(Debug, Clone)]
pub enum Function {
    Native {
        identifier: String,
        arity: u8,
        callable: fn(&[LoxObject]) -> Result<LoxObject, String>,
    },
    Defined {
        name: String,
        parameters: Vec<String>,
        code_block: Box<Statement>,
    },
}

impl Function {
    pub fn arity(&self) -> u8 {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::Defined { parameters, .. } => parameters.len() as u8,
        }
    }

    pub fn format(&self) -> String {
        match self {
            Function::Native { identifier, .. } => format!("function '{}'", identifier),
            Function::Defined { name, .. } => format!("function '{}'", name),
        }
    }

    pub fn call(&self, args: &[LoxObject], interpreter: &Interpreter) -> Result<LoxObject, String> {
        if self.arity() as usize != args.len() {
            return Err(format!(
                "{} takes {} arguments, but {} provided",
                self.format(),
                self.arity(),
                args.len()
            ));
        }
        match self {
            Function::Native { callable, .. } => Ok(callable(args)?),
            Function::Defined {
                name,
                parameters,
                code_block,
            } => {
                let mut environment = Environment::new();
                std::iter::zip(parameters, args)
                    .map(|(name, value)| environment.define(name.clone(), value.clone()))
                    .collect();

                interpreter.enter_environment(Box::new(environment));
                interpreter.exec_statement(code_block)?;
                interpreter.exit_environment();
                Ok(LoxObject::Nil)
            }
        }
    }
}
