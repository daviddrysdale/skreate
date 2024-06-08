//! Skating move definitions.

use crate::{
    code, Code, Edge, Foot, Input, Move, MoveData, OwnedInput, ParseError, Position, RenderOptions,
    Rotation, SkatingDirection, SkatingDirection::*, Transition,
};
use log::{info, warn};
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
//  |
//  v  y-axis

const ARROW: Option<i32> = Some(45);

const ARROW_LEN: i64 = 10; // cm

fn arrow_at(len: i64, dir: i32, arrow_angle: i32) -> String {
    let len = len as f64;
    let angle = (dir + arrow_angle) as f64 * std::f64::consts::PI / 180.0;
    let d1x = (len * angle.sin()) as i32;
    let d1y = (-len * angle.cos()) as i32;

    let angle = (dir - arrow_angle) as f64 * std::f64::consts::PI / 180.0;
    let d2x = (len * angle.sin()) as i32;
    let d2y = (-len * angle.cos()) as i32;
    format!(
        "{} {} {} {} {} {} {} {}",
        d1x, d1y, -d1x, -d1y, d2x, d2y, -d2x, -d2y
    )
}

fn arrow(dir: i32) -> String {
    if let Some(arrow_angle) = ARROW {
        arrow_at(ARROW_LEN, dir, arrow_angle)
    } else {
        "".to_string()
    }
}

standard_move!(Lf, LF => LF, "LF", Position { x: 0, y: 100 }, Rotation(0), format!("l 0 35 {} 0 65", arrow(0)));
standard_move!(Rf, RF => RF, "RF", Position { x: 0, y: 100 }, Rotation(0), format!("l 0 35 {} 0 65", arrow(0)));
standard_move!(Lb, LB => LB, "LB", Position { x: 0, y: 100 }, Rotation(0), format!("l 0 35 {} 0 65", arrow(0)));
standard_move!(Rb, RB => RB, "RB", Position { x: 0, y: 100 }, Rotation(0), format!("l 0 35 {} 0 65", arrow(0)));
move_and_xf!(Lfo, XfLfo, LFO => LFO, "LFO", Position { x: 200, y: 200 }, Rotation(-90), "c 0 100 100 200 200 200");
move_and_xf!(Lfi, XfLfi, LFI => LFI, "LFI", Position { x: -180, y: 180 }, Rotation(90), "c 0 90 -90 180 -180 180");
move_and_xf!(Rfo, XfRfo, RFO => RFO, "RFO", Position { x: -200, y: 200 }, Rotation(90), "c 0 100 -100 200 -200 200");
move_and_xf!(Rfi, XfRfi, RFI => RFI, "RFI", Position { x: 180, y: 180 }, Rotation(-90), "c 0 90 90 180 180 180");
move_and_xb!(Lbo, XbLbo, LBO => LBO, "LBO", Position { x: -200, y: 200 }, Rotation(-90), "c 0 100 -100 200 -200 200");
move_and_xb!(Lbi, XbLbi, LBI => LBI, "LBI", Position { x: 180, y: 180 }, Rotation(90), "c 0 90 90 180 180 180");
move_and_xb!(Rbo, XbRbo, RBO => RBO, "RBO", Position { x: 200, y: 200 }, Rotation(90), "c 0 100 100 200 200 200");
move_and_xb!(Rbi, XbRbi, RBI => RBI, "RBI", Position { x: -180, y: 180 }, Rotation(-90), "c 0 90 -90 180 -180 180");
standard_move!(Bf, BF => BF, "BF", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW} 0 l 0 35 {0} 0 65 m -{HW} -100 l 0 35 {0} 0 65", arrow(0)));
standard_move!(Bb, BF => BF, "BB", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW} 0 l 0 35 {0} 0 65 m -{HW} -100 l 0 35 {0} 0 65", arrow(0)));

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

/// Half-width of a standard stance.
const HW: i64 = 18; // cm
/// Width of a standard stance.
const W: i64 = 2 * HW; // cm

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
        delta: Position { x, y },
        rotate: Rotation(rotation),
        code: to,
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
                    y = HW
                }
                (Foot::Right, Foot::Left) => {
                    x = -HW;
                    y = HW
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
                    x = -HW;
                    y = HW
                }
                (Foot::Right, Foot::Left) => {
                    x = HW;
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

    #[test]
    fn test_arrow_at() {
        let tests = [
            (0, 45, "7 -7 -7 7 -7 -7 7 7"),
            (0, 90, "10 0 -10 0 -10 0 10 0"),
            (45, 45, "10 0 -10 0 0 -10 0 10"),
            (90, 45, "7 7 -7 -7 7 -7 -7 7"),
        ];
        for (dir, arrow_angle, want) in tests {
            let got = arrow_at(10, dir, arrow_angle);
            assert_eq!(got, want, "for {dir}, {arrow_angle}");
        }
    }
}
