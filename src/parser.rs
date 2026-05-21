use crate::{
    ast::{
        BinaryOperator, Expression, LiteralValue, LogicalOperator, Program, Statement,
        UnaryOperator,
    },
    token::{Token, TokenType},
};

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    is_inside_loop: bool, // break is possible
}

type ParserError = String;
type ParserResult<T> = Result<T, ParserError>;

pub fn parse_program(tokens: Vec<Token>) -> ParserResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.program()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            is_inside_loop: false,
        }
    }

    fn program(&mut self) -> ParserResult<Program> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while !self.is_at_end() {
            // TODO: synchronize
            program.statements.push(self.declaration()?);
        }
        Ok(program)
    }
    fn declaration(&mut self) -> ParserResult<Statement> {
        match self.peek().token_type {
            TokenType::Fun => {
                self.advance();
                self.fun_declaration()
            }
            TokenType::Var => {
                self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn fun_declaration(&mut self) -> ParserResult<Statement> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err("Expected function name after 'fun'.".to_string());
        };
        self.advance();
        self.expect_token(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err("Function can't have more than 255 params.".to_string());
                }

                let param = if let TokenType::Identifier(param) = &self.peek().token_type {
                    param.clone()
                } else {
                    return Err("Expected function parameter to be identifier.".to_string());
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

        let block = self.block_statement()?;

        return Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            body: Box::new(block),
        });
    }

    fn var_declaration(&mut self) -> ParserResult<Statement> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err("Expected variable name.".to_string());
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

    fn statement(&mut self) -> ParserResult<Statement> {
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
                self.advance();
                self.expect_token(TokenType::Semicolon, "Expected ';' after 'break'.")?;
                if self.is_inside_loop {
                    Ok(Statement::Break)
                } else {
                    Err("'break' outside of loop body.".to_string())
                }
            }
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self) -> ParserResult<Statement> {
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

    fn print_statement(&mut self) -> ParserResult<Statement> {
        let expr = self.expression()?;
        self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
        Ok(Statement::Print {
            expression: Box::new(expr),
        })
    }

    fn if_statement(&mut self) -> ParserResult<Statement> {
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

    fn while_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expected ')' after loop condition.")?;
        self.is_inside_loop = true;
        let body = Box::new(self.statement()?);
        self.is_inside_loop = false;

        Ok(Statement::WhileLoop { condition, body })
    }

    fn for_statement(&mut self) -> ParserResult<Statement> {
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

    fn expression_statement(&mut self) -> ParserResult<Statement> {
        let expr = self.expression()?;

        self.expect_token(TokenType::Semicolon, "Expected ';' after statement.")?;
        Ok(Statement::Expression {
            expression: Box::new(expr),
        })
    }

    fn expect_token(&mut self, expected: TokenType, error_msg: &str) -> ParserResult<()> {
        if self.peek().token_type == expected {
            self.advance();
            Ok(())
        } else {
            Err(error_msg.to_string())
        }
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expr = self.or()?;

        if let TokenType::Equal = self.peek().token_type {
            self.advance();
            match expr {
                Expression::Identifier { name } => {
                    let value = self.assignment()?;
                    return Ok(Expression::Assignment {
                        name: name.clone(),
                        expression: Box::new(value),
                    });
                }
                _ => return Err("Invalid assignment target.".to_string()),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParserResult<Expression> {
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

    fn and(&mut self) -> ParserResult<Expression> {
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

    fn equality(&mut self) -> ParserResult<Expression> {
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

    fn comparison(&mut self) -> ParserResult<Expression> {
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

    fn term(&mut self) -> ParserResult<Expression> {
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

    fn factor(&mut self) -> ParserResult<Expression> {
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

    fn unary(&mut self) -> ParserResult<Expression> {
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

    fn call(&mut self) -> ParserResult<Expression> {
        let mut expr = self.primary()?;

        loop {
            match self.peek().token_type {
                TokenType::LeftParen => {
                    self.advance();
                    expr = self.finish_call(Box::new(expr))?;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expression>) -> ParserResult<Expression> {
        let mut args = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err("Can't have more than 255 arguments.".to_string());
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

        return Ok(Expression::Call {
            callee,
            arguments: args,
        });
    }

    fn primary(&mut self) -> ParserResult<Expression> {
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
            TokenType::Identifier(name) => Identifier { name: name.clone() },
            _ => return self.grouping(),
        };
        self.advance();
        Ok(expression)
    }

    fn grouping(&mut self) -> ParserResult<Expression> {
        if matches!(self.peek().token_type, TokenType::LeftParen) {
            self.advance();
            let expr = self.expression()?;
            if matches!(self.peek().token_type, TokenType::RightParen) {
                self.advance();
                Ok(Expression::Grouping {
                    expression: Box::new(expr),
                })
            } else {
                Err(format!("Expected: ')', got: {}", self.peek().lexeme))
            }
        } else {
            self.fallback()
        }
    }

    fn fallback(&mut self) -> ParserResult<Expression> {
        match self.peek().token_type {
            TokenType::Eof => Err("Unexpected EOF".to_string()),
            _ => Err(format!("Unexpected token: {}", self.peek().lexeme)),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.current];
        self.current += 1;
        token
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }
}
