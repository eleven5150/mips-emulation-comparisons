use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;
use nom_locate::position;
use serde::{Deserialize, Serialize};

use crate::misc::{comment_multispace0, parse_ident};
use crate::number::{parse_char, parse_u32};
use crate::parser::Position;
use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct MpConst {
    label: String,
    value: MpConstValueLoc,
    line: u32,
    col: u32,
    line_end: u32,
    col_end: u32,
}

impl MpConst {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> &MpConstValueLoc {
        &self.value
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn line_end(&self) -> u32 {
        self.line_end
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MpConstValueLoc(pub MpConstValue, pub Position);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MpConstValue {
    Value(u64),
    Const(String),
    Minus(Box<MpConstValueLoc>),
    Mult(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Sum(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Sub(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Div(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Mod(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    And(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Or(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Xor(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Neg(Box<MpConstValueLoc>),
    Shl(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Shr(Box<MpConstValueLoc>, Box<MpConstValueLoc>),
}

impl Display for MpConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpConstValue::Value(x) => write!(f, "{}", x),
            MpConstValue::Const(x) => write!(f, "{}", x),
            MpConstValue::Minus(x) => write!(f, "-{}", x.0),
            MpConstValue::Mult(x, y) => write!(f, "{} * {}", x.0, y.0),
            MpConstValue::Sum(x, y) => write!(f, "{} + {}", x.0, y.0),
            MpConstValue::Sub(x, y) => write!(f, "{} - {}", x.0, y.0),
            MpConstValue::Div(x, y) => write!(f, "{} / {}", x.0, y.0),
            MpConstValue::Mod(x, y) => write!(f, "{} % {}", x.0, y.0),
            MpConstValue::And(x, y) => write!(f, "{} & {}", x.0, y.0),
            MpConstValue::Or(x, y) => write!(f, "{} | {}", x.0, y.0),
            MpConstValue::Xor(x, y) => write!(f, "{} ^ {}", x.0, y.0),
            MpConstValue::Neg(x) => write!(f, "~{}", x.0),
            MpConstValue::Shl(x, y) => write!(f, "{} << {}", x.0, y.0),
            MpConstValue::Shr(x, y) => write!(f, "{} >> {}", x.0, y.0),
        }
    }
}

pub fn parse_constant_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    parse_or_value(i)
}

pub fn parse_or_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_xor_value,
                many0(map(
                    tuple((
                        comment_multispace0,
                        char('|'),
                        comment_multispace0,
                        parse_xor_value,
                    )),
                    |(_, _, _, value)| value,
                )),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    MpConstValueLoc(
                        MpConstValue::Or(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    )
                })
            },
        ),
        parse_xor_value,
    ))(i)
}

pub fn parse_xor_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_and_value,
                many0(map(
                    tuple((
                        comment_multispace0,
                        char('^'),
                        comment_multispace0,
                        parse_and_value,
                    )),
                    |(_, _, _, value)| value,
                )),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    MpConstValueLoc(
                        MpConstValue::Xor(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    )
                })
            },
        ),
        parse_and_value,
    ))(i)
}

pub fn parse_and_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_shift_value,
                many0(map(
                    tuple((
                        comment_multispace0,
                        char('&'),
                        comment_multispace0,
                        parse_shift_value,
                    )),
                    |(_, _, _, value)| value,
                )),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    MpConstValueLoc(
                        MpConstValue::And(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    )
                })
            },
        ),
        parse_shift_value,
    ))(i)
}

pub fn parse_shift_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_add_sub_value,
            many0(map(
                tuple((
                    comment_multispace0,
                    alt((tag("<<"), tag(">>"))),
                    comment_multispace0,
                    parse_add_sub_value,
                )),
                |(_, tag, _, value)| (tag, value),
            )),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values
                .into_iter()
                .fold(value, |acc, (tag, value)| match *tag.fragment() {
                    b"<<" => MpConstValueLoc(
                        MpConstValue::Shl(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    b">>" => MpConstValueLoc(
                        MpConstValue::Shr(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    _ => unreachable!(),
                })
        },
    )(i)
}

pub fn parse_add_sub_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_mul_div_mod_value,
            many0(map(
                tuple((
                    comment_multispace0,
                    alt((char('+'), char('-'))),
                    comment_multispace0,
                    parse_mul_div_mod_value,
                )),
                |(_, tag, _, value)| (tag, value),
            )),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values
                .into_iter()
                .fold(value, |acc, (tag, value)| match tag {
                    '+' => MpConstValueLoc(
                        MpConstValue::Sum(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    '-' => MpConstValueLoc(
                        MpConstValue::Sub(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    _ => unreachable!(),
                })
        },
    )(i)
}

pub fn parse_mul_div_mod_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_unary_op_value,
            many0(map(
                tuple((
                    comment_multispace0,
                    alt((char('*'), char('/'), char('%'))),
                    comment_multispace0,
                    parse_unary_op_value,
                )),
                |(_, tag, _, value)| (tag, value),
            )),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values
                .into_iter()
                .fold(value, |acc, (tag, value)| match tag {
                    '*' => MpConstValueLoc(
                        MpConstValue::Mult(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    '/' => MpConstValueLoc(
                        MpConstValue::Div(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    '%' => MpConstValueLoc(
                        MpConstValue::Mod(Box::new(acc), Box::new(value)),
                        Position::from_positions(pos_start, pos_end),
                    ),
                    _ => unreachable!(),
                })
        },
    )(i)
}

pub fn parse_unary_op_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        parse_plus_value,
        parse_minus_value,
        parse_neg_value,
        parse_value,
    ))(i)
}

pub fn parse_plus_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((char('+'), comment_multispace0, parse_unary_op_value)),
        |(_, _, value)| value,
    )(i)
}

pub fn parse_minus_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            char('-'),
            comment_multispace0,
            parse_unary_op_value,
            position,
        )),
        |(pos_start, _, _, value, pos_end)| {
            MpConstValueLoc(
                MpConstValue::Minus(Box::new(value)),
                Position::from_positions(pos_start, pos_end),
            )
        },
    )(i)
}

pub fn parse_neg_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            char('~'),
            comment_multispace0,
            parse_unary_op_value,
            position,
        )),
        |(pos_start, _, _, value, pos_end)| {
            MpConstValueLoc(
                MpConstValue::Neg(Box::new(value)),
                Position::from_positions(pos_start, pos_end),
            )
        },
    )(i)
}

pub fn parse_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((position, parse_u32, position)),
            |(pos_start, value, pos_end)| {
                MpConstValueLoc(
                    MpConstValue::Value(value as u64),
                    Position::from_positions(pos_start, pos_end),
                )
            },
        ),
        map(
            tuple((position, parse_char, position)),
            |(pos_start, value, pos_end)| {
                MpConstValueLoc(
                    MpConstValue::Value(value as u64),
                    Position::from_positions(pos_start, pos_end),
                )
            },
        ),
        map(
            tuple((position, parse_ident, position)),
            |(pos_start, value, pos_end)| {
                MpConstValueLoc(
                    MpConstValue::Const(value),
                    Position::from_positions(pos_start, pos_end),
                )
            },
        ),
        map(
            tuple((
                char('('),
                comment_multispace0,
                parse_constant_value,
                comment_multispace0,
                char(')'),
            )),
            |(_, _, value, _, _)| value,
        ),
    ))(i)
}
