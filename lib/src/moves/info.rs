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
    markers: bool,
    bounds: bool,
    grid: Option<i32>,
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
            name: "markers",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "bounds",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "grid",
            default: Value::Number(0),
            range: params::Range::Positive,
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
        let grid = params[5].value.as_i32().unwrap();

        Ok(Box::new(Self {
            input: input.owned(),
            label: params[0].value.as_str().unwrap().to_string(),
            label_pos: Position {
                x: params[1].value.as_i32().unwrap() as i64,
                y: params[2].value.as_i32().unwrap() as i64,
            },
            markers: params[3].value.as_bool().unwrap(),
            bounds: params[4].value.as_bool().unwrap(),
            grid: if grid > 0 { Some(grid) } else { None },
        }))
    }
}

impl Move for Info {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.label),
            param!("label-x" = (self.label_pos.x as i32)),
            param!("label-y" = (self.label_pos.y as i32)),
            param!(self.markers),
            param!(self.bounds),
            param!("grid" = (self.grid.unwrap_or(0))),
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
        if self.markers {
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
        opts.markers = self.markers;
        opts.bounds = self.bounds;
        opts.grid = self.grid.map(|g| g as usize);
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
