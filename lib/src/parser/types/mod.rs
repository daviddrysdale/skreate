// Copyright 2024-2025 David Drysdale

//! Parsers for base types.

use crate::{Code, Edge, Foot, PreTransition, SkatingDirection};
use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

#[cfg(test)]
mod tests;

/// Parse input as [`Foot`].
pub fn parse_foot(input: &str) -> IResult<&str, Foot> {
    alt((
        value(Foot::Left, tag("L")),
        value(Foot::Right, tag("R")),
        value(Foot::Both, tag("B")),
    ))(input)
}

/// Parse input as [`SkatingDirection`].
pub fn parse_direction(input: &str) -> IResult<&str, SkatingDirection> {
    alt((
        value(SkatingDirection::Forward, tag("F")),
        value(SkatingDirection::Backward, tag("B")),
    ))(input)
}

/// Parse input for single-foot move as [`Edge`].
pub fn parse_edge(input: &str) -> IResult<&str, Edge> {
    alt((
        value(Edge::Outside, tag("O")),
        value(Edge::Inside, tag("I")),
        // no parse for Self::Flat
    ))(input)
}

/// Parse input for double-foot forward edge as [`Edge`]. By convention, the edge of the right foot is returned.
pub fn parse_bf_edge(input: &str) -> IResult<&str, Edge> {
    alt((
        value(Edge::Outside, tag("R")),
        value(Edge::Inside, tag("L")),
        // no parse for Self::Flat
    ))(input)
}

/// Parse input for double-foot backward edge as [`Edge`]. By convention, the edge of the right foot is returned.
pub fn parse_bb_edge(input: &str) -> IResult<&str, Edge> {
    alt((
        value(Edge::Outside, tag("L")),
        value(Edge::Inside, tag("R")),
        // no parse for Self::Flat
    ))(input)
}

/// Parse input as [`Code`].
pub fn parse_code(input: &str) -> IResult<&str, Code> {
    let (mut rest, (foot, dir)) = nom::sequence::pair(parse_foot, parse_direction)(input)?;

    // The edge codes for double-footed edges ("L", "R") are different from those for single-footed edges ("I", "O").
    let edge_parser = match (foot, dir) {
        (Foot::Both, SkatingDirection::Forward) => parse_bf_edge,
        (Foot::Both, SkatingDirection::Backward) => parse_bb_edge,
        _ => parse_edge,
    };

    let edge = if let Ok((more, edge)) = edge_parser(rest) {
        rest = more;
        edge
    } else {
        Edge::Flat
    };
    Ok((rest, Code { foot, dir, edge }))
}

/// Parse a possible transition prefix.
pub fn parse_pre_transition(input: &str) -> IResult<&str, PreTransition> {
    alt((
        value(PreTransition::CrossFront, tag("xf-")),
        value(PreTransition::CrossBehind, tag("xb-")),
        value(PreTransition::Wide, tag("wd-")),
        value(PreTransition::Normal, tag("")),
    ))(input)
}
