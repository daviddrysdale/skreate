//! Comment parser.

use nom::{
    bytes::complete::is_not, character::complete::char, combinator::value, sequence::pair, IResult,
};

#[cfg(test)]
mod tests;

/// Parse an end-of-line comment marked with '#'.
pub fn parse(input: &str) -> IResult<&str, ()> {
    value(
        (), // Output dropped
        pair(char('#'), is_not("\n\r")),
    )(input)
}
