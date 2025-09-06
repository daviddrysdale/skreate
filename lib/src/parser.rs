// Copyright 2024-2025 David Drysdale

//! Parser.

use crate::{ParseError, TextPosition};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, one_of, space0},
    combinator::{map, map_res, opt, recognize},
    multi::{many1, separated_list0},
    sequence::tuple,
    IResult, Parser,
};

pub mod comment;
pub mod mv;
pub mod params;
pub mod string;
pub mod timing;
pub mod types;

/// Parsing error.
pub(crate) type InnErr<'a> = nom::error::Error<&'a str>;

/// Parsing error.
pub(crate) type Error<'a> = nom::Err<InnErr<'a>>;

fn parse_i32(input: &str) -> IResult<&str, i32> {
    map_res(
        recognize(tuple((
            // May start with + or -
            opt(one_of("-+")),
            // Followed by at least one digit
            many1(one_of("0123456789")),
        ))),
        // Convert `&str` to `i32` on the way out.
        |out: &str| out.parse::<i32>(),
    )
    .parse(input)
}

fn parse_separator(input: &str) -> IResult<&str, Vec<&str>> {
    // Separate moves by...
    many1(alt((
        // Whitespace including at least one newline.
        map(
            tuple((space0, nom::bytes::complete::is_a("\n\r"), multispace0)),
            |(_sp0, eol, _sp1)| eol,
        ),
        // Semi-colon
        tag(";"),
        // Comment to newline (inclusive).
        comment::parse,
    )))(input)
}

pub(crate) fn parse(start: &str) -> IResult<&str, Vec<mv::TimedInputs>> {
    // Allow separators before the move text
    let (rest, (_, moves)) = tuple((
        opt(parse_separator),
        separated_list0(parse_separator, |input| mv::parse_move(start, input)),
    ))(start)?;
    // The `separated_list0` combinator will leave a final separator in place if the thing after it doesn't parse as a
    // move, so consume that (and any leading whitespace) too.
    let rest = rest.trim_start();
    let result = parse_separator(rest);
    if let Ok((rest, _)) = result {
        Ok((rest, moves))
    } else {
        Ok((rest, moves))
    }
}

/// Convert a nom error into a [`ParseError`].
pub(crate) fn err(err: nom::Err<nom::error::Error<&str>>, input: &str) -> ParseError {
    ParseError {
        pos: match &err {
            nom::Err::Incomplete(_) => TextPosition::default(),
            nom::Err::Error(e) | nom::Err::Failure(e) => TextPosition::new(input, e.input, e.input),
        },
        msg: format!("{err:?}"),
    }
}

/// Parser wrapper to help in debugging, for when the output implements `Debug`.
#[allow(dead_code)]
fn dbg<'a, O, E, F>(tag: &'static str, mut f: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: nom::Parser<&'a str, O, E>,
    E: nom::error::ParseError<&'a str>,
    O: std::fmt::Debug,
{
    move |i: &'a str| {
        log::warn!("[{tag}] attempt to parse '{}'", starts(i));
        let result = f.parse(i);
        match &result {
            Ok((rest, output)) => {
                let start = i.as_ptr() as usize;
                let end = rest.as_ptr() as usize;
                log::warn!(
                    "[{tag}] consumed {} bytes to produce {output:?}, now at '{}'",
                    end - start,
                    starts(rest)
                );
            }
            Err(_e) => {
                log::debug!("[{tag}] parser rejected input from '{}'", starts(i));
            }
        }
        result
    }
}

/// Parser wrapper to help in debugging, for when the output does not implement `Debug`.
#[allow(dead_code)]
fn dbg_raw<'a, O, E, F>(
    tag: &'static str,
    mut f: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: nom::Parser<&'a str, O, E>,
    E: nom::error::ParseError<&'a str>,
{
    move |i: &'a str| {
        log::warn!("[{tag}]: attempt to parse '{}'", starts(i));
        let result = f.parse(i);
        match &result {
            Ok((rest, _output)) => {
                let start = i.as_ptr() as usize;
                let end = rest.as_ptr() as usize;
                log::warn!(
                    "[{tag}] consumed {} bytes , now at '{}'",
                    end - start,
                    starts(rest)
                );
            }
            Err(_e) => {
                log::debug!("[{tag}] parser rejected input from '{}'", starts(i));
            }
        }
        result
    }
}

#[allow(dead_code)]
fn starts(text: &str) -> String {
    let len = std::cmp::min(4, text.len());
    if len < text.len() {
        format!("{}...", &text[..len])
    } else {
        text[..len].to_string()
    }
}
