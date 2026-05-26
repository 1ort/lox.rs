use core::fmt;
use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::Statement,
    environment::{EnvRef, Environment},
    interruption::Interruption,
    object::ObjRef,
};

#[derive(Debug, Clone)]
pub enum Function {
    Native {
        identifier: String,
        arity: u8,
        callable: fn(&[ObjRef]) -> Result<ObjRef, Interruption>,
    },
    Defined {
        name: String,
        parameters: Vec<String>,
        code_block: Rc<Statement>,
        closure: EnvRef,
        is_initializer: bool,
    },
}

impl Function {
    pub fn name(&self) -> String {
        match self {
            Self::Native { identifier, .. } => identifier,
            Self::Defined { name, .. } => name,
        }
        .clone()
    }
    pub fn arity(&self) -> u8 {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::Defined { parameters, .. } => parameters.len() as u8,
        }
    }
    pub fn bind(&self, obj: &ObjRef) -> Self {
        match self {
            Function::Native { .. } => unreachable!(),
            Function::Defined {
                name,
                parameters,
                code_block,
                closure,
                is_initializer,
            } => {
                let mut new_env = Environment::new_local(Rc::clone(closure));
                new_env.define("this".to_owned(), obj.clone());

                Function::Defined {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    code_block: code_block.clone(),
                    closure: Rc::new(RefCell::new(new_env)),
                    is_initializer: *is_initializer,
                }
            }
        }
    }
}
impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { identifier, .. } => {
                f.write_fmt(format_args!("function '{}'", identifier))
            }
            Function::Defined { name, .. } => f.write_fmt(format_args!("function '{}'", name)),
        }
    }
}
