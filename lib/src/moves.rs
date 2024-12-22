//! Skating move definitions.

use crate::{
    pos, Code, Foot, Input, Move, MoveParam, ParseError, Position, PreTransition, Rotation,
    SkatingDirection::*, SpatialTransition, Transition,
};
use log::{info, warn};
use serde::Serialize;

pub(crate) mod bracket;
pub(crate) mod coe;
pub(crate) mod compound;
pub(crate) mod counter;
pub(crate) mod edge;
pub(crate) mod info;
pub(crate) mod label;
pub(crate) mod mohawk;
pub(crate) mod rink;
pub(crate) mod rocker;
pub(crate) mod shift;
pub(crate) mod straight;
pub(crate) mod text;
pub(crate) mod three;
pub(crate) mod title;
pub(crate) mod twizzle;
pub(crate) mod warp;

#[cfg(test)]
mod tests;

/// Errors arising from attempting to create move instances.
#[derive(Debug, Clone)]
pub(crate) enum Error {
    /// Indicates that the constructor doesn't apply to this input.
    Unrecognized,
    /// Indicates that the constructor does apply to this input, but failed to parse correctly.
    Failed(String),
}

impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(_err: nom::Err<nom::error::Error<&str>>) -> Self {
        // TODO: move location into error
        Error::Unrecognized
    }
}

/// Information about a class of moves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Info {
    /// Name of the move.
    pub name: &'static str,
    /// Summary of the move.
    pub summary: &'static str,
    /// Example input for move.
    pub example: &'static str,
    /// Whether the move is visible.
    pub visible: bool,
    /// Move parameter information.
    pub params: &'static [crate::params::Info],
}

pub(crate) fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    crate::parser::mv::parse_move(input.text)
        .map(|(_rest, mv)| mv)
        .map_err(|_e| ParseError::from_input(input, &format!("unknown move {}", input.text)))
}

// Coordinates:
//
//  0-------------------> x axis
//  |
//  |
//  |            90 <---x Direction
//  |                  /|
//  |                 / |
//  |             45 L  v 0
//  |
//  v  y-axis

/// Macro to build a [`Path`] with a "d" attribute set to the formatted arguments.
#[macro_export]
macro_rules! path {
    { $($arg:tt)+ } => {
        svg::node::element::Path::new().set("d", format!("{}", format_args!($($arg)+)))
    }
}

/// List of static move information.
pub static INFO: &[Info] = &[
    // Insert moves in order of importance, as they will appear in the manual.
    // First skating moves.
    edge::Curve::INFO,
    straight::StraightEdge::INFO,
    three::ThreeTurn::INFO,
    mohawk::OpenMohawk::INFO,
    bracket::Bracket::INFO,
    rocker::Rocker::INFO,
    counter::Counter::INFO,
    coe::ChangeOfEdge::INFO,
    twizzle::Twizzle::INFO,
    // Then pseudo-moves.
    warp::Warp::INFO,
    shift::Shift::INFO,
    rink::Rink::INFO,
    info::Info::INFO,
    title::Title::INFO,
    text::Text::INFO,
    label::Label::INFO,
];

/// Identifier for skating moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkatingMoveId {
    /// Curved edge
    Curve,
    /// Straight Edge
    StraightEdge,
    /// Three Turn
    ThreeTurn,
    /// Open Mohawk
    OpenMohawk,
    /// Bracket
    Bracket,
    /// Rocker
    Rocker,
    /// Counter
    Counter,
    /// Change Of Edge
    ChangeOfEdge,
    /// Twizzle with count of half-turns.
    Twizzle(u32),
}

