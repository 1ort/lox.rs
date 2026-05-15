use crate::{
    ast::{BinaryOperator, Expression, LiteralValue, Program, Statement, UnaryOperator},
    scanner::{Token, TokenType},
};

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

type ParserError = String;
type ParserResult<T> = Result<T, ParserError>;

pub fn parse_program(tokens: Vec<Token>) -> ParserResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.program()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
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
            TokenType::Var => {
                self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
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
        self.expect_semicolon()?;
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
        self.expect_closing_brace()?;
        Ok(Statement::Block { statements })
    }

    fn print_statement(&mut self) -> ParserResult<Statement> {
        let expr = self.expression()?;
        self.expect_semicolon()?;
        Ok(Statement::Print {
            expression: Box::new(expr),
        })
    }

    fn expression_statement(&mut self) -> ParserResult<Statement> {
        let expr = self.expression()?;
        self.expect_semicolon()?;
        Ok(Statement::Expression {
            expression: Box::new(expr),
        })
    }
    fn expect_semicolon(&mut self) -> ParserResult<()> {
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
                Ok(())
            }
            _ => Err("Expected ';' after statement".to_string()),
        }
    }

    fn expect_closing_brace(&mut self) -> ParserResult<()> {
        match self.peek().token_type {
            TokenType::RightBrace => {
                self.advance();
                Ok(())
            }
            _ => Err("Expected '}' after block".to_string()),
        }
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expr = self.equality()?;

        if let TokenType::Equal = self.peek().token_type {
            self.advance();
            match expr {
                Expression::Variable { name } => {
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
            _ => return self.primary(),
        };
        self.advance();
        let expr = self.unary()?;
        Ok(Expression::Unary {
            operator: unary_operator,
            expression: Box::new(expr),
        })
    }

    fn primary(&mut self) -> ParserResult<Expression> {
        use Expression::{Literal, Variable};
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
            TokenType::Identifier(name) => Variable { name: name.clone() },
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
