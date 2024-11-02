//! Pseudo-move definition for diagram label.

use super::Error;
use crate::{
    moves, param, params, params::Value, Bounds, Input, Move, MoveParam, OwnedInput, Position,
    RenderOptions, Skater,
};
use std::borrow::Cow;
use svg::{node::element::Text, Document};

pub struct Label {
    input: OwnedInput,
    text: String,
    pos: Position,
}

impl Label {
    pub const INFO: moves::Info = moves::Info {
        name: "Label",
        summary: "Diagram label",
        example: "Label[text=\"Start\",x=500,y=200]",
        params: &[
            params::Info {
                name: "text",
                doc: "Text to display",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "x",
                doc: "Horizontal location of text",
                default: Value::Number(100),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "y",
                doc: "Vertical location of text (increasing down)",
                default: Value::Number(100),
                range: params::Range::Any,
                short: None,
            },
        ],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let Some(rest) = input.text.strip_prefix(Self::INFO.name) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::INFO.params, rest).map_err(Error::Failed)?;

        Ok(Box::new(Self {
            input: input.owned(),
            text: params[0].value.as_str().unwrap().to_string(),
            pos: Position::from_params(&params[1], &params[2]),
        }))
    }
}

impl Move for Label {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(&self, doc: Document, _start: &Skater, opts: &mut RenderOptions) -> Document {
        doc.add(
            Text::new(self.text.clone())
                .set("x", self.pos.x)
                .set("y", self.pos.y)
                .set("style", format!("font-size:{}pt;", opts.font_size())),
        )
    }
}
