#[allow(unused)]
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{
        alpha1, alphanumeric1, char, digit1, hex_digit1, line_ending, multispace1, none_of,
    },
    combinator::{all_consuming, complete, cut, map, map_res, not, opt, recognize, success, value},
    error::{context, convert_error, VerboseError},
    multi::{fold_many0, many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use nom_locate::{position, LocatedSpan};

use std::{ops::Range, collections::HashSet};
use std::sync::Arc;

use crate::ast::*;

type Span<'a> = LocatedSpan<&'a str>;
//type Result<'a, T> = nom::IResult<Span<'a>, T, VerboseError<Span<'a>>>;
type Result<'a, T> = nom::IResult<Span<'a>, T, ErrorDetail<'a>>;

#[derive(Clone, Debug)]
pub struct ErrorDetail<'a>(Span<'a>, String);

impl<'a> nom::error::ParseError<Span<'a>> for ErrorDetail<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        ErrorDetail(input, format!("ErrorKind: {:?}", kind))
    }

    fn append(input: Span<'a>, kind: nom::error::ErrorKind, other: Self) -> Self {
        ErrorDetail(input, format!("ErrorKind: {:?}, {}", kind, other.0))
    }
}

impl<'a> nom::error::FromExternalError<Span<'a>, String> for ErrorDetail<'a> {
    fn from_external_error(input: Span<'a>, kind: nom::error::ErrorKind, err: String) -> Self {
        ErrorDetail(input, format!("ExternalError: {:?} {}", kind, err))
    }
}

fn ws_or_comment(i: Span) -> Result<&str> {
    value(
        "",
        many0_count(alt((
            delimited(tag("//"), many0_count(none_of("\r\n")), line_ending),
            delimited(tag("/*"), many0_count(none_of("*/")), tag("*/")),
            value(0, multispace1),
        ))),
    )(i)
}

// terminated by whitespace or comment
fn tws<'a, O, F>(f: F) -> impl FnMut(Span<'a>) -> Result<'a, O>
where
    F: FnMut(Span<'a>) -> Result<'a, O>,
{
    terminated(f, ws_or_comment)
}

// capture parsed position
fn pos<'a, O, F>(f: F) -> impl FnMut(Span<'a>) -> Result<(Range<usize>, O)>
where
    F: FnMut(Span<'a>) -> Result<'a, O>,
{
    map(tuple((position, f, position)), |(start, val, end)| {
        (start.location_offset()..end.location_offset(), val)
    })
}

fn node(pos: Range<usize>, expr: Expr) -> Arc<Node> {
    Arc::new(Node { pos, expr })
}

fn ident(i: Span) -> Result<Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(i)
}

fn expr_const(i: Span) -> Result<Arc<Node>> {
    map(tws(pos(nom::number::complete::double)), |(pos, num)| {
        node(pos, Expr::Num(num))
    })(i)
}

fn expr_var(i: Span) -> Result<Arc<Node>> {
    alt((
        map(tws(pos(ident)), |(pos, name)| {
            node(pos, Expr::Var(name.to_string()))
        }),
        expr_const,
    ))(i)
}

fn expr_parens(i: Span) -> Result<Arc<Node>> {
    alt((delimited(tws(tag("(")), expr, tws(tag(")"))), expr_var))(i)
}

/*fn expr_call(i: &str) -> Result<Expr> {
    alt((
        map(call, |(name, args)| Expr::Call(name, args, vec![])),
        expr_parens,
    ))(i)
}*/

fn expr_call(i: Span) -> Result<Arc<Node>> {
    alt((
        map(
            pos(tuple((
                ident,
                delimited(
                    tws(tag("(")),
                    cut(map_res(
                        terminated(
                            separated_list0(
                                tws(tag(",")),
                                alt((
                                    map(
                                        pair(tws(ident), preceded(tws(char('=')), expr)),
                                        |(name, value)| (Some(name.to_string()), value),
                                    ),
                                    map(expr, |expr| (None, expr)),
                                )),
                            ),
                            opt(tws(tag(","))),
                        ),
                        |args| {
                            let mut named: HashSet<&str> = HashSet::new();

                            for a in &args {
                                match a {
                                    (None, _) => {
                                        if !named.is_empty() {
                                            return Err(String::from("positional arguments must come before named arguments"));
                                        }
                                    },
                                    (Some(name), _) => {
                                        if !named.insert(name.as_str()) {
                                            return Err(format!(
                                                "duplicate named argument: {}",
                                                name
                                            ));
                                        }
                                    }
                                }
                            }

                            Ok(args)
                        }
                    )),
                    tws(tag(")")),
                ),
                alt((map(expr_call, |child| vec![child]), block, success(vec![]))),
            ))),
            |(pos, (name, args, body))| {
                node(
                    pos,
                    Expr::Call(CallExpr {
                        name: name.to_string(),
                        body,
                        args,
                    }),
                )
            },
        ),
        expr_parens,
    ))(i)
}

