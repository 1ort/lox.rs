use std::collections::HashMap;

use crate::object::LoxObject;

#[derive(Debug)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn set_enclosing(&mut self, enclosing: Option<Box<Environment>>) {
        self.enclosing = enclosing;
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
