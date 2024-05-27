use crate::direction::Rotation;
use crate::{Foot, Input, Move, OwnedInput, ParseError, Position, RenderOptions, Transition};
use log::info;
use svg::node::element::{Group, Path};

pub fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    match input.text {
        "lf" | "LF" => Ok(Box::new(Lf::new(input))),
        "rf" | "RF" => Ok(Box::new(Rf::new(input))),
        "lfo" | "LFO" => Ok(Box::new(Lfo::new(input))),
        "lfi" | "LFI" => Ok(Box::new(Lfi::new(input))),
        "rfo" | "RFO" => Ok(Box::new(Rfo::new(input))),
        "rfi" | "RFI" => Ok(Box::new(Rfi::new(input))),
        m => Err(ParseError::from_input(input, &format!("unknown move {m}"))),
    }
}

/// Macro to populate standard boilerplate for moves.
macro_rules! standard_move {
    { $name:ident, $start_foot:ident => $end_foot:ident, $text:literal, $pos:expr, $rotate:expr, $path:literal } => {
        standard_move_with_id!($name, $start_foot => $end_foot, $text, $text, $pos, $rotate, $path);
    }
}

/// Macro to populate standard boilerplate for moves, but with an shared move ID that differs from the text form
/// (e.g. because it's not valid in an XML attribute).
macro_rules! standard_move_with_id {
    { $name:ident, $start_foot:ident => $end_foot:ident, $def_id:literal, $text:literal, $pos:expr, $rotate:expr, $path:literal } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
            const ID: &'static str = $def_id;
            pub fn new(input: &Input) -> Self {
                Self { input: input.owned() }
            }
        }
        impl Move for $name {
            fn start_foot(&self) -> Foot { Foot::$start_foot }
            fn end_foot(&self) -> Foot { Foot::$end_foot }
            fn def_id(&self) -> &'static str { Self::ID }
            fn text(&self) -> String { $text.to_string() }
            fn input(&self) -> Option<OwnedInput> { Some(self.input.clone()) }
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
    Position { x: -200, y: 200 }, Rotation(90),
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
    Position { x: 180, y: 180 }, Rotation(90),
    "c 0 90 90 180 180 180"
);

pub fn pre_transition(from: Foot, to: Foot) -> Transition {
    Transition {
        delta: Position {
            x: match (from, to) {
                (Foot::Left, Foot::Left) | (Foot::Right, Foot::Right) => 0,
                (Foot::Both, _) => 0,
                (Foot::Left, Foot::Right) => -36,
                (Foot::Left, Foot::Both) => -18,
                (Foot::Right, Foot::Left) => 36,
                (Foot::Right, Foot::Both) => 18,
            },
            y: 0,
        },
        rotate: Rotation(0),
        foot: to,
    }
}

// TODO: check consistency .. transition.foot should equal end_foot
