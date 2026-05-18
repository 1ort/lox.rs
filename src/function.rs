use crate::object::LoxObject;

#[derive(Debug, Clone)]
pub enum Function {
    Native {
        identifier: String,
        arity: u8,
        callable: fn(&[LoxObject]) -> Result<LoxObject, String>,
    },
    Defined {
        name: String,
    },
}

impl Function {
    pub fn arity(&self) -> u8 {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::Defined { .. } => todo!(),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Function::Native { identifier, .. } => format!("function '{}'", identifier),
            Function::Defined { name } => format!("function '{}'", name),
        }
    }

    pub fn call(&self, args: &[LoxObject]) -> Result<LoxObject, String> {
        if self.arity() as usize != args.len() {
            return Err(format!(
                "{} takes {} arguments, but {} provided",
                self.format(),
                self.arity(),
                args.len()
            ));
        }
        match self {
            Function::Native { callable, .. } => Ok(callable(args)?),
            Function::Defined { .. } => todo!(),
        }
    }
}
