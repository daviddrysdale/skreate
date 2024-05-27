use crate::direction::Rotation;
use crate::{Foot, Input, Move, ParseError, Position, RenderOptions, Transition};
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
    { $name:ident, $def_id:literal, $text:literal, $transition:block, $def:block } => {
        struct $name {
            input: String,
        }
        impl $name {
            const ID: &'static str = $def_id;

            pub fn new(input: &Input) -> Self {
                Self { input: input.text.to_string(), }
            }
        }
        impl Move for $name {
            fn def_id(&self) -> &'static str { Self::ID }
            fn text(&self) -> String { $text.to_string() }
            fn input_text(&self) -> Option<String> { Some(self.input.clone()) }
            fn transition(&self) -> Transition { $transition }
            fn def(&self, _opts: &RenderOptions) -> Group { $def }
        }
    }
}

standard_move!(
    Lf,
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
