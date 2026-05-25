use core::slice::Iter;
use std::{cell::RefCell, iter::Peekable};

use crate::{
    ast::{
        BinaryOperator, Expression, FunctionStatement, LiteralValue, LogicalOperator, Program,
        Statement, UnaryOperator,
    },
    interruption::{Interruption, brake_inter, parser_error},
    token::{Token, TokenType},
};

type TokensPeekable<'a> = Peekable<Iter<'a, Token>>;

struct Parser<'a> {
    tokens: TokensPeekable<'a>,
    is_inside_loop: bool,
    is_inside_function_body: bool,
}

pub fn parse_program(tokens: Vec<Token>) -> Result<Program, Interruption> {
    let tokens_it = tokens.iter().peekable();
    let mut parser = Parser::new(tokens_it);
    parser.program()
}

impl<'a> Parser<'a> {
    fn new(tokens: TokensPeekable<'a>) -> Parser<'a> {
        Parser {
            tokens,
            is_inside_loop: false,
            is_inside_function_body: false,
        }
    }

    fn program(&mut self) -> Result<Program, Interruption> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while !self.is_at_end() {
            // TODO: synchronize
            program.statements.push(self.declaration()?);
        }
        Ok(program)
    }
    fn declaration(&mut self) -> Result<Statement, Interruption> {
        match self.peek().token_type {
            TokenType::Fun => {
                self.advance();
                self.fun_declaration().map(Statement::FunctionDeclaration)
            }
            TokenType::Var => {
                self.advance();
                self.var_declaration()
            }
            TokenType::Class => {
                self.advance();
                self.class_declaration()
            }
            _ => self.statement(),
        }
    }

