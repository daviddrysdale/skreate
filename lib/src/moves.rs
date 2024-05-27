use crate::direction::Rotation;
use crate::{Foot, Input, Move, ParseError, Position, RenderOptions, Transition};
use log::info;
use svg::node::element::{Group, Path};

pub fn factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    match input.text {
        "lf" | "LF" => Ok(Box::new(Lf::new(input))),
        m => Err(ParseError::from_input(input, &format!("unknown move {m}"))),
    }
}

pub struct Lf {
    input: String,
}

impl Lf {
    const ID: &'static str = "lf";

    pub fn new(input: &Input) -> Self {
        Self {
            input: input.text.to_string(),
        }
    }
}

impl Move for Lf {
    fn transition(&self) -> Transition {
        Transition {
            delta: Position { x: 0, y: 100 },
            rotate: Rotation(0),
            foot: Foot::Left,
        }
    }

    fn def(&self, _opts: &RenderOptions) -> Group {
        Group::new()
            .set("stroke", "black")
            .add(Path::new().set("d", "M 0 0 l 0 100"))
    }

    fn def_id(&self) -> &'static str {
        Self::ID
    }

    fn text(&self) -> String {
        "LF".to_string()
    }

    fn input_text(&self) -> Option<String> {
        Some(self.input.clone())
    }
}
