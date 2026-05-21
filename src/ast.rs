#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression {
        expression: Box<Expression>,
    },
    Print {
        expression: Box<Expression>,
    },
    Block {
        statements: Vec<Statement>,
    },
    VarDeclaration {
        name: String,
        initializer: Option<Box<Expression>>,
    },
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    WhileLoop {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    Break,
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Box<Statement>,
    },
    Return {
        expresstion: Option<Box<Expression>>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Identifier {
        name: String,
    },
    Assignment {
        name: String,
        expression: Box<Expression>,
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
    Logical {
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
    },
    Grouping {
        expression: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    Or,
    And,
}
