//! Skating move definitions.

use crate::{
    code, label, Code, Foot, Input, Label, Move, MoveParam, OwnedInput, ParseError, Position,
    RenderOptions, Rotation, SkatingDirection::*, SpatialTransition, Transition,
};
use log::{info, warn};
use std::sync::OnceLock;
use svg::node::element::{Group, Path};

mod edge;
mod info;
mod label;
mod rink;
mod straight;
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    /// Name of the move.
    pub name: &'static str,
    /// Summary of the move.
    pub summary: &'static str,
    /// Example input for move.
    pub example: &'static str,
    /// Move parameter information.
    pub params: &'static [crate::params::Info],
}

pub(crate) fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");

    for constructor in constructors() {
        match constructor(input) {
            Ok(mv) => return Ok(mv),
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

/// Macro to populate standard boilerplate for moves.
macro_rules! move_and_xf {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr, $($labels:expr),* } => {
        move_definition!($name, code!($start) => code!($end), $text, $pos, $rotate, $path, vec![$($labels),*], pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xf-", $text), $pos, $rotate, $path, vec![$($labels),*, label!("xf" @ 10,10)], cross_transition);
    }
}
macro_rules! move_and_xb {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr, $($labels:expr),* } => {
        move_definition!($name, code!($start) => code!($end), $text, $pos, $rotate, $path, vec![$($labels),*], pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xb-", $text), $pos, $rotate, $path, vec![$($labels),*, label!("xb" @ 10,10)], cross_transition);
    }
}

/// Macro to populate a structure that implements [`Move`] and [`Info`].
macro_rules! move_definition {
    { $name:ident, $start:expr => $end:expr, $text:expr, $pos:expr, $rotate:expr, $path:expr, $labels:expr, $pre_trans:ident } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
            const START: Code = $start;
            const END: Code = $end;
            const ID: &'static str = $text;
            const INFO: Info = Info {
                name: stringify!($name),
                summary: stringify!($name),
                example: $text,
                params: &[],
            };
            pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
                if input.text == Self::ID {
                    Ok(Box::new(Self { input: input.owned()}))
                } else {
                    Err(Error::Unrecognized)
                }
            }
        }

        impl Move for $name {
            fn params(&self) -> Vec<MoveParam> {vec![]}
            fn start(&self) -> Option<Code> { Some(Self::START) }
            fn end(&self) -> Option<Code> { Some(Self::END) }
            fn text(&self) -> String { $text.to_string() }
            fn input(&self) -> Option<OwnedInput> { Some(self.input.clone()) }
            fn pre_transition(&self, from: Code) -> Transition {
                $pre_trans(from, Self::START)
            }
            fn transition(&self) -> Transition {
                Transition {
                    spatial: SpatialTransition::Relative {
                        delta: $pos,
                        rotate: $rotate,
                    },
                    code: Some(Self::END),
                }
            }
            fn def(&self, _opts: &mut RenderOptions) -> Option<Group> {
                Some(Group::new().add(Path::new().set("d", format!("M 0,0 {}", $path))))
            }
            fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
                $labels
            }
        }
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

move_and_xf!(Lfo3, XfLfo3, LFO => LBI, "LFO3", Position { x: 200, y: 200 }, Rotation(-90), "c 15,80 90,90 130,70 c -20,40 -10,115 70,130", label!("LFO" @ 50,40), label!("3" @ 140,60), label!("LBI" @ 160,160));
move_and_xb!(Lbi3, XbLbi3, LBI => LFO, "LBI3", Position { x: 200, y: 200 }, Rotation(-90), "c 15,80 90,90 130,70 c -20,40 -10,115 70,130", label!("LBI" @ 50,40), label!("3" @ 140,60), label!("LFO" @ 160,160));
move_and_xf!(Rfi3, XfRfi3, RFI => RBO, "RFI3", Position { x: 200, y: 200 }, Rotation(-90), "c 15,80 90,90 130,70 c -20,40 -10,115 70,130", label!("RFI" @ 50,40), label!("3" @ 140,60), label!("RBO" @ 160,160));
move_and_xb!(Rbo3, XbRbo3, RBO => RFI, "RBO3", Position { x: 200, y: 200 }, Rotation(-90), "c 15,80 90,90 130,70 c -20,40 -10,115 70,130", label!("RBO" @ 50,40), label!("3" @ 140,60), label!("RFI" @ 160,160));

