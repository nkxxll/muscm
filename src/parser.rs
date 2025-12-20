use crate::interpreter::Interpreter;
use crate::tokenizer::Token;
use anyhow::{anyhow, Result};

pub trait AstNode: std::fmt::Debug {
    fn eval(&self, interpreter: &mut Interpreter) -> Result<Datum>;
}

#[derive(Debug)]
pub struct Constant {
    pub value: ConstantValue,
}

#[derive(Debug)]
pub enum ConstantValue {
    Boolean(bool),
    Number(Number),
    Character(char),
    String(String),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub struct Application {
    pub func: Box<dyn AstNode>,
    pub args: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Lambda {
    pub formals: Formals,
    pub body: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct If {
    pub condition: Box<dyn AstNode>,
    pub then_expr: Box<dyn AstNode>,
    pub else_expr: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct If2 {
    pub condition: Box<dyn AstNode>,
    pub then_expr: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct SetBang {
    pub variable: String,
    pub value: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct VariableDefinition {
    pub variable: String,
    pub value: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct SyntaxDefinition {
    pub keyword: String,
    pub transformer: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct SyntaxBinding {
    pub keyword: String,
    pub transformer: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct LetSyntax {
    pub syntax_bindings: Vec<SyntaxBinding>,
    pub definitions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct LetrecSyntax {
    pub syntax_bindings: Vec<SyntaxBinding>,
    pub definitions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Begin {
    pub expressions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct And {
    pub expressions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Or {
    pub expressions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Cond {
    pub clauses: Vec<CondClause>,
}

#[derive(Debug)]
pub struct CondClause {
    pub test: Box<dyn AstNode>,
    pub expressions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Case {
    pub expr: Box<dyn AstNode>,
    pub clauses: Vec<CaseClause>,
}

#[derive(Debug)]
pub struct CaseClause {
    pub datums: Vec<Datum>,
    pub expressions: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Do {
    pub iterations: Vec<DoIteration>,
    pub test: Box<dyn AstNode>,
    pub commands: Vec<Box<dyn AstNode>>,
    pub body: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct DoIteration {
    pub variable: String,
    pub init: Box<dyn AstNode>,
    pub step: Option<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Let {
    pub bindings: Vec<LetBinding>,
    pub body: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct LetBinding {
    pub variable: String,
    pub value: Box<dyn AstNode>,
}

#[derive(Debug)]
pub struct LetStar {
    pub bindings: Vec<LetBinding>,
    pub body: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Letrec {
    pub bindings: Vec<LetBinding>,
    pub body: Vec<Box<dyn AstNode>>,
}

#[derive(Debug)]
pub struct Quote {
    pub datum: Datum,
}

#[derive(Debug)]
pub struct Delay {
    pub expr: Box<dyn AstNode>,
}

// ============ Shared Types ============

#[derive(Debug, Clone)]
pub enum Formals {
    Variable(String),
    List(Vec<String>),
    DottedList { params: Vec<String>, rest: String },
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

pub type Program = Vec<Box<dyn AstNode>>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub fn peekn(&self, n: usize) -> Result<Vec<&Token>> {
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

    pub fn advance(&mut self) -> &Token {
        assert!(self.pos < self.tokens.len());
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    pub fn parse(&mut self) -> Result<Program> {
        self.parse_program()
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program: Program = Vec::new();
        loop {
            match self.peek().token_type {
                crate::tokenizer::TokenType::Eof => break,
                _ => {
                    program.push(self.parse_form()?);
                }
            }
        }
        Ok(program)
    }

    pub fn is_definition(&self) -> Result<bool> {
        // TODO: implement
        Ok(false)
    }

    pub fn parse_form(&mut self) -> Result<Box<dyn AstNode>> {
        // decide whether to parse definition or expression
        if self.is_definition()? {
            self.parse_definition()
        } else {
            self.parse_expression()
        }
    }

    pub fn parse_definition(&mut self) -> Result<Box<dyn AstNode>> {
        // TODO: implement
        Err(anyhow!("parse_definition not implemented"))
    }

    pub fn parse_expression(&mut self) -> Result<Box<dyn AstNode>> {
        // TODO: implement
        Err(anyhow!("parse_expression not implemented"))
    }
}

impl AstNode for Constant {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // TODO: implement
        Err(anyhow!("Constant::eval not implemented"))
    }
}

impl AstNode for Variable {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // TODO: implement
        Err(anyhow!("Variable::eval not implemented"))
    }
}

impl AstNode for Application {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // Now you can just call eval recursively:
        // let func_val = self.func.eval(interpreter)?;
        // let arg_vals: Result<Vec<_>> = self.args.iter().map(|arg| arg.eval(interpreter)).collect();
        // TODO: implement
        Err(anyhow!("Application::eval not implemented"))
    }
}

impl AstNode for Lambda {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // TODO: implement
        Err(anyhow!("Lambda::eval not implemented"))
    }
}

impl AstNode for If {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // TODO: implement
        Err(anyhow!("If::eval not implemented"))
    }
}

impl AstNode for VariableDefinition {
    fn eval(&self, _interpreter: &mut crate::interpreter::Interpreter) -> Result<Datum> {
        // TODO: implement
        Err(anyhow!("VariableDefinition::eval not implemented"))
    }
}
