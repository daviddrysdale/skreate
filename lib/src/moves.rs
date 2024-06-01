//! Skating move definitions.

use crate::direction::Rotation;
use crate::{
    Foot, Input, Move, MoveData, OwnedInput, ParseError, Position, RenderOptions, Transition,
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
macro_rules! standard_move {
    { $name:ident, $start_foot:ident => $end_foot:ident, $text:literal, $pos:expr, $rotate:expr, $path:literal } => {
        move_definition!($name, $start_foot => $end_foot, $text, $text, $pos, $rotate, $path, pre_transition);
    }
}
macro_rules! xf_move {
    { $name:ident, $start_foot:ident => $end_foot:ident, $text:literal, $pos:expr, $rotate:expr, $path:literal } => {
        move_definition!($name, $start_foot => $end_foot, $text, $text, $pos, $rotate, $path, cross_transition);
    }
}
macro_rules! xb_move {
    { $name:ident, $start_foot:ident => $end_foot:ident, $text:literal, $pos:expr, $rotate:expr, $path:literal } => {
        move_definition!($name, $start_foot => $end_foot, $text, $text, $pos, $rotate, $path, cross_transition);
    }
}

/// Macro to populate a structure that implements [`Move`].
macro_rules! move_definition {
    { $name:ident, $start_foot:ident => $end_foot:ident, $def_id:literal, $text:literal, $pos:expr, $rotate:expr, $path:literal, $pre_trans:ident } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
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
            fn start_foot(&self) -> Foot { Foot::$start_foot }
            fn end_foot(&self) -> Foot { Foot::$end_foot }
            fn def_id(&self) -> &'static str { Self::ID }
            fn text(&self) -> String { $text.to_string() }
            fn input(&self) -> Option<OwnedInput> { Some(self.input.clone()) }
            fn pre_transition(&self, from: Foot) -> Transition {
                $pre_trans(from, self.start_foot())
            }
            fn transition(&self) -> Transition {
                Transition {
                    delta: $pos,
                    rotate: $rotate,
                    foot: Foot::$end_foot,
                }
            }
            fn def(&self, _opts: &RenderOptions) -> Group {
                Group::new()
                    .set("stroke", "black")
                    .set("fill", "none")
                    .add(Path::new().set("d", concat!("M 0 0 ", $path)))
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

standard_move!(
    Lf, Left => Left, "LF",
    Position { x: 0, y: 100 }, Rotation(0),
    "l 0 100"
);

standard_move!(
    Rf, Right => Right, "RF",
    Position { x: 0, y: 100 }, Rotation(0),
    "l 0 100"
);

standard_move!(
    Lfo, Left => Left, "LFO",
    Position { x: 200, y: 200 }, Rotation(-90),
    "c 0 100 100 200 200 200"
);

standard_move!(
    Lfi, Left => Left, "LFI",
    Position { x: -180, y: 180 }, Rotation(90),
    "c 0 90 -90 180 -180 180"
);

standard_move!(
    Rfo, Right => Right, "RFO",
    Position { x: -200, y: 200 }, Rotation(90),
    "c 0 100 -100 200 -200 200"
);

standard_move!(
    Rfi, Right => Right, "RFI",
    Position { x: 180, y: 180 }, Rotation(-90),
    "c 0 90 90 180 180 180"
);

xf_move!(
    XfRfi, Right => Right, "xf-RFI",
    Position { x: 180, y: 180 }, Rotation(-90),
    "c 0 90 90 180 180 180"
);

/// Macro to register a move constructor by name (and lowercased name).
macro_rules! register {
    { $ids:ident, $m:ident, $typ:ident } => {
        $ids.insert($typ::ID.to_string());
        $m.insert($typ::ID.to_string(), $typ::new_box as Constructor);
        $m.insert($typ::ID.to_lowercase(), $typ::new_box as Constructor);
    }
}

fn initialize() -> (HashSet<String>, HashMap<String, Constructor>) {
    let mut m = HashMap::new();
    let mut ids = HashSet::new();
    register!(ids, m, Lf);
    register!(ids, m, Rf);
    register!(ids, m, Lfo);
    register!(ids, m, Lfi);
    register!(ids, m, Rfo);
    register!(ids, m, Rfi);
    register!(ids, m, XfRfi);
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

/// Standard pre-transition is just to change foot.
fn pre_transition(from: Foot, to: Foot) -> Transition {
    let x = match (from, to) {
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
        foot: to,
    }
}

/// Standard pre-transition is just to change foot.
fn cross_transition(from: Foot, to: Foot) -> Transition {
    let (x, y) = match (from, to) {
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
        foot: to,
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
                mv.pre_transition(Foot::Both).foot,
                mv.start_foot(),
                "for '{name}'"
            );
            assert_eq!(mv.transition().foot, mv.end_foot(), "for '{name}'");
            assert_eq!(mv.input(), Some(input.owned()));
            assert_eq!(mv.text(), *name);
        }
    }
}