move_and_xf!(Rfo3, XfRfo3, RFO => RBI, "RFO3", Position { x: -200, y: 200 }, Rotation(90), "c -15,80 -90,90 -130,70 c 20,40 10,115 -70,130", label!("RFO" @ -50,40), label!("3" @ -140,60), label!("RBI" @ -160,160));
move_and_xb!(Rbi3, XbRbi3, RBI => RFO, "RBI3", Position { x: -200, y: 200 }, Rotation(90), "c -15,80 -90,90 -130,70 c 20,40 10,115 -70,130", label!("RBI" @ -50,40), label!("3" @ -140,60), label!("RFO" @ -160,160));
move_and_xf!(Lfi3, XfLfi3, LFI => LBO, "LFI3", Position { x: -200, y: 200 }, Rotation(90), "c -15,80 -90,90 -130,70 c 20,40 10,115 -70,130", label!("LFI" @ -50,40), label!("3" @ -140,60), label!("LBO" @ -160,160));
move_and_xb!(Lbo3, XbLbo3, LBO => LFI, "LBO3", Position { x: -200, y: 200 }, Rotation(90), "c -15,80 -90,90 -130,70 c 20,40 10,115 -70,130", label!("LBO" @ -50,40), label!("3" @ -140,60), label!("LFI" @ -160,160));

move_and_xf!(LfoRk, XfLfoRk, LFO => LBO, "LFO-Rk", Position { x: 200, y: 180 }, Rotation(0), "c 15,80 70,100 100,70 c 10,40 80,0 100,80", label!("LFO" @ 50,40), label!("Rk" @ 110,60), label!("LBO" @ 150,130));
move_and_xb!(LbiRk, XbLbiRk, LBI => LFI, "LBI-Rk", Position { x: 200, y: 180 }, Rotation(0), "c 15,80 70,100 100,70 c 10,40 80,0 100,80", label!("LBI" @ 50,40), label!("Rk" @ 110,60), label!("LFI" @ 150,130));
move_and_xf!(RfiRk, XfRfiRk, RFI => RBI, "RFI-Rk", Position { x: 200, y: 180 }, Rotation(0), "c 15,80 70,100 100,70 c 10,40 80,0 100,80", label!("RFI" @ 50,40), label!("Rk" @ 110,60), label!("RBI" @ 150,130));
move_and_xb!(RboRk, XbRboRk, RBO => RFO, "RBO-Rk", Position { x: 200, y: 180 }, Rotation(0), "c 15,80 70,100 100,70 c 10,40 80,0 100,80", label!("RBO" @ 50,40), label!("Rk" @ 110,60), label!("RFO" @ 150,130));

move_and_xf!(RfoRk, XfRfoRk, RFO => RBO, "RFO-Rk", Position { x: -200, y: 180 }, Rotation(0), "c -15,80 -70,100 -100,70 c -10,40 -80,0 -100,80", label!("RFO" @ -50,40), label!("Rk" @ -110,60), label!("RBO" @ -150,130));
move_and_xb!(RbiRk, XbRbiRk, RBI => RFI, "RBI-Rk", Position { x: -200, y: 180 }, Rotation(0), "c -15,80 -70,100 -100,70 c -10,40 -80,0 -100,80", label!("RBI" @ -50,40), label!("Rk" @ -110,60), label!("RFI" @ -150,130));
move_and_xf!(LfiRk, XfLfiRk, LFI => LBI, "LFI-Rk", Position { x: -200, y: 180 }, Rotation(0), "c -15,80 -70,100 -100,70 c -10,40 -80,0 -100,80", label!("LFI" @ -50,40), label!("Lk" @ -110,60), label!("LBI" @ -150,130));
move_and_xb!(LboRk, XbLboRk, LBO => LFO, "LBO-Rk", Position { x: -200, y: 180 }, Rotation(0), "c -15,80 -70,100 -100,70 c -10,40 -80,0 -100,80", label!("LBO" @ -50,40), label!("Lk" @ -110,60), label!("LFO" @ -150,130));

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

    register!(cons, info, warp::Warp);
    register!(cons, info, rink::Rink);
    register!(cons, info, info::Info);
    register!(cons, info, title::Title);
    register!(cons, info, label::Label);

    register!(cons, info, Lfo3, XfLfo3, Lbi3, XbLbi3, Rfi3, XfRfi3, Rbo3, XbRbo3);
    register!(cons, info, Rfo3, XfRfo3, Rbi3, XbRbi3, Lfi3, XfLfi3, Lbo3, XbLbo3);
    register!(cons, info, LfoRk, XfLfoRk, LbiRk, XbLbiRk, RfiRk, XfRfiRk, RboRk, XbRboRk);
    register!(cons, info, RfoRk, XfRfoRk, RbiRk, XbRbiRk, LfiRk, XfLfiRk, LboRk, XbLboRk);
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
/// Width of a standard stance.
const W: i64 = 2 * HW; // cm
/// Length of skate.
const SL: i64 = 30; // cm

