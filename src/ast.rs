enum Expression {
    Literal {
        value: LiteralValue,
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
    grouping {
        expression: Box<Expression>,
    },
}

enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

enum UnaryOperator {
    Bang,
    Minus,
}

enum BinaryOperator {
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
