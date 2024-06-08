//! Skating move definitions.

use crate::{
    code, Code, Edge, Foot, Input, Move, MoveData, OwnedInput, ParseError, Position, RenderOptions,
    Rotation, SkatingDirection, Transition,
};
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use svg::node::element::{Group, Path};

pub(crate) fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    if let Some(factory) = registry().get(input.text) {
        Ok(factory(input))
    } else {
        Err(ParseError::from_input(
            input,
            &format!("unknown move {}", input.text),
        ))
    }
}

/// Macro to populate standard boilerplate for moves.
macro_rules! move_and_xf {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xf-", $text), concat!("xf-", $text), $pos, $rotate, $path, cross_transition);
    }
}
macro_rules! move_and_xb {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xb-", $text), concat!("xb-", $text), $pos, $rotate, $path, cross_transition);
    }
}
macro_rules! standard_move {
    { $name:ident, $start:ident => $end:ident, $text:expr, $pos:expr, $rotate:expr, $path:expr } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, pre_transition);
    }
}

/// Macro to populate a structure that implements [`Move`].
macro_rules! move_definition {
    { $name:ident, $start:expr => $end:expr, $def_id:expr, $text:expr, $pos:expr, $rotate:expr, $path:expr, $pre_trans:ident } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
            const START: Code = $start;
            const END: Code = $end;
            pub fn new(input: &Input) -> Self {
                Self { input: input.owned() }
            }
            pub fn new_box(input: &Input) -> Box<dyn Move> {
                Box::new(Self::new(input))
            }
        }
        impl MoveData for $name {
            const ID: &'static str = $text;
        }
        impl Move for $name {
            fn start(&self) -> Code { Self::START }
            fn end(&self) -> Code { Self::END }
            fn def_id(&self) -> &'static str { Self::ID }
            fn text(&self) -> String { $text.to_string() }
            fn input(&self) -> Option<OwnedInput> { Some(self.input.clone()) }
            fn pre_transition(&self, from: Code) -> Transition {
                $pre_trans(from, self.start())
            }
            fn transition(&self) -> Transition {
                Transition {
                    delta: $pos,
                    rotate: $rotate,
                    code: Self::END,
                }
            }
            fn def(&self, _opts: &RenderOptions) -> Group {
                Group::new()
                    .set("stroke", "black")
                    .set("fill", "none")
                    .add(Path::new().set("d", format!("M 0 0 {}", $path)))
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
//  v  y-axis

standard_move!(Lf, LF => LF, "LF", Position { x: 0, y: 100 }, Rotation(0), "l 0 100");
standard_move!(Rf, RF => RF, "RF", Position { x: 0, y: 100 }, Rotation(0), "l 0 100");
standard_move!(Lb, LB => LB, "LB", Position { x: 0, y: 100 }, Rotation(0), "l 0 100");
standard_move!(Rb, RB => RB, "RB", Position { x: 0, y: 100 }, Rotation(0), "l 0 100");
move_and_xf!(Lfo, XfLfo, LFO => LFO, "LFO", Position { x: 200, y: 200 }, Rotation(-90), "c 0 100 100 200 200 200");
move_and_xf!(Lfi, XfLfi, LFI => LFI, "LFI", Position { x: -180, y: 180 }, Rotation(90), "c 0 90 -90 180 -180 180");
move_and_xf!(Rfo, XfRfo, RFO => RFO, "RFO", Position { x: -200, y: 200 }, Rotation(90), "c 0 100 -100 200 -200 200");
move_and_xf!(Rfi, XfRfi, RFI => RFI, "RFI", Position { x: 180, y: 180 }, Rotation(-90), "c 0 90 90 180 180 180");
move_and_xb!(Lbo, XbLbo, LBO => LBO, "LBO", Position { x: -200, y: 200 }, Rotation(-90), "c 0 100 -100 200 -200 200");
move_and_xb!(Lbi, XbLbi, LBI => LBI, "LBI", Position { x: 180, y: 180 }, Rotation(90), "c 0 90 90 180 180 180");
move_and_xb!(Rbo, XbRbo, RBO => RBO, "RBO", Position { x: 200, y: 200 }, Rotation(90), "c 0 100 100 200 200 200");
move_and_xb!(Rbi, XbRbi, RBI => RBI, "RBI", Position { x: -180, y: 180 }, Rotation(-90), "c 0 90 -90 180 -180 180");
standard_move!(Bf, BF => BF, "BF", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW} 0 l 0 100 m -{HW} -100 l 0 100"));
standard_move!(Bb, BF => BF, "BB", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW} 0 l 0 100 m -{HW} -100 l 0 100"));

/// Macro to register a move constructor by name (and lowercased name).
macro_rules! register {
    { $ids:ident, $m:ident, $( $typ:ident ),* } => {
        $(
            $ids.insert($typ::ID.to_string());
            $m.insert($typ::ID.to_string(), $typ::new_box as Constructor);
            $m.insert($typ::ID.to_lowercase(), $typ::new_box as Constructor);
        )*
    }
}

fn initialize() -> (HashSet<String>, HashMap<String, Constructor>) {
    let mut m = HashMap::new();
    let mut ids = HashSet::new();
    register!(ids, m, Lf, Rf, Lb, Rb);
    register!(ids, m, Lfo, XfLfo, Lfi, XfLfi, Rfo, XfRfo, Rfi, XfRfi);
    register!(ids, m, Lbo, XbLbo, Lbi, XbLbi, Rbo, XbRbo, Rbi, XbRbi);
    register!(ids, m, Bf, Bb);
    (ids, m)
}

/// Function that constructs a move from an [`Input`].
type Constructor = fn(&Input) -> Box<dyn Move>;

/// Registry of move names and name-or-alias to constructor mapping.
static REGISTRY: OnceLock<(HashSet<String>, HashMap<String, Constructor>)> = OnceLock::new();

/// Return the set of move names.
pub fn ids() -> &'static HashSet<String> {
    &REGISTRY.get_or_init(|| initialize()).0
}

fn registry() -> &'static HashMap<String, Constructor> {
    &REGISTRY.get_or_init(|| initialize()).1
}

