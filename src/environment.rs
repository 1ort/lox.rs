use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interruption::{Interruption, runtime_error},
    object::LoxObject,
};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    pub enclosing: Option<EnvRef>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new_global() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_local(enclosing: EnvRef) -> Self {
        Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LoxObject) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Result<LoxObject, Interruption> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow().get(name)
            } else {
                Err(runtime_error(format!("Undefined variable: {} .", name)))
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: &String) -> Result<LoxObject, Interruption> {
        if distance == 0
            && let Some(value) = self.values.get(name)
        {
            Ok(value.clone())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.borrow().get_at(distance - 1, name)
        } else {
            Err(runtime_error(format!("Undefined variable: {} .", name)))
        }
    }

    pub fn assign(&mut self, name: &String, value: LoxObject) -> Result<(), Interruption> {
        if self.values.contains_key(name) {
            self.values.insert(name.clone(), value);
            Ok(())
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow_mut().assign(name, value)
            } else {
                Err(runtime_error(format!("Undefined variable: {} .", name)))
            }
        }
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &String,
        value: LoxObject,
    ) -> Result<(), Interruption> {
        if distance == 0 && self.values.contains_key(name) {
            self.values.insert(name.clone(), value);
            Ok(())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.borrow_mut().assign_at(distance - 1, name, value)
        } else {
            Err(runtime_error(format!("Undefined variable: {} .", name)))
        }
    }
}
