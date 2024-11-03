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
    font_size: Option<u32>,
    rotate: i32,
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
            params::Info {
                name: "font-size",
                doc: "Font size for label; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "rotate",
                doc: "Angle to rotate text by",
                default: Value::Number(0),
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
        let font_size = params[3].value.as_i32().unwrap();

        Ok(Box::new(Self {
            input: input.owned(),
            text: params[0].value.as_str().unwrap().to_string(),
            pos: Position::from_params(&params[1], &params[2]),
            font_size: if font_size > 0 {
                Some(font_size as u32)
            } else {
                None
            },
            rotate: params[4].value.as_i32().unwrap(),
        }))
    }

    fn font_size(&self, opts: &RenderOptions) -> u32 {
        self.font_size.unwrap_or_else(|| opts.font_size())
    }
}

impl Move for Label {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
            param!(self.rotate),
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
        let mut text = Text::new(self.text.clone())
            .set("x", self.pos.x)
            .set("y", self.pos.y)
            .set("style", format!("font-size:{}pt;", self.font_size(opts)));
        if self.rotate != 0 {
            text = text.set(
                "transform",
                format!("rotate({},{},{})", self.rotate, self.pos.x, self.pos.y),
            )
        }
        doc.add(text)
    }
}
