use std::cell::{Ref, RefCell};

use crate::class::Instance;

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
        superclass: Option<Identifier>,
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
pub struct Identifier {
    pub name: String,
    pub resolved_depth: Option<usize>,
}

impl Identifier {
    pub fn new(name: String) -> Self {
        Identifier {
            name,
            resolved_depth: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Identifier(Identifier),
    Assignment {
        identifier: Identifier,
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
    Get {
        object: Box<Expression>,
        name: String,
    },
    Set {
        object: Box<Expression>,
        name: String,
        expression: Box<Expression>,
    },
    This(Identifier),
    Super {
        identifier: Identifier,
        method: String,
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
