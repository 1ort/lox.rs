use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{function::Function, interruption::Interruption, object::LoxObject};

#[derive(Debug)]
pub struct Class {
    name: String,
    methods: HashMap<String, Rc<Function>>,
    superclass: Option<Rc<Class>>,
}

impl Class {
    pub fn new(
        name: String,
        methods: Vec<(String, Rc<Function>)>,
        superclass: Option<Rc<Class>>,
    ) -> Self {
        Self {
            name,
            methods: HashMap::from_iter(methods),
            superclass,
        }
    }

    pub fn arity(&self) -> u8 {
        match self.get_method("init") {
            Some(func) => func.arity(),
            None => 0,
        }
    }

    pub fn get_method(&self, name: &str) -> Option<Rc<Function>> {
        let self_meth = &self.methods.get(name);
        if self_meth.is_some() {
            self_meth.cloned()
        } else {
            self.superclass
                .clone()
                .and_then(|superclass| superclass.get_method(name))
        }
    }

    pub fn get_initializer(&self) -> Option<Rc<Function>> {
        self.methods.get("init").cloned()
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug)]
pub struct Instance {
    class: Rc<Class>,
    fields: RefCell<HashMap<String, LoxObject>>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &str) -> Option<LoxObject> {
        if let Some(ref field) = self.fields.borrow().get(name).cloned() {
            return Some(field.clone());
        }
        self.class.get_method(name).map(LoxObject::Function)
    }

    pub fn set(&self, name: String, value: LoxObject) -> Result<(), Interruption> {
        self.fields.borrow_mut().insert(name, value);
        Ok(())
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.class.fmt(f)?;
        f.write_fmt(format_args!(" instance"))
    }
}
