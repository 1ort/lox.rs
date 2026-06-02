use crate::{
    error::error::{LoxError, new_runtime_error},
    runtime::object::LoxObject,
};
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    fmt,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct Environment(Rc<RefCell<EnvironmentScope>>);

#[derive(Debug)]
struct EnvironmentScope {
    tag: String,
    enclosing: Option<Environment>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new_global() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope {
            enclosing: None,
            values: HashMap::new(),
            tag: "global".to_string(),
        })))
    }

    pub fn enter_scope(&self, tag: String) -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope {
            enclosing: Some(self.clone()),
            values: HashMap::new(),
            tag,
        })))
    }

    pub fn define(&mut self, name: String, value: LoxObject) {
        self.0.borrow_mut().values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Result<LoxObject, LoxError> {
        if let Some(value) = self.0.borrow().values.get(name) {
            Ok(value.clone())
        } else {
            Err(new_runtime_error(
                format!("Undefined variable '{}'.", name),
                None,
            ))
        }
    }

    pub fn get_at(&self, distance: usize, name: &String) -> Result<LoxObject, LoxError> {
        let scope = &self.0.borrow();
        if distance == 0
            && let Some(value) = scope.values.get(name)
        {
            Ok(value.clone())
        } else if let Some(ref enclosing) = scope.enclosing {
            enclosing.get_at(distance - 1, name)
        } else {
            Err(new_runtime_error(
                format!("Undefined variable '{}'.", name),
                None,
            ))
        }
    }

    pub fn assign(&mut self, name: String, value: LoxObject) -> Result<(), LoxError> {
        match self
            .0
            .borrow_mut()
            .values
            .entry(name.clone())
            .and_modify(|x| *x = value)
        {
            Entry::Vacant(_) => Err(new_runtime_error(
                format!("Undefined variable '{}'.", name),
                None,
            )),
            Entry::Occupied(_) => Ok(()),
        }
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &String,
        value: LoxObject,
    ) -> Result<(), LoxError> {
        let this = &mut self.0.borrow_mut();
        if distance == 0 && this.values.contains_key(name) {
            this.values.insert(name.clone(), value);
            Ok(())
        } else if let Some(ref mut enclosing) = this.enclosing {
            enclosing.assign_at(distance - 1, name, value)
        } else {
            Err(new_runtime_error(
                format!("Undefined variable '{}'.", name),
                None,
            ))
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = self.0.borrow();
        write!(f, "{}", scope)
    }
}

impl fmt::Display for EnvironmentScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{{ ", self.tag)?;
        for (i, (key, value)) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, " }}")?;

        if let Some(encl) = &self.enclosing {
            write!(f, " -> {}", encl)?;
        }
        Ok(())
    }
}