fn binop<'a>(
    i: Span<'a>,
    mut next: impl FnMut(Span<'a>) -> Result<Arc<Node>>,
    parse_op: impl FnMut(Span<'a>) -> Result<Span<'a>>,
) -> Result<Arc<Node>> {
    let (i, init) = next(i)?;

    fold_many0(
        pair(tws(pos(parse_op)), next),
        move || init.clone(),
        |prev, ((pos, op), expr)| {
            node(
                pos,
                /*Expr::BinOp(BinOpExpr {
                    left: prev,
                    op: op.to_string(),
                    right: expr,
                }),*/
                Expr::Call(CallExpr {
                    name: op.to_string(),
                    args: vec![(None, prev), (None, expr)],
                    body: vec![],
                }),
            )
        },
    )(i)
}

fn expr_mul_div(i: Span) -> Result<Arc<Node>> {
    binop(i, expr_call, alt((tag("*"), tag("/"), tag("%"))))
}

fn expr_add_sub(i: Span) -> Result<Arc<Node>> {
    binop(i, expr_mul_div, alt((tag("+"), tag("-"))))
}

fn expr(i: Span) -> Result<Arc<Node>> {
    tws(expr_add_sub)(i)
}

fn block_body(i: Span) -> Result<Vec<Arc<Node>>> {
    map(
        pair(
            many0(
                alt((
                    terminated(
                        alt((
                            map(
                                pos(pair(tws(ident), preceded(tws(char('=')), cut(expr)))),
                                |(pos, (name, value))| {
                                    node(
                                        pos,
                                        Expr::Let(LetExpr {
                                            name: name.to_string(),
                                            value,
                                            body: vec![],
                                        }),
                                    )
                                },
                            ),
                            expr,
                        )),
                        many1(tws(tag(";")))),
                    expr_call
                ))
            ),
            opt(expr),
        ),
        |(mut nodes, ret): (Vec<Arc<Node>>, Option<Arc<Node>>)| {
            if let Some(r) = ret {
                nodes.push(node(r.pos.clone(), Expr::Return(r)));
            }

            let mut result = vec![];
            for n in nodes.into_iter().rev() {
                if let Node {
                    pos,
                    expr: Expr::Let(LetExpr { name, value, .. }),
                } = &*n
                {
                    let body = std::mem::take(&mut result);
                    result.push(node(
                        pos.clone(),
                        Expr::Let(LetExpr {
                            name: name.clone(),
                            value: value.clone(),
                            body,
                        }),
                    ));
                } else {
                    result.push(n);
                }
            }

            result
        },
    )(i)
}

fn block(i: Span) -> Result<Vec<Arc<Node>>> {
    delimited(tws(tag("{")), block_body, tws(tag("}")))(i)
}

pub fn parse_source(i: &str) -> Result<Vec<Arc<Node>>> {
    all_consuming(preceded(ws_or_comment, cut(block_body)))(Span::new(i))
}

#[cfg(test)]
mod test {
    //use super::*;

    /*fn check(code: &str, result: Vec<Arc<Node>>) {
        match parse_source(code) {
            Ok((_, out)) => {
                println!("{:#?}", out);
                assert_eq!(out, result);
            }
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                //panic!("{}", convert_error(code, e));
            }
            Err(nom::Err::Incomplete(_)) => panic!("Incomplete"),
        }
    }*/

    #[test]
    fn test_parse_source() {
        //check("", vec![]);

        //panic!("{:#?}", parse_source("x = 1; b = 2; a(2, 3, a=2); 2+ 1"));

        /*check(
            "x = 1; b = 2; a(x + 1) { x }",
            vec![Expr::Let(
                String::from("x"),
                Box::new(Expr::Num(1.0)),
                vec![],
            )],
        );

        check(
            "x = 1; /* hi */
        a(a=x, y * 2 + 3) { 1 + 2 } a(a()) b();",
            vec![],
        );*/
    }
}
