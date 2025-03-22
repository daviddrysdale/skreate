// Copyright 2024-2025 David Drysdale

//! Parsing for timing info.

use crate::parser::{parse_i32, InnErr};
use crate::{Count, Duration};
use nom::{
    character::complete::char,
    combinator::{map_res, value},
    sequence::tuple,
    IResult, Parser,
};

#[cfg(test)]
mod tests;

/// Parse a [`Count`] from text.
pub fn parse_count(input: &str) -> IResult<&str, Count> {
    map_res(
        tuple((parse_i32, value((), char(')')))),
        // Convert to `Count` on the way out
        |(n, _)| Ok::<_, InnErr>(Count(n)),
    )
    .parse(input)
}

/// Parse a [`Duration`] from text.
pub fn parse_duration(input: &str) -> IResult<&str, Duration> {
    map_res(
        tuple((value((), char('/')), parse_i32)),
        // Convert to `Duration` on the way out
        |(_, n)| Ok::<_, InnErr>(Duration(n)),
    )
    .parse(input)
}
