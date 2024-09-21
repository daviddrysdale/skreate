//! Pseudo-move definition for diagram label.

use crate::{
    param, params, params::Value, Bounds, Input, Move, MoveParam, OwnedInput, ParseError, Position,
    RenderOptions, Skater,
};
use std::borrow::Cow;
use svg::Document;

pub struct Label {
    input: OwnedInput,
    text: String,
    pos: Position,
}

const NAME: &str = "Label";

impl Label {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "text",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
        params::Info {
            name: "x",
            default: Value::Number(100),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "y",
            default: Value::Number(100),
            range: params::Range::Any,
            short: None,
        },
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(ParseError {
                pos: input.pos,
                msg: format!("No {NAME} prefix"),
            });
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(|msg| ParseError {
            pos: input.pos,
            msg,
        })?;

        Ok(Box::new(Self {
            input: input.owned(),
            text: params[0].value.as_str().unwrap().to_string(),
            pos: Position {
                x: params[1].value.as_i32().unwrap() as i64,
                y: params[2].value.as_i32().unwrap() as i64,
            },
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
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME}{params}")
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn encompass_bounds(
        &self,
        skater: &Skater,
        _include_pre: bool,
        _bounds: &mut Bounds,
    ) -> Skater {
        *skater
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<crate::Label> {
        // TODO: label position is relative to current coordinate system, needs to be absolute
        vec![crate::Label {
            text: self.text.clone(),
            pos: self.pos,
        }]
    }
    fn render(&self, doc: Document, _start: &Skater, _opts: &mut RenderOptions) -> Document {
        doc
    }
}
