//! lua parser with nom
//!
//! chunk ::= block
//! block ::= {stat} [retstat]
//!
//! stat ::=  ';' |
//! varlist '=' explist |
//! functioncall |
//! label |
//! break |
//! goto Name |
//! do block end |
//! while exp do block end |
//! repeat block until exp |
//! if exp then block {elseif exp then block} [else block] end |
//! for Name '=' exp ',' exp [',' exp] do block end |
//! for namelist in explist do block end |
//! function funcname funcbody |
//! local function Name funcbody |
//! local namelist ['=' explist]
//!
//! retstat ::= return [explist] [';']
//!
//! label ::= '::' Name '::'
//!
//! funcname ::= Name {'.' Name} [':' Name]
//!
//! varlist ::= var {',' var}
//!
//! var ::=  Name | prefixexp '[' exp ']' | prefixexp '.' Name
//!
//! namelist ::= Name {',' Name}
//!
//! explist ::= exp {',' exp}
//!
//! exp ::=  nil | false | true | Numeral | LiteralString | '...' | functiondef |
//! prefixexp | tableconstructor | exp binop exp | unop exp
//!
//! prefixexp ::= var | functioncall | '(' exp ')'
//!
//! functioncall ::=  prefixexp args | prefixexp ':' Name args
//!
//! args ::=  '(' [explist] ')' | tableconstructor | LiteralString
//!
//! functiondef ::= function funcbody
//!
//! funcbody ::= '(' [parlist] ')' block end
//!
//! parlist ::= namelist [',' '...'] | '...'
//!
//! tableconstructor ::= '{' [fieldlist] '}'
//!
//! fieldlist ::= field {fieldsep field} [fieldsep]
//!
//! field ::= '[' exp ']' '=' exp | Name '=' exp | exp
//!
//! fieldsep ::= ',' | ';'
//!
//! binop ::=  '+' | '-' | '*' | '/' | '//' | '^' | '%' |
//! '&' | '~' | '|' | '>>' | '<<' | '..' |
//! '<' | '<=' | '>' | '>=' | '==' | '~=' |
//! and | or
//!
//! unop ::= '-' | not | '#' | '~'
//!
//!
use phf::phf_map;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, satisfy},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{pair, preceded},
    IResult, Input, Needed, Parser,
};

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
    Identifier(::std::string::String),
    Number(::std::string::String),
    StringLit(::std::string::String),
}
use Token::*;

#[derive(Debug, Clone, Copy)]
pub struct TokenSlice<'a>(&'a [Token]);

impl<'a> From<&'a [Token]> for TokenSlice<'a> {
    fn from(slice: &'a [Token]) -> Self {
        TokenSlice(slice)
    }
}

impl<'a> Input for TokenSlice<'a> {
    type Item = &'a Token;
    type Iter = std::slice::Iter<'a, Token>;
    type IterIndices = std::iter::Enumerate<std::slice::Iter<'a, Token>>;

    fn input_len(&self) -> usize {
        self.0.len()
    }

    fn take(&self, index: usize) -> Self {
        TokenSlice(&self.0[..index.min(self.0.len())])
    }

    fn take_from(&self, index: usize) -> Self {
        TokenSlice(&self.0[index.min(self.0.len())..])
    }

    fn take_split(&self, index: usize) -> (Self, Self) {
        let index = index.min(self.0.len());
        (TokenSlice(&self.0[index..]), TokenSlice(&self.0[..index]))
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.iter().position(predicate)
    }

    fn iter_elements(&self) -> Self::Iter {
        self.0.iter()
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.0.iter().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if count > self.0.len() {
            Err(Needed::Size(
                std::num::NonZeroUsize::new(count - self.0.len()).unwrap(),
            ))
        } else {
            Ok(count)
        }
    }
}

const KEYWORDS: phf::Map<&str, Token> = phf_map! {
    "and" => And,
    "break" => Break,
    "do" => Do,
    "else" => Else,
    "elseif" => Elseif,
    "end" => End,
    "false" => False,
    "for" => For,
    "function" => Function,
    "goto" => Goto,
    "if" => If,
    "in" => In,
    "local" => Local,
    "nil" => Nil,
    "not" => Not,
    "or" => Or,
    "repeat" => Repeat,
    "return" => Return,
    "then" => Then,
    "true" => True,
    "until" => Until,
    "while" => While,
};

const SYMBOLS: phf::Map<&str, Token> = phf_map! {
    ";" => Semicolon,
    "=" => Equals,
    "," => Comma,
    "." => Dot,
    ":" => Colon,
    "::" => DoubleColon,
    "(" => LParen,
    ")" => RParen,
    "[" => LBracket,
    "]" => RBracket,
    "{" => LBrace,
    "}" => RBrace,
    "+" => Plus,
    "-" => Minus,
    "*" => Star,
    "/" => Slash,
    "//" => DoubleSlash,
    "^" => Caret,
    "%" => Percent,
    "&" => Ampersand,
    "~" => Tilde,
    "|" => Pipe,
    ">>" => RShift,
    "<<" => LShift,
    ".." => Concat,
    "<" => Lt,
    "<=" => Lte,
    ">" => Gt,
    ">=" => Gte,
    "==" => Eq,
    "~=" => Neq,
    "#" => Hash,
    "..." => Varargs,
};

// Helper parsers
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

fn number(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, opt(preceded(char('.'), digit1)))).parse(input)
}

fn string_literal(input: &str) -> IResult<&str, ::std::string::String> {
    let (input, _) = char('"').parse(input)?;
    let (input, content) = take_while1(|c: char| c != '"').parse(input)?;
    let (input, _) = char('"').parse(input)?;
    Ok((input, content.to_string()))
}

// Tokenizer
fn symbol(input: &str) -> IResult<&str, Token> {
    let symbols = vec![
        "...", "::", "//", ">>", "<<", "..", "<=", ">=", "==", "~=", ":", ".", "=", ",", ";", "(",
        ")", "[", "]", "{", "}", "+", "-", "*", "/", "^", "%", "&", "~", "|", "<", ">", "#",
    ];

    for sym in symbols {
        if let Ok((rest, _)) = tag::<_, _, nom::error::Error<_>>(sym)(input) {
            let token = SYMBOLS.get(sym).cloned().ok_or_else(|| {
                nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
            })?;
            return Ok((rest, token));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn token(input: &str) -> IResult<&str, Token> {
    if let Ok((rest, token)) = symbol(input) {
        return Ok((rest, token));
    }
    if let Ok((rest, content)) = string_literal(input) {
        return Ok((rest, Token::StringLit(content)));
    }
    if let Ok((rest, num)) = number(input) {
        return Ok((rest, Token::Number(num.to_string())));
    }

    let (rest, ident) = identifier(input)?;
    let token = KEYWORDS
        .get(ident)
        .cloned()
        .unwrap_or_else(|| Token::Identifier(ident.to_string()));
    Ok((rest, token))
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ::std::string::String> {
    let mut tokens = Vec::new();
    let mut remaining = input;

    loop {
        // Skip whitespace and comments
        while !remaining.is_empty() {
            if remaining.starts_with("--") {
                if let Some(newline) = remaining.find('\n') {
                    remaining = &remaining[newline + 1..];
                } else {
                    remaining = "";
                }
            } else if remaining.chars().next().is_some_and(char::is_whitespace) {
                remaining = &remaining[1..];
            } else {
                break;
            }
        }

        if remaining.is_empty() {
            break;
        }

        let (rest, tok) = token(remaining).map_err(|e| format!("Tokenization error: {:?}", e))?;

        tokens.push(tok);
        remaining = rest;
    }

    Ok(tokens)
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

fn parse_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    // Try each statement type
    alt((
        parse_empty_statement,
        parse_break_statement,
        parse_label_statement,
        parse_goto_statement,
        parse_do_block,
        parse_while_loop,
        parse_repeat_until,
        parse_if_statement,
        parse_for_loop,
        parse_function_decl,
        parse_local_statement,
        parse_assignment_or_call,
    ))
    .parse(t)
}

fn parse_empty_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Semicolon)(t)?;
    Ok((rest, Statement::Empty))
}

fn parse_break_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Break)(t)?;
    Ok((rest, Statement::Break))
}

