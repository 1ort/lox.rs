use std::fmt;
use std::rc::Rc;

use crate::class::{Class, Instance};
use crate::function::{NativeFunction, UserFunction};
use crate::interruption::{LoxError, new_runtime_error};

#[derive(Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    UserFunction(Rc<UserFunction>),
    NativeFunction(Rc<NativeFunction>),
    Class(Rc<Class>),
    Instance(Rc<Instance>),
}

impl LoxObject {
    pub fn bool_native(&self) -> bool {
        match self.bool().unwrap() {
            Self::Boolean(a) => a,
            _ => false,
        }
    }

    // Why return result, if no errors?
    pub fn bool(&self) -> Result<LoxObject, LoxError> {
        match self {
            LoxObject::Boolean(b) => Ok(LoxObject::Boolean(*b)),
            LoxObject::Nil => Ok(LoxObject::Boolean(false)),
            _ => Ok(LoxObject::Boolean(true)),
        }
    }

    // -x
    pub fn neg(&self) -> Result<LoxObject, LoxError> {
        match self {
            LoxObject::Number(num) => Ok(LoxObject::Number(-num)),
            _ => Err(new_runtime_error(
                "Operand must be a number.".to_string(),
                None,
            )),
        }
    }

    // !x
    pub fn not(&self) -> Result<LoxObject, LoxError> {
        Ok(LoxObject::Boolean(!self.bool_native()))
    }

    // ==
    pub fn eq(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => {
                if x == y {
                    Ok(Boolean(true))
                } else {
                    Ok(Boolean(false))
                }
            }
            (String(x), String(y)) => {
                if x == y {
                    Ok(Boolean(true))
                } else {
                    Ok(Boolean(false))
                }
            }
            (Boolean(x), Boolean(y)) => {
                if x == y {
                    Ok(Boolean(true))
                } else {
                    Ok(Boolean(false))
                }
            }
            (Nil, Nil) => Ok(Boolean(true)),
            (UserFunction(this), UserFunction(other)) => {
                Ok(Boolean(std::ptr::eq(this.as_ref(), other.as_ref())))
            }
            (NativeFunction(this), NativeFunction(other)) => {
                Ok(Boolean(std::ptr::eq(this.as_ref(), other.as_ref())))
            }
            (Class(this), Class(other)) => Ok(Boolean(std::ptr::eq(this.as_ref(), other.as_ref()))),
            (Instance(this), Instance(other)) => {
                Ok(Boolean(std::ptr::eq(this.as_ref(), other.as_ref())))
            }
            _ => Ok(Boolean(false)),
        }
    }

    pub fn neq(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        self.eq(other)?.not()
    }

    pub fn gt(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a > b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn ge(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a >= b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn lt(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a < b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn le(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a <= b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn sub(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a - b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn add(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a + b)),
            (LoxObject::String(a), LoxObject::String(b)) => {
                Ok(LoxObject::String(format!("{}{}", a, b)))
            }
            _ => Err(new_runtime_error(
                "Operands must be two numbers or two strings.".to_string(),
                None,
            )),
        }
    }

    pub fn div(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => {
                if *b != 0.0 {
                    Ok(LoxObject::Number(a / b))
                } else {
                    Err(new_runtime_error(
                        "Can not divide by zero".to_string(),
                        None,
                    ))
                }
            }
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }

    pub fn mul(&self, other: &LoxObject) -> Result<LoxObject, LoxError> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a * b)),
            _ => Err(new_runtime_error(
                "Operands must be numbers.".to_string(),
                None,
            )),
        }
    }
}

impl std::fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxObject::Number(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::String(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::Boolean(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::Nil => f.write_str("nil"),
            LoxObject::UserFunction(function) => f.write_fmt(format_args!("{}", function)),
            LoxObject::NativeFunction(function) => f.write_fmt(format_args!("{}", function)),
            LoxObject::Class(class) => f.write_fmt(format_args!("{}", class)),
            LoxObject::Instance(instance) => f.write_fmt(format_args!("{}", instance)),
        }
    }
}
