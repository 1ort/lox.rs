use crate::{
    ast::{
        BinaryOperator, Expression, FunctionStatement, Identifier, LiteralValue, LogicalOperator,
        Program, Statement, UnaryOperator,
    },
    compile::error_reporter::ErrorReporter,
    compile::token::{Token, TokenType},
    error::{LoxError, new_syntax_error},
    span::Span,
};
use core::slice::Iter;
use std::iter::Peekable;

pub fn parse_program(
    tokens: Vec<Token>,
    error_reporter: &impl ErrorReporter,
) -> Result<Program, ()> {
    let parser = Parser {
        tokens: tokens.iter().peekable(),
        is_inside_loop: false,
        is_inside_function_body: false,
        error_reporter,
        has_errors: false,
    };

    parser.program()
}

struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    is_inside_loop: bool,
    is_inside_function_body: bool,
    error_reporter: &'a dyn ErrorReporter,
    has_errors: bool,
}

impl<'a> Parser<'a> {
    fn program(mut self) -> Result<Program, ()> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => program.statements.push(stmt),
                Err(err) => {
                    self.error_reporter.report(&err);
                    self.has_errors = true;
                    self.synchronize();
                }
            }
        }
        if self.has_errors {
            Err(())
        } else {
            Ok(program)
        }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::Semicolon => {
                    self.advance();
                    return;
                }
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn declaration(&mut self) -> Result<Statement, LoxError> {
        match self.peek().token_type {
            TokenType::Fun => {
                let start_span = self.advance().span.clone();
                let fun = self.fun_declaration()?;
                Ok(Statement::FunctionDeclaration {
                    span: start_span.union(&fun.span),
                    function: fun,
                })
            }
            TokenType::Var => self.var_declaration(),
            TokenType::Class => self.class_declaration(),
            _ => self.statement(),
        }
    }

    fn class_declaration(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(new_syntax_error(
                "Expect class name after 'class'.".to_owned(),
                self.peek().clone(),
            ));
        };
        self.advance();
        let need_superclass = matches!(self.peek().token_type, TokenType::Less);
        let superclass = if need_superclass {
            self.advance();
            let superclass = match &self.peek().token_type {
                TokenType::Identifier(superclass) => superclass.clone(),
                _ => {
                    return Err(new_syntax_error(
                        "Expect superclass name.".to_owned(),
                        self.peek().clone(),
                    ));
                }
            };
            let ident_span = &self.advance().span;
            Some(Identifier::new(superclass, ident_span))
        } else {
            None
        };
        self.expect_token(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods: Vec<FunctionStatement> = Vec::new();
        loop {
            if matches!(
                self.peek().token_type,
                TokenType::Eof | TokenType::RightBrace
            ) {
                break;
            }
            match self.fun_declaration() {
                Ok(stmt) => methods.push(stmt),
                Err(err) => {
                    self.error_reporter.report(&err);
                    self.has_errors = true;
                    self.synchronize();
                }
            }
        }
        let end_span = &self
            .expect_token(TokenType::RightBrace, "Expect '}' after class body.")?
            .span;
        Ok(Statement::ClassDeclaration {
            name,
            superclass,
            methods,
            span: start_span.union(end_span),
        })
    }

    fn fun_declaration(&mut self) -> Result<FunctionStatement, LoxError> {
        let start_span = self.span();
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(new_syntax_error(
                "Expected function name.".to_owned(),
                self.peek().clone(),
            ));
        };
        self.advance();
        self.expect_token(TokenType::LeftParen, "Expected '(' after function name")?;
        let mut parameters = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(new_syntax_error(
                        "Can't have more than 255 parameters.".to_owned(),
                        self.peek().clone(),
                    ));
                }

                let param = if let TokenType::Identifier(param) = &self.peek().token_type {
                    param.clone()
                } else {
                    return Err(new_syntax_error(
                        "Expect function parameter to be identifier.".to_owned(),
                        self.peek().clone(),
                    ));
                };

                let param = Identifier::new(param, &self.span());
                parameters.push(param);
                self.advance();

                if matches!(self.peek().token_type, TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(TokenType::RightParen, "Expect ')' after parameters.")?;
        let block_start_span = self
            .expect_token(TokenType::LeftBrace, "Expect '{' before function body.")?
            .span
            .clone();

        let enclosing = self.is_inside_function_body;
        self.is_inside_function_body = true;
        let block = self.block_statement(&block_start_span)?;
        self.is_inside_function_body = enclosing;

        Ok(FunctionStatement {
            name,
            parameters,
            span: start_span.union(block.span()),
            body: Box::new(block),
        })
    }

    fn var_declaration(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(new_syntax_error(
                "Expect variable name.".to_owned(),
                self.peek().clone(),
            ));
        };

        self.advance();

        let initializer = if let TokenType::Equal = &self.peek().token_type {
            self.advance();
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        let end_span = &self
            .expect_token(TokenType::Semicolon, "Expected ';' after statement.")?
            .span;

        Ok(Statement::VarDeclaration {
            name,
            initializer,
            span: start_span.union(end_span),
        })
    }

    fn statement(&mut self) -> Result<Statement, LoxError> {
        match self.peek().token_type {
            TokenType::LeftBrace => {
                let span = self.advance().span.clone();
                self.block_statement(&span)
            }
            TokenType::Print => self.print_statement(),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::Break => {
                let tok = self.advance().clone();
                if self.is_inside_loop {
                    let end_span = &self
                        .expect_token(TokenType::Semicolon, "Expected ';' after 'break'.")?
                        .span;
                    Ok(Statement::Break {
                        span: tok.span.union(end_span),
                    })
                } else {
                    Err(new_syntax_error(
                        "'break' outside of loop body.".to_owned(),
                        tok,
                    ))
                }
            }
            TokenType::Return => self.return_statement(),
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self, span: &Span) -> Result<Statement, LoxError> {
        let mut statements = Vec::new();
        loop {
            if matches!(self.peek().token_type, TokenType::RightBrace) || self.is_at_end() {
                break;
            }
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.error_reporter.report(&err);
                    self.has_errors = true;

                    self.synchronize();
                }
            }
        }
        let end_span = &self
            .expect_token(TokenType::RightBrace, "Expected '}' after block.")?
            .span;
        Ok(Statement::Block {
            statements,
            span: span.union(end_span),
        })
    }

    fn print_statement(&mut self) -> Result<Statement, LoxError> {
        let span = self.advance().span.clone();
        let expr = self.expression()?;
        let end_span = &self
            .expect_token(TokenType::Semicolon, "Expected ';' after statement.")?
            .span;
        Ok(Statement::Print {
            expression: Box::new(expr),
            span: span.union(end_span),
        })
    }

    fn if_statement(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expected ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);
        let mut end_span = then_branch.span().clone();

        let else_branch = if matches!(self.peek().token_type, TokenType::Else) {
            self.advance();
            let stmt = Box::new(self.statement()?);
            end_span = stmt.span().clone();
            Some(stmt)
        } else {
            None
        };

        Ok(Statement::Conditional {
            condition,
            then_branch,
            else_branch,
            span: start_span.union(&end_span),
        })
    }

    fn while_statement(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expected ')' after loop condition.")?;

        let enclosing = self.is_inside_loop;
        self.is_inside_loop = true;
        let body = Box::new(self.statement()?);
        self.is_inside_loop = enclosing;

        Ok(Statement::WhileLoop {
            condition,
            span: start_span.union(body.span()),
            body,
        })
    }

    fn for_statement(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'for'.")?;
        let initializer = match self.peek().token_type {
            TokenType::Semicolon => {
                let span = self.advance().span.clone();
                Statement::Pass { span }
            }
            TokenType::Var => self.var_declaration()?,
            _ => self.expression_statement()?,
        };
        let condition = match self.peek().token_type {
            TokenType::Semicolon => Expression::Literal {
                span: self.span(),
                value: LiteralValue::Boolean(true),
            },
            _ => self.expression()?,
        };
        self.expect_token(TokenType::Semicolon, "Expected ';' after condition.")?;
        let increment = match self.peek().token_type {
            TokenType::RightParen => Statement::Pass { span: self.span() },
            _ => {
                let expr = self.expression()?;
                Statement::Expression {
                    span: expr.span().clone(),
                    expression: Box::new(expr),
                }
            }
        };

        self.expect_token(TokenType::RightParen, "Expected ')' after 'for' clauses.")?;
        self.is_inside_loop = true;
        let body = self.statement()?;
        self.is_inside_loop = false;

        let while_body = Statement::Block {
            span: body.span().clone(),
            statements: vec![body, increment],
        };
        let while_loop = Statement::WhileLoop {
            condition: Box::new(condition),
            span: start_span.union(while_body.span()),
            body: Box::new(while_body),
        };
        let stmt = Statement::Block {
            span: while_loop.span().clone(),
            statements: vec![initializer, while_loop],
        };
        Ok(stmt)
    }

    fn return_statement(&mut self) -> Result<Statement, LoxError> {
        let start_span = self.advance().span.clone();
        match self.peek().token_type {
            TokenType::Semicolon => {
                let end_span = self.advance().span.clone();
                Ok(Statement::Return {
                    expresstion: None,
                    span: end_span,
                })
            }
            _ => {
                let expr = Box::new(self.expression()?);
                let end_span = &self
                    .expect_token(TokenType::Semicolon, "Expect ';' after statement.")?
                    .span;

                Ok(Statement::Return {
                    span: start_span.union(end_span),
                    expresstion: Some(expr),
                })
            }
        }
    }

    fn expression_statement(&mut self) -> Result<Statement, LoxError> {
        let expr = self.expression()?;
        let end_span = &self
            .expect_token(TokenType::Semicolon, "Expect ';' after expression.")?
            .span;
        Ok(Statement::Expression {
            span: expr.span().union(end_span),
            expression: Box::new(expr),
        })
    }

    fn expect_token(&mut self, expected: TokenType, error_msg: &str) -> Result<&Token, LoxError> {
        if self.peek().token_type == expected {
            Ok(self.advance())
        } else {
            Err(new_syntax_error(error_msg.to_owned(), self.peek().clone()))
        }
    }

    fn expression(&mut self) -> Result<Expression, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, LoxError> {
        let expr = self.or()?;

        if let TokenType::Equal = self.peek().token_type {
            let tok = self.advance();
            match expr {
                Expression::Identifier { identifier, span } => {
                    let value = self.assignment()?;
                    return Ok(Expression::Assignment {
                        identifier,
                        span: span.union(value.span()),
                        expression: Box::new(value),
                    });
                }
                Expression::Get { object, name, span } => {
                    let value = self.assignment()?;
                    return Ok(Expression::Set {
                        object,
                        name,
                        span: span.union(value.span()),
                        expression: Box::new(value),
                    });
                }

                _ => {
                    return Err(new_syntax_error(
                        "Invalid assignment target.".to_owned(),
                        tok.clone(),
                    ));
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.and()?;

        loop {
            if matches!(self.peek().token_type, TokenType::Or) {
                self.advance();
                let right = self.and()?;
                expr = Expression::Logical {
                    span: expr.span().union(right.span()),
                    left: Box::new(expr),
                    operator: LogicalOperator::Or,
                    right: Box::new(right),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.equality()?;

        loop {
            if matches!(self.peek().token_type, TokenType::And) {
                self.advance();
                let right = self.equality()?;
                expr = Expression::Logical {
                    span: expr.span().union(right.span()),
                    left: Box::new(expr),
                    operator: LogicalOperator::And,
                    right: Box::new(right),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.comparison()?;
        loop {
            let binary_operator = match self.peek().token_type {
                TokenType::BangEqual => BinaryOperator::BangEqual,
                TokenType::EqualEqual => BinaryOperator::EqualEqual,
                _ => break,
            };
            self.advance();
            let right = self.comparison()?;
            expr = Expression::Binary {
                span: expr.span().union(right.span()),
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.term()?;
        loop {
            let binary_operator = match self.peek().token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                _ => break,
            };
            self.advance();
            let right = self.term()?;
            expr = Expression::Binary {
                span: expr.span().union(right.span()),
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.factor()?;
        loop {
            let binary_operator = match self.peek().token_type {
                TokenType::Minus => BinaryOperator::Minus,
                TokenType::Plus => BinaryOperator::Plus,
                _ => break,
            };
            self.advance();
            let right = self.factor()?;
            expr = Expression::Binary {
                span: expr.span().union(right.span()),
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.unary()?;
        loop {
            let binary_operator = match self.peek().token_type {
                TokenType::Slash => BinaryOperator::Slash,
                TokenType::Star => BinaryOperator::Star,
                _ => break,
            };
            self.advance();
            let right = self.unary()?;
            expr = Expression::Binary {
                span: expr.span().union(right.span()),
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, LoxError> {
        let unary_operator = match self.peek().token_type {
            TokenType::Bang => UnaryOperator::Bang,
            TokenType::Minus => UnaryOperator::Minus,
            _ => return self.call(),
        };
        let start_span = self.advance().span.clone();
        let expr = self.unary()?;
        Ok(Expression::Unary {
            span: start_span.union(expr.span()),
            operator: unary_operator,
            expression: Box::new(expr),
        })
    }

    fn call(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.primary()?;
        loop {
            expr = match self.peek().token_type {
                TokenType::LeftParen => {
                    self.advance();
                    self.finish_call(Box::new(expr))?
                }
                TokenType::Dot => {
                    self.advance();
                    self.finish_get(Box::new(expr))?
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn finish_get(&mut self, object: Box<Expression>) -> Result<Expression, LoxError> {
        let field_name = match &self.peek().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => {
                return Err(new_syntax_error(
                    "Expect property name after '.'.".to_owned(),
                    self.peek().clone(),
                ));
            }
        };
        let end_span = &self.advance().span;
        Ok(Expression::Get {
            span: object.span().union(end_span),
            object,
            name: field_name,
        })
    }

    fn finish_call(&mut self, callee: Box<Expression>) -> Result<Expression, LoxError> {
        let mut args = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(new_syntax_error(
                        "Can't have more than 255 arguments.".to_owned(),
                        self.peek().clone(),
                    ));
                }
                args.push(self.expression()?);

                if matches!(self.peek().token_type, TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        let end_span = &self
            .expect_token(TokenType::RightParen, "Expect ')' after arguments.")?
            .span;
        Ok(Expression::Call {
            span: callee.span().union(end_span),
            callee,
            arguments: args,
        })
    }

    fn primary(&mut self) -> Result<Expression, LoxError> {
        use Expression::{Identifier, Literal, This};
        use LiteralValue::*;
        let span = self.span();
        let expression = match &self.peek().token_type {
            TokenType::False => Literal {
                value: Boolean(false),
                span,
            },
            TokenType::True => Literal {
                value: Boolean(true),
                span,
            },
            TokenType::Nil => Literal { value: Nil, span },
            TokenType::Number(num) => {
                let int_part = num.clone();
                self.advance();
                if !matches!(self.peek().token_type, TokenType::Dot) {
                    return Ok(Literal {
                        value: Number(int_part.parse::<f64>().unwrap()),
                        span,
                    });
                }
                self.advance();
                match &self.peek().token_type {
                    TokenType::Number(fract_part) => Literal {
                        value: Number(
                            format!("{}.{}", int_part, fract_part)
                                .parse::<f64>()
                                .unwrap(),
                        ),
                        span,
                    },
                    _ => {
                        return self.finish_get(Box::new(Literal {
                            value: Number(int_part.parse::<f64>().unwrap()),
                            span,
                        }));
                    }
                }
            }
            TokenType::String(string) => Literal {
                value: String(string.clone()),
                span,
            },
            TokenType::Identifier(name) => Identifier {
                identifier: crate::ast::Identifier::new(name.clone(), &span),
                span,
            },
            TokenType::This => This {
                identifier: crate::ast::Identifier::new("this".to_string(), &span),
                span,
            },
            TokenType::Super => {
                self.advance();
                self.expect_token(TokenType::Dot, "Expect '.' after 'super'.")?;

                let method = if let TokenType::Identifier(ref name) = self.peek().token_type {
                    name.clone()
                } else {
                    return Err(new_syntax_error(
                        "Expect superclass method name.".to_owned(),
                        self.peek().clone(),
                    ));
                };
                Expression::Super {
                    identifier: crate::ast::Identifier::new("super".to_string(), &span),
                    method,
                    span: span.union(&self.span()),
                }
            }
            _ => return self.grouping(),
        };
        self.advance();
        Ok(expression)
    }

    fn grouping(&mut self) -> Result<Expression, LoxError> {
        if matches!(self.peek().token_type, TokenType::LeftParen) {
            let start_span = self.advance().span.clone();
            let expr = self.expression()?;
            if matches!(self.peek().token_type, TokenType::RightParen) {
                let end_span = &self.advance().span;
                Ok(Expression::Grouping {
                    span: start_span.union(end_span),
                    expression: Box::new(expr),
                })
            } else {
                Err(new_syntax_error(
                    "Expected ')'".to_owned(),
                    self.peek().clone(),
                ))
            }
        } else {
            self.fallback()
        }
    }

    fn fallback(&mut self) -> Result<Expression, LoxError> {
        match self.peek().token_type {
            TokenType::Eof => Err(new_syntax_error(
                "Unexpected EOF".to_owned(),
                self.peek().clone(),
            )),
            TokenType::Unexpected(ref message) => {
                Err(new_syntax_error(message.clone(), self.peek().clone()))
            }
            _ => Err(new_syntax_error(
                "Expect expression.".to_owned(),
                self.peek().clone(),
            )),
        }
    }

    fn peek(&mut self) -> &Token {
        self.tokens.peek().unwrap()
    }

    fn advance(&mut self) -> &Token {
        self.tokens.next().unwrap()
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn span(&mut self) -> Span {
        self.peek().span.clone()
    }
}
