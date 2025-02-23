//! Comment parser.

use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not},
    character::complete::char,
    combinator::{all_consuming, map, opt},
    sequence::{pair, terminated},
    IResult,
};

#[cfg(test)]
mod tests;

/// Parse an end-of-line comment marked with '#' to an included newline.
fn parse_to_eol(input: &str) -> IResult<&str, &str> {
    map(
        terminated(
            // Start from '#' to end-of-line
            pair(char('#'), opt(is_not("\n\r"))),
            // Also consume and discard the end-of-line
            is_a("\n\r"),
        ),
        // Only interested in the non-hash result
        |(_hash, comment)| {
            log::debug!("found comment '{comment:?}'");
            comment.unwrap_or_default()
        },
    )(input)
}

/// Parse an end-of-line comment that finishes the input.
fn parse_to_eof(input: &str) -> IResult<&str, &str> {
    all_consuming(map(
        pair(char('#'), opt(is_not("\n\r"))),
        // Only interested in the non-hash result
        |(_hash, comment)| {
            log::debug!("found comment '{comment:?}'");
            comment.unwrap_or_default()
        },
    ))(input)
}

/// Parse an end-of-line comment marked with '#'.
pub fn parse(input: &str) -> IResult<&str, &str> {
    alt((parse_to_eol, parse_to_eof))(input)
}
