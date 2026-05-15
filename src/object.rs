pub type EvalResult<T> = Result<T, String>;
#[derive(Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl LoxObject {
    pub fn bool(&self) -> EvalResult<LoxObject> {
        match self {
            LoxObject::Boolean(b) => Ok(LoxObject::Boolean(*b)),
            LoxObject::Nil => Ok(LoxObject::Boolean(false)),
            _ => Ok(LoxObject::Boolean(true)),
        }
    }

    // -x
    pub fn neg(&self) -> EvalResult<LoxObject> {
        match self {
            LoxObject::Number(num) => Ok(LoxObject::Number(-num)),
            x => Err(format!("Can not apply unary '-': {:?} is not a number.", x)),
        }
    }

    // !x
    pub fn not(&self) -> EvalResult<LoxObject> {
        match self.bool() {
            Ok(LoxObject::Boolean(val)) => Ok(LoxObject::Boolean(!val)),
            _ => Err(format!("Can not apply unary '!'.",)),
        }
    }

    // ==
    pub fn eq(&self, other: &LoxObject) -> EvalResult<LoxObject> {
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
            _ => Ok(Boolean(false)),
        }
    }

    pub fn neq(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        self.eq(other)?.not()
    }

    pub fn gt(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a > b)),
            _ => Err(format!("Can not compare {:?} > {:?}", self, other)),
        }
    }

    pub fn ge(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a >= b)),
            _ => Err(format!("Can not compare {:?} >= {:?}", self, other)),
        }
    }

    pub fn lt(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a < b)),
            _ => Err(format!("Can not compare {:?} < {:?}", self, other)),
        }
    }

    pub fn le(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a <= b)),
            _ => Err(format!("Can not compare {:?} <= {:?}", self, other)),
        }
    }

    pub fn sub(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a - b)),
            _ => Err(format!("Can not substract {:?} - {:?}", self, other)),
        }
    }

    pub fn add(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a + b)),
            (LoxObject::String(a), LoxObject::String(b)) => Ok(LoxObject::String(a.to_owned() + b)),
            _ => Err(format!("Can not add {:?} + {:?}", self, other)),
        }
    }

    pub fn div(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => {
                if *b != 0.0 {
                    Ok(LoxObject::Number(a / b))
                } else {
                    Err("Can not divide by zero".to_string())
                }
            }
            _ => Err(format!("Can not divide {:?} / {:?}", self, other)),
        }
    }

    pub fn mul(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a * b)),
            _ => Err(format!("Can not multiply {:?} * {:?}", self, other)),
        }
    }

    pub fn format(&self) -> String {
        match self {
            LoxObject::Number(val) => format!("{}", val),
            LoxObject::String(val) => format!("{}", val),
            LoxObject::Boolean(val) => format!("{}", val),
            LoxObject::Nil => "Nil".to_string(),
        }
    }
}
