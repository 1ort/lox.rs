use std::rc::Rc;

use crate::ast::{
    BinaryOperator, Expression, FunctionStatement, LiteralValue, LogicalOperator, Program,
    Statement, UnaryOperator,
};
use crate::class::{Class, Instance};
use crate::environment::Environment;
use crate::function::Function;
use crate::interruption::{Interruption, brake_inter, retun_inter, runtime_error};
use crate::object::LoxObject;

use crate::globals;

pub struct Interpreter {
    pub environment: Environment,
    pub globals: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = globals::build_globals();
        Interpreter {
            globals: globals.clone(),
            environment: globals.clone(),
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
                self.environment.define(name.clone(), value);
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
                let func = self.eval_function_statement(func_stmt, self.environment.clone(), false);

                self.environment
                    .define(func.name().to_owned(), LoxObject::Function(func));
                Ok(())
            }
            Statement::Return { expresstion } => {
                let expr_result = match expresstion {
                    None => LoxObject::Nil,
                    Some(expr) => self.eval_expression(expr)?,
                };
                Err(retun_inter(expr_result))
            }
            Statement::ClassDeclaration { name, methods } => {
                self.environment.define(name.clone(), LoxObject::Nil);

                let closure = self.environment.enter_scope();

                let methods_vec = methods
                    .iter()
                    .map(|meth_stmt| {
                        let method = self.eval_function_statement(meth_stmt, closure.clone(), true);
                        (method.name().to_owned(), method)
                    })
                    .collect();

                let class = Rc::new(Class::new(name.clone(), methods_vec));
                self.environment
                    .assign(name.clone(), LoxObject::Class(class))?;
                Ok(())
            }
        }
    }

    fn eval_function_statement(
        &mut self,
        func_stmt: &FunctionStatement,
        closure: Environment,
        is_method: bool,
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
            closure,
            is_initializer: is_method && name.eq("init"),
        })
    }

    fn exec_block(&mut self, statements: &[Statement]) -> Result<(), Interruption> {
        let enclosing = self.environment.clone();
        self.environment = self.environment.enter_scope();

        let res: Result<(), Interruption> = statements
            .iter()
            .try_for_each(|stmt| self.exec_statement(stmt));

        self.environment = enclosing;
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

    fn eval_get(&mut self, object: &Expression, name: &String) -> Result<LoxObject, Interruption> {
        let obj = self.eval_expression(object)?;

        let attr = match &obj {
            LoxObject::Instance(instance) => instance.get(name),
            _ => return Err(runtime_error("Only instances have attributes.".to_string())),
        }
        .ok_or(runtime_error(format!("Undefined property '{}'.", name)))?;
        if let LoxObject::Function(function) = attr {
            let bound_method = function.bind(obj);
            Ok(LoxObject::Function(Rc::new(bound_method)))
        } else {
            Ok(attr)
        }
    }

    fn eval_set(
        &mut self,
        object: &Expression,
        name: &str,
        expression: &Expression,
    ) -> Result<LoxObject, Interruption> {
        let obj = self.eval_expression(object)?;
        if let LoxObject::Instance(instance) = obj {
            let value_ref = self.eval_expression(expression)?;
            instance.set(name.to_owned(), value_ref.clone())?;
            Ok(value_ref)
        } else {
            Err(runtime_error("Only instances have fields.".to_string()))
        }
    }

    fn eval_variable(
        &mut self,
        name: &String,
        scope_depth: &Option<usize>,
    ) -> Result<LoxObject, Interruption> {
        let obj_ref = if let Some(distance) = *scope_depth {
            self.environment.get_at(distance, name)?
        } else {
            self.globals.get(name)?
        };
        Ok(obj_ref)
    }

    fn eval_assignment(
        &mut self,
        name: &String,
        expression: &Expression,
        scope_depth: &Option<usize>,
    ) -> Result<LoxObject, Interruption> {
        let value = self.eval_expression(expression)?;
        if let Some(distance) = *scope_depth {
            self.environment.assign_at(distance, name, value.clone())?;
        } else {
            self.globals.assign(name.clone(), value.clone())?
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
        match &callee_obj {
            LoxObject::Class(class) => self.instantiate(class, &args),
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
                        is_initializer,
                        ..
                    } => self.eval_call(parameters, code_block, &args, closure, *is_initializer),
                }
            }
            _ => Err(runtime_error(format!("'{}' is not callable", callee_obj))),
        }
    }

    fn instantiate(
        &mut self,
        class: &Rc<Class>,
        arguments: &[LoxObject],
    ) -> Result<LoxObject, Interruption> {
        if class.arity() as usize != arguments.len() {
            return Err(runtime_error(format!(
                "{} takes {} arguments, but {} provided",
                class,
                class.arity(),
                arguments.len()
            )));
        }
        let instance_rc = Rc::new(Instance::new(Rc::clone(class)));
        let object = LoxObject::Instance(instance_rc);

        if let Some(initializer) = class.get_initializer() {
            let bound_method = initializer.bind(object.clone());
            match bound_method {
                Function::Defined {
                    parameters,
                    code_block,
                    closure,
                    is_initializer,
                    ..
                } => self.eval_call(
                    &parameters,
                    &code_block,
                    arguments,
                    &closure,
                    is_initializer,
                )?,
                _ => unreachable!(), // only user functions could be initializers
            };
        }
        Ok(object)
    }

    fn eval_call(
        &mut self,
        parameters: &[String],
        code_block: &Statement,
        args: &[LoxObject],
        closure: &Environment,
        is_initializer: bool,
    ) -> Result<LoxObject, Interruption> {
        let enclosing = self.environment.clone();
        self.environment = closure.enter_scope();

        let _ = std::iter::zip(parameters, args)
            .map(|(name, value)| self.environment.define(name.clone(), value.clone()))
            .collect::<Vec<_>>();

        let result = match self.exec_statement(code_block) {
            Ok(_) => {
                if is_initializer {
                    closure.get_at(0, &"this".to_string())?
                } else {
                    LoxObject::Nil
                }
            }
            Err(Interruption::Return { object }) => {
                if is_initializer {
                    closure.get_at(0, &"this".to_string())?
                } else {
                    object
                }
            }
            Err(error) => return Err(error),
        };

        self.environment = enclosing;
        Ok(result)
    }
}
