use std::{cell::RefCell, collections::HashMap, collections::hash_map::Entry, rc::Rc};

use crate::{
    interruption::{Interruption, runtime_error},
    object::LoxObject,
};

#[derive(Debug, Clone)]
pub struct Environment(Rc<RefCell<EnvironmentScope>>);

#[derive(Debug, Clone)]
struct EnvironmentScope {
    enclosing: Option<Environment>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new_global() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope {
            enclosing: None,
            values: HashMap::new(),
        })))
    }

    pub fn enter_scope(&self) -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope {
            enclosing: Some(self.clone()),
            values: HashMap::new(),
        })))
    }

    pub fn define(&mut self, name: String, value: LoxObject) {
        self.0.borrow_mut().define(name, value)
    }

    pub fn get(&self, name: &String) -> Result<LoxObject, Interruption> {
        self.0.borrow().get(name)
    }

    pub fn get_at(&self, distance: usize, name: &String) -> Result<LoxObject, Interruption> {
        self.0.borrow().get_at(distance, name)
    }

    pub fn assign(&mut self, name: String, value: LoxObject) -> Result<(), Interruption> {
        self.0.borrow_mut().assign(name, value)
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &String,
        value: LoxObject,
    ) -> Result<(), Interruption> {
        self.0.borrow_mut().assign_at(distance, name, value)
    }
}

impl EnvironmentScope {
    fn define(&mut self, name: String, value: LoxObject) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &String) -> Result<LoxObject, Interruption> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else {
            Err(runtime_error(format!("Undefined variable: {} .", name)))
        }
    }

    fn get_at(&self, distance: usize, name: &String) -> Result<LoxObject, Interruption> {
        if distance == 0
            && let Some(value) = self.values.get(name)
        {
            Ok(value.clone())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.get_at(distance - 1, name)
        } else {
            Err(runtime_error(format!("Undefined variable: {} .", name)))
        }
    }

    fn assign(&mut self, name: String, value: LoxObject) -> Result<(), Interruption> {
        match self.values.entry(name.clone()).and_modify(|x| *x = value) {
            Entry::Vacant(_) => Err(runtime_error(format!("Undefined variable: {} .", name))),
            Entry::Occupied(_) => Ok(()),
        }
    }

    fn assign_at(
        &mut self,
        distance: usize,
        name: &String,
        value: LoxObject,
    ) -> Result<(), Interruption> {
        if distance == 0 && self.values.contains_key(name) {
            self.values.insert(name.clone(), value);
            Ok(())
        } else if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign_at(distance - 1, name, value)
        } else {
            Err(runtime_error(format!("Undefined variable: {} .", name)))
        }
    }
}
