use crate::{
    ast::{BinaryOperator, Expression, LiteralValue, UnaryOperator},
    scanner::{Token, TokenType},
};

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

type ParserError = String;
type ParserResult<T> = Result<T, ParserError>;

pub fn parse_tokens(tokens: Vec<Token>) -> ParserResult<Expression> {
    let mut parser = Parser::new(tokens);
    parser.expression()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        let expr = self.equality()?;
        match self.peek().token_type {
            TokenType::Eof => Ok(expr),
            _ => Err(format!(
                "Unexpected token: {}, EOF expected",
                self.peek().lexeme
            )),
        }
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
        let literal = match &self.peek().token_type {
            TokenType::False => LiteralValue::Boolean(false),
            TokenType::True => LiteralValue::Boolean(true),
            TokenType::Nil => LiteralValue::Nil,
            TokenType::Number(num) => LiteralValue::Number(*num),
            TokenType::String(string) => LiteralValue::String(string.clone()),
            _ => return self.grouping(),
        };
        self.advance();
        Ok(Expression::Literal { value: literal })
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
}
