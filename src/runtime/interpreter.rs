use crate::ast::{
    BinaryOperator, Expression, FunctionStatement, Identifier, LiteralValue, LogicalOperator,
    Program, Statement, UnaryOperator,
};
use crate::{
    error::{LoxError, new_runtime_error},
    runtime::{
        class::{Class, Instance},
        environment::Environment,
        function::{NativeFunction, UserFunction},
        globals,
        object::LoxObject,
    },
    span::Span,
};
use std::rc::Rc;

pub struct Interpreter {
    pub environment: Environment,
    pub globals: Environment,
}

enum JumpKind {
    Return(LoxObject),
    Brake,
    None,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = globals::build_globals();
        Interpreter {
            globals: globals.clone(),
            environment: globals.clone(),
        }
    }

    pub fn exec(&mut self, program: &Program) -> Result<(), LoxError> {
        let mut iterator = program.statements.iter();

        for result in iterator.by_ref().map(|stmt| self.exec_statement(stmt)) {
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    fn exec_statement(&mut self, statement: &Statement) -> Result<JumpKind, LoxError> {
        match statement {
            Statement::Block { statements, .. } => self.exec_block(statements),
            Statement::Expression { expression, .. } => {
                self.eval_expression(expression)?;
                Ok(JumpKind::None)
            }
            Statement::Print { expression, .. } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj);
                Ok(JumpKind::None)
            }
            Statement::VarDeclaration {
                name, initializer, ..
            } => self.exec_var_declaration(name, initializer),
            Statement::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.exec_conditional(condition, then_branch, else_branch),
            Statement::WhileLoop {
                condition, body, ..
            } => self.exec_while_loop(condition, body),
            Statement::Break { .. } => Ok(JumpKind::Brake),
            Statement::FunctionDeclaration { function, .. } => {
                self.exec_function_declaration(function)
            }
            Statement::Return { expresstion, .. } => {
                let expr_result = match expresstion {
                    None => LoxObject::Nil,
                    Some(expr) => self.eval_expression(expr)?,
                };
                Ok(JumpKind::Return(expr_result))
            }
            Statement::ClassDeclaration {
                name,
                superclass,
                methods,
                span,
            } => self
                .exec_class_declaration(name, superclass, methods)
                .map_err(|err| err.with_span(span.clone())),
            Statement::Pass { .. } => Ok(JumpKind::None),
        }
    }

    fn exec_class_declaration(
        &mut self,
        name: &str,
        superclass: &Option<Identifier>,
        methods: &[FunctionStatement],
    ) -> Result<JumpKind, LoxError> {
        self.environment.define(name.to_owned(), LoxObject::Nil);
        let mut class_scope = self.environment.enter_scope("class".to_string());

        let superclass = if let Some(superclass_identifier) = superclass {
            let superclass = self.eval_variable(superclass_identifier)?;
            match superclass {
                LoxObject::Class(class_ref) => {
                    class_scope.define("super".to_string(), LoxObject::Class(class_ref.clone()));
                    Some(class_ref)
                }
                _ => {
                    return Err(new_runtime_error(
                        "Superclass must be a class.".to_string(),
                        Some(&superclass_identifier.span),
                    ));
                }
            }
        } else {
            None
        };

        let methods_vec = methods
            .iter()
            .map(|meth_stmt| {
                let method = self.eval_function_statement(meth_stmt, class_scope.clone(), true);
                (method.name().to_owned(), method)
            })
            .collect();

        let class = Rc::new(Class::new(name.to_owned(), methods_vec, superclass));
        self.environment
            .assign(name.to_owned(), LoxObject::Class(class))?;
        Ok(JumpKind::None)
    }

    fn exec_function_declaration(
        &mut self,
        func_stmt: &FunctionStatement,
    ) -> Result<JumpKind, LoxError> {
        let func = self.eval_function_statement(func_stmt, self.environment.clone(), false);

        self.environment
            .define(func.name().to_owned(), LoxObject::UserFunction(func));
        Ok(JumpKind::None)
    }

    fn exec_while_loop(
        &mut self,
        condition: &Expression,
        body: &Statement,
    ) -> Result<JumpKind, LoxError> {
        while self.eval_expression(condition)?.bool_native() {
            let jump_kind = self.exec_statement(body)?;
            match jump_kind {
                JumpKind::Brake => return Ok(JumpKind::None),
                JumpKind::Return(..) => {
                    return Ok(jump_kind);
                }
                JumpKind::None => continue,
            };
        }
        Ok(JumpKind::None)
    }

    fn exec_conditional(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> Result<JumpKind, LoxError> {
        if self.eval_expression(condition)?.bool_native() {
            let jump_kind = self.exec_statement(then_branch)?;
            if !matches!(jump_kind, JumpKind::None) {
                return Ok(jump_kind);
            }
        } else if let Some(else_branch_unwrapped) = else_branch {
            let jump_kind = self.exec_statement(else_branch_unwrapped)?;
            if !matches!(jump_kind, JumpKind::None) {
                return Ok(jump_kind);
            }
        }
        Ok(JumpKind::None)
    }

    fn exec_var_declaration(
        &mut self,
        name: &str,
        initializer: &Option<Box<Expression>>,
    ) -> Result<JumpKind, LoxError> {
        let value = if let Some(expression) = initializer {
            self.eval_expression(expression)?
        } else {
            LoxObject::Nil
        };
        self.environment.define(name.to_owned(), value);
        Ok(JumpKind::None)
    }

    fn exec_block(&mut self, statements: &[Statement]) -> Result<JumpKind, LoxError> {
        let enclosing = self.environment.clone();
        self.environment = self.environment.enter_scope("block".to_string());

        let mut block_result = Ok(JumpKind::None);

        for result in statements.iter().map(|stmt| self.exec_statement(stmt)) {
            let jump_kind = result?;
            match jump_kind {
                JumpKind::None => continue,
                _ => {
                    block_result = Ok(jump_kind);
                    break;
                }
            }
        }
        self.environment = enclosing;
        block_result
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<LoxObject, LoxError> {
        match expr {
            Expression::Grouping { expression, .. } => self.eval_expression(expression),
            Expression::Literal { value, .. } => self.eval_literal_value(value),
            Expression::Unary {
                operator,
                expression,
                span,
            } => self.eval_unary(operator, expression.as_ref(), span),
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => self.eval_binary(left, operator, right, span),
            Expression::Identifier { identifier, .. } => self.eval_variable(identifier),
            Expression::Assignment {
                identifier,
                expression,
                ..
            } => self.eval_assignment(expression, identifier),
            Expression::Logical {
                left,
                operator,
                right,
                ..
            } => self.eval_logical(left, operator, right),
            Expression::Call {
                callee,
                arguments,
                span,
            } => self.eval_call_expr(callee, arguments, span),
            Expression::Get { object, name, span } => self.eval_get(object, name, span),
            Expression::Set {
                object,
                name,
                expression,
                ..
            } => self.eval_set(object, name, expression),
            Expression::This { identifier, .. } => self.eval_variable(identifier),
            Expression::Super {
                identifier,
                method,
                span,
            } => self.eval_super_method(identifier, method, span),
        }
    }

    fn eval_function_statement(
        &mut self,
        func_stmt: &FunctionStatement,
        closure: Environment,
        is_method: bool,
    ) -> Rc<UserFunction> {
        let FunctionStatement {
            name,
            parameters,
            body,
            ..
        } = func_stmt;
        Rc::new(UserFunction {
            name: name.clone(),
            parameters: parameters.iter().map(|ident| ident.name.clone()).collect(),
            code_block: Rc::new(*body.clone()),
            closure,
            is_initializer: is_method && name.eq("init"),
            is_bound: false,
        })
    }

    fn eval_super_method(
        &mut self,
        super_identifier: &Identifier,
        method_name: &str,
        span: &Span,
    ) -> Result<LoxObject, LoxError> {
        let class_obj = self.eval_variable(super_identifier)?;
        let method_func = if let LoxObject::Class(class) = class_obj {
            if let Some(func) = class.get_method(method_name) {
                func
            } else {
                return Err(new_runtime_error(
                    format!("Undefined property '{}'.", method_name),
                    Some(span),
                ));
            }
        } else {
            return Err(new_runtime_error(
                "Can't resolve 'super': not a class.".to_string(),
                Some(&super_identifier.span),
            ));
        };

        let mut instance_identifier = Identifier::new("this".to_string(), span);
        instance_identifier.resolved_depth = super_identifier.resolved_depth.map(|x| x - 1);
        let instance = self.eval_variable(&instance_identifier)?;

        let callable = if method_func.is_bound() {
            method_func
        } else {
            Rc::new(method_func.bind(instance))
        };
        Ok(LoxObject::UserFunction(callable))
    }

    fn eval_get(
        &mut self,
        object: &Expression,
        name: &String,
        span: &Span,
    ) -> Result<LoxObject, LoxError> {
        let obj = self.eval_expression(object)?;

        let attr = match &obj {
            LoxObject::Instance(instance) => instance.get(name),
            _ => {
                return Err(new_runtime_error(
                    "Only instances have properties.".to_string(),
                    Some(object.span()),
                ));
            }
        }
        .ok_or(new_runtime_error(
            format!("Undefined property '{}'.", name),
            Some(span),
        ))?;
        if let LoxObject::UserFunction(function) = attr {
            let callable = if function.is_bound() {
                function
            } else {
                Rc::new(function.bind(obj))
            };
            Ok(LoxObject::UserFunction(callable))
        } else {
            Ok(attr)
        }
    }

    fn eval_set(
        &mut self,
        object: &Expression,
        name: &str,
        expression: &Expression,
    ) -> Result<LoxObject, LoxError> {
        let obj = self.eval_expression(object)?;
        if let LoxObject::Instance(instance) = obj {
            let value_ref = self.eval_expression(expression)?;
            instance.set(name.to_owned(), value_ref.clone())?;
            Ok(value_ref)
        } else {
            Err(new_runtime_error(
                "Only instances have fields.".to_string(),
                Some(object.span()),
            ))
        }
    }

    fn eval_variable(&mut self, identifier: &Identifier) -> Result<LoxObject, LoxError> {
        let Identifier {
            resolved_depth,
            name,
            span,
        } = identifier;

        let obj_ref = if let Some(distance) = *resolved_depth {
            self.environment
                .get_at(distance, name)
                .map_err(|err| err.with_span(span.clone()))?
        } else {
            self.globals
                .get(name)
                .map_err(|err| err.with_span(span.clone()))?
        };
        Ok(obj_ref)
    }

    fn eval_assignment(
        &mut self,
        expression: &Expression,
        identifier: &Identifier,
    ) -> Result<LoxObject, LoxError> {
        let Identifier {
            resolved_depth,
            name,
            span,
        } = identifier;

        let value = self.eval_expression(expression)?;
        if let Some(distance) = *resolved_depth {
            self.environment
                .assign_at(distance, name, value.clone())
                .map_err(|err| err.with_span(span.clone()))?;
        } else {
            self.globals
                .assign(name.clone(), value.clone())
                .map_err(|err| err.with_span(span.clone()))?
        }
        Ok(value)
    }

    fn eval_literal_value(&mut self, val: &LiteralValue) -> Result<LoxObject, LoxError> {
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
        span: &Span,
    ) -> Result<LoxObject, LoxError> {
        let expr_value = self.eval_expression(expression)?;
        match operator {
            UnaryOperator::Minus => expr_value.neg(),
            UnaryOperator::Bang => expr_value.not(),
        }
        .map_err(|err| err.with_span(span.clone()))
    }

    fn eval_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        span: &Span,
    ) -> Result<LoxObject, LoxError> {
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
        .map_err(|err| err.with_span(span.clone()))
    }

    fn eval_logical(
        &mut self,
        left: &Expression,
        operator: &LogicalOperator,
        right: &Expression,
    ) -> Result<LoxObject, LoxError> {
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
        span: &Span,
    ) -> Result<LoxObject, LoxError> {
        let callee_obj = self.eval_expression(callee)?;
        let args = argument_expressions
            .iter()
            .map(|expr| self.eval_expression(expr))
            .collect::<Result<Vec<LoxObject>, LoxError>>()?;
        match &callee_obj {
            LoxObject::Class(class) => self
                .instantiate(class, &args)
                .map_err(|err| err.with_span(span.clone())),
            LoxObject::NativeFunction(func) => {
                let NativeFunction { callable, .. } = func.as_ref();
                Ok(
                    callable(&args, &self.environment)
                        .map_err(|err| err.with_span(span.clone()))?,
                )
            }
            LoxObject::UserFunction(func) => {
                let UserFunction {
                    parameters,
                    code_block,
                    closure,
                    is_initializer,
                    ..
                } = func.as_ref();
                if parameters.len() != args.len() {
                    return Err(new_runtime_error(
                        format!(
                            "Expected {} arguments but got {}.",
                            parameters.len(),
                            args.len()
                        ),
                        Some(span),
                    ));
                };
                self.eval_call(parameters, code_block, &args, closure, *is_initializer)
            }
            _ => Err(new_runtime_error(
                "Can only call functions and classes.".to_string(),
                Some(callee.span()),
            )),
        }
    }

    fn instantiate(
        &mut self,
        class: &Rc<Class>,
        arguments: &[LoxObject],
    ) -> Result<LoxObject, LoxError> {
        if class.arity() as usize != arguments.len() {
            return Err(new_runtime_error(
                format!(
                    "Expected {} arguments but got {}.",
                    class.arity(),
                    arguments.len()
                ),
                None,
            ));
        }
        let instance_rc = Rc::new(Instance::new(Rc::clone(class)));
        let object = LoxObject::Instance(instance_rc);

        if let Some(initializer) = class.get_initializer() {
            let callable = if initializer.is_bound() {
                initializer
            } else {
                Rc::new(initializer.bind(object.clone()))
            };
            let UserFunction {
                parameters,
                code_block,
                closure,
                is_initializer,
                ..
            } = callable.as_ref();
            self.eval_call(parameters, code_block, arguments, closure, *is_initializer)?;
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
    ) -> Result<LoxObject, LoxError> {
        let enclosing = self.environment.clone();
        self.environment = closure.enter_scope("arguments".to_string());

        let _ = std::iter::zip(parameters, args)
            .map(|(name, value)| self.environment.define(name.clone(), value.clone()))
            .collect::<Vec<_>>();

        let jump_kind = self.exec_statement(code_block)?;
        if is_initializer {
            let result = closure.get_at(0, &"this".to_string())?;
            self.environment = enclosing;
            return Ok(result);
        }
        let result = match jump_kind {
            JumpKind::Return(object) => object,
            _ => LoxObject::Nil,
        };

        self.environment = enclosing;
        Ok(result)
    }
}
