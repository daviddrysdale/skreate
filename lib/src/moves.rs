//! Skating move definitions.

use crate::{
    code, Code, Edge, Foot, Input, Label, Move, MoveParam, OwnedInput, ParseError, Position,
    RenderOptions, Rotation, SkatingDirection, SkatingDirection::*, Transition,
};
use log::{info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use svg::node::element::{Group, Path};

pub(crate) fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    if let Some(factory) = registry().get(input.text) {
        factory(input)
    } else {
        Err(ParseError::from_input(
            input,
            &format!("unknown move {}", input.text),
        ))
    }
}

/// Macro to build a [`Label`]
macro_rules! label {
    { $text:literal @ $x:literal, $y:literal } => {
        Label {
            text: $text.to_string(),
            pos: Position {
                x: $x,
                y: $y,
            }
        }
    }
}

/// Macro to populate standard boilerplate for moves.
macro_rules! move_and_xf {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr, $($labels:expr),* } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, vec![$($labels),*], pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xf-", $text), concat!("xf-", $text), $pos, $rotate, $path, vec![$($labels),*, label!("xf" @ 10,10)], cross_transition);
    }
}
macro_rules! move_and_xb {
    { $name:ident, $xname:ident, $start:ident => $end:ident, $text:literal, $pos:expr, $rotate:expr, $path:expr, $($labels:expr),* } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, vec![$($labels),*], pre_transition);
        move_definition!($xname, code!($start) => code!($end), concat!("xb-", $text), concat!("xb-", $text), $pos, $rotate, $path, vec![$($labels),*, label!("xb" @ 10,10)], cross_transition);
    }
}
macro_rules! standard_move {
    { $name:ident, $start:ident => $end:ident, $text:expr, $pos:expr, $rotate:expr, $path:expr, $($labels:expr),* } => {
        move_definition!($name, code!($start) => code!($end), $text, $text, $pos, $rotate, $path, vec![$($labels),*], pre_transition);
    }
}

/// Macro to populate a structure that implements [`Move`].
macro_rules! move_definition {
    { $name:ident, $start:expr => $end:expr, $def_id:expr, $text:expr, $pos:expr, $rotate:expr, $path:expr, $labels:expr, $pre_trans:ident } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
            const START: Code = $start;
            const END: Code = $end;
            const ID: &'static str = $text;
            pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
                Ok(Box::new(Self { input: input.owned()}))
            }
        }
        impl Move for $name {
            fn params(&self) -> &[MoveParam] {&[]}
            fn start(&self) -> Code { Self::START }
            fn end(&self) -> Code { Self::END }
            fn def_id(&self) -> String { Self::ID.to_string() }
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
                Group::new().add(Path::new().set("d", format!("M 0,0 {}", $path)))
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

standard_move!(Lf, LF => LF, "LF", Position { x: 0, y: 100 }, Rotation(0), format!("l 0,100"), label!("LF" @ 30,50));
standard_move!(Rf, RF => RF, "RF", Position { x: 0, y: 100 }, Rotation(0), format!("l 0,100"), label!("RF" @ 30,50));
standard_move!(Lb, LB => LB, "LB", Position { x: 0, y: 100 }, Rotation(0), format!("l 0,100"), label!("LB" @ 30,50));
standard_move!(Rb, RB => RB, "RB", Position { x: 0, y: 100 }, Rotation(0), format!("l 0,100"), label!("RB" @ 30,50));

move_and_xf!(Lfo, XfLfo, LFO => LFO, "LFO", Position { x: 200, y: 200 }, Rotation(-90), "c 0,100 100,200 200,200", label!("LFO" @ 100,100));
move_and_xf!(Lfi, XfLfi, LFI => LFI, "LFI", Position { x: -180, y: 180 }, Rotation(90), "c 0,90 -90,180 -180,180", label!("LFI" @ -90,90));
move_and_xf!(Rfo, XfRfo, RFO => RFO, "RFO", Position { x: -200, y: 200 }, Rotation(90), "c 0,100 -100,200 -200,200", label!("RFO" @ -100,100));
move_and_xf!(Rfi, XfRfi, RFI => RFI, "RFI", Position { x: 180, y: 180 }, Rotation(-90), "c 0,90 90 180,180,180", label!("RFI" @ 90,90));
move_and_xb!(Lbo, XbLbo, LBO => LBO, "LBO", Position { x: -200, y: 200 }, Rotation(90), "c 0,100 -100,200 -200,200", label!("LBO" @ -100,100));
move_and_xb!(Lbi, XbLbi, LBI => LBI, "LBI", Position { x: 180, y: 180 }, Rotation(-90), "c 0,90 90 180,180,180", label!("LBI" @ 90,90));
move_and_xb!(Rbo, XbRbo, RBO => RBO, "RBO", Position { x: 200, y: 200 }, Rotation(-90), "c 0,100 100,200 200,200", label!("RBO" @ 100,100));
move_and_xb!(Rbi, XbRbi, RBI => RBI, "RBI", Position { x: -180, y: 180 }, Rotation(90), "c 0,90 -90,180 -180,180", label!("RBI" @ -90,90));

standard_move!(Bf, BF => BF, "BF", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW},0 l 0,100 m -{HW},-100 l 0,100"), );
standard_move!(Bb, BF => BF, "BB", Position { x: 0, y: 100 }, Rotation(0), format!("m {HW},0 l 0,100 m -{HW},-100 l 0,100"), );

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
    { $ids:ident, $m:ident, $( $typ:ident ),* } => {
        $(
            $ids.insert($typ::ID.to_string());
            $m.insert($typ::ID.to_string(), $typ::construct as Constructor);
            $m.insert($typ::ID.to_lowercase(), $typ::construct as Constructor);
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
    register!(ids, m, Lfo3, XfLfo3, Lbi3, XbLbi3, Rfi3, XfRfi3, Rbo3, XbRbo3);
    register!(ids, m, Rfo3, XfRfo3, Rbi3, XbRbi3, Lfi3, XfLfi3, Lbo3, XbLbo3);
    register!(ids, m, LfoRk, XfLfoRk, LbiRk, XbLbiRk, RfiRk, XfRfiRk, RboRk, XbRboRk);
    register!(ids, m, RfoRk, XfRfoRk, RbiRk, XbRbiRk, LfiRk, XfLfiRk, LboRk, XbLboRk);
    (ids, m)
}

/// Function that constructs a move from an [`Input`].
type Constructor = fn(&Input) -> Result<Box<dyn Move>, ParseError>;

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
            let mv = constructor(&input).unwrap();
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
