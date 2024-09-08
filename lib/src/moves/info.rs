//! Pseudo-move definition for diagram info.

use crate::{
    param, params, params::Value, path, Bounds, Document, Input, Label, Move, MoveParam,
    OwnedInput, ParseError, Position, RenderOptions, Skater,
};
use std::borrow::Cow;
use svg::node::element::Group;

pub struct Info {
    input: OwnedInput,
    label: String,
    label_pos: Position,
    debug: bool,
}

const NAME: &str = "Info";

impl Info {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "label",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "label-x",
            default: Value::Number(100),
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "label-y",
            default: Value::Number(100),
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "debug",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: params::Abbrev::None,
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
            label: params[0].value.as_str().unwrap().to_string(),
            label_pos: Position {
                x: params[1].value.as_i32().unwrap() as i64,
                y: params[1].value.as_i32().unwrap() as i64,
            },
            debug: params[3].value.as_bool().unwrap(),
        }))
    }
}

impl Move for Info {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.label),
            param!("label-x" = (self.label_pos.x as i32)),
            param!("label-y" = (self.label_pos.y as i32)),
            param!(self.debug),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME} {params}")
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
    fn def(&self, _opts: &RenderOptions) -> Group {
        let mut defs = Group::new();
        if self.debug {
            defs = defs.add(
                path!(
                    "M 0,0 l 10,0 l -20,0 l 10,0 l 0,20 l 8,-8 l -8,8 l-8,-8 l 8,8 l 0,-30 l 0,10",
                )
                .set("style", "stroke:red;")
                .set("id", "end-mark"),
            );
            defs = defs.add(
                path!(
                    "M 0,0 l 10,0 l -20,0 l 10,0 l 0,20 l 8,-8 l -8,8 l-8,-8 l 8,8 l 0,-30 l 0,10",
                )
                .set("style", "stroke:green;")
                .set("id", "start-mark"),
            );
        }
        defs
    }
    fn render(&self, doc: Document, _start: &Skater, opts: &mut RenderOptions) -> Document {
        opts.debug = self.debug;
        doc
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        if self.label.is_empty() {
            Vec::new()
        } else {
            vec![Label {
                text: self.label.clone(),
                pos: self.label_pos,
            }]
        }
    }
}
