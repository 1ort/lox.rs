use std::collections::HashMap;

use crate::object::LoxObject;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LoxObject) -> () {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &String) -> Result<&LoxObject, String> {
        // println!("{:?}", self.values);
        self.values
            .get(name)
            .ok_or(format!("Undefined variable: {} .", name))
    }

    pub fn assign(&mut self, name: &String, value: LoxObject) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.clone(), value);
            Ok(())
        } else {
            Err(format!("Undefined variable: {} .", name))
        }
    }
}