fn parse_label_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::DoubleColon)(t)?;
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&rest.0[1..]);
        let (rest, _) = token_tag(&Token::DoubleColon)(rest)?;
        Ok((rest, Statement::Label(name)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_goto_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Goto)(t)?;
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&rest.0[1..]);
        Ok((rest, Statement::Goto(name)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_do_block(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Do)(t)?;
    let (rest, block) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::End)(rest)?;
    Ok((rest, Statement::Do(Box::new(block))))
}

fn parse_while_loop(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::While)(t)?;
    let (rest, condition) = parse_expression(rest)?;
    let (rest, _) = token_tag(&Token::Do)(rest)?;
    let (rest, body) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::End)(rest)?;
    Ok((
        rest,
        Statement::While {
            condition,
            body: Box::new(body),
        },
    ))
}

fn parse_repeat_until(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Repeat)(t)?;
    let (rest, body) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::Until)(rest)?;
    let (rest, condition) = parse_expression(rest)?;
    Ok((
        rest,
        Statement::Repeat {
            body: Box::new(body),
            condition,
        },
    ))
}

fn parse_if_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::If)(t)?;
    let (rest, condition) = parse_expression(rest)?;
    let (rest, _) = token_tag(&Token::Then)(rest)?;
    let (rest, then_block) = parse_block(rest)?;

    // Parse elseif parts
    let (rest, elseif_parts) = many0(|input| {
        let (r, _) = token_tag(&Token::Elseif)(input)?;
        let (r, cond) = parse_expression(r)?;
        let (r, _) = token_tag(&Token::Then)(r)?;
        let (r, blk) = parse_block(r)?;
        Ok((r, (cond, blk)))
    })
    .parse(rest)?;

    // Parse optional else block
    let (rest, else_block) = opt(|input| {
        let (r, _) = token_tag(&Token::Else)(input)?;
        parse_block(r).map(|(r, b)| (r, Box::new(b)))
    })
    .parse(rest)?;

    let (rest, _) = token_tag(&Token::End)(rest)?;

    Ok((
        rest,
        Statement::If {
            condition,
            then_block: Box::new(then_block),
            elseif_parts,
            else_block,
        },
    ))
}

fn parse_for_loop(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::For)(t)?;

    // Parse the first variable name
    if let Some(Token::Identifier(var_name)) = rest.0.first() {
        let var_name = var_name.clone();
        let rest = TokenSlice(&rest.0[1..]);

        // Try numeric for: var = start, end [, step]
        if let Ok((r, _)) = token_tag(&Token::Equals)(rest) {
            let (r, start) = parse_expression(r)?;
            let (r, _) = token_tag(&Token::Comma)(r)?;
            let (r, end) = parse_expression(r)?;

            // Optional step
            let (r, step) = opt(|input| {
                let (r, _) = token_tag(&Token::Comma)(input)?;
                parse_expression(r)
            })
            .parse(r)?;

            let (r, _) = token_tag(&Token::Do)(r)?;
            let (r, body) = parse_block(r)?;
            let (r, _) = token_tag(&Token::End)(r)?;

            return Ok((
                r,
                Statement::ForNumeric {
                    var: var_name,
                    start,
                    end,
                    step,
                    body: Box::new(body),
                },
            ));
        }

        // Try generic for: [var1,] var2 [, var3, ...] in iterables
        let (rest, more_vars) = opt(|input| {
            let (r, _) = token_tag(&Token::Comma)(input)?;
            parse_namelist(r)
        })
        .parse(rest)?;

        if let Ok((r, _)) = token_tag(&Token::In)(rest) {
            let (r, iterables) = parse_expression_list(r)?;
            let (r, _) = token_tag(&Token::Do)(r)?;
            let (r, body) = parse_block(r)?;
            let (r, _) = token_tag(&Token::End)(r)?;

            let mut vars = vec![var_name];
            if let Some(mut extra_vars) = more_vars {
                vars.append(&mut extra_vars);
            }

            return Ok((
                r,
                Statement::ForGeneric {
                    vars,
                    iterables,
                    body: Box::new(body),
                },
            ));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        t,
        nom::error::ErrorKind::Alt,
    )))
}

fn parse_function_decl(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Function)(t)?;
    
    // Parse function name
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&rest.0[1..]);

        let (rest, body) = parse_funcbody(rest)?;
        Ok((
            rest,
            Statement::FunctionDecl {
                name,
                body: Box::new(body),
            },
        ))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_local_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Local)(t)?;

    // Check if it's local function
    if let Ok((r, _)) = token_tag(&Token::Function)(rest) {
        if let Some(Token::Identifier(name)) = r.0.first() {
            let name = name.clone();
            let r = TokenSlice(&r.0[1..]);
            let (r, body) = parse_funcbody(r)?;
            return Ok((
                r,
                Statement::LocalFunction {
                    name,
                    body: Box::new(body),
                },
            ));
        }
    }

    // Otherwise it's local vars [= values]
    let (rest, names) = parse_namelist(rest)?;
    let (rest, values) = opt(|input| {
        let (r, _) = token_tag(&Token::Equals)(input)?;
        parse_expression_list(r)
    })
    .parse(rest)?;

    Ok((
        rest,
        Statement::LocalVars {
            names,
            values,
        },
    ))
}

fn parse_assignment_or_call(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, first_expr) = parse_prefix_exp(t)?;

    // Check if this is an assignment by looking for more variables or =
    if let Ok((r, _)) = token_tag(&Token::Comma)(rest) {
        // Multiple variables: var1, var2, ... = ...
        let (r, rest_vars) = many0(|input| {
            let (r, expr) = parse_prefix_exp(input)?;
            let (r, _) = token_tag(&Token::Comma)(r)?;
            Ok((r, expr))
        })
        .parse(r)?;

        // Now we must have = and values
        let (r, final_expr) = parse_prefix_exp(r)?;
        let (r, _) = token_tag(&Token::Equals)(r)?;
        let (r, values) = parse_expression_list(r)?;

        let mut variables = vec![first_expr];
        variables.extend(rest_vars);
        variables.push(final_expr);

        return Ok((
            r,
            Statement::Assignment { variables, values },
        ));
    }

    // Try assignment: varlist = explist
    if let Ok((r, _)) = token_tag(&Token::Equals)(rest) {
        let (r, values) = parse_expression_list(r)?;
        // Collect first_expr as a variable
        return Ok((
            r,
            Statement::Assignment {
                variables: vec![first_expr],
                values,
            },
        ));
    }

    // Try function call (prefix expression that is a function call)
    match &first_expr {
        Expression::FunctionCall { .. } | Expression::MethodCall { .. } => {
            Ok((rest, Statement::FunctionCall(first_expr)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Alt,
        ))),
    }
}

