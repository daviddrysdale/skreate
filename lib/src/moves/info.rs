//! Pseudo-move definition for diagram info.

use super::Error;
use crate::{
    param, params, params::Value, path, Bounds, Document, Input, Label, Move, MoveParam,
    OwnedInput, Position, RenderOptions, Skater,
};
use svg::node::element::Group;

pub struct Info {
    input: OwnedInput,
    markers: bool,
    bounds: bool,
    grid: Option<i32>,
    margin: Position,
}

const NAME: &str = "Info";

impl Info {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "markers",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: None,
        },
        params::Info {
            name: "bounds",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: None,
        },
        params::Info {
            name: "grid",
            default: Value::Number(0),
            range: params::Range::Positive,
            short: None,
        },
        params::Info {
            name: "margin-x",
            default: Value::Number(crate::MARGIN as i32),
            range: params::Range::Positive,
            short: None,
        },
        params::Info {
            name: "margin-y",
            default: Value::Number(crate::MARGIN as i32),
            range: params::Range::Positive,
            short: None,
        },
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(Error::Failed)?;
        let grid = params[2].value.as_i32().unwrap();

        Ok(Box::new(Self {
            input: input.owned(),
            markers: params[0].value.as_bool().unwrap(),
            bounds: params[1].value.as_bool().unwrap(),
            grid: if grid > 0 { Some(grid) } else { None },
            margin: Position::from_params(&params[3], &params[4]),
        }))
    }
}

impl Move for Info {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.markers),
            param!(self.bounds),
            param!("grid" = (self.grid.unwrap_or(0))),
            param!("label-x" = (self.margin.x as i32)),
            param!("label-y" = (self.margin.y as i32)),
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
    fn def(&self, opts: &mut RenderOptions) -> Option<Group> {
        // Change some options once and for all in the prelude.
        opts.show_bounds = self.bounds;
        opts.grid = self.grid.map(|g| g as usize);
        opts.offset = self.margin;

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
        Some(defs)
    }
    fn render(&self, doc: Document, _start: &Skater, opts: &mut RenderOptions) -> Document {
        // Some options can be toggled on/off as we go along.
        opts.markers = self.markers;

        doc
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        Vec::new()
    }
}
