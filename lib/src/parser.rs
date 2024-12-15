//! Parser.

use nom::{
    bytes::complete::is_not, character::complete::char, combinator::value, sequence::pair, IResult,
};

/// Parse an end-of-line comment marked with '#'.
pub fn parse_comment(input: &str) -> IResult<&str, ()> {
    value(
        (), // Output dropped
        pair(char('#'), is_not("\n\r")),
    )(input)
}
