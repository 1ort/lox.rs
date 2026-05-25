use std::cell::RefCell;

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
    FunctionDeclaration(FunctionStatement),
    Return {
        expresstion: Option<Box<Expression>>,
    },
    ClassDeclaration {
        name: String,
        methods: Vec<FunctionStatement>,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Identifier {
        name: String,
        resolved_scope_depth: RefCell<Option<usize>>,
    },
    Assignment {
        name: String,
        expression: Box<Expression>,
        resolved_scope_depth: RefCell<Option<usize>>,
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
    Get {
        object: Box<Expression>,
        name: String,
    },
    Set {
        object: Box<Expression>,
        name: String,
        expression: Box<Expression>,
    },
    This {
        resolved_scope_depth: RefCell<Option<usize>>,
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
