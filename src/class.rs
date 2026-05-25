use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    function::Function,
    interruption::{Interruption, runtime_error},
    object::{LoxObject, ObjRef},
};

#[derive(Debug)]
pub struct Class {
    name: String,
    methods: HashMap<String, Rc<Function>>,
}

impl Class {
    pub fn new(name: String, methods: Vec<(String, Rc<Function>)>) -> Self {
        Self {
            name,
            methods: HashMap::from_iter(methods),
        }
    }

    pub fn arity(&self) -> usize {
        0
    }

    pub fn get_method(&self, name: &String) -> Option<Rc<Function>> {
        self.methods.get(name).cloned()
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Class '{}'", self.name))
    }
}

#[derive(Debug)]
pub struct Instance {
    class: Rc<Class>,
    fields: RefCell<HashMap<String, ObjRef>>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &String) -> Result<ObjRef, Interruption> {
        let maybe_field = self.fields.borrow().get(name).cloned();
        if let Some(ref field) = maybe_field {
            return Ok(Rc::clone(field));
        }
        let maybe_method = self.class.get_method(name);
        if let Some(method) = maybe_method {
            Ok(Rc::new(LoxObject::Function(method)))
        } else {
            Err(runtime_error(format!("Undefined property '{}'.", name)))
        }
    }

    pub fn set(&self, name: String, value: ObjRef) -> Result<(), Interruption> {
        self.fields.borrow_mut().insert(name, value);
        Ok(())
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Instance of "))?;
        self.class.fmt(f)
    }
}
