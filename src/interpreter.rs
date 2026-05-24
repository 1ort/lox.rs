use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{
    BinaryOperator, Expression, LiteralValue, LogicalOperator, Program, Statement, UnaryOperator,
};
use crate::environment::{EnvRef, Environment};
use crate::function::Function;
use crate::interruption::{Interruption, brake_inter, retun_inter, runtime_error};
use crate::object::LoxObject;

use crate::globals;

pub struct Interpreter {
    pub environment: EnvRef,
    pub globals: EnvRef,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(globals::build_globals()));
        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::clone(&globals),
        }
    }

    pub fn exec(&mut self, program: &Program) -> Result<(), Interruption> {
        let mut iterator = program.statements.iter();

        for result in iterator.by_ref().map(|stmt| self.exec_statement(stmt)) {
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    pub fn exec_statement(&mut self, statement: &Statement) -> Result<(), Interruption> {
        match statement {
            Statement::Block { statements } => {
                self.exec_block(statements)?;
                Ok(())
            }
            Statement::Expression { expression } => {
                self.eval_expression(expression)?;
                Ok(())
            }
            Statement::Print { expression } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj);
                Ok(())
            }
            Statement::VarDeclaration { name, initializer } => {
                let value = if let Some(expression) = initializer {
                    self.eval_expression(expression)?
                } else {
                    LoxObject::Nil
                };
                self.environment.borrow_mut().define(name.clone(), value);
                Ok(())
            }
            Statement::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.eval_expression(condition)?.bool_native() {
                    self.exec_statement(then_branch)?;
                } else if let Some(else_branch_unwrapped) = else_branch {
                    self.exec_statement(else_branch_unwrapped)?;
                }
                Ok(())
            }
            Statement::WhileLoop { condition, body } => {
                while self.eval_expression(condition)?.bool_native() {
                    if let Err(err) = self.exec_statement(body) {
                        match err {
                            Interruption::Break => {
                                break;
                            }
                            _ => return Err(err),
                        }
                    }
                }
                Ok(())
            }
            Statement::Break => Err(brake_inter()),
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                self.environment.borrow_mut().define(
                    name.clone(),
                    LoxObject::Function(crate::function::Function::Defined {
                        name: name.clone(),
                        parameters: parameters.clone(),
                        code_block: body.clone(),
                        closure: Rc::clone(&self.environment),
                    }),
                );
                Ok(())
            }
            Statement::Return { expresstion } => {
                let expr_result = match expresstion {
                    None => LoxObject::Nil,
                    Some(expr) => self.eval_expression(expr)?,
                };
                Err(retun_inter(expr_result))
            }
        }
    }

    fn exec_block(&mut self, statements: &[Statement]) -> Result<(), Interruption> {
        let old_env_ref = Rc::clone(&self.environment);
        let new_env = Environment::new_local(Rc::clone(&self.environment));
        self.environment = Rc::new(RefCell::new(new_env));

        let res: Result<(), Interruption> = statements
            .iter()
            .try_for_each(|stmt| self.exec_statement(stmt));

        self.environment = old_env_ref;
        res
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<LoxObject, Interruption> {
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
            Expression::Identifier {
                name,
                resolved_scope_depth,
            } => self.eval_variable(name, &resolved_scope_depth.borrow()),
            Expression::Assignment {
                name,
                expression,
                resolved_scope_depth,
            } => self.eval_assignment(name, expression, &resolved_scope_depth.borrow()),
            Expression::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(left, operator, right),
            Expression::Call { callee, arguments } => self.eval_call_expr(callee, arguments),
        }
    }

    fn eval_variable(
        &mut self,
        name: &String,
        scope_depth: &Option<usize>,
    ) -> Result<LoxObject, Interruption> {
        let obj_ref = if let Some(distance) = *scope_depth {
            self.environment.borrow().get_at(distance, name)?
        } else {
            self.globals.borrow().get(name)?
        };
        Ok(obj_ref.clone())
    }

    fn eval_assignment(
        &mut self,
        name: &String,
        expression: &Expression,
        scope_depth: &Option<usize>,
    ) -> Result<LoxObject, Interruption> {
        let value = self.eval_expression(expression)?;
        if let Some(distance) = *scope_depth {
            self.environment
                .borrow_mut()
                .assign_at(distance, name, value.clone())?;
        } else {
            self.globals.borrow_mut().assign(name, value.clone())?
        }
        Ok(value)
    }

    fn eval_literal_value(&mut self, val: &LiteralValue) -> Result<LoxObject, Interruption> {
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
    ) -> Result<LoxObject, Interruption> {
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
    ) -> Result<LoxObject, Interruption> {
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

    fn eval_logical(
        &mut self,
        left: &Expression,
        operator: &LogicalOperator,
        right: &Expression,
    ) -> Result<LoxObject, Interruption> {
        let left_result = self.eval_expression(left)?;
        match operator {
            LogicalOperator::Or => {
                if left_result.bool_native() {
                    return Ok(left_result);
                }
            }
            LogicalOperator::And => {
                if !left_result.bool_native() {
                    return Ok(left_result);
                }
            }
        }
        self.eval_expression(right)
    }

    fn eval_call_expr(
        &mut self,
        callee: &Expression,
        argument_expressions: &[Expression],
    ) -> Result<LoxObject, Interruption> {
        let callee_obj = self.eval_expression(callee)?;
        let args = argument_expressions
            .iter()
            .map(|expr| self.eval_expression(expr))
            .collect::<Result<Vec<LoxObject>, Interruption>>()?;
        match callee_obj {
            LoxObject::Function(func) => {
                if func.arity() as usize != args.len() {
                    return Err(runtime_error(format!(
                        "{} takes {} arguments, but {} provided",
                        func.format(),
                        func.arity(),
                        args.len()
                    )));
                };

                match func {
                    Function::Native { callable, .. } => Ok(callable(&args)?),
                    Function::Defined {
                        parameters,
                        code_block,
                        closure,
                        ..
                    } => self.eval_call(&parameters, &code_block, &args, closure),
                }
            }
            _ => Err(runtime_error(format!("'{}' is not callable", callee_obj))),
        }
    }

    fn eval_call(
        &mut self,
        parameters: &[String],
        code_block: &Statement,
        args: &[LoxObject],
        closure: EnvRef,
    ) -> Result<LoxObject, Interruption> {
        let environment = Environment::new_local(Rc::clone(&closure));
        let old_environment = Rc::clone(&self.environment);

        self.environment = Rc::new(RefCell::new(environment));

        let _ = std::iter::zip(parameters, args)
            .map(|(name, value)| {
                self.environment
                    .borrow_mut()
                    .define(name.clone(), value.clone())
            })
            .collect::<Vec<_>>();

        let result = match self.exec_statement(code_block) {
            Ok(_) => LoxObject::Nil,
            Err(Interruption::Return { object }) => object,
            Err(error) => return Err(error),
        };

        self.environment = old_environment;
        Ok(result)
    }
}
