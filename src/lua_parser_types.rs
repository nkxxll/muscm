//! AST Types for Lua parser

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    // Symbols
    Semicolon,
    Equals,
    Comma,
    Dot,
    Colon,
    DoubleColon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Caret,
    Percent,
    Ampersand,
    Tilde,
    Pipe,
    RShift,
    LShift,
    Concat,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    Hash,
    Varargs,
    // Values
    Identifier(String),
    Number(String),
    StringLit(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_statement: Option<ReturnStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Empty,
    Assignment {
        variables: Vec<Expression>,
        values: Vec<Expression>,
    },
    FunctionCall(Expression),
    Break,
    Label(String),
    Goto(String),
    Do(Box<Block>),
    While {
        condition: Expression,
        body: Box<Block>,
    },
    Repeat {
        body: Box<Block>,
        condition: Expression,
    },
    If {
        condition: Expression,
        then_block: Box<Block>,
        elseif_parts: Vec<(Expression, Block)>,
        else_block: Option<Box<Block>>,
    },
    ForNumeric {
        var: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: Box<Block>,
    },
    ForGeneric {
        vars: Vec<String>,
        iterables: Vec<Expression>,
        body: Box<Block>,
    },
    FunctionDecl {
        name: String,
        body: Box<FunctionBody>,
    },
    LocalFunction {
        name: String,
        body: Box<FunctionBody>,
    },
    LocalVars {
        names: Vec<String>,
        values: Option<Vec<Expression>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub expression_list: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Nil,
    Boolean(bool),
    Number(String),
    String(String),
    Varargs,
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    TableIndexing {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    FunctionCall {
        function: Box<Expression>,
        args: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    TableConstructor {
        fields: Vec<Field>,
    },
    FunctionDef(Box<FunctionBody>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    FloorDivide,
    Modulo,
    Power,
    Concat,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
    BitNot,
    Length,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: FieldKey,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKey {
    Bracket(Box<Expression>),
    Identifier(String),
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionBody {
    pub params: Vec<String>,
    pub varargs: bool,
    pub block: Box<Block>,
}