    fn class_declaration(&mut self) -> Result<Statement, Interruption> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(parser_error(
                self.peek().clone(),
                "Expected class name after 'class'.",
            ));
        };
        self.advance();

        self.expect_token(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods: Vec<FunctionStatement> = Vec::new();

        loop {
            if matches!(
                self.peek().token_type,
                TokenType::Eof | TokenType::RightBrace
            ) {
                break;
            }
            methods.push(self.fun_declaration()?);
        }

        self.expect_token(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Statement::ClassDeclaration { name, methods })
    }

    fn fun_declaration(&mut self) -> Result<FunctionStatement, Interruption> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(parser_error(self.peek().clone(), "Expected function name."));
        };
        self.advance();
        self.expect_token(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(parser_error(
                        self.peek().clone(),
                        "Function can't have more than 255 params.",
                    ));
                }

                let param = if let TokenType::Identifier(param) = &self.peek().token_type {
                    param.clone()
                } else {
                    return Err(parser_error(
                        self.peek().clone(),
                        "Expected function parameter to be identifier.",
                    ));
                };
                parameters.push(param);
                self.advance();

                if matches!(self.peek().token_type, TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(TokenType::RightParen, "Expect ')' after function params.")?;
        self.expect_token(TokenType::LeftBrace, "Expect '{' before function body.")?;

        let block = if self.is_inside_function_body {
            self.block_statement()?
        } else {
            self.is_inside_function_body = true;
            let stmt = self.block_statement()?;
            self.is_inside_function_body = false;
            stmt
        };

        Ok(FunctionStatement {
            name,
            parameters,
            body: Box::new(block),
        })
    }

    fn var_declaration(&mut self) -> Result<Statement, Interruption> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(parser_error(self.peek().clone(), "Variable name expected."));
        };

        self.advance();

        let initializer = if let TokenType::Equal = &self.peek().token_type {
            self.advance();
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
        Ok(Statement::VarDeclaration { name, initializer })
    }

    fn statement(&mut self) -> Result<Statement, Interruption> {
        match self.peek().token_type {
            TokenType::LeftBrace => {
                self.advance();
                self.block_statement()
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            TokenType::For => {
                self.advance();
                self.for_statement()
            }
            TokenType::Break => {
                let tok = self.advance().clone();
                self.expect_token(TokenType::Semicolon, "Expected ';' after 'break'.")?;
                if self.is_inside_loop {
                    Ok(Statement::Break)
                } else {
                    Err(parser_error(tok, "'break' outside of loop body."))
                }
            }
            TokenType::Return => {
                self.advance();
                self.return_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self) -> Result<Statement, Interruption> {
        let mut statements = Vec::new();
        loop {
            if matches!(self.peek().token_type, TokenType::RightBrace) || self.is_at_end() {
                break;
            }
            statements.push(self.declaration()?);
        }

        self.expect_token(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(Statement::Block { statements })
    }

    fn print_statement(&mut self) -> Result<Statement, Interruption> {
        let expr = self.expression()?;
        self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
        Ok(Statement::Print {
            expression: Box::new(expr),
        })
    }

    fn if_statement(&mut self) -> Result<Statement, Interruption> {
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expected ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);

        let else_branch = if matches!(self.peek().token_type, TokenType::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::Conditional {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement, Interruption> {
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expected ')' after loop condition.")?;

        let body = if self.is_inside_loop {
            Box::new(self.statement()?)
        } else {
            self.is_inside_loop = true;
            let stmt = Box::new(self.statement()?);
            self.is_inside_loop = false;
            stmt
        };

        Ok(Statement::WhileLoop { condition, body })
    }

    fn for_statement(&mut self) -> Result<Statement, Interruption> {
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'for'.")?;
        let maybe_initializer = match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
                None
            }
            TokenType::Var => {
                self.advance();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };

        let condition = match self.peek().token_type {
            TokenType::Semicolon => Expression::Literal {
                value: LiteralValue::Boolean(true),
            },
            _ => self.expression()?,
        };
        self.expect_token(TokenType::Semicolon, "Expected ';' after condition.")?;

        let maybe_increment = match self.peek().token_type {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.expect_token(TokenType::RightParen, "Expected ')' after 'for' clauses.")?;
        self.is_inside_loop = true;
        let body = self.statement()?;
        self.is_inside_loop = false;

        let while_body = if let Some(increment) = maybe_increment {
            Statement::Block {
                statements: vec![
                    body,
                    Statement::Expression {
                        expression: Box::new(increment),
                    },
                ],
            }
        } else {
            body
        };

        let while_loop = Statement::WhileLoop {
            condition: Box::new(condition),
            body: Box::new(while_body),
        };

        let statement = if let Some(initializer) = maybe_initializer {
            Statement::Block {
                statements: vec![initializer, while_loop],
            }
        } else {
            while_loop
        };

        Ok(statement)
    }

    fn return_statement(&mut self) -> Result<Statement, Interruption> {
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
                Ok(Statement::Return { expresstion: None })
            }
            _ => {
                let stmt = Ok(Statement::Return {
                    expresstion: Some(Box::new(self.expression()?)),
                });
                self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
                stmt
            }
        }
    }

    fn expression_statement(&mut self) -> Result<Statement, Interruption> {
        let expr = self.expression()?;

        self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
        Ok(Statement::Expression {
            expression: Box::new(expr),
        })
    }

    fn expect_token(&mut self, expected: TokenType, error_msg: &str) -> Result<(), Interruption> {
        if self.peek().token_type == expected {
            self.advance();
            Ok(())
        } else {
            Err(parser_error(self.peek().clone(), error_msg))
        }
    }

    fn expression(&mut self) -> Result<Expression, Interruption> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, Interruption> {
        let expr = self.or()?;

        if let TokenType::Equal = self.peek().token_type {
            let tok = self.advance();
            match expr {
                Expression::Identifier { name, .. } => {
                    let value = self.assignment()?;
                    return Ok(Expression::Assignment {
                        name: name.clone(),
                        expression: Box::new(value),
                        resolved_scope_depth: RefCell::new(None),
                    });
                }
                Expression::Get { object, name } => {
                    let value = self.assignment()?;
                    return Ok(Expression::Set {
                        object,
                        name,
                        expression: Box::new(value),
                    });
                }

                _ => return Err(parser_error(tok.clone(), "Invalid assignment target.")),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expression, Interruption> {
        let mut expr = self.and()?;

        loop {
            if matches!(self.peek().token_type, TokenType::Or) {
                self.advance();
                let right = self.and()?;
                expr = Expression::Logical {
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

    fn and(&mut self) -> Result<Expression, Interruption> {
        let mut expr = self.equality()?;

        loop {
            if matches!(self.peek().token_type, TokenType::And) {
                self.advance();
                let right = self.equality()?;
                expr = Expression::Logical {
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

    fn equality(&mut self) -> Result<Expression, Interruption> {
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
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, Interruption> {
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
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, Interruption> {
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
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, Interruption> {
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
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, Interruption> {
        let unary_operator = match self.peek().token_type {
            TokenType::Bang => UnaryOperator::Bang,
            TokenType::Minus => UnaryOperator::Minus,
            _ => return self.call(),
        };
        self.advance();
        let expr = self.unary()?;
        Ok(Expression::Unary {
            operator: unary_operator,
            expression: Box::new(expr),
        })
    }

    fn call(&mut self) -> Result<Expression, Interruption> {
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

    fn finish_get(&mut self, object: Box<Expression>) -> Result<Expression, Interruption> {
        let field_name = match &self.peek().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => {
                return Err(parser_error(
                    self.peek().clone(),
                    "Expected field name after '.'",
                ));
            }
        };
        self.advance();
        Ok(Expression::Get {
            object,
            name: field_name,
        })
    }

    fn finish_call(&mut self, callee: Box<Expression>) -> Result<Expression, Interruption> {
        let mut args = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(parser_error(
                        self.peek().clone(),
                        "Can't have more than 255 arguments.",
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
        self.expect_token(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expression::Call {
            callee,
            arguments: args,
        })
    }

    fn primary(&mut self) -> Result<Expression, Interruption> {
        use Expression::{Identifier, Literal};
        use LiteralValue::*;
        let expression = match &self.peek().token_type {
            TokenType::False => Literal {
                value: Boolean(false),
            },
            TokenType::True => Literal {
                value: Boolean(true),
            },
            TokenType::Nil => Literal { value: Nil },
            TokenType::Number(num) => Literal {
                value: Number(*num),
            },
            TokenType::String(string) => Literal {
                value: String(string.clone()),
            },
            TokenType::Identifier(name) => Identifier {
                name: name.clone(),
                resolved_scope_depth: RefCell::new(None),
            },
            TokenType::This => Expression::This {
                resolved_scope_depth: RefCell::new(None),
            },
            _ => return self.grouping(),
        };
        self.advance();
        Ok(expression)
    }

    fn grouping(&mut self) -> Result<Expression, Interruption> {
        if matches!(self.peek().token_type, TokenType::LeftParen) {
            self.advance();
            let expr = self.expression()?;
            if matches!(self.peek().token_type, TokenType::RightParen) {
                self.advance();
                Ok(Expression::Grouping {
                    expression: Box::new(expr),
                })
            } else {
                Err(parser_error(self.peek().clone(), "Expected ')'"))
            }
        } else {
            self.fallback()
        }
    }

    fn fallback(&mut self) -> Result<Expression, Interruption> {
        match self.peek().token_type {
            TokenType::Eof => Err(parser_error(self.peek().clone(), "Unexpected EOF")),
            _ => Err(parser_error(self.peek().clone(), "Unexpected token")),
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
}