impl SkatingMoveId {
    /// Return the static move information for the given move.
    pub fn info(&self) -> &'static Info {
        match self {
            Self::Curve => &edge::Curve::INFO,
            Self::StraightEdge => &straight::StraightEdge::INFO,
            Self::ThreeTurn => &three::ThreeTurn::INFO,
            Self::OpenMohawk => &mohawk::OpenMohawk::INFO,
            Self::Bracket => &bracket::Bracket::INFO,
            Self::Rocker => &rocker::Rocker::INFO,
            Self::Counter => &counter::Counter::INFO,
            Self::ChangeOfEdge => &coe::ChangeOfEdge::INFO,
            Self::Twizzle(_count) => &twizzle::Twizzle::INFO,
        }
    }
    /// Construct an instance of a skating move.
    pub(crate) fn construct(
        &self,
        input: &Input,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
    ) -> Result<Box<dyn Move>, Error> {
        Ok(match self {
            Self::Curve => Box::new(edge::Curve::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::StraightEdge => Box::new(straight::StraightEdge::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::ThreeTurn => Box::new(three::ThreeTurn::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::OpenMohawk => Box::new(mohawk::OpenMohawk::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::Bracket => Box::new(bracket::Bracket::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::Rocker => Box::new(rocker::Rocker::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::Counter => Box::new(counter::Counter::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::ChangeOfEdge => Box::new(coe::ChangeOfEdge::from_params(
                input,
                pre_transition,
                entry_code,
                params,
            )?),
            Self::Twizzle(count) => Box::new(twizzle::Twizzle::from_params(
                input,
                pre_transition,
                entry_code,
                *count,
                params,
            )?),
        })
    }
}

/// Identifier for pseudo-moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoMoveId {
    /// Warp
    Warp,
    /// Shift
    Shift,
    /// Rink
    Rink,
    /// Info
    Info,
    /// Title
    Title,
    /// Text
    Text,
    /// Label
    Label,
}

impl PseudoMoveId {
    /// Return the static move information for the given move.
    pub fn info(&self) -> &'static Info {
        match self {
            Self::Warp => &warp::Warp::INFO,
            Self::Shift => &shift::Shift::INFO,
            Self::Rink => &rink::Rink::INFO,
            Self::Info => &info::Info::INFO,
            Self::Title => &title::Title::INFO,
            Self::Text => &text::Text::INFO,
            Self::Label => &label::Label::INFO,
        }
    }

    /// Construct an instance of a pseudo-move.
    pub(crate) fn construct(
        &self,
        input: &Input,
        params: Vec<MoveParam>,
    ) -> Result<Box<dyn Move>, Error> {
        Ok(match self {
            Self::Warp => Box::new(warp::Warp::from_params(input, params)?),
            Self::Shift => Box::new(shift::Shift::from_params(input, params)?),
            Self::Rink => Box::new(rink::Rink::from_params(input, params)?),
            Self::Info => Box::new(info::Info::from_params(input, params)?),
            Self::Title => Box::new(title::Title::from_params(input, params)?),
            Self::Text => Box::new(text::Text::from_params(input, params)?),
            Self::Label => Box::new(label::Label::from_params(input, params)?),
        })
    }
}

/// Move identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveId {
    /// Skating move.
    Skating(SkatingMoveId),
    /// Pseudo-move.
    Pseudo(PseudoMoveId),
}

impl MoveId {
    /// Return the static move information for the given move.
    pub fn info(&self) -> &'static Info {
        match self {
            Self::Skating(id) => id.info(),
            Self::Pseudo(id) => id.info(),
        }
    }
}

/// Half-width of a standard stance.
const HW: i64 = 15; // cm
/// Length of skate.
const SL: i64 = 30; // cm

/// Standard pre-transition with plain step.
pub fn pre_transition(from: Code, to: Code) -> Transition {
    standard_transition(from, to, HW)
}

/// Pre-transition with wide step.
pub fn wide_transition(from: Code, to: Code) -> Transition {
    standard_transition(from, to, 2 * HW)
}

fn standard_transition(from: Code, to: Code, half_width: i64) -> Transition {
    let mut x = 0;
    let mut y = 0;
    let mut rotation = 0;
    match (from.dir, to.dir) {
        (Forward, Forward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Right) => x = -(2 * half_width),
            (Foot::Left, Foot::Both) => x = -half_width,
            (Foot::Both, Foot::Left) => x = half_width,
            (Foot::Both, Foot::Right) => x = -half_width,
            (Foot::Right, Foot::Left) => x = 2 * half_width,
            (Foot::Right, Foot::Both) => x = half_width,
            _ => {}
        },
        (Backward, Backward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Right) => x = 2 * half_width,
            (Foot::Left, Foot::Both) => x = half_width,
            (Foot::Both, Foot::Left) => x = -half_width,
            (Foot::Both, Foot::Right) => x = half_width,
            (Foot::Right, Foot::Left) => x = -(2 * half_width),
            (Foot::Right, Foot::Both) => x = -half_width,
            _ => {}
        },
        (Forward, Backward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Left) => rotation = 180,
            (Foot::Left, Foot::Right) => {
                x = -half_width;
                y = half_width;
                rotation = 90;
            }
            (Foot::Left, Foot::Both) => {
                x = -half_width;
                y = half_width / 2;
                rotation = 90;
            }
            (Foot::Both, Foot::Left) => {
                x = half_width;
                rotation = -90;
            }
            (Foot::Both, Foot::Right) => {
                x = -half_width;
                rotation = 90;
            }
            (Foot::Both, Foot::Both) => rotation = 180,
            (Foot::Right, Foot::Left) => {
                x = half_width;
                y = half_width;
                rotation = -90;
            }
            (Foot::Right, Foot::Right) => rotation = 180,
            (Foot::Right, Foot::Both) => {
                x = half_width;
                y = half_width / 2;
                rotation = -90;
            }
        },
        (Backward, Forward) => {
            match (from.foot, to.foot) {
                (Foot::Left, Foot::Left) => rotation = 180, // reverse direction
                (Foot::Left, Foot::Right) => {
                    x = half_width;
                    y = half_width;
                    rotation = -90;
                }
                (Foot::Left, Foot::Both) => {
                    x = half_width;
                    y = half_width / 2;
                    rotation = 90;
                }
                (Foot::Both, Foot::Left) => {
                    x = -half_width;
                    rotation = 90;
                }
                (Foot::Both, Foot::Right) => {
                    x = half_width;
                    rotation = -90;
                }
                (Foot::Both, Foot::Both) => rotation = 90,
                (Foot::Right, Foot::Left) => {
                    x = -half_width;
                    y = half_width;
                    rotation = 90;
                }
                (Foot::Right, Foot::Right) => rotation = 180, // reverse direction
                (Foot::Right, Foot::Both) => {
                    x = -half_width;
                    y = half_width / 2;
                    rotation = -90;
                }
            }
        }
    }
    Transition {
        spatial: SpatialTransition::Relative {
            delta: pos!(x, y),
            rotate: Rotation(rotation),
        },
        code: Some(to),
    }
}

