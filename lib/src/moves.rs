//! Skating move definitions.

use crate::{
    pos, Code, Foot, Input, Move, ParseError, Position, Rotation, SkatingDirection::*,
    SpatialTransition, Transition,
};
use log::{info, warn};
use serde::Serialize;
use std::sync::OnceLock;

mod bracket;
mod coe;
mod compound;
mod counter;
mod edge;
mod info;
mod label;
mod mohawk;
mod rink;
mod rocker;
mod shift;
mod straight;
mod text;
mod three;
mod title;
mod warp;

/// Errors arising from attempting to create move instances.
#[derive(Debug, Clone)]
enum Error {
    /// Indicates that the constructor doesn't apply to this input.
    Unrecognized,
    /// Indicates that the constructor does apply to this input, but failed to parse correctly.
    Failed(String),
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

    for constructor in constructors() {
        match constructor(input) {
            Ok(mv) => {
                info!("success: canonical form {}", mv.text());
                return Ok(mv);
            }
            Err(Error::Unrecognized) => {}
            Err(Error::Failed(msg)) => {
                warn!("constructor {constructor:?} failed: {msg}",);
                return Err(ParseError {
                    pos: input.pos,
                    msg,
                });
            }
        }
    }

    Err(ParseError::from_input(
        input,
        &format!("unknown move {}", input.text),
    ))
}

/// Macro to build a [`Path`] with a "d" attribute set to the formatted arguments.
#[macro_export]
macro_rules! path {
    { $($arg:tt)+ } => {
        svg::node::element::Path::new().set("d", format!("{}", format_args!($($arg)+)))
    }
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

/// Macro to register a move constructor by name (and lowercased name).
macro_rules! register {
    {  $constructors:ident, $info:ident, $( $typ:ty ),* } => {
        $( $constructors.push(
            <$typ>::construct as Constructor
        ); )*
        $( $info.push(
            <$typ>::INFO,
        ); )*
    }
}

#[allow(clippy::vec_init_then_push)]
fn initialize() -> (Vec<Info>, Vec<Constructor>) {
    let mut cons = Vec::new();
    let mut info = Vec::new();

    // Insert moves in order of importance, as they will appear in the manual.
    register!(cons, info, edge::Curve);
    register!(cons, info, straight::StraightEdge);
    register!(cons, info, three::ThreeTurn);
    register!(cons, info, mohawk::OpenMohawk);
    register!(cons, info, bracket::Bracket);
    register!(cons, info, rocker::Rocker);
    register!(cons, info, counter::Counter);
    register!(cons, info, coe::ChangeOfEdge);

    register!(cons, info, warp::Warp);
    register!(cons, info, shift::Shift);
    register!(cons, info, rink::Rink);
    register!(cons, info, info::Info);
    register!(cons, info, title::Title);
    register!(cons, info, text::Text);
    register!(cons, info, label::Label);

    (info, cons)
}

/// Function that constructs a move from an [`Input`].
type Constructor = fn(&Input) -> Result<Box<dyn Move>, Error>;

/// Registry of move information and constructors.
static REGISTRY: OnceLock<(Vec<Info>, Vec<Constructor>)> = OnceLock::new();

/// Return a collection of move constructors.
fn constructors() -> &'static Vec<Constructor> {
    let (_info, constructors) = REGISTRY.get_or_init(|| initialize());
    constructors
}

/// Return a collection of move [`Info`] structures.
pub fn info() -> &'static Vec<Info> {
    let (info, _constructors) = REGISTRY.get_or_init(|| initialize());
    info
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code;

    fn check_consistent(mv: &dyn Move, input: &Input) {
        assert_eq!(
            mv.pre_transition(code!(BF)).code,
            mv.start(),
            "for '{}'",
            input.text
        );
        assert_eq!(mv.transition().code, mv.end(), "for '{}'", input.text);
        assert_eq!(mv.input(), Some(input.owned()));
        assert_eq!(mv.text(), input.text);
    }

    #[test]
    fn test_examples() {
        for info in info() {
            let input = Input {
                pos: Default::default(),
                text: info.example,
            };
            let mv = factory(&input)
                .unwrap_or_else(|e| panic!("example for {} doesn't construct!: {e:?}", info.name));
            check_consistent(&*mv, &input);
        }
    }
}
