use std::collections::HashMap;

use crate::{
    ast::{Expression, FunctionStatement, Identifier, Program, Statement},
    interruption::{LoxError, new_resolver_error},
};

#[derive(Clone, Copy, Debug)]
enum DeclarationState {
    Declared,
    Defined,
}

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Initializer,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, DeclarationState>>,
    current_function_type: FunctionType,
    current_class_type: ClassType,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            current_function_type: FunctionType::None,
            current_class_type: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) -> Result<(), LoxError> {
        if matches!(self.get_state_in_current_scope(name), Some(..)) {
            return Err(new_resolver_error(
                format!(
                    "Error at '{}': Already a variable with this name in this scope.",
                    name
                ),
                None,
            ));
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), DeclarationState::Declared);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn define(&mut self, name: &str) {
        self.scopes
            .last_mut()
            .and_then(|scope| scope.insert(name.to_string(), DeclarationState::Defined));
    }

    fn get_state_in_current_scope(&self, name: &str) -> Option<DeclarationState> {
        if let Some(scope) = self.scopes.last() {
            scope.get(&name.to_string()).cloned()
        } else {
            None
        }
    }

    fn resolve_local(&mut self, identifier: &mut Identifier) {
        if let Some(depth) = self
            .scopes
            .iter()
            .rev()
            .position(|scope| scope.contains_key(&identifier.name))
        {
            self.fill_expression_depth(identifier, depth);
        }
    }

    fn fill_expression_depth(&mut self, identifier: &mut Identifier, depth: usize) {
        identifier.resolved_depth = Some(depth);
        //println!("resolved depth for {:?}", identifier);
    }

    pub fn resolve_program(&mut self, program: &mut Program) -> Result<(), LoxError> {
        let mut iterator = program.statements.iter_mut();

        for result in iterator.by_ref().map(|stmt| self.resolve_statement(stmt)) {
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &mut Statement) -> Result<(), LoxError> {
        match stmt {
            Statement::Block { statements, .. } => self.resolve_block_stmt(statements),
            Statement::VarDeclaration {
                name,
                initializer,
                span,
            } => self
                .resolve_var_declaration(name, initializer)
                .map_err(|err| err.with_span(span.clone())),
            Statement::FunctionDeclaration {
                function:
                    FunctionStatement {
                        name,
                        parameters,
                        body,
                        ..
                    },
                span,
            } => {
                self.declare(name)
                    .map_err(|err| err.with_span(span.clone()))?;
                self.define(name);
                self.resolve_function(parameters, body, FunctionType::Function)
            }
            Statement::Expression { expression, .. } => self.resolve_expression(expression),
            Statement::Print { expression, .. } => self.resolve_expression(expression),
            Statement::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(stmt) = else_branch {
                    self.resolve_statement(stmt)?;
                }
                Ok(())
            }
            Statement::WhileLoop {
                condition, body, ..
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)
            }
            Statement::Return {
                expresstion: expression,
                span,
            } => {
                if matches!(self.current_function_type, FunctionType::None) {
                    Err(new_resolver_error(
                        "Error at 'return': Can't return from top-level code.".to_string(),
                        Some(span),
                    ))
                } else if matches!(self.current_function_type, FunctionType::Initializer)
                    && expression.is_some()
                {
                    Err(new_resolver_error(
                        "Error at 'return': Can't return a value from an initializer.".to_string(),
                        Some(span),
                    ))
                } else if let Some(expr) = expression {
                    self.resolve_expression(expr)
                } else {
                    Ok(())
                }
            }
            Statement::Break { .. } => Ok(()),
            Statement::ClassDeclaration {
                name,
                superclass,
                methods,
                span,
            } => {
                self.declare(name)
                    .map_err(|err| err.with_span(span.clone()))?;
                self.define(name);

                if let Some(identifier) = superclass {
                    if identifier.name.eq(name) {
                        return Err(new_resolver_error(
                            format!("Error at '{}': A class can't inherit from itself.", name),
                            Some(&identifier.span),
                        ));
                    }
                    self.resolve_local(identifier);
                }
                self.begin_scope();

                let enclosing_class_type = self.current_class_type;
                self.current_class_type = if superclass.is_some() {
                    self.define("super");
                    ClassType::SubClass
                } else {
                    ClassType::Class
                };

                methods.iter_mut().try_for_each(|fun_stmt| {
                    let FunctionStatement {
                        parameters,
                        body,
                        name,
                        ..
                    } = fun_stmt;
                    self.begin_scope();
                    self.define("this");
                    self.resolve_function(
                        parameters,
                        body,
                        if name.eq(&"init") {
                            FunctionType::Initializer
                        } else {
                            FunctionType::Function
                        },
                    )?;
                    self.end_scope();
                    Ok(())
                })?;
                self.current_class_type = enclosing_class_type;
                self.end_scope();
                Ok(())
            }
            Statement::Pass { .. } => Ok(()),
        }
    }

    fn resolve_block_stmt(&mut self, statements: &mut [Statement]) -> Result<(), LoxError> {
        self.begin_scope();
        let result = statements
            .iter_mut()
            .try_for_each(|stmt| self.resolve_statement(stmt));
        self.end_scope();
        result
    }

    fn resolve_function(
        &mut self,
        parameters: &Vec<Identifier>,
        body: &mut Statement,
        function_type: FunctionType,
    ) -> Result<(), LoxError> {
        let enclosing_function_type = self.current_function_type;
        self.current_function_type = function_type;

        self.begin_scope();
        for param in parameters {
            self.declare(&param.name)
                .map_err(|err| err.with_span(param.span.clone()))?;
            self.define(&param.name);
        }
        self.resolve_statement(body)?;
        self.end_scope();

        self.current_function_type = enclosing_function_type;
        Ok(())
    }

    fn resolve_var_declaration(
        &mut self,
        name: &str,
        initializer: &mut Option<Box<Expression>>,
    ) -> Result<(), LoxError> {
        self.declare(name)?;
        if let Some(expr) = initializer {
            self.resolve_expression(expr)?;
        }
        self.define(name);
        Ok(())
    }

    fn resolve_expression(&mut self, expression: &mut Expression) -> Result<(), LoxError> {
        match expression {
            Expression::Identifier { identifier, .. } => {
                self.resolve_identifier_expression(identifier)
            }
            Expression::Assignment {
                identifier,
                expression,
                ..
            } => self.resolve_assignment_expression(identifier, expression),
            Expression::Unary { expression, .. } => self.resolve_expression(expression),
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            }
            Expression::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            }
            Expression::Grouping { expression, .. } => self.resolve_expression(expression),
            Expression::Call {
                callee, arguments, ..
            } => {
                if let Expression::Identifier {
                    identifier: Identifier { name, .. },
                    ..
                } = callee.as_ref()
                    && name.eq("dbgenv")
                {
                    println!("{:?}", {
                        let mut scopes = self.scopes.clone();
                        scopes.reverse();
                        scopes
                    });
                }
                self.resolve_expression(callee)?;
                arguments
                    .iter_mut()
                    .try_for_each(|expr| self.resolve_expression(expr))
            }
            Expression::Literal { .. } => Ok(()),
            Expression::Get { object, .. } => self.resolve_expression(object),
            Expression::Set {
                object, expression, ..
            } => {
                self.resolve_expression(object)?;
                self.resolve_expression(expression)
            }
            Expression::This { identifier, .. } => {
                if !matches!(self.current_class_type, ClassType::None) {
                    self.resolve_local(identifier);
                    Ok(())
                } else {
                    Err(new_resolver_error(
                        "Error at 'this': Can't use 'this' outside of a class.".to_string(),
                        Some(&identifier.span),
                    ))
                }
            }
            Expression::Super { identifier, .. } => match self.current_class_type {
                ClassType::None => Err(new_resolver_error(
                    "Error at 'super': Can't use 'super' outside of a class.".to_string(),
                    Some(&identifier.span),
                )),
                ClassType::Class => Err(new_resolver_error(
                    "Error at 'super': Can't use 'super' in a class with no superclass."
                        .to_string(),
                    Some(&identifier.span),
                )),
                ClassType::SubClass => {
                    self.resolve_local(identifier);
                    Ok(())
                }
            },
        }
    }

    fn resolve_identifier_expression(
        &mut self,
        identifier: &mut Identifier,
    ) -> Result<(), LoxError> {
        if !self.scopes.is_empty()
            && matches!(
                self.get_state_in_current_scope(&identifier.name),
                Some(DeclarationState::Declared)
            )
        {
            return Err(new_resolver_error(
                format!(
                    "Error at '{}': Can't read local variable in its own initializer.",
                    identifier.name
                ),
                Some(&identifier.span),
            ));
        }

        self.resolve_local(identifier);
        Ok(())
    }
    fn resolve_assignment_expression(
        &mut self,
        identifier: &mut Identifier,
        expression: &mut Expression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(expression)?;
        self.resolve_local(identifier);
        Ok(())
    }
}
