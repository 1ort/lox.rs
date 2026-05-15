use std::collections::HashMap;

use crate::object::LoxObject;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LoxObject) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &String) -> Result<&LoxObject, String> {
        if let Some(value) = self.values.get(name) {
            return Ok(value);
        }
        match self.enclosing.as_mut() {
            Some(env) => env.get(name),
            None => Err(format!("Undefined variable: {} .", name)),
        }
    }

    pub fn assign(&mut self, name: &String, value: LoxObject) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.clone(), value);
            return Ok(());
        }
        match self.enclosing.as_mut() {
            Some(env) => env.assign(name, value),
            None => Err(format!("Undefined variable: {} .", name)),
        }
    }
}