/// Pre-transition with feet crossing over.
pub fn cross_transition(from: Code, to: Code) -> Transition {
    let mut x = 0;
    let mut y = 0;
    match (from.dir, to.dir) {
        (Forward, Forward) => {
            // Cross in front.
            match (from.foot, to.foot) {
                (Foot::Left, Foot::Right) => {
                    x = HW;
                    y = SL / 2;
                }
                (Foot::Right, Foot::Left) => {
                    x = -HW;
                    y = SL / 2;
                }
                (Foot::Left, Foot::Left) | (Foot::Right, Foot::Right) => {
                    warn!("XF transition but no foot change ({from}->{to})!");
                }
                (Foot::Both, _) => {
                    warn!("XF transition from two feet ({from}->{to})!");
                }
                (Foot::Left, Foot::Both) => {
                    warn!("XF transition to two feet ({from}->{to})!");
                    x = -HW;
                }
                (Foot::Right, Foot::Both) => {
                    warn!("XF transition to two feet ({from}->{to})!");
                    x = HW;
                }
            }
        }
        (Backward, Backward) => {
            // Cross behind.
            match (from.foot, to.foot) {
                (Foot::Left, Foot::Right) => {
                    x = -SL / 2;
                    y = HW
                }
                (Foot::Right, Foot::Left) => {
                    x = SL / 2;
                    y = HW
                }
                (Foot::Left, Foot::Left) | (Foot::Right, Foot::Right) => {
                    warn!("XB transition but no foot change ({from}->{to})!");
                }
                (Foot::Both, _) => {
                    warn!("XB transition from two feet ({from}->{to})!");
                }
                (Foot::Left, Foot::Both) => {
                    warn!("XB transition to two feet ({from}->{to})!");
                    x = HW;
                }
                (Foot::Right, Foot::Both) => {
                    warn!("XB transition to two feet ({from}->{to})!");
                    x = -HW;
                }
            }
        }
        // A cross transition that changes skating direction doesn't make much sense, so fall back to the standard
        // transition here.
        (Forward, Backward) | (Backward, Forward) => return pre_transition(from, to),
    }
    Transition {
        spatial: SpatialTransition::Relative {
            delta: pos!(x, y),
            rotate: Rotation(0),
        },
        code: Some(to),
    }
}
