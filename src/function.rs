use crate::{ast::Statement, interruption::Interruption, object::LoxObject};

#[derive(Debug, Clone)]
pub enum Function {
    Native {
        identifier: String,
        arity: u8,
        callable: fn(&[LoxObject]) -> Result<LoxObject, Interruption>,
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
}
