// Copyright 2024-2025 David Drysdale

//! Move parsing.

use crate::{
    moves::{self, PseudoMoveId, SkatingMoveId},
    parser::timing::{parse_count, parse_duration},
    parser::{self, InnErr},
    JumpCount, Move, TextPosition, TimedMove,
};
use log::info;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, map_res, opt, value},
    error,
    sequence::{preceded, tuple},
    IResult, Parser,
};

#[cfg(test)]
mod tests;

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

fn parse_jump_id(input: &str) -> IResult<&str, SkatingMoveId> {
    // '-' [1234] {S,T,Lo,F,Lz,A}
    let (rest, _) = tag("-")(input)?;
    let (rest, count) = alt((
        value(JumpCount::Single, tag("1")),
        value(JumpCount::Double, tag("2")),
        value(JumpCount::Triple, tag("3")),
        value(JumpCount::Quad, tag("4")),
    ))(rest)?;
    alt((
        value(
            SkatingMoveId::Salchow(count),
            tag(moves::jump::Salchow::JUMP),
        ),
        value(
            SkatingMoveId::ToeLoop(count),
            tag(moves::jump::ToeLoop::JUMP),
        ),
        value(SkatingMoveId::LoopJump(count), tag(moves::jump::Loop::JUMP)),
        value(SkatingMoveId::Flip(count), tag(moves::jump::Flip::JUMP)),
        value(SkatingMoveId::Axel(count), tag(moves::jump::Axel::JUMP)),
    ))(rest)
}

fn parse_skating_move_id(edge: crate::Edge, input: &str) -> IResult<&str, SkatingMoveId> {
    if edge == crate::Edge::Flat {
        alt((
            value(SkatingMoveId::Hop, tag(moves::hop::Hop::MOVE)),
            value(SkatingMoveId::StraightEdge, tag("")),
        ))(input)
    } else {
        alt((
            value(SkatingMoveId::ThreeTurn, tag(moves::three::ThreeTurn::MOVE)),
            value(
                SkatingMoveId::OpenMohawk,
                tag(moves::mohawk::OpenMohawk::MOVE),
            ),
            value(
                SkatingMoveId::ClosedMohawk,
                tag(moves::mohawk::ClosedMohawk::MOVE),
            ),
            value(SkatingMoveId::Bracket, tag(moves::bracket::Bracket::MOVE)),
            value(SkatingMoveId::Rocker, tag(moves::rocker::Rocker::MOVE)),
            value(SkatingMoveId::Counter, tag(moves::counter::Counter::MOVE)),
            value(
                SkatingMoveId::OpenChoctaw,
                tag(moves::choctaw::OpenChoctaw::MOVE),
            ),
            value(
                SkatingMoveId::ClosedChoctaw,
                tag(moves::choctaw::ClosedChoctaw::MOVE),
            ),
            value(
                SkatingMoveId::ChangeOfEdge,
                tag(moves::coe::ChangeOfEdge::MOVE),
            ),
            value(
                SkatingMoveId::ChangeOfEdge,
                tag(moves::coe::ChangeOfEdge::MOVE_ALT),
            ),
            parse_twizzle_id,
            parse_jump_id,
            value(SkatingMoveId::Loop, tag(moves::loopfig::Loop::MOVE)),
            value(SkatingMoveId::Hop, tag(moves::loopfig::Loop::MOVE)),
            // Match an empty string last.
            value(SkatingMoveId::Curve, tag("")),
        ))(input)
    }
}

/// Parse a skating move.
fn parse_skating_move<'a>(start: &'a str, input: &'a str) -> IResult<&'a str, Box<dyn Move>> {
    let (rest, _) = space0(input)?;
    let cur = rest;
    let (rest, pre_transition) = parser::types::parse_pre_transition(rest)?;
    let (rest, code) = parser::types::parse_code(rest)?;
    let (rest, move_id) = parse_skating_move_id(code.edge, rest)?;
    let info = move_id.info();
    let (rest, (plus_minus, more_less, vals)) = parser::params::parse(rest)?;
    let params = crate::params::populate_from(info.params, input, plus_minus, more_less, vals)
        .map_err(|_e| fail(input))?;
    let text_pos = TextPosition::new(start, cur, rest);
    info!("found {move_id:?} at {text_pos:?}");
    Ok((
        rest,
        move_id
            .construct(input, text_pos, pre_transition, code, params)
            .map_err(|_e| fail(input))?,
    ))
}

/// Parse a skating move with optional timing info beforehand.
fn parse_timed_skating_move<'a>(start: &'a str, input: &'a str) -> IResult<&'a str, TimedMove> {
    map_res(
        tuple((
            space0,
            opt(parse_count),
            space0,
            opt(parse_duration),
            |input| parse_skating_move(start, input),
        )),
        |(_, count, _, duration, mv)| {
            Ok::<_, InnErr>(TimedMove {
                count,
                duration,
                mv,
            })
        },
    )
    .parse(input)
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
pub(crate) fn parse_pseudo_move<'a>(start: &'a str, input: &'a str) -> IResult<&'a str, TimedMove> {
    let (rest, _) = space0(input)?;
    let cur = rest;
    let (rest, move_id) = parse_pseudo_move_id(rest)?;
    let info = move_id.info();
    let (rest, (plus_minus, more_less, vals)) = parser::params::parse(rest)?;
    let params = crate::params::populate_from(info.params, input, plus_minus, more_less, vals)
        .map_err(|_e| fail(input))?;
    let text_pos = TextPosition::new(start, cur, rest);
    info!("found {move_id:?} at {text_pos:?}");
    Ok((
        rest,
        move_id
            .construct(input, text_pos, params)
            .map_err(|_e| fail(input))?
            .into(),
    ))
}

/// Parse a move.
pub(crate) fn parse_move<'a>(start: &'a str, input: &'a str) -> IResult<&'a str, TimedMove> {
    alt((
        |input| parse_timed_skating_move(start, input),
        |input| parse_pseudo_move(start, input),
    ))(input)
}

fn fail(input: &str) -> nom::Err<error::Error<&str>> {
    nom::Err::Failure(error::Error::new(input, error::ErrorKind::Fail))
}