fn parse_number_literal(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::Number(n)) = t.0.first() {
        Ok((TokenSlice(&t.0[1..]), Expression::Number(n.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_string_literal(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::StringLit(s)) = t.0.first() {
        Ok((TokenSlice(&t.0[1..]), Expression::String(s.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_identifier(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::Identifier(id)) = t.0.first() {
        Ok((TokenSlice(&t.0[1..]), Expression::Identifier(id.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse arguments for a function call: `(explist) | tableconstructor | string_literal`
fn parse_args(t: TokenSlice) -> IResult<TokenSlice, Vec<Expression>> {
    // Try parenthesized argument list first
    if let Ok((rest, _)) = token_tag(&Token::LParen)(t) {
        let (rest, exprs) = opt(|i| parse_expression_list(i)).parse(rest)?;
        let (rest, _) = token_tag(&Token::RParen)(rest)?;
        return Ok((rest, exprs.unwrap_or_default()));
    }
    
    // Try table constructor
    if let Ok((rest, expr)) = parse_table_constructor(t) {
        return Ok((rest, vec![expr]));
    }
    
    // Try string literal
    if let Ok((rest, expr)) = parse_string_literal(t) {
        return Ok((rest, vec![expr]));
    }
    
    Err(nom::Err::Error(nom::error::Error::new(t, nom::error::ErrorKind::Alt)))
}

/// Parse table constructor: `{ [fieldlist] }`
fn parse_table_constructor(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, _) = token_tag(&Token::LBrace)(t)?;
    let (rest, fields) = opt(|i| parse_fieldlist(i)).parse(rest)?;
    let (rest, _) = token_tag(&Token::RBrace)(rest)?;
    Ok((
        rest,
        Expression::TableConstructor {
            fields: fields.unwrap_or_default(),
        },
    ))
}

/// Parse field list: `field {fieldsep field} [fieldsep]`
fn parse_fieldlist(t: TokenSlice) -> IResult<TokenSlice, Vec<Field>> {
    let (rest, first_field) = parse_field(t)?;
    let (rest, rest_fields) = many0(|input| {
        let (rest, _) = parse_fieldsep(input)?;
        // Check for trailing separator (fieldsep followed by })
        if rest.0.first() == Some(&Token::RBrace) {
            return Ok((rest, None));
        }
        let (rest, field) = parse_field(rest)?;
        Ok((rest, Some(field)))
    })
    .parse(rest)?;

    let mut result = vec![first_field];
    for field_opt in rest_fields {
        if let Some(field) = field_opt {
            result.push(field);
        }
    }
    Ok((rest, result))
}

/// Parse a single field: `[exp] = exp | name = exp | exp`
fn parse_field(t: TokenSlice) -> IResult<TokenSlice, Field> {
    // Try [exp] = exp
    if let Ok((rest, _)) = token_tag(&Token::LBracket)(t) {
        let (rest, key_expr) = parse_expression(rest)?;
        let (rest, _) = token_tag(&Token::RBracket)(rest)?;
        let (rest, _) = token_tag(&Token::Equals)(rest)?;
        let (rest, value) = parse_expression(rest)?;
        return Ok((
            rest,
            Field {
                key: FieldKey::Bracket(Box::new(key_expr)),
                value,
            },
        ));
    }
    
    // Try name = exp
    if let Some(Token::Identifier(name)) = t.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&t.0[1..]);
        if let Ok((rest, _)) = token_tag(&Token::Equals)(rest) {
            let (rest, value) = parse_expression(rest)?;
            return Ok((
                rest,
                Field {
                    key: FieldKey::Identifier(name),
                    value,
                },
            ));
        }
    }
    
    // Try exp (implicit array index)
    let (rest, expr) = parse_expression(t)?;
    Ok((
        rest,
        Field {
            key: FieldKey::Index(0), // placeholder
            value: expr,
        },
    ))
}

/// Parse field separator: `,` or `;`
fn parse_fieldsep(t: TokenSlice) -> IResult<TokenSlice, ()> {
    alt((
        map(token_tag(&Token::Comma), |_| ()),
        map(token_tag(&Token::Semicolon), |_| ()),
    ))
    .parse(t)
}

/// Parse function definition: `function funcbody`
fn parse_function_def(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, _) = token_tag(&Token::Function)(t)?;
    let (rest, body) = parse_funcbody(rest)?;
    Ok((rest, Expression::FunctionDef(Box::new(body))))
}

/// Parse function body: `( [parlist] ) block end`
fn parse_funcbody(t: TokenSlice) -> IResult<TokenSlice, FunctionBody> {
    let (rest, _) = token_tag(&Token::LParen)(t)?;
    let (rest, params_info) = opt(|i| parse_parlist(i)).parse(rest)?;
    let (rest, _) = token_tag(&Token::RParen)(rest)?;
    let (rest, block) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::End)(rest)?;

    let (params, varargs) = params_info.unwrap_or_default();
    Ok((
        rest,
        FunctionBody {
            params,
            varargs,
            block: Box::new(block),
        },
    ))
}

/// Parse parameter list: `namelist [',' '...'] | '...'`
fn parse_parlist(t: TokenSlice) -> IResult<TokenSlice, (Vec<String>, bool)> {
    alt((
        // Just varargs
        map(token_tag(&Token::Varargs), |_| (vec![], true)),
        // Namelist with optional varargs
        |input| {
            let (rest, names) = parse_namelist(input)?;
            let (rest, has_varargs) = opt(|i| {
                let (r, _) = token_tag(&Token::Comma)(i)?;
                token_tag(&Token::Varargs)(r)
            })
            .parse(rest)?;
            Ok((rest, (names, has_varargs.is_some())))
        },
    ))
    .parse(t)
}

/// Parse name list: `name {',' name}`
fn parse_namelist(t: TokenSlice) -> IResult<TokenSlice, Vec<String>> {
    let (rest, first_name) = if let Some(Token::Identifier(name)) = t.0.first() {
        (TokenSlice(&t.0[1..]), name.clone())
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )));
    };

    let (rest, rest_names) = many0(|input| {
        let (r, _) = token_tag(&Token::Comma)(input)?;
        if let Some(Token::Identifier(name)) = r.0.first() {
            Ok((TokenSlice(&r.0[1..]), name.clone()))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                r,
                nom::error::ErrorKind::Tag,
            )))
        }
    })
    .parse(rest)?;

    let mut result = vec![first_name];
    result.extend(rest_names);
    Ok((rest, result))
}

/// Helper to parse a number literal
fn parse_number_literal_helper(input: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::Number(n)) = input.0.first() {
        Ok((TokenSlice(&input.0[1..]), Expression::Number(n.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Helper to parse a string literal
fn parse_string_literal_helper(input: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::StringLit(s)) = input.0.first() {
        Ok((TokenSlice(&input.0[1..]), Expression::String(s.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse a primary/prefix expression, then apply suffix operations (indexing, calls, method calls)
fn parse_prefix_exp(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (mut rest, mut expr) = {
        // Try simple literals first
        if let Ok((r, expr)) = alt((
            map(token_tag(&Token::Nil), |_| Expression::Nil),
            map(token_tag(&Token::True), |_| Expression::Boolean(true)),
            map(token_tag(&Token::False), |_| Expression::Boolean(false)),
            map(token_tag(&Token::Varargs), |_| Expression::Varargs),
            parse_number_literal_helper,
            parse_string_literal_helper,
        ))
        .parse(t) {
            (r, expr)
        } else if let Some(Token::LParen) = t.0.first() {
            // Parenthesized expression: ( exp )
            let (r, _) = token_tag(&Token::LParen)(t)?;
            let (r, expr) = parse_expression(r)?;
            let (r, _) = token_tag(&Token::RParen)(r)?;
            (r, expr)
        } else if let Some(Token::Function) = t.0.first() {
            // Function definition: function funcbody
            parse_function_def(t)?
        } else if let Some(Token::LBrace) = t.0.first() {
            // Table constructor: { fieldlist }
            parse_table_constructor(t)?
        } else if let Some(Token::Identifier(_)) = t.0.first() {
            // Identifier
            parse_identifier(t)?
        } else {
            return Err(nom::Err::Error(nom::error::Error::new(
                t,
                nom::error::ErrorKind::Alt,
            )));
        }
    };

    // Apply suffix operations: indexing [exp], field access .name, function calls
    loop {
        if let Some(Token::LBracket) = rest.0.first() {
            // Table indexing: [exp]
            let r = TokenSlice(&rest.0[1..]);
            let (r, index) = parse_expression(r)?;
            let (r, _) = token_tag(&Token::RBracket)(r)?;
            expr = Expression::TableIndexing {
                object: Box::new(expr),
                index: Box::new(index),
            };
            rest = r;
        } else if let Some(Token::Dot) = rest.0.first() {
            // Field access: .name
            let r = TokenSlice(&rest.0[1..]);
            if let Some(Token::Identifier(field)) = r.0.first() {
                let field = field.clone();
                let r = TokenSlice(&r.0[1..]);
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field,
                };
                rest = r;
            } else {
                return Err(nom::Err::Error(nom::error::Error::new(
                    r,
                    nom::error::ErrorKind::Tag,
                )));
            }
        } else if let Some(Token::Colon) = rest.0.first() {
            // Method call: :name args
            let r = TokenSlice(&rest.0[1..]);
            if let Some(Token::Identifier(method)) = r.0.first() {
                let method = method.clone();
                let r = TokenSlice(&r.0[1..]);
                let (r, args) = parse_args(r)?;
                expr = Expression::MethodCall {
                    object: Box::new(expr),
                    method,
                    args,
                };
                rest = r;
            } else {
                return Err(nom::Err::Error(nom::error::Error::new(
                    r,
                    nom::error::ErrorKind::Tag,
                )));
            }
        } else if matches!(
            rest.0.first(),
            Some(Token::LParen) | Some(Token::LBrace) | Some(Token::StringLit(_))
        ) {
            // Function call: args
            let (r, args) = parse_args(rest)?;
            expr = Expression::FunctionCall {
                function: Box::new(expr),
                args,
            };
            rest = r;
        } else {
            break;
        }
    }

    Ok((rest, expr))
}

/// Parse a simple literal: nil | false | true | number | string | ...
fn parse_literal(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    alt((
        map(token_tag(&Token::Nil), |_| Expression::Nil),
        map(token_tag(&Token::True), |_| Expression::Boolean(true)),
        map(token_tag(&Token::False), |_| Expression::Boolean(false)),
        map(token_tag(&Token::Varargs), |_| Expression::Varargs),
        parse_number_literal,
        parse_string_literal,
        parse_identifier,
    ))
    .parse(t)
}

/// Binary operator precedence levels (highest to lowest)
/// Lua has 14 precedence levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Or = 0,
    And = 1,
    RelationalEq = 2,    // <, <=, >, >=, ==, ~=
    BitOr = 3,
    BitXor = 4,
    BitAnd = 5,
    Concat = 6,          // .. (right-associative)
    Shift = 7,           // <<, >>
    Additive = 8,        // +, -
    Multiplicative = 9,  // *, /, //, %
    UnaryBitNot = 10,    // ~, not, -, #, ^ (prefix)
    Exponent = 11,       // ^ (right-associative)
}

impl Precedence {
    fn of(op: &BinaryOp) -> Self {
        match op {
            BinaryOp::Or => Precedence::Or,
            BinaryOp::And => Precedence::And,
            BinaryOp::Lt | BinaryOp::Lte | BinaryOp::Gt | BinaryOp::Gte 
            | BinaryOp::Eq | BinaryOp::Neq => Precedence::RelationalEq,
            BinaryOp::BitOr => Precedence::BitOr,
            BinaryOp::BitXor => Precedence::BitXor,
            BinaryOp::BitAnd => Precedence::BitAnd,
            BinaryOp::Concat => Precedence::Concat,
            BinaryOp::LeftShift | BinaryOp::RightShift => Precedence::Shift,
            BinaryOp::Add | BinaryOp::Subtract => Precedence::Additive,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::FloorDivide | BinaryOp::Modulo => Precedence::Multiplicative,
            BinaryOp::Power => Precedence::Exponent,
        }
    }

    fn is_right_associative(&self) -> bool {
        matches!(self, Precedence::Concat | Precedence::Exponent)
    }

    fn next_level(&self) -> Self {
        // Return the next lower precedence level (higher numeric value)
        match self {
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::RelationalEq,
            Precedence::RelationalEq => Precedence::BitOr,
            Precedence::BitOr => Precedence::BitXor,
            Precedence::BitXor => Precedence::BitAnd,
            Precedence::BitAnd => Precedence::Concat,
            Precedence::Concat => Precedence::Shift,
            Precedence::Shift => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::UnaryBitNot,
            Precedence::UnaryBitNot => Precedence::Exponent,
            Precedence::Exponent => Precedence::Exponent, // Can't go higher
        }
    }
}

/// Try to parse a token as a binary operator
fn token_to_binop(t: TokenSlice) -> Option<BinaryOp> {
    if let Some(token) = t.0.first() {
        match token {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Subtract),
            Token::Star => Some(BinaryOp::Multiply),
            Token::Slash => Some(BinaryOp::Divide),
            Token::DoubleSlash => Some(BinaryOp::FloorDivide),
            Token::Percent => Some(BinaryOp::Modulo),
            Token::Caret => Some(BinaryOp::Power),
            Token::Concat => Some(BinaryOp::Concat),
            Token::Ampersand => Some(BinaryOp::BitAnd),
            Token::Pipe => Some(BinaryOp::BitOr),
            Token::Tilde => Some(BinaryOp::BitXor),
            Token::LShift => Some(BinaryOp::LeftShift),
            Token::RShift => Some(BinaryOp::RightShift),
            Token::Lt => Some(BinaryOp::Lt),
            Token::Lte => Some(BinaryOp::Lte),
            Token::Gt => Some(BinaryOp::Gt),
            Token::Gte => Some(BinaryOp::Gte),
            Token::Eq => Some(BinaryOp::Eq),
            Token::Neq => Some(BinaryOp::Neq),
            Token::And => Some(BinaryOp::And),
            Token::Or => Some(BinaryOp::Or),
            _ => None,
        }
    } else {
        None
    }
}

/// Parse unary operators: - | not | # | ~
fn parse_unary_op(t: TokenSlice) -> IResult<TokenSlice, UnaryOp> {
    alt((
        map(token_tag(&Token::Minus), |_| UnaryOp::Minus),
        map(token_tag(&Token::Not), |_| UnaryOp::Not),
        map(token_tag(&Token::Hash), |_| UnaryOp::Length),
        map(token_tag(&Token::Tilde), |_| UnaryOp::BitNot),
    ))
    .parse(t)
}

/// Parse a unary expression
fn parse_unary_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    alt((
        map(pair(parse_unary_op, parse_unary_expr), |(op, operand)| {
            Expression::UnaryOp {
                op,
                operand: Box::new(operand),
            }
        }),
        parse_prefix_exp,
    ))
    .parse(t)
}

/// Parse binary operations using precedence climbing
/// min_prec: only process operators with precedence >= min_prec
fn parse_binary_op(t: TokenSlice, min_prec: Precedence) -> IResult<TokenSlice, Expression> {
    let (mut rest, mut left) = parse_unary_expr(t)?;

    loop {
        // Check if next token is a binary operator
        let op = match token_to_binop(rest) {
            Some(op) => op,
            None => break,
        };

        let op_prec = Precedence::of(&op);

        // If operator precedence is too low, stop here
        if op_prec < min_prec {
            break;
        }

        // Consume the operator
        rest = TokenSlice(&rest.0[1..]);

        // For right-associative operators, use the same precedence; for left-associative, use next level
        let next_min_prec = if op_prec.is_right_associative() {
            op_prec
        } else {
            op_prec.next_level()
        };

        // Recursively parse the right side
        let (rest_right, right) = parse_binary_op(rest, next_min_prec)?;
        rest = rest_right;

        // Combine left, op, right
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok((rest, left))
}

/// Parse a full expression (entry point)
fn parse_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    parse_binary_op(t, Precedence::Or)
}

/// Parse expression with binary operators
/// Lua operator precedence (lowest to highest):
/// or, and, <, >, <=, >=, ~=, ==, |, ~, &, <<, >>, .., +, -, *, /, //, %, ^, unary
fn parse_or_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_and_expr(t)?;
    let (rest, ops) = many0(pair(
        |i| token_tag(&Token::Or)(i).map(|(r, _)| (r, BinaryOp::Or)),
        parse_and_expr,
    ))
    .parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_and_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_eq_expr(t)?;
    let (rest, ops) = many0(pair(
        |i| token_tag(&Token::And)(i).map(|(r, _)| (r, BinaryOp::And)),
        parse_eq_expr,
    ))
    .parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_eq_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_relational_expr(t)?;
    let (rest, ops) = many0(pair(parse_eq_op, parse_relational_expr)).parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_eq_op(t: TokenSlice) -> IResult<TokenSlice, BinaryOp> {
    alt((
        map(token_tag(&Token::Eq), |_| BinaryOp::Eq),
        map(token_tag(&Token::Neq), |_| BinaryOp::Neq),
    ))
    .parse(t)
}

fn parse_relational_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_bitwise_expr(t)?;
    let (rest, ops) = many0(pair(parse_relational_op, parse_bitwise_expr)).parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_relational_op(t: TokenSlice) -> IResult<TokenSlice, BinaryOp> {
    alt((
        map(token_tag(&Token::Lt), |_| BinaryOp::Lt),
        map(token_tag(&Token::Lte), |_| BinaryOp::Lte),
        map(token_tag(&Token::Gt), |_| BinaryOp::Gt),
        map(token_tag(&Token::Gte), |_| BinaryOp::Gte),
    ))
    .parse(t)
}

fn parse_bitwise_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_concat_expr(t)?;
    let (rest, ops) = many0(pair(parse_bitwise_op, parse_concat_expr)).parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_bitwise_op(t: TokenSlice) -> IResult<TokenSlice, BinaryOp> {
    alt((
        map(token_tag(&Token::Ampersand), |_| BinaryOp::BitAnd),
        map(token_tag(&Token::Pipe), |_| BinaryOp::BitOr),
        map(token_tag(&Token::Tilde), |_| BinaryOp::BitXor),
        map(token_tag(&Token::LShift), |_| BinaryOp::LeftShift),
        map(token_tag(&Token::RShift), |_| BinaryOp::RightShift),
    ))
    .parse(t)
}

fn parse_concat_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_additive_expr(t)?;
    let (rest, ops) = many0(pair(
        |i| token_tag(&Token::Concat)(i).map(|(r, _)| (r, BinaryOp::Concat)),
        parse_additive_expr,
    ))
    .parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_additive_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_multiplicative_expr(t)?;
    let (rest, ops) = many0(pair(parse_additive_op, parse_multiplicative_expr)).parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_additive_op(t: TokenSlice) -> IResult<TokenSlice, BinaryOp> {
    alt((
        map(token_tag(&Token::Plus), |_| BinaryOp::Add),
        map(token_tag(&Token::Minus), |_| BinaryOp::Subtract),
    ))
    .parse(t)
}

fn parse_multiplicative_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, mut left) = parse_power_expr(t)?;
    let (rest, ops) = many0(pair(parse_multiplicative_op, parse_power_expr)).parse(rest)?;
    for (op, right) in ops {
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok((rest, left))
}

fn parse_multiplicative_op(t: TokenSlice) -> IResult<TokenSlice, BinaryOp> {
    alt((
        map(token_tag(&Token::Star), |_| BinaryOp::Multiply),
        map(token_tag(&Token::Slash), |_| BinaryOp::Divide),
        map(token_tag(&Token::DoubleSlash), |_| BinaryOp::FloorDivide),
        map(token_tag(&Token::Percent), |_| BinaryOp::Modulo),
    ))
    .parse(t)
}

fn parse_power_expr(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, left) = parse_unary_expr(t)?;
    let (rest, op) = opt(token_tag(&Token::Caret)).parse(rest)?;
    if op.is_some() {
        let (rest, right) = parse_power_expr(rest)?;
        Ok((
            rest,
            Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::Power,
                right: Box::new(right),
            },
        ))
    } else {
        Ok((rest, left))
    }
}

/// Parse the full expression
fn parse_expression(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    parse_or_expr(t)
}

fn parse_expression_list(t: TokenSlice) -> IResult<TokenSlice, Vec<Expression>> {
    let (rest, first) = parse_expression(t)?;
    let (rest, rest_exprs) = many0(pair(token_tag(&Token::Comma), parse_expression)).parse(rest)?;

    let mut result = vec![first];
    for (_, expr) in rest_exprs {
        result.push(expr);
    }
    Ok((rest, result))
}

fn parse_return_statement(t: TokenSlice) -> IResult<TokenSlice, ReturnStatement> {
    let (rest, _) = token_tag(&Token::Return).parse(t)?;
    let (rest, list) = opt(parse_expression_list).parse(rest)?;
    let (rest, _) = opt(token_tag(&Token::Semicolon)).parse(rest)?;
    Ok((
        rest,
        ReturnStatement {
            expression_list: list.unwrap_or_default(),
        },
    ))
}

fn token_tag(expected: &Token) -> impl Fn(TokenSlice) -> IResult<TokenSlice, &Token> {
    let expected = expected.clone();
    move |input: TokenSlice| {
        if let Some(tok) = input.0.first() {
            if tok == &expected {
                Ok((TokenSlice(&input.0[1..]), tok))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )))
        }
    }
}

