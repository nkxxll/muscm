use crate::tokenizer::Token;
use anyhow::{anyhow, Result};

// Program: sequence of forms
pub type Program = Vec<Form>;

pub enum Form {
    Definition(Definition),
    Expression(Expression),
}

// Definitions
pub enum Definition {
    VariableDefinition(VariableDefinition),
    SyntaxDefinition(SyntaxDefinition),
    Begin(Vec<Definition>),
    LetSyntax(LetSyntax),
    LetrecSyntax(LetrecSyntax),
}

pub struct VariableDefinition {
    pub variable: String, // identifier
    pub value: Expression,
}

pub struct FunctionDefinition {
    pub name: String,
    pub params: Formals,
    pub body: Body,
}

pub struct SyntaxDefinition {
    pub keyword: String, // identifier
    pub transformer: Expression,
}

pub struct SyntaxBinding {
    pub keyword: String, // identifier
    pub transformer: Expression,
}

pub struct LetSyntax {
    pub syntax_bindings: Vec<SyntaxBinding>,
    pub definitions: Vec<Definition>,
}

pub struct LetrecSyntax {
    pub syntax_bindings: Vec<SyntaxBinding>,
    pub definitions: Vec<Definition>,
}

pub struct Body {
    pub definitions: Vec<Definition>,
    pub expressions: Vec<Expression>,
}

// Expressions
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Quote(Datum),
    QuasiQuote(Datum),
    Unquote(Datum),
    UnquoteSplicing(Datum),
    Lambda(Lambda),
    If(If),
    If2(If2),
    SetBang(SetBang),
    Application(Application),
    LetSyntax(LetSyntax),
    LetrecSyntax(LetrecSyntax),
    Begin(Vec<Expression>),
    And(Vec<Expression>),
    Or(Vec<Expression>),
    Cond(Vec<CondClause>),
    Case(Case),
    Delay(Box<Expression>),
    Do(Do),
    Let(Let),
    LetStar(LetStar),
    Letrec(Letrec),
}

pub enum Constant {
    Boolean(bool),
    Number(Number),
    Character(char),
    String(String),
}

pub struct Lambda {
    pub formals: Formals,
    pub body: Body,
}

pub struct If {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
}

pub struct If2 {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
}

pub struct SetBang {
    pub variable: String,
    pub value: Box<Expression>,
}

pub struct Application {
    pub func: Box<Expression>,
    pub args: Vec<Expression>,
}

pub enum Formals {
    Variable(String),
    List(Vec<String>),
    DottedList { params: Vec<String>, rest: String },
}

pub struct CondClause {
    pub test: Box<Expression>,
    pub expressions: Vec<Expression>,
}

pub struct Case {
    pub expr: Box<Expression>,
    pub clauses: Vec<CaseClause>,
}

pub struct CaseClause {
    pub datums: Vec<Datum>,
    pub expressions: Vec<Expression>,
}

pub struct Do {
    pub iterations: Vec<DoIteration>,
    pub test: Box<Expression>,
    pub commands: Vec<Expression>,
    pub body: Vec<Expression>,
}

pub struct DoIteration {
    pub variable: String,
    pub init: Box<Expression>,
    pub step: Option<Box<Expression>>,
}

pub struct Let {
    pub bindings: Vec<LetBinding>,
    pub body: Body,
}

pub struct LetBinding {
    pub variable: String,
    pub value: Box<Expression>,
}

pub struct LetStar {
    pub bindings: Vec<LetBinding>,
    pub body: Body,
}

pub struct Letrec {
    pub bindings: Vec<LetBinding>,
    pub body: Body,
}

// Data
pub enum Datum {
    Boolean(bool),
    Number(Number),
    Character(char),
    String(String),
    Symbol(String),
    List(Vec<Datum>),
    DottedList {
        elements: Vec<Datum>,
        tail: Box<Datum>,
    },
    Vector(Vec<Datum>),
    Quote(Box<Datum>),
    QuasiQuote(Box<Datum>),
    Unquote(Box<Datum>),
    UnquoteSplicing(Box<Datum>),
}

pub enum Number {
    Integer(i64),
    Float(f64),
    Rational {
        numerator: i64,
        denominator: i64,
    },
    Complex {
        real: Box<Number>,
        imag: Box<Number>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn peek(self: &Self) -> &Token {
        &self.tokens[self.pos + 1]
    }

    pub fn peekn(self: &Self, n: usize) -> Result<Vec<&Token>> {
        let mut res = Vec::new();
        for i in self.pos..self.pos + n {
            if i < self.tokens.len() {
                res.push(&self.tokens[i]);
            } else {
                return Err(anyhow!("Peeked too far"));
            }
        }
        Ok(res)
    }

    pub fn advance(self: &mut Self) -> &Token {
        assert!(self.pos + 1 < self.tokens.len());
        self.pos += 1;
        &self.tokens[self.pos]
    }

    pub fn parse(self: &mut Self) -> Result<Program> {
        self.parse_program()
    }

    pub fn parse_program(self: &mut Self) -> Result<Program> {
        // parse one or more form
        let mut program: Vec<Form> = Vec::new();
        loop {
            match self.peek().token_type {
                crate::tokenizer::TokenType::Eof => break,
                _ => {}
            }
            program.push(self.parse_form()?);
        }
        Ok(program)
    }

    pub fn is_definition(self: &Self) -> Result<bool> {}

    pub fn parse_definition(self: &mut Self) -> Result<Definition> {}
    pub fn parse_expression(self: &mut Self) -> Result<Expression> {}

    pub fn parse_form(self: &mut Self) -> Result<Form> {
        // descide wether to parse definition or expression
        if self.is_definition()? {
            Ok(Form::Definition(self.parse_definition()?))
        } else {
            Ok(Form::Expression(self.parse_expression()?))
        }
    }
}

pub fn parse_tokens_to_ast() {}
