//! Pseudo-move definition for diagram title.

use super::Error;
use crate::{
    param, params, params::Value, Bounds, Input, Move, MoveParam, OwnedInput, Position,
    RenderOptions, Skater,
};
use std::borrow::Cow;
use svg::{node::element::Text, Document};

pub struct Title {
    input: OwnedInput,
    text: String,
    pos: Position,
}

const NAME: &str = "Title";

impl Title {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "text",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
        // (-1, -1) used to indicate auto-positioning.
        params::Info {
            name: "x",
            default: Value::Number(-1),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "y",
            default: Value::Number(-1),
            range: params::Range::Any,
            short: None,
        },
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(Error::Failed)?;

        Ok(Box::new(Self {
            input: input.owned(),
            text: params[0].value.as_str().unwrap().to_string(),
            pos: Position::from_params(&params[1], &params[2]),
        }))
    }
}

impl Move for Title {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME}{params}")
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(&self, doc: Document, _start: &Skater, opts: &mut RenderOptions) -> Document {
        let x = if self.pos.x >= 0 {
            self.pos.x
        } else {
            opts.bounds.midpoint().x
        };
        let y = if self.pos.y >= 0 { self.pos.y } else { 100 };
        doc.add(
            Text::new(self.text.clone())
                .set("x", x)
                .set("y", y)
                .set("style", format!("font-size: {}pt;", opts.font_size() * 2)),
        )
    }
}
