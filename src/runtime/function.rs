use crate::{
    ast::Statement, interruption::LoxError, runtime::environment::Environment,
    runtime::object::LoxObject,
};
use core::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct NativeFunction {
    pub name: String,
    pub callable: fn(Vec<LoxObject>, &Environment) -> Result<LoxObject, LoxError>,
}

impl std::fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

#[derive(Debug)]
pub struct UserFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub code_block: Rc<Statement>,
    pub closure: Environment,
    pub is_initializer: bool,
    pub is_bound: bool,
}

impl UserFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arity(&self) -> u8 {
        self.parameters.len() as u8
    }

    pub fn is_bound(&self) -> bool {
        self.is_bound
    }

    pub fn bind(&self, obj: LoxObject) -> Self {
        let mut new_env = self.closure.enter_scope("bound".to_string());
        new_env.define("this".to_owned(), obj);

        UserFunction {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            code_block: self.code_block.clone(),
            closure: new_env,
            is_initializer: self.is_initializer,
            is_bound: true,
        }
    }
}

impl std::fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
