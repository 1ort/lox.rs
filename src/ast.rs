#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Box<Expression>,
    },
    Print {
        expression: Box<Expression>,
    },
    VarDeclaration {
        name: String,
        initializer: Option<Box<Expression>>,
    },
}

#[derive(Debug)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Variable {
        name: String,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Grouping {
        expression: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug)]
pub enum BinaryOperator {
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Minus,
    Plus,
    Slash,
    Star,
}
