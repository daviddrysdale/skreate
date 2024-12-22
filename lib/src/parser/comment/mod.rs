//! Comment parser.

use nom::{
    bytes::complete::{is_a, is_not},
    character::complete::char,
    combinator::map,
    sequence::{pair, terminated},
    IResult,
};

#[cfg(test)]
mod tests;

/// Parse an end-of-line comment marked with '#'.
pub fn parse(input: &str) -> IResult<&str, &str> {
    map(
        terminated(
            // Start from '#' to end-of-line
            pair(char('#'), is_not("\n\r")),
            // Also consume and discard the end-of-line
            is_a("\n\r"),
        ),
        // Only interested in the non-hash result
        |(_hash, comment)| {
            log::debug!("found comment '{comment}'");
            comment
        },
    )(input)
}