/// Standard pre-transition with plain step.
fn pre_transition(from: Code, to: Code) -> Transition {
    let mut x = 0;
    let mut y = 0;
    let mut rotation = 0;
    match (from.dir, to.dir) {
        (Forward, Forward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Right) => x = -W,
            (Foot::Left, Foot::Both) => x = -HW,
            (Foot::Both, Foot::Left) => x = HW,
            (Foot::Both, Foot::Right) => x = -HW,
            (Foot::Right, Foot::Left) => x = W,
            (Foot::Right, Foot::Both) => x = HW,
            _ => {}
        },
        (Backward, Backward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Right) => x = W,
            (Foot::Left, Foot::Both) => x = HW,
            (Foot::Both, Foot::Left) => x = -HW,
            (Foot::Both, Foot::Right) => x = HW,
            (Foot::Right, Foot::Left) => x = -W,
            (Foot::Right, Foot::Both) => x = -HW,
            _ => {}
        },
        (Forward, Backward) => match (from.foot, to.foot) {
            (Foot::Left, Foot::Left) => rotation = 180,
            (Foot::Left, Foot::Right) => {
                x = -HW;
                y = HW;
                rotation = 90;
            }
            (Foot::Left, Foot::Both) => {
                x = -HW;
                y = HW / 2;
                rotation = 90;
            }
            (Foot::Both, Foot::Left) => {
                x = HW;
                rotation = -90;
            }
            (Foot::Both, Foot::Right) => {
                x = -HW;
                rotation = 90;
            }
            (Foot::Both, Foot::Both) => rotation = 180,
            (Foot::Right, Foot::Left) => {
                x = HW;
                y = HW;
                rotation = -90;
            }
            (Foot::Right, Foot::Right) => rotation = 180,
            (Foot::Right, Foot::Both) => {
                x = HW;
                y = HW / 2;
                rotation = -90;
            }
        },
        (Backward, Forward) => {
            match (from.foot, to.foot) {
                (Foot::Left, Foot::Left) => rotation = 180, // reverse direction
                (Foot::Left, Foot::Right) => {
                    x = HW;
                    y = HW;
                    rotation = -90;
                }
                (Foot::Left, Foot::Both) => {
                    x = HW;
                    y = HW / 2;
                    rotation = 90;
                }
                (Foot::Both, Foot::Left) => {
                    x = -HW;
                    rotation = 90;
                }
                (Foot::Both, Foot::Right) => {
                    x = HW;
                    rotation = -90;
                }
                (Foot::Both, Foot::Both) => rotation = 90,
                (Foot::Right, Foot::Left) => {
                    x = -HW;
                    y = HW;
                    rotation = 90;
                }
                (Foot::Right, Foot::Right) => rotation = 180, // reverse direction
                (Foot::Right, Foot::Both) => {
                    x = -HW;
                    y = HW / 2;
                    rotation = -90;
                }
            }
        }
    }
    Transition {
        spatial: SpatialTransition::Relative {
            delta: Position { x, y },
            rotate: Rotation(rotation),
        },
        code: Some(to),
    }
}

/// Pre-transition with feet crossing over.
fn cross_transition(from: Code, to: Code) -> Transition {
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
            delta: Position { x, y },
            rotate: Rotation(0),
        },
        code: Some(to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let mv =
                factory(&input).expect(&format!("example for {} doesn't construct!", info.name));
            check_consistent(&*mv, &input);
        }
    }
}
