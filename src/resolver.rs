use std::collections::HashMap;

use crate::{
    ast::{Expression, FunctionStatement, Program, Statement},
    class::Class,
    interruption::{Interruption, resolver_error},
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

    fn declare(&mut self, name: &str) -> Result<(), Interruption> {
        if matches!(self.get_state_in_current_scope(name), Some(..)) {
            return Err(resolver_error(format!(
                "Already a variable with this name in this scope: {}",
                name
            )));
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), DeclarationState::Declared);
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), DeclarationState::Defined);
        } else {
            unreachable!()
        }
    }

    fn get_state_in_current_scope(&self, name: &str) -> Option<DeclarationState> {
        if let Some(scope) = self.scopes.last() {
            scope.get(&name.to_string()).cloned()
        } else {
            None
        }
    }

    fn resolve_local(&mut self, expr: &Expression, name: &str) {
        if let Some(depth) = self
            .scopes
            .iter()
            .rev()
            .position(|scope| scope.contains_key(name))
        {
            self.fill_expression_depth(expr, depth);
        }
    }

    fn fill_expression_depth(&mut self, expr: &Expression, depth: usize) {
        match expr {
            Expression::Identifier {
                resolved_scope_depth,
                ..
            }
            | Expression::This {
                resolved_scope_depth,
                ..
            }
            | Expression::Assignment {
                resolved_scope_depth,
                ..
            } => {
                let mut expr_value = resolved_scope_depth.borrow_mut();
                *expr_value = Some(depth);
            }
            _ => (),
        }
    }

    pub fn resolve_program(&mut self, program: &Program) -> Result<(), Interruption> {
        self.begin_scope();
        let mut iterator = program.statements.iter();

        for result in iterator.by_ref().map(|stmt| self.resolve_statement(stmt)) {
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement) -> Result<(), Interruption> {
        match stmt {
            Statement::Block { statements } => self.resolve_block_stmt(statements),
            Statement::VarDeclaration { name, initializer } => {
                self.resolve_var_declaration(name, initializer)
            }
            Statement::FunctionDeclaration(FunctionStatement {
                name,
                parameters,
                body,
            }) => {
                self.declare(name)?;
                self.define(name);
                self.resolve_function(parameters, body, FunctionType::Function)
            }
            Statement::Expression { expression } => self.resolve_expression(expression),
            Statement::Print { expression } => self.resolve_expression(expression),
            Statement::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(stmt) = else_branch {
                    self.resolve_statement(stmt)?;
                }
                Ok(())
            }
            Statement::WhileLoop { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)
            }
            Statement::Return {
                expresstion: expression,
            } => {
                if matches!(self.current_function_type, FunctionType::None) {
                    Err(resolver_error(
                        "Can't return from top-level code.".to_string(),
                    ))
                } else if matches!(self.current_function_type, FunctionType::Initializer)
                    && expression.is_some()
                {
                    Err(resolver_error(
                        "Can't return value from 'init' method.".to_string(),
                    ))
                } else if let Some(expr) = expression {
                    self.resolve_expression(expr)
                } else {
                    Ok(())
                }
            }
            Statement::Break => Ok(()),
            Statement::ClassDeclaration { name, methods } => {
                self.declare(name)?;
                self.define(name);
                self.begin_scope();

                let enclosing_class_type = self.current_class_type;
                self.current_class_type = ClassType::Class;

                self.define("this");
                methods.iter().try_for_each(|fun_stmt| {
                    let FunctionStatement {
                        parameters,
                        body,
                        name,
                    } = fun_stmt;
                    self.resolve_function(
                        parameters,
                        body,
                        if name.eq("init") {
                            FunctionType::Initializer
                        } else {
                            FunctionType::Function
                        },
                    )
                })?;
                self.current_class_type = enclosing_class_type;
                self.end_scope();
                Ok(())
            }
        }
    }

    fn resolve_block_stmt(&mut self, statements: &[Statement]) -> Result<(), Interruption> {
        self.begin_scope();
        let result = statements
            .iter()
            .try_for_each(|stmt| self.resolve_statement(stmt));
        self.end_scope();
        result
    }

    fn resolve_function(
        &mut self,
        parameters: &Vec<String>,
        body: &Statement,
        function_type: FunctionType,
    ) -> Result<(), Interruption> {
        let enclosing_function_type = self.current_function_type;
        self.current_function_type = function_type;

        self.begin_scope();
        for param in parameters {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statement(body)?;
        self.end_scope();

        self.current_function_type = enclosing_function_type;
        Ok(())
    }

    fn resolve_var_declaration(
        &mut self,
        name: &str,
        initializer: &Option<Box<Expression>>,
    ) -> Result<(), Interruption> {
        self.declare(name)?;
        if let Some(expr) = initializer {
            self.resolve_expression(expr)?;
        }
        self.define(name);
        Ok(())
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), Interruption> {
        match expression {
            Expression::Identifier { .. } => self.resolve_identifier_expression(expression),
            Expression::Assignment { .. } => self.resolve_assignment_expression(expression),
            Expression::Unary { expression, .. } => self.resolve_expression(expression),
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            }
            Expression::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            }
            Expression::Grouping { expression } => self.resolve_expression(expression),
            Expression::Call { callee, arguments } => {
                self.resolve_expression(callee)?;
                arguments
                    .iter()
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
            Expression::This { .. } => {
                if matches!(self.current_class_type, ClassType::Class) {
                    self.resolve_local(expression, "this");
                    Ok(())
                } else {
                    Err(resolver_error(
                        "Can't use 'this' outside of class method.".to_string(),
                    ))
                }
            }
        }
    }

    fn resolve_identifier_expression(&mut self, expr: &Expression) -> Result<(), Interruption> {
        let Expression::Identifier { name, .. } = expr else {
            unreachable!()
        };

        if matches!(
            self.get_state_in_current_scope(name),
            Some(DeclarationState::Declared)
        ) {
            return Err(resolver_error(format!(
                "Can't read local variable in its own initializer: '{}'.",
                name
            )));
        }

        self.resolve_local(expr, name);
        Ok(())
    }
    fn resolve_assignment_expression(&mut self, expr: &Expression) -> Result<(), Interruption> {
        let Expression::Assignment {
            name, expression, ..
        } = expr
        else {
            unreachable!()
        };
        self.resolve_expression(expression)?;
        self.resolve_local(expr, name);
        Ok(())
    }
}