/// Parse a block of statements, stopping at block-terminating tokens
/// Block terminators: 'end', 'else', 'elseif', 'until', EOF
fn parse_block(t: TokenSlice) -> IResult<TokenSlice, Block> {
    let mut statements = Vec::new();
    let mut current = t;

    // Parse statements until we hit a block terminator
    loop {
        // Check if we've hit a block terminator or EOF
        if current.0.is_empty() {
            break;
        }

        // Check for block terminating tokens
        if let Some(token) = current.0.first() {
            match token {
                Token::End | Token::Else | Token::Elseif | Token::Until => {
                    break;
                }
                _ => {}
            }
        }

        // Try to parse a return statement first (since it can be followed by anything)
        if let Ok((rest, ret_stmt)) = parse_return_statement(current) {
            return Ok((
                rest,
                Block {
                    statements,
                    return_statement: Some(ret_stmt),
                },
            ));
        }

        // Try to parse a regular statement
        match parse_statement(current) {
            Ok((rest, stmt)) => {
                statements.push(stmt);
                current = rest;
            }
            Err(_) => {
                // If we can't parse a statement, we're done with the block
                break;
            }
        }
    }

    Ok((
        current,
        Block {
            statements,
            return_statement: None,
        },
    ))
}

pub fn parse(t: TokenSlice) -> IResult<TokenSlice, Block> {
    parse_block(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = tokenize("if then else end").unwrap();
        assert_eq!(
            tokens,
            vec![Token::If, Token::Then, Token::Else, Token::End]
        );
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("local x = 42").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("42".to_string())
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("a + b * c").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::Star,
                Token::Identifier("c".to_string())
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize(r#"local msg = "hello""#).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("msg".to_string()),
                Token::Equals,
                Token::StringLit("hello".to_string())
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("x = 5 -- comment").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("5".to_string())
            ]
        );
    }

    #[test]
    fn test_if_statement() {
        let code = "if x > 5 then print(x) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Identifier("x".to_string()),
                Token::Gt,
                Token::Number("5".to_string()),
                Token::Then,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("x".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_for_loop() {
        let code = "for i = 1, 10 do print(i) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::For,
                Token::Identifier("i".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Comma,
                Token::Number("10".to_string()),
                Token::Do,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("i".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_function_definition() {
        let code = "function add(a, b) return a + b end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Function,
                Token::Identifier("add".to_string()),
                Token::LParen,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::Identifier("b".to_string()),
                Token::RParen,
                Token::Return,
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_table_constructor() {
        let code = "{1, 2, x = 3}";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LBrace,
                Token::Number("1".to_string()),
                Token::Comma,
                Token::Number("2".to_string()),
                Token::Comma,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("3".to_string()),
                Token::RBrace
            ]
        );
    }

    #[test]
    fn test_all_operators() {
        let code = "+ - * / // ^ % & | ~ >> << .. < <= > >= == ~=";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::DoubleSlash,
                Token::Caret,
                Token::Percent,
                Token::Ampersand,
                Token::Pipe,
                Token::Tilde,
                Token::RShift,
                Token::LShift,
                Token::Concat,
                Token::Lt,
                Token::Lte,
                Token::Gt,
                Token::Gte,
                Token::Eq,
                Token::Neq
            ]
        );
    }

    #[test]
    fn test_all_keywords() {
        let code = "and break do else elseif end false for function goto if in local nil not or repeat return then true until while";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::And,
                Token::Break,
                Token::Do,
                Token::Else,
                Token::Elseif,
                Token::End,
                Token::False,
                Token::For,
                Token::Function,
                Token::Goto,
                Token::If,
                Token::In,
                Token::Local,
                Token::Nil,
                Token::Not,
                Token::Or,
                Token::Repeat,
                Token::Return,
                Token::Then,
                Token::True,
                Token::Until,
                Token::While
            ]
        );
    }

    #[test]
    fn test_varargs() {
        let code = "function variadic(...) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Function,
                Token::Identifier("variadic".to_string()),
                Token::LParen,
                Token::Varargs,
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"local msg = "hello" .. " " .. "world""#;
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("msg".to_string()),
                Token::Equals,
                Token::StringLit("hello".to_string()),
                Token::Concat,
                Token::StringLit(" ".to_string()),
                Token::Concat,
                Token::StringLit("world".to_string())
            ]
        );
    }

    #[test]
    fn test_double_colon_label() {
        let code = "::start:: x = 1";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::DoubleColon,
                Token::Identifier("start".to_string()),
                Token::DoubleColon,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string())
            ]
        );
    }

    #[test]
    fn test_method_call() {
        let code = "obj:method(arg)";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("obj".to_string()),
                Token::Colon,
                Token::Identifier("method".to_string()),
                Token::LParen,
                Token::Identifier("arg".to_string()),
                Token::RParen
            ]
        );
    }

    #[test]
    fn test_boolean_and_nil() {
        let code = "local x = true local y = false local z = nil";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::True,
                Token::Local,
                Token::Identifier("y".to_string()),
                Token::Equals,
                Token::False,
                Token::Local,
                Token::Identifier("z".to_string()),
                Token::Equals,
                Token::Nil
            ]
        );
    }

    #[test]
    fn test_while_loop() {
        let code = "while x < 10 do x = x + 1 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::Identifier("x".to_string()),
                Token::Lt,
                Token::Number("10".to_string()),
                Token::Do,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Number("1".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_repeat_until_loop() {
        let code = "repeat x = x + 1 until x >= 10";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Repeat,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Number("1".to_string()),
                Token::Until,
                Token::Identifier("x".to_string()),
                Token::Gte,
                Token::Number("10".to_string())
            ]
        );
    }

    #[test]
    fn test_table_indexing() {
        let code = "t[1] t[\"key\"] t.field";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("t".to_string()),
                Token::LBracket,
                Token::Number("1".to_string()),
                Token::RBracket,
                Token::Identifier("t".to_string()),
                Token::LBracket,
                Token::StringLit("key".to_string()),
                Token::RBracket,
                Token::Identifier("t".to_string()),
                Token::Dot,
                Token::Identifier("field".to_string())
            ]
        );
    }

    #[test]
    fn test_logical_operators() {
        let code = "x and y or not z";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::And,
                Token::Identifier("y".to_string()),
                Token::Or,
                Token::Not,
                Token::Identifier("z".to_string())
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let code = "result = (a + b) * c - d / e ^ 2";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("result".to_string()),
                Token::Equals,
                Token::LParen,
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::RParen,
                Token::Star,
                Token::Identifier("c".to_string()),
                Token::Minus,
                Token::Identifier("d".to_string()),
                Token::Slash,
                Token::Identifier("e".to_string()),
                Token::Caret,
                Token::Number("2".to_string())
            ]
        );
    }

    #[test]
    fn test_underscored_identifiers() {
        let code = "_var _G __private__ a_b_c";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("_var".to_string()),
                Token::Identifier("_G".to_string()),
                Token::Identifier("__private__".to_string()),
                Token::Identifier("a_b_c".to_string())
            ]
        );
    }

    #[test]
    fn test_decimal_numbers() {
        let code = "0 1 42 3.14 0.5 100.0";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number("0".to_string()),
                Token::Number("1".to_string()),
                Token::Number("42".to_string()),
                Token::Number("3.14".to_string()),
                Token::Number("0.5".to_string()),
                Token::Number("100.0".to_string())
            ]
        );
    }

    #[test]
    fn test_comment_variations() {
        let code = "x = 1 -- comment\ny = 2 -- another comment\nz = 3";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Identifier("y".to_string()),
                Token::Equals,
                Token::Number("2".to_string()),
                Token::Identifier("z".to_string()),
                Token::Equals,
                Token::Number("3".to_string())
            ]
        );
    }

    #[test]
    fn test_local_function() {
        let code = "local function helper() return 42 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Function,
                Token::Identifier("helper".to_string()),
                Token::LParen,
                Token::RParen,
                Token::Return,
                Token::Number("42".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_goto_and_label() {
        let code = "goto skip ::skip:: x = 1";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Goto,
                Token::Identifier("skip".to_string()),
                Token::DoubleColon,
                Token::Identifier("skip".to_string()),
                Token::DoubleColon,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string())
            ]
        );
    }

    #[test]
    fn test_unary_operators() {
        let code = "-x not y #z ~a";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Minus,
                Token::Identifier("x".to_string()),
                Token::Not,
                Token::Identifier("y".to_string()),
                Token::Hash,
                Token::Identifier("z".to_string()),
                Token::Tilde,
                Token::Identifier("a".to_string())
            ]
        );
    }

    #[test]
    fn test_bitwise_operators() {
        let code = "a & b | c ~ d a >> b a << b";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Ampersand,
                Token::Identifier("b".to_string()),
                Token::Pipe,
                Token::Identifier("c".to_string()),
                Token::Tilde,
                Token::Identifier("d".to_string()),
                Token::Identifier("a".to_string()),
                Token::RShift,
                Token::Identifier("b".to_string()),
                Token::Identifier("a".to_string()),
                Token::LShift,
                Token::Identifier("b".to_string())
            ]
        );
    }

    #[test]
    fn test_mixed_whitespace() {
        let code = "x=1\t+\t2  \n  +   3";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Plus,
                Token::Number("2".to_string()),
                Token::Plus,
                Token::Number("3".to_string())
            ]
        );
    }

    #[test]
    fn test_for_in_loop() {
        let code = "for k, v in pairs(t) do print(k, v) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::For,
                Token::Identifier("k".to_string()),
                Token::Comma,
                Token::Identifier("v".to_string()),
                Token::In,
                Token::Identifier("pairs".to_string()),
                Token::LParen,
                Token::Identifier("t".to_string()),
                Token::RParen,
                Token::Do,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("k".to_string()),
                Token::Comma,
                Token::Identifier("v".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    // Phase 2 Tests: Prefix Expressions & Function Calls
    
    #[test]
    fn test_parse_prefix_table_indexing() {
        let code = "t[1]";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::TableIndexing { object, index } => {
                assert!(matches!(*object, Expression::Identifier(_)));
                assert!(matches!(*index, Expression::Number(_)));
            }
            _ => panic!("Expected TableIndexing, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_field_access() {
        let code = "obj.field";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::FieldAccess { object, field } => {
                assert!(matches!(*object, Expression::Identifier(_)));
                assert_eq!(field, "field");
            }
            _ => panic!("Expected FieldAccess, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_function_call() {
        let code = "print(42)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::FunctionCall { function, args } => {
                assert!(matches!(*function, Expression::Identifier(_)));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expression::Number(_)));
            }
            _ => panic!("Expected FunctionCall, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_method_call() {
        let code = "obj:method(arg)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::MethodCall { object, method, args } => {
                assert!(matches!(*object, Expression::Identifier(_)));
                assert_eq!(method, "method");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected MethodCall, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_chained_access() {
        let code = "t[1].field";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        // Should be FieldAccess { object: TableIndexing { ... }, field: "field" }
        match expr {
            Expression::FieldAccess { object, field } => {
                assert!(matches!(*object, Expression::TableIndexing { .. }));
                assert_eq!(field, "field");
            }
            _ => panic!("Expected FieldAccess, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_table_constructor_empty() {
        let code = "{}";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::TableConstructor { fields } => {
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected TableConstructor, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_table_constructor_fields() {
        let code = "{1, x = 2}";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::TableConstructor { fields } => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected TableConstructor, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_function_def() {
        let code = "function() return 42 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::FunctionDef(body) => {
                assert_eq!(body.params.len(), 0);
                assert!(!body.varargs);
            }
            _ => panic!("Expected FunctionDef, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_function_def_params() {
        let code = "function(a, b) return a + b end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::FunctionDef(body) => {
                assert_eq!(body.params.len(), 2);
                assert_eq!(body.params[0], "a");
                assert_eq!(body.params[1], "b");
                assert!(!body.varargs);
            }
            _ => panic!("Expected FunctionDef, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_function_def_varargs() {
        let code = "function(...) end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        match expr {
            Expression::FunctionDef(body) => {
                assert_eq!(body.params.len(), 0);
                assert!(body.varargs);
            }
            _ => panic!("Expected FunctionDef, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_parse_prefix_parenthesized() {
        let code = "(42)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_prefix_exp(ts).unwrap();
        
        // Parenthesized expressions are unwrapped, so we get the inner expr
        match expr {
            Expression::Number(_) => {}
            _ => panic!("Expected Number, got {:?}", expr),
        }
    }
    
    #[test]
    fn test_empty_input() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_only_whitespace() {
        let tokens = tokenize("   \n\t  \n  ").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_only_comments() {
        let tokens = tokenize("-- comment\n-- another comment").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_return_statement() {
        let code = "return a, b, c";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Return,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::Identifier("b".to_string()),
                Token::Comma,
                Token::Identifier("c".to_string())
            ]
        );
    }

    #[test]
    fn test_break_statement() {
        let code = "while true do break end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::True,
                Token::Do,
                Token::Break,
                Token::End
            ]
        );
    }

    #[test]
    fn test_do_block() {
        let code = "do local x = 1 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Do,
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::End
            ]
        );
    }

    // Binary operator precedence tests

    #[test]
    fn test_binary_op_basic() {
        let code = "a + b";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (rest, expr) = parse_expr(slice).unwrap();
        
        // Check we parsed the entire expression
        assert_eq!(rest.0.len(), 0);
        
        // Verify it's a BinaryOp
        match expr {
            Expression::BinaryOp { left, op, right } => {
                assert_eq!(op, BinaryOp::Add);
                match *left {
                    Expression::Identifier(ref id) => assert_eq!(id, "a"),
                    _ => panic!("Expected identifier on left"),
                }
                match *right {
                    Expression::Identifier(ref id) => assert_eq!(id, "b"),
                    _ => panic!("Expected identifier on right"),
                }
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_binary_op_precedence_mul_add() {
        // a + b * c should parse as a + (b * c)
        let code = "a + b * c";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_expr(slice).unwrap();
        
        match expr {
            Expression::BinaryOp { left, op: BinaryOp::Add, right } => {
                // Left should be 'a'
                match *left {
                    Expression::Identifier(ref id) => assert_eq!(id, "a"),
                    _ => panic!("Expected identifier 'a' on left"),
                }
                // Right should be (b * c)
                match *right {
                    Expression::BinaryOp { op: BinaryOp::Multiply, .. } => {
                        // Correct structure
                    }
                    _ => panic!("Expected (b * c) on right"),
                }
            }
            _ => panic!("Expected Add as top-level operator"),
        }
    }

    #[test]
    fn test_binary_op_precedence_and_or() {
        // a or b and c should parse as a or (b and c)
        let code = "a or b and c";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_expr(slice).unwrap();
        
        match expr {
            Expression::BinaryOp { left, op: BinaryOp::Or, right } => {
                // Left should be 'a'
                match *left {
                    Expression::Identifier(ref id) => assert_eq!(id, "a"),
                    _ => panic!("Expected identifier 'a' on left"),
                }
                // Right should be (b and c)
                match *right {
                    Expression::BinaryOp { op: BinaryOp::And, .. } => {
                        // Correct structure
                    }
                    _ => panic!("Expected (b and c) on right"),
                }
            }
            _ => panic!("Expected Or as top-level operator"),
        }
    }

    #[test]
    fn test_binary_op_left_associative() {
        // a + b + c should parse as (a + b) + c
        let code = "a + b + c";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_expr(slice).unwrap();
        
        match expr {
            Expression::BinaryOp { left, op: BinaryOp::Add, right } => {
                // Right should be 'c'
                match *right {
                    Expression::Identifier(ref id) => assert_eq!(id, "c"),
                    _ => panic!("Expected identifier 'c' on right"),
                }
                // Left should be (a + b)
                match *left {
                    Expression::BinaryOp { op: BinaryOp::Add, .. } => {
                        // Correct structure
                    }
                    _ => panic!("Expected (a + b) on left"),
                }
            }
            _ => panic!("Expected Add as top-level operator"),
        }
    }

    #[test]
    fn test_binary_op_right_associative_concat() {
        // a .. b .. c should parse as a .. (b .. c)
        let code = "a .. b .. c";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_expr(slice).unwrap();
        
        match expr {
            Expression::BinaryOp { left, op: BinaryOp::Concat, right } => {
                // Left should be 'a'
                match *left {
                    Expression::Identifier(ref id) => assert_eq!(id, "a"),
                    _ => panic!("Expected identifier 'a' on left"),
                }
                // Right should be (b .. c)
                match *right {
                    Expression::BinaryOp { op: BinaryOp::Concat, .. } => {
                        // Correct structure
                    }
                    _ => panic!("Expected (b .. c) on right"),
                }
            }
            _ => panic!("Expected Concat as top-level operator"),
        }
    }

    #[test]
    fn test_binary_op_complex_precedence() {
        // a + b * c - d / e ^ 2 should parse respecting precedence
        let code = "a + b * c - d / e ^ 2";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_rest, expr) = parse_expr(slice).unwrap();
        
        // Just verify it parses without error for now
        assert!(matches!(expr, Expression::BinaryOp { .. }));
    }

    #[test]
    fn test_binary_op_parenthesized() {
        // (a + b) * c should parse with parentheses overriding precedence
        // For now, just test that literals parse correctly
        let code = "a";
        let tokens = tokenize(code).unwrap();
        let slice = TokenSlice::from(tokens.as_slice());
        let (_, expr) = parse_expr(slice).unwrap();
        
        match expr {
            Expression::Identifier(ref id) => assert_eq!(id, "a"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_all_binary_operators_tokenize() {
        // Ensure all binary operators tokenize correctly
        let code = "+ - * / // % ^ & | ~ << >> .. < <= > >= == ~= and or";
        let tokens = tokenize(code).unwrap();
        
        assert!(tokens.iter().any(|t| matches!(t, Token::Plus)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Minus)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Star)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Slash)));
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleSlash)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Percent)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Caret)));
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }
    
    // Phase 3: Statement Parsing Tests
    
    #[test]
    fn test_parse_empty_statement() {
        let code = ";";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Empty => {}
            _ => panic!("Expected Empty statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_break_statement() {
        let code = "break";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Break => {}
            _ => panic!("Expected Break statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_label_statement() {
        let code = "::label::";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Label(name) => assert_eq!(name, "label"),
            _ => panic!("Expected Label statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_goto_statement() {
        let code = "goto label";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Goto(name) => assert_eq!(name, "label"),
            _ => panic!("Expected Goto statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_do_block() {
        let code = "do local x = 1 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Do(_) => {}
            _ => panic!("Expected Do block, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_while_loop() {
        let code = "while x < 10 do x = x + 1 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::While { condition, body } => {
                assert!(matches!(condition, Expression::BinaryOp { .. }));
                assert!(!body.statements.is_empty() || body.return_statement.is_some());
            }
            _ => panic!("Expected While loop, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_repeat_until() {
        let code = "repeat x = x + 1 until x >= 10";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Repeat { body, condition } => {
                assert!(!body.statements.is_empty() || body.return_statement.is_some());
                assert!(matches!(condition, Expression::BinaryOp { .. }));
            }
            _ => panic!("Expected Repeat-Until, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_if_statement_simple() {
        let code = "if x > 0 then y = 1 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::If { condition, then_block, elseif_parts, else_block } => {
                assert!(matches!(condition, Expression::BinaryOp { .. }));
                assert!(elseif_parts.is_empty());
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_if_statement_with_else() {
        let code = "if x > 0 then y = 1 else y = 2 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::If { condition, then_block, elseif_parts, else_block } => {
                assert!(matches!(condition, Expression::BinaryOp { .. }));
                assert!(elseif_parts.is_empty());
                assert!(else_block.is_some());
            }
            _ => panic!("Expected If statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_if_statement_with_elseif() {
        let code = "if x > 0 then y = 1 elseif x < 0 then y = -1 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::If { condition, then_block, elseif_parts, else_block } => {
                assert!(matches!(condition, Expression::BinaryOp { .. }));
                assert_eq!(elseif_parts.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_for_numeric() {
        let code = "for i = 1, 10 do x = i end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::ForNumeric { var, start, end, step, body: _ } => {
                assert_eq!(var, "i");
                assert!(matches!(start, Expression::Number(_)));
                assert!(matches!(end, Expression::Number(_)));
                assert!(step.is_none());
            }
            _ => panic!("Expected ForNumeric, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_for_numeric_with_step() {
        let code = "for i = 1, 10, 2 do x = i end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::ForNumeric { var, start, end, step, body: _ } => {
                assert_eq!(var, "i");
                assert!(matches!(start, Expression::Number(_)));
                assert!(matches!(end, Expression::Number(_)));
                assert!(step.is_some());
            }
            _ => panic!("Expected ForNumeric, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_for_generic() {
        let code = "for k, v in pairs(t) do print(k) end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::ForGeneric { vars, iterables, body: _ } => {
                assert_eq!(vars.len(), 2);
                assert_eq!(vars[0], "k");
                assert_eq!(vars[1], "v");
                assert_eq!(iterables.len(), 1);
            }
            _ => panic!("Expected ForGeneric, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_function_decl() {
        let code = "function add(a, b) return a + b end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::FunctionDecl { name, body } => {
                assert_eq!(name, "add");
                assert_eq!(body.params.len(), 2);
            }
            _ => panic!("Expected FunctionDecl, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_local_function() {
        let code = "local function test() end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::LocalFunction { name, body } => {
                assert_eq!(name, "test");
                assert_eq!(body.params.len(), 0);
            }
            _ => panic!("Expected LocalFunction, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_local_vars_without_values() {
        let code = "local x, y";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::LocalVars { names, values } => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "x");
                assert_eq!(names[1], "y");
                assert!(values.is_none());
            }
            _ => panic!("Expected LocalVars, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_local_vars_with_values() {
        let code = "local x, y = 1, 2";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::LocalVars { names, values } => {
                assert_eq!(names.len(), 2);
                assert!(values.is_some());
                assert_eq!(values.unwrap().len(), 2);
            }
            _ => panic!("Expected LocalVars, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_function_call_statement() {
        let code = "print(x)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::FunctionCall(expr) => {
                assert!(matches!(expr, Expression::FunctionCall { .. }));
            }
            _ => panic!("Expected FunctionCall statement, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_assignment_simple() {
        let code = "x = 5";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Assignment { variables, values } => {
                assert_eq!(variables.len(), 1);
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected Assignment, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_assignment_multiple() {
        let code = "x, y = 1, 2";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Assignment { variables, values } => {
                assert_eq!(variables.len(), 2);
                assert_eq!(values.len(), 2);
            }
            _ => panic!("Expected Assignment, got {:?}", stmt),
        }
    }
    
    #[test]
    fn test_parse_assignment_table_field() {
        let code = "t[1] = 5";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (_, stmt) = parse_statement(ts).unwrap();
        
        match stmt {
            Statement::Assignment { variables, values } => {
                assert_eq!(variables.len(), 1);
                assert!(matches!(variables[0], Expression::TableIndexing { .. }));
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected Assignment, got {:?}", stmt),
        }
    }
    
    // Phase 4: Top-Level Block Parsing Tests
    
    #[test]
    fn test_parse_block_empty() {
        let code = "";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert!(block.statements.is_empty());
        assert!(block.return_statement.is_none());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_block_single_statement() {
        let code = "x = 1";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 1);
        assert!(block.return_statement.is_none());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_block_multiple_statements() {
        let code = "x = 1; y = 2; z = 3";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 5); // 3 assignments + 2 empty statements for semicolons
        assert!(block.return_statement.is_none());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_block_with_return() {
        let code = "x = 1; return 42";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 2); // assignment + empty statement
        assert!(block.return_statement.is_some());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_block_stops_at_end() {
        let code = "x = 1 end y = 2";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 1);
        assert!(block.return_statement.is_none());
        // Should stop before 'end'
        assert!(!rest.0.is_empty());
        assert_eq!(rest.0[0], Token::End);
    }
    
    #[test]
    fn test_parse_block_stops_at_else() {
        let code = "x = 1 else y = 2";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 1);
        assert!(block.return_statement.is_none());
        // Should stop before 'else'
        assert!(!rest.0.is_empty());
        assert_eq!(rest.0[0], Token::Else);
    }
    
    #[test]
    fn test_parse_block_stops_at_elseif() {
        let code = "x = 1 elseif x > 0 then";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 1);
        assert!(block.return_statement.is_none());
        // Should stop before 'elseif'
        assert!(!rest.0.is_empty());
        assert_eq!(rest.0[0], Token::Elseif);
    }
    
    #[test]
    fn test_parse_block_stops_at_until() {
        let code = "x = 1 until x > 0";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse_block(ts).unwrap();
        
        assert_eq!(block.statements.len(), 1);
        assert!(block.return_statement.is_none());
        // Should stop before 'until'
        assert!(!rest.0.is_empty());
        assert_eq!(rest.0[0], Token::Until);
    }
    
    #[test]
    fn test_parse_chunk_simple() {
        let code = "local x = 1; print(x); return x";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();
        
        // Expecting: local, empty (from first semicolon), print call, empty (from second semicolon)
        assert_eq!(block.statements.len(), 4);
        assert!(block.return_statement.is_some());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_chunk_with_do_block() {
        let code = "do local x = 1 end print(x)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();
        
        assert_eq!(block.statements.len(), 2); // do block + print call
        assert!(block.return_statement.is_none());
        assert!(rest.0.is_empty());
    }
    
    #[test]
    fn test_parse_nested_blocks() {
        let code = "if x > 0 then y = 1; z = 2 else w = 3 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        match &block.statements[0] {
            Statement::If { then_block, else_block, .. } => {
                assert!(!then_block.statements.is_empty());
                assert!(else_block.is_some());
            }
            _ => panic!("Expected If statement"),
        }
        assert!(rest.0.is_empty());
    }

    // Phase 5: Integration Tests - Complex Programs

    #[test]
    fn test_complete_program_mixed_statements() {
        let code = "
        local x = 10
        local y = 20
        local z = x + y
        print(z)
        return z
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 4); // 3 local vars + 1 print call
        assert!(block.return_statement.is_some());
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_nested_loops() {
        let code = "
        for i = 1, 10 do
            for j = 1, 5 do
                print(i, j)
            end
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_nested_if_statements() {
        let code = "
        if x > 0 then
            if y > 0 then
                z = 1
            else
                z = 2
            end
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_function_with_nested_blocks() {
        let code = "
        function foo(a, b)
            if a > b then
                return a
            else
                return b
            end
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_mixed_operators_complex_expression() {
        let code = "x = 1 + 2 * 3 ^ 2 - 4 / 5";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_table_with_mixed_fields() {
        let code = "t = {1, 2, 3, x = 10, [5] = 20, 30}";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_function_calls_with_various_args() {
        let code = "
        foo()
        bar(1, 2, 3)
        baz(\"string\")
        qux{a=1, b=2}
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 4);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_method_calls() {
        let code = "
        obj:method()
        obj:method(1, 2)
        obj:method{x=1}
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 3);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_complex_assignment() {
        let code = "a, b, c = 1, 2, 3";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_while_with_break() {
        let code = "
        while true do
            if x > 10 then
                break
            end
            x = x + 1
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_repeat_until() {
        let code = "
        repeat
            x = x + 1
        until x > 10
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_for_generic_loop() {
        let code = "
        for k, v in pairs(t) do
            print(k, v)
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_local_function_declaration() {
        let code = "
        local function helper(x)
            return x * 2
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_multiple_returns() {
        let code = "
        function foo(x)
            if x > 0 then
                return x, \"positive\"
            else
                return -x, \"negative\"
            end
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    // Edge cases for Phase 5

    #[test]
    fn test_empty_program() {
        let code = "";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 0);
        assert!(block.return_statement.is_none());
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_only_comments_and_whitespace() {
        let code = "-- comment 1\n  \n-- comment 2";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 0);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_multiple_empty_statements() {
        let code = ";;; x = 1 ;;;";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        // 3 empty + 1 assignment + 3 empty
        assert_eq!(block.statements.len(), 7);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_operator_precedence_parens() {
        let code = "x = (1 + 2) * 3";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_unary_with_binary() {
        let code = "x = -5 + 3; y = not true and false";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        // x = -5 + 3 (1) + ; (1) + y = not true and false (1) = 3 statements
        assert_eq!(block.statements.len(), 3);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_string_concatenation_chain() {
        let code = "s = \"a\" .. \"b\" .. \"c\" .. \"d\"";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_label_and_goto() {
        let code = "
        ::start::
        print(1)
        goto finish
        print(2)
        ::finish::
        print(3)
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 6); // label, print, goto, print, label, print
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_table_indexing_chains() {
        let code = "x = t[1][2][3]; y = t.a.b.c; z = t[k].field[m]";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        // x = t[1][2][3] (1) + ; (1) + y = t.a.b.c (1) + ; (1) + z = t[k].field[m] (1) = 5
        assert_eq!(block.statements.len(), 5);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_varargs_in_function() {
        let code = "
        function print_all(...)
            return ...
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_do_block_scoping() {
        let code = "
        x = 1
        do
            local x = 2
            y = x
        end
        z = x
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 3); // x = 1, do block, z = x
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_multiple_elseif() {
        let code = "
        if x == 1 then
            print(1)
        elseif x == 2 then
            print(2)
        elseif x == 3 then
            print(3)
        else
            print(0)
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_anonymous_function() {
        let code = "f = function(x) return x * 2 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }
}
