//! Expression parsing - binary ops, unary ops, literals, prefix expressions

use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::many0,
    sequence::pair,
    IResult, Parser,
};

use super::{
    Token, TokenSlice, Expression, BinaryOp, UnaryOp, Field, FieldKey, FunctionBody,
    token_tag,
};

/// Parse number literal from token
pub fn parse_number_literal(input: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::Number(n)) = input.0.first() {
        Ok((TokenSlice(&input.0[1..]), Expression::Number(n.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse string literal from token
pub fn parse_string_literal(input: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::StringLit(s)) = input.0.first() {
        Ok((TokenSlice(&input.0[1..]), Expression::String(s.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse identifier
pub fn parse_identifier(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    if let Some(Token::Identifier(id)) = t.0.first() {
        Ok((TokenSlice(&t.0[1..]), Expression::Identifier(id.clone())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse table constructor: `{ [fieldlist] }`
pub fn parse_table_constructor(t: TokenSlice) -> IResult<TokenSlice, Expression> {
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
pub fn parse_function_def(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (rest, _) = token_tag(&Token::Function)(t)?;
    let (rest, body) = parse_funcbody(rest)?;
    Ok((rest, Expression::FunctionDef(Box::new(body))))
}

/// Parse function body: `( [parlist] ) block end`
pub fn parse_funcbody(t: TokenSlice) -> IResult<TokenSlice, FunctionBody> {
    let (rest, _) = token_tag(&Token::LParen)(t)?;
    let (rest, params_info) = opt(|i| parse_parlist(i)).parse(rest)?;
    let (rest, _) = token_tag(&Token::RParen)(rest)?;
    let (rest, block) = super::statement::parse_block(rest)?;
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

/// Parse arguments for a function call: `(explist) | tableconstructor | string_literal`
pub fn parse_args(t: TokenSlice) -> IResult<TokenSlice, Vec<Expression>> {
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

    Err(nom::Err::Error(nom::error::Error::new(
        t,
        nom::error::ErrorKind::Alt,
    )))
}

/// Parse a primary/prefix expression, then apply suffix operations (indexing, calls, method calls)
pub fn parse_prefix_exp(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    let (mut rest, mut expr) = {
        // Try simple literals first
        if let Ok((r, expr)) = alt((
            map(token_tag(&Token::Nil), |_| Expression::Nil),
            map(token_tag(&Token::True), |_| Expression::Boolean(true)),
            map(token_tag(&Token::False), |_| Expression::Boolean(false)),
            map(token_tag(&Token::Varargs), |_| Expression::Varargs),
            parse_number_literal,
            parse_string_literal,
        ))
        .parse(t)
        {
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
pub fn parse_expression(t: TokenSlice) -> IResult<TokenSlice, Expression> {
    parse_or_expr(t)
}

pub fn parse_expression_list(t: TokenSlice) -> IResult<TokenSlice, Vec<Expression>> {
    let (rest, first) = parse_expression(t)?;
    let (rest, rest_exprs) = many0(pair(token_tag(&Token::Comma), parse_expression)).parse(rest)?;

    let mut result = vec![first];
    for (_, expr) in rest_exprs {
        result.push(expr);
    }
    Ok((rest, result))
}
