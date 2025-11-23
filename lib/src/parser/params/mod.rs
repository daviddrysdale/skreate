// Copyright 2024-2025 David Drysdale

//! Parameter parsing.

use crate::params::{DetentLevel, MoveParamRef, Value};
use crate::parser::{self, parse_i32};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, one_of, space0},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::borrow::Cow;

#[cfg(test)]
mod tests;

fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(true, tag("y")),
        value(true, tag("Y")),
        value(false, tag("false")),
        value(false, tag("n")),
        value(false, tag("N")),
    ))(input)
}

/// Parse a [`Value`] from text.
pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        |input| parser::string::parse(input).map(|(rest, v)| (rest, Value::Text(Cow::Owned(v)))),
        |input| parse_bool(input).map(|(rest, v)| (rest, Value::Boolean(v))),
        |input| parse_i32(input).map(|(rest, v)| (rest, Value::Number(v))),
    ))(input)
}

/// Parse a parameter name from test.
fn parse_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        // Starts with a letter
        alpha1,
        // Followed by zero or more letters, numbers, underscores or dashes.
        many0(alt((alphanumeric1, tag("_"), tag("-")))),
    ))
    .parse(input)
}

/// Parse a "name=value" parameter.
fn parse_name_value(input: &str) -> IResult<&str, MoveParamRef<'_>> {
    map(
        tuple((
            parse_name,
            space0,
            value((), char('=')),
            space0,
            parse_value,
        )),
        // We're only interested in the name and value, not the equals sign nor any whitespace.
        |(name, _, _, _, value)| MoveParamRef { name, value },
    )
    .parse(input)
}

/// Parse explicit "[name1=val1, name2=val2]" parameters.
fn parse_name_values(input: &str) -> IResult<&str, Vec<MoveParamRef<'_>>> {
    delimited(
        value((), terminated(tag("["), space0)),
        separated_list0(
            // Separator is a comma, optionally with whitespace
            tuple((space0, tag(","), space0)),
            // Entries are name=value
            parse_name_value,
        ),
        value((), preceded(space0, tag("]"))),
    )(input)
}

fn parse_plus_minus(input: &str) -> IResult<&str, Option<DetentLevel>> {
    // Larger strings first to prevent early exit.
    opt(alt((
        value(DetentLevel::Raise3, tag("+++")),
        value(DetentLevel::Raise2, tag("++")),
        value(DetentLevel::Raise1, tag("+")),
        value(DetentLevel::Lower3, tag("---")),
        value(DetentLevel::Lower2, tag("--")),
        value(DetentLevel::Lower1, tag("-")),
    )))
    .parse(input)
}

fn parse_more_less(input: &str) -> IResult<&str, Option<DetentLevel>> {
    // Larger strings first to prevent early exit.
    opt(alt((
        value(DetentLevel::Raise3, tag(">>>")),
        value(DetentLevel::Raise2, tag(">>")),
        value(DetentLevel::Raise1, tag(">")),
        value(DetentLevel::Lower3, tag("<<<")),
        value(DetentLevel::Lower2, tag("<<")),
        value(DetentLevel::Lower1, tag("<")),
    )))
    .parse(input)
}

/// Parse short codes.  Return code is (plus_minus, more_less).
fn parse_short_codes(input: &str) -> IResult<&str, (Option<DetentLevel>, Option<DetentLevel>)> {
    // Can't use `permutation` because the parsers will match the empty string and are applied greedily, so try both
    // combinations manually.
    let (rest1, (_, plus1, more1, _)) =
        tuple((space0, parse_plus_minus, parse_more_less, space0)).parse(input)?;
    let (rest2, (_, more2, plus2, _)) =
        tuple((space0, parse_more_less, parse_plus_minus, space0)).parse(input)?;

    let result = match (&more1, &more2) {
        (None, None) => (rest2, (plus2, more2)),
        (Some(_m1), None) => (rest1, (plus1, more1)),
        (None, Some(_m2)) => (rest2, (plus2, more2)),
        (Some(m1), Some(m2)) => {
            if m2.abs() < m1.abs() {
                (rest1, (plus1, more1))
            } else {
                (rest2, (plus2, more2))
            }
        }
    };
    Ok(result)
}

/// Parse a parameter specification.  Returns (plus_minus, more_less, params).
#[allow(clippy::type_complexity)]
pub fn parse(
    input: &str,
) -> IResult<
    &str,
    (
        Option<DetentLevel>,
        Option<DetentLevel>,
        Vec<MoveParamRef<'_>>,
    ),
> {
    map(
        tuple((parse_short_codes, space0, opt(parse_name_values))),
        |((plus_minus, more_less), _, params)| (plus_minus, more_less, params.unwrap_or_default()),
    )
    .parse(input)
}

/// Parse a turn count. Returns the number of half-turns.
pub fn parse_turn_count(input: &str) -> IResult<&str, u32> {
    map(
        tuple((one_of("123456789"), opt(preceded(tag("."), tag("5"))))),
        |(n, half)| 2 * n.to_digit(10).unwrap() + if half.is_some() { 1 } else { 0 },
    )
    .parse(input)
}
