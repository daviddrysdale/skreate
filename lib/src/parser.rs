//! Parser.

use crate::Move;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    combinator::map,
    multi::{many1, separated_list0},
    sequence::tuple,
    IResult,
};

pub mod comment;
pub mod mv;
pub mod params;
pub mod string;
pub mod types;

pub(crate) fn parse(input: &str) -> IResult<&str, Vec<Box<dyn Move>>> {
    separated_list0(
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
        ))),
        mv::parse_move,
    )(input)
}
