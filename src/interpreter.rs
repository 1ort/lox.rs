use crate::ast::{BinaryOperator, Expression, LiteralValue, UnaryOperator};
type RuntimeError = String;
type EvalResult<T> = Result<T, String>;

pub struct Interpreter {}

#[derive(Debug)]
pub enum LoxObject {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl LoxObject {
    fn bool(&self) -> EvalResult<LoxObject> {
        match self {
            LoxObject::Boolean(b) => Ok(LoxObject::Boolean(*b)),
            LoxObject::Nil => Ok(LoxObject::Boolean(false)),
            _ => Ok(LoxObject::Boolean(true)),
        }
    }

    // -x
    fn neg(&self) -> EvalResult<LoxObject> {
        match self {
            LoxObject::Number(num) => Ok(LoxObject::Number(-num)),
            x => Err(format!("Can not apply unary '-': {:?} is not a number.", x)),
        }
    }

    // !x
    fn not(&self) -> EvalResult<LoxObject> {
        match self.bool() {
            Ok(LoxObject::Boolean(val)) => Ok(LoxObject::Boolean(!val)),
            _ => Err(format!("Can not apply unary '!'.",)),
        }
    }

    // ==
    fn eq(&self, other: &LoxObject) -> EvalResult<LoxObject> {
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

    fn neq(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        self.eq(other)?.not()
    }

    fn gt(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a > b)),
            _ => Err(format!("Can not compare {:?} > {:?}", self, other)),
        }
    }

    fn ge(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a >= b)),
            _ => Err(format!("Can not compare {:?} >= {:?}", self, other)),
        }
    }

    fn lt(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a < b)),
            _ => Err(format!("Can not compare {:?} < {:?}", self, other)),
        }
    }

    fn le(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Boolean(a <= b)),
            _ => Err(format!("Can not compare {:?} <= {:?}", self, other)),
        }
    }

    fn sub(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a - b)),
            _ => Err(format!("Can not substract {:?} - {:?}", self, other)),
        }
    }

    fn add(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a + b)),
            (LoxObject::String(a), LoxObject::String(b)) => Ok(LoxObject::String(a.to_owned() + b)),
            _ => Err(format!("Can not add {:?} + {:?}", self, other)),
        }
    }

    fn div(&self, other: &LoxObject) -> EvalResult<LoxObject> {
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

    fn mul(&self, other: &LoxObject) -> EvalResult<LoxObject> {
        match (self, other) {
            (LoxObject::Number(a), LoxObject::Number(b)) => Ok(LoxObject::Number(a / b)),
            _ => Err(format!("Can not multiply {:?} * {:?}", self, other)),
        }
    }
}

impl Interpreter {
    pub fn eval(&mut self, expr: &Expression) -> EvalResult<LoxObject> {
        match expr {
            Expression::Grouping { expression } => self.eval(expression),
            Expression::Literal { value } => self.eval_literal_value(value),
            Expression::Unary {
                operator,
                expression,
            } => self.eval_unary(operator, expression.as_ref()),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
        }
    }

    fn eval_literal_value(&mut self, val: &LiteralValue) -> EvalResult<LoxObject> {
        Ok(match val {
            LiteralValue::Number(num) => LoxObject::Number(*num),
            LiteralValue::String(s) => LoxObject::String(s.clone()),
            LiteralValue::Boolean(b) => LoxObject::Boolean(*b),
            LiteralValue::Nil => LoxObject::Nil,
        })
    }

    fn eval_unary(
        &mut self,
        operator: &UnaryOperator,
        expression: &Expression,
    ) -> EvalResult<LoxObject> {
        let expr_value = self.eval(expression)?;
        match operator {
            UnaryOperator::Minus => expr_value.neg(),
            UnaryOperator::Bang => expr_value.not(),
        }
    }

    fn eval_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
    ) -> EvalResult<LoxObject> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;

        match operator {
            BinaryOperator::EqualEqual => left.eq(&right),
            BinaryOperator::BangEqual => left.neq(&right),
            BinaryOperator::Greater => left.gt(&right),
            BinaryOperator::GreaterEqual => left.ge(&right),
            BinaryOperator::Less => left.lt(&right),
            BinaryOperator::LessEqual => left.le(&right),
            BinaryOperator::Minus => left.sub(&right),
            BinaryOperator::Plus => left.add(&right),
            BinaryOperator::Slash => left.div(&right),
            BinaryOperator::Star => left.mul(&right),
        }
    }
}
