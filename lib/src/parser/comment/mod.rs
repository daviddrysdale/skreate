//! Comment parser.

use nom::{
    bytes::complete::is_not, character::complete::char, combinator::map, sequence::pair, IResult,
};

#[cfg(test)]
mod tests;

/// Parse an end-of-line comment marked with '#'.
pub fn parse(input: &str) -> IResult<&str, &str> {
    map(
        // Start from '#' to end-of-line
        pair(char('#'), is_not("\n\r")),
        // Only interested in the non-hash result
        |(_hash, comment)| comment,
    )(input)
}
