//! Common parsing helpers and token utilities

use phf::phf_map;
use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, satisfy},
    combinator::{opt, recognize},
    sequence::{pair, preceded},
    IResult, Parser,
};

use super::Token;
use super::Token::*;

// Keywords and symbols lookup tables
pub const KEYWORDS: phf::Map<&str, Token> = phf_map! {
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

pub const SYMBOLS: phf::Map<&str, Token> = phf_map! {
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

// Low-level tokenization helpers
pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

pub fn number(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, opt(preceded(char('.'), digit1)))).parse(input)
}

pub fn string_literal(input: &str) -> IResult<&str, String> {
    if input.starts_with('\'') {
        let (input, _) = char('\'').parse(input)?;
        let (input, content) = take_while(|c: char| c != '\'').parse(input)?;
        let (input, _) = char('\'').parse(input)?;
        let processed = process_escape_sequences(content);
        Ok((input, processed))
    } else {
        let (input, _) = char('"').parse(input)?;
        let (input, content) = take_while(|c: char| c != '"').parse(input)?;
        let (input, _) = char('"').parse(input)?;
        let processed = process_escape_sequences(content);
        Ok((input, processed))
    }
}

pub fn process_escape_sequences(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    '"' => {
                        result.push('"');
                        chars.next();
                    }
                    '\'' => {
                        result.push('\'');
                        chars.next();
                    }
                    _ => {
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn symbol(input: &str) -> IResult<&str, Token> {
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

pub fn tokenize_single(input: &str) -> IResult<&str, Token> {
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
