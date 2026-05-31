use crate::span::Span;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression {
        expression: Box<Expression>,
        // Span is used in every enum variant
        // i'd convert to:
        // ```rust
        // struct Statement {
        //     expression: Box<Expression>,
        //     span: Span,
        // }
        // ```
        // or even:
        // ```rust
        // struct Spanned<T> {
        //     value: T,
        //     span: Span,
        // }
        // ```
        // 
        // Or (because it also used in Expression and other types)
        // Move it inside inner-structs and use some trait to get info.
        // ```rust
        // trait HasSpan {
        //     fn span(&self) -> &Span;
        // }
        // ```
        span: Span,
    },
    Print {
        expression: Box<Expression>,
        span: Span,
    },
    Block {
        statements: Vec<Statement>,
        span: Span,
    },
    VarDeclaration {
        name: String,
        initializer: Option<Box<Expression>>,
        span: Span,
    },
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
        span: Span,
    },
    WhileLoop {
        condition: Box<Expression>,
        body: Box<Statement>,
        span: Span,
    },
    Break {
        span: Span,
    },
    FunctionDeclaration {
        function: FunctionStatement,
        span: Span,
    },
    Return {
        expresstion: Option<Box<Expression>>,
        span: Span,
    },
    ClassDeclaration {
        name: String,
        superclass: Option<Identifier>,
        methods: Vec<FunctionStatement>,
        span: Span,
    },
    Pass {
        span: Span,
    },
}

impl Statement {
    // Convert to trait?
    pub fn span(&self) -> &Span {
        match self {
            Statement::Expression { span, .. } => span,
            Statement::Print { span, .. } => span,
            Statement::Block { span, .. } => span,
            Statement::VarDeclaration { span, .. } => span,
            Statement::Conditional { span, .. } => span,
            Statement::WhileLoop { span, .. } => span,
            Statement::Break { span } => span,
            Statement::FunctionDeclaration { span, .. } => span,
            Statement::Return { span, .. } => span,
            Statement::ClassDeclaration { span, .. } => span,
            Statement::Pass { span } => span,
        }
    }
}

// To big enums?
// Best if enum can be <= 16 bytes
// Good <= 64 bytes
const ASSERT_SIZEOF_STMT: () = {
    assert!(std::mem::size_of::<Statement>() == 152);
};
const ASSERT_SIZEOF_EXPR: () = {
    assert!(std::mem::size_of::<Expression>() == 128);
};

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<Identifier>,
    pub body: Box<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub resolved_depth: Option<usize>,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: String, span: &Span) -> Self {
        Identifier {
            name,
            resolved_depth: None,
            span: span.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: LiteralValue,
        span: Span,
    },
    Identifier {
        identifier: Identifier,
        span: Span,
    },
    Assignment {
        identifier: Identifier,
        expression: Box<Expression>,
        span: Span,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
        span: Span,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,

        span: Span,
    },
    Logical {
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
        span: Span,
    },
    Grouping {
        expression: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    Get {
        object: Box<Expression>,
        name: String,
        span: Span,
    },
    Set {
        object: Box<Expression>,
        name: String,
        expression: Box<Expression>,
        span: Span,
    },
    This {
        identifier: Identifier,
        span: Span,
    },
    Super {
        identifier: Identifier,
        method: String,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal { span, .. } => span,
            Expression::Identifier { span, .. } => span,
            Expression::Assignment { span, .. } => span,
            Expression::Unary { span, .. } => span,
            Expression::Binary { span, .. } => span,
            Expression::Logical { span, .. } => span,
            Expression::Grouping { span, .. } => span,
            Expression::Call { span, .. } => span,
            Expression::Get { span, .. } => span,
            Expression::Set { span, .. } => span,
            Expression::This { span, .. } => span,
            Expression::Super { span, .. } => span,
        }
    }
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
