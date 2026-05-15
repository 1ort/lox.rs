use crate::ast::{BinaryOperator, Expression, LiteralValue, Program, Statement, UnaryOperator};
use crate::environment::Environment;
use crate::object::{EvalResult, LoxObject};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(None),
        }
    }

    pub fn exec(&mut self, program: &Program) -> EvalResult<()> {
        let mut iterator = program.statements.iter();

        for result in iterator.by_ref().map(|stmt| self.exec_statement(stmt)) {
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    pub fn exec_statement(&mut self, statement: &Statement) -> EvalResult<()> {
        match statement {
            Statement::Expression { expression } => {
                self.eval_expression(expression)?;
                Ok(())
            }
            Statement::Print { expression } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj.format());
                Ok(())
            }
            Statement::VarDeclaration { name, initializer } => {
                let value = if let Some(expression) = initializer {
                    self.eval_expression(expression)?
                } else {
                    LoxObject::Nil
                };
                self.environment.define(name.clone(), value);
                Ok(())
            }
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> EvalResult<LoxObject> {
        match expr {
            Expression::Grouping { expression } => self.eval_expression(expression),
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
            Expression::Variable { name } => self.eval_variable(name),
            Expression::Assignment { name, expression } => self.eval_assignment(name, expression),
        }
    }

    fn eval_variable(&mut self, name: &String) -> EvalResult<LoxObject> {
        let obj_ref = self.environment.get(name)?;
        Ok(obj_ref.clone())
    }

    fn eval_assignment(&mut self, name: &String, expression: &Expression) -> EvalResult<LoxObject> {
        let value = self.eval_expression(expression)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
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
        let expr_value = self.eval_expression(expression)?;
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
        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

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
