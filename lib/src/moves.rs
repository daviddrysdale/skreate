use crate::direction::Rotation;
use crate::{Foot, Input, Move, OwnedInput, ParseError, Position, RenderOptions, Transition};
use log::info;
use svg::node::element::{Group, Path};

pub fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    match input.text {
        "lf" | "LF" => Ok(Box::new(Lf::new(input))),
        "rf" | "RF" => Ok(Box::new(Rf::new(input))),
        m => Err(ParseError::from_input(input, &format!("unknown move {m}"))),
    }
}

/// Macro to populate standard boilerplate for moves.
macro_rules! standard_move {
    { $name:ident, $start_foot:ident, $end_foot:ident, $def_id:literal, $text:literal, $transition:block, $def:block } => {
        struct $name {
            input: OwnedInput,
        }
        impl $name {
            const ID: &'static str = $def_id;

            pub fn new(input: &Input) -> Self {
                Self { input: input.owned(), }
            }
        }
        impl Move for $name {
            fn start_foot(&self) -> Foot { Foot::$start_foot }
            fn end_foot(&self) -> Foot { Foot::$end_foot }
            fn def_id(&self) -> &'static str { Self::ID }
            fn text(&self) -> String { $text.to_string() }
            fn input(&self) -> Option<OwnedInput> { Some(self.input.clone()) }
            fn transition(&self) -> Transition { $transition }
            fn def(&self, _opts: &RenderOptions) -> Group { $def }
        }
    }
}

standard_move!(
    Lf,
    Left,
    Left,
    "lf",
    "LF",
    {
        Transition {
            delta: Position { x: 0, y: 100 },
            rotate: Rotation(0),
            foot: Foot::Left,
        }
    },
    {
        Group::new()
            .set("stroke", "black")
            .add(Path::new().set("d", "M 0 0 l 0 100"))
    }
);

standard_move!(
    Rf,
    Right,
    Right,
    "rf",
    "RF",
    {
        Transition {
            delta: Position { x: 0, y: 100 },
            rotate: Rotation(0),
            foot: Foot::Right,
        }
    },
    {
        Group::new()
            .set("stroke", "black")
            .add(Path::new().set("d", "M 0 0 l 0 100"))
    }
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
