use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{
    BinaryOperator, Expression, FunctionStatement, LiteralValue, LogicalOperator, Program,
    Statement, UnaryOperator,
};
use crate::class::{Class, Instance};
use crate::environment::{EnvRef, Environment};
use crate::function::Function;
use crate::interruption::{Interruption, brake_inter, retun_inter, runtime_error};
use crate::object::{self, LoxObject, ObjRef, objref};

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
                    objref(LoxObject::Nil)
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
            Statement::FunctionDeclaration(func_stmt) => {
                let func = self.eval_function_statement(func_stmt, Rc::clone(&self.environment));

                self.environment
                    .borrow_mut()
                    .define(func.name(), objref(LoxObject::Function(func)));
                Ok(())
            }
            Statement::Return { expresstion } => {
                let expr_result = match expresstion {
                    None => objref(LoxObject::Nil),
                    Some(expr) => self.eval_expression(expr)?,
                };
                Err(retun_inter(expr_result))
            }
            Statement::ClassDeclaration { name, methods } => {
                self.environment
                    .borrow_mut()
                    .define(name.clone(), objref(LoxObject::Nil));

                let closure = Rc::new(RefCell::new(Environment::new_local(Rc::clone(
                    &self.environment,
                ))));

                let methods_vec = methods
                    .iter()
                    .map(|meth_stmt| {
                        let method = self.eval_function_statement(meth_stmt, Rc::clone(&closure));
                        (method.name(), method)
                    })
                    .collect();

                let class = Rc::new(Class::new(name.clone(), methods_vec));
                self.environment
                    .borrow_mut()
                    .assign(name.clone(), objref(LoxObject::Class(class)))?;
                Ok(())
            }
        }
    }

    fn eval_function_statement(
        &mut self,
        func_stmt: &FunctionStatement,
        closure: EnvRef,
    ) -> Rc<Function> {
        let FunctionStatement {
            name,
            parameters,
            body,
        } = func_stmt;
        Rc::new(Function::Defined {
            name: name.clone(),
            parameters: parameters.clone(),
            code_block: Rc::new(*body.clone()),
            closure: closure,
        })
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

    fn eval_expression(&mut self, expr: &Expression) -> Result<ObjRef, Interruption> {
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
            Expression::Get { object, name } => self.eval_get(object, name),
            Expression::Set {
                object,
                name,
                expression,
            } => self.eval_set(object, name, expression),
            Expression::This {
                resolved_scope_depth,
            } => self.eval_variable(&"this".to_string(), &resolved_scope_depth.borrow()),
        }
    }

    fn eval_get(&mut self, object: &Expression, name: &String) -> Result<ObjRef, Interruption> {
        let instance_ref = self.eval_expression(object)?;
        let attr = match &*instance_ref.clone() {
            LoxObject::Instance(instance) => instance.get(name)?,
            _ => return Err(runtime_error("Only instances have attributes.".to_string())),
        };

        if let LoxObject::Function(function) = attr.as_ref() {
            Ok(objref(LoxObject::Function(Rc::new(
                function.bind(&instance_ref),
            ))))
        } else {
            Ok(attr)
        }
    }

    fn eval_set(
        &mut self,
        object: &Expression,
        name: &str,
        expression: &Expression,
    ) -> Result<ObjRef, Interruption> {
        let instance_ref = self.eval_expression(object)?;
        if let LoxObject::Instance(instance) = &*instance_ref {
            let value_ref = self.eval_expression(expression)?;
            instance.set(name.to_owned(), value_ref)?
        } else {
            return Err(runtime_error("Only instances have fields.".to_string()));
        }

        Ok(objref(LoxObject::Nil))
    }

    fn eval_variable(
        &mut self,
        name: &String,
        scope_depth: &Option<usize>,
    ) -> Result<ObjRef, Interruption> {
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
    ) -> Result<ObjRef, Interruption> {
        let value = self.eval_expression(expression)?;
        if let Some(distance) = *scope_depth {
            self.environment
                .borrow_mut()
                .assign_at(distance, name, value.clone())?;
        } else {
            self.globals
                .borrow_mut()
                .assign(name.clone(), value.clone())?
        }
        Ok(value)
    }

    fn eval_literal_value(&mut self, val: &LiteralValue) -> Result<ObjRef, Interruption> {
        Ok(objref(match val {
            LiteralValue::Number(num) => LoxObject::Number(*num),
            LiteralValue::String(s) => LoxObject::String(s.clone()),
            LiteralValue::Boolean(b) => LoxObject::Boolean(*b),
            LiteralValue::Nil => LoxObject::Nil,
        }))
    }

    fn eval_unary(
        &mut self,
        operator: &UnaryOperator,
        expression: &Expression,
    ) -> Result<ObjRef, Interruption> {
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
    ) -> Result<ObjRef, Interruption> {
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
    ) -> Result<ObjRef, Interruption> {
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
    ) -> Result<ObjRef, Interruption> {
        let callee_obj = self.eval_expression(callee)?;
        let args = argument_expressions
            .iter()
            .map(|expr| self.eval_expression(expr))
            .collect::<Result<Vec<ObjRef>, Interruption>>()?;
        match callee_obj.as_ref() {
            LoxObject::Class(class) => {
                if class.arity() != args.len() {
                    return Err(runtime_error(format!(
                        "{} takes {} arguments, but {} provided",
                        class,
                        class.arity(),
                        args.len()
                    )));
                }

                Ok(objref(LoxObject::Instance(Instance::new(Rc::clone(class)))))
            }
            LoxObject::Function(func) => {
                if func.arity() as usize != args.len() {
                    return Err(runtime_error(format!(
                        "{} takes {} arguments, but {} provided",
                        func,
                        func.arity(),
                        args.len()
                    )));
                };

                match func.as_ref() {
                    Function::Native { callable, .. } => Ok(callable(&args)?),
                    Function::Defined {
                        parameters,
                        code_block,
                        closure,
                        ..
                    } => self.eval_call(parameters, code_block, &args, closure),
                }
            }
            _ => Err(runtime_error(format!("'{}' is not callable", callee_obj))),
        }
    }

    fn eval_call(
        &mut self,
        parameters: &[String],
        code_block: &Statement,
        args: &[ObjRef],
        closure: &EnvRef,
    ) -> Result<ObjRef, Interruption> {
        let environment = Environment::new_local(Rc::clone(closure));
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
            Ok(_) => objref(LoxObject::Nil),
            Err(Interruption::Return { object }) => object,
            Err(error) => return Err(error),
        };

        self.environment = old_environment;
        Ok(result)
    }
}
