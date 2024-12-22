//! Move parsing.

use crate::{
    moves::{self, PseudoMoveId, SkatingMoveId},
    parser, Move,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, value},
    error,
    sequence::preceded,
    IResult, Parser,
};

fn parse_twizzle_id(input: &str) -> IResult<&str, SkatingMoveId> {
    map(
        preceded(
            tag(moves::twizzle::Twizzle::MOVE),
            parser::params::parse_turn_count,
        ),
        SkatingMoveId::Twizzle,
    )
    .parse(input)
}

fn parse_skating_move_id(input: &str) -> IResult<&str, SkatingMoveId> {
    alt((
        value(SkatingMoveId::ThreeTurn, tag(moves::three::ThreeTurn::MOVE)),
        value(
            SkatingMoveId::OpenMohawk,
            tag(moves::mohawk::OpenMohawk::MOVE),
        ),
        value(SkatingMoveId::Bracket, tag(moves::bracket::Bracket::MOVE)),
        value(SkatingMoveId::Rocker, tag(moves::rocker::Rocker::MOVE)),
        value(SkatingMoveId::Counter, tag(moves::counter::Counter::MOVE)),
        value(
            SkatingMoveId::ChangeOfEdge,
            tag(moves::coe::ChangeOfEdge::MOVE),
        ),
        value(
            SkatingMoveId::ChangeOfEdge,
            tag(moves::coe::ChangeOfEdge::MOVE_ALT),
        ),
        parse_twizzle_id,
        // Match an empty string last.
        value(SkatingMoveId::Curve, tag("")),
    ))(input)
}

/// Parse a skating move.
pub(crate) fn parse_skating_move(input: &str) -> IResult<&str, Box<dyn Move>> {
    let (rest, _) = space0(input)?;
    let (rest, pre_transition) = parser::types::parse_pre_transition(rest)?;
    let (rest, code) = parser::types::parse_code(rest)?;
    let (rest, move_id) = if code.edge == crate::Edge::Flat {
        (rest, SkatingMoveId::StraightEdge)
    } else {
        parse_skating_move_id(rest)?
    };
    let info = move_id.info();
    let (rest, (plus_minus, more_less, vals)) = parser::params::parse(rest)?;
    let params = crate::params::populate_from(info.params, plus_minus, more_less, vals)
        .map_err(|_e| fail(input))?;
    Ok((
        rest,
        move_id
            .construct(&input.into(), pre_transition, code, params)
            .map_err(|_e| fail(input))?,
    ))
}

fn parse_pseudo_move_id(input: &str) -> IResult<&str, PseudoMoveId> {
    alt((
        value(PseudoMoveId::Warp, tag(moves::warp::Warp::MOVE)),
        value(PseudoMoveId::Shift, tag(moves::shift::Shift::MOVE)),
        value(PseudoMoveId::Rink, tag(moves::rink::Rink::MOVE)),
        value(PseudoMoveId::Info, tag(moves::info::Info::MOVE)),
        value(PseudoMoveId::Title, tag(moves::title::Title::MOVE)),
        value(PseudoMoveId::Text, tag(moves::text::Text::MOVE)),
        value(PseudoMoveId::Label, tag(moves::label::Label::MOVE)),
    ))(input)
}

/// Parse a pseudo-move.
pub(crate) fn parse_pseudo_move(input: &str) -> IResult<&str, Box<dyn Move>> {
    let (rest, _) = space0(input)?;
    let (rest, move_id) = parse_pseudo_move_id(rest)?;
    let info = move_id.info();
    let (rest, (plus_minus, more_less, vals)) = parser::params::parse(rest)?;
    let params = crate::params::populate_from(info.params, plus_minus, more_less, vals)
        .map_err(|_e| fail(input))?;
    Ok((
        rest,
        move_id
            .construct(&input.into(), params)
            .map_err(|_e| fail(input))?,
    ))
}

/// Parse a move.
pub(crate) fn parse_move(input: &str) -> IResult<&str, Box<dyn Move>> {
    alt((parse_skating_move, parse_pseudo_move))(input)
}

fn fail(input: &str) -> nom::Err<error::Error<&str>> {
    nom::Err::Failure(error::Error::new(input, error::ErrorKind::Fail))
}