const HW: i64 = 18; // cm
const W: i64 = 2 * HW; // cm

/// Standard pre-transition is just to change foot.
fn pre_transition(from: Code, to: Code) -> Transition {
    let x = match (from.foot, to.foot) {
        (Foot::Left, Foot::Left) | (Foot::Right, Foot::Right) => 0,
        (Foot::Both, _) => 0,
        (Foot::Left, Foot::Right) => -36,
        (Foot::Left, Foot::Both) => -18,
        (Foot::Right, Foot::Left) => 36,
        (Foot::Right, Foot::Both) => 18,
    };
    Transition {
        delta: Position { x, y: 0 },
        rotate: Rotation(0),
        code: to,
    }
}

/// Standard pre-transition is just to change foot.
fn cross_transition(from: Code, to: Code) -> Transition {
    let (x, y) = match (from.foot, to.foot) {
        (Foot::Left, Foot::Right) => (18, 18),
        (Foot::Right, Foot::Left) => (-18, 18),
        (Foot::Left, Foot::Left) | (Foot::Right, Foot::Right) => {
            error!("XF transition but no foot change ({from}->{to})!");
            (0, 0)
        }
        (Foot::Both, _) => {
            error!("XF transition from two feet ({from}->{to})!");
            (0, 0)
        }
        (Foot::Left, Foot::Both) => {
            error!("XF transition to two feet ({from}->{to})!");
            (-18, 0)
        }
        (Foot::Right, Foot::Both) => {
            error!("XF transition to two feet ({from}->{to})!");
            (18, 0)
        }
    };
    Transition {
        delta: Position { x, y },
        rotate: Rotation(0),
        code: to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_consistency() {
        for name in ids() {
            let constructor = registry().get(name).unwrap();
            let input = Input {
                pos: Default::default(),
                text: name,
            };
            let mv = constructor(&input);
            assert_eq!(
                mv.pre_transition(code!(BF)).code,
                mv.start(),
                "for '{name}'"
            );
            assert_eq!(mv.transition().code, mv.end(), "for '{name}'");
            assert_eq!(mv.input(), Some(input.owned()));
            assert_eq!(mv.text(), *name);
        }
    }
}
