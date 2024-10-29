//! Pseudo-move definition for moving skater to new position.

use super::Error;
use crate::{
    param, params, params::Value, Bounds, Direction, Document, Input, Move, MoveParam, OwnedInput,
    Position, RenderOptions, Skater, SpatialTransition, Transition,
};

const NAME: &str = "Warp";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    input: OwnedInput,
    pos: Position,
    dir: Direction,
}

impl Warp {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "x",
            default: Value::Number(6 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: None,
        },
        params::Info {
            name: "y",
            default: Value::Number(6 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: None,
        },
        params::Info {
            name: "dir",
            default: Value::Number(0),
            range: params::Range::Positive,
            short: None,
        },
    ];

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        Ok(Box::new(Self::new(input)?))
    }

    pub fn new(input: &Input) -> Result<Self, Error> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(Error::Failed)?;
        Ok(Self {
            input: input.owned(),
            pos: Position::from_params(&params[0], &params[1]),
            dir: Direction(params[2].value.as_i32().unwrap() as u32),
        })
    }
}

impl Move for Warp {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
            param!("dir" = (self.dir.0 as i32)),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME}{params}")
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn transition(&self) -> Transition {
        Transition {
            spatial: SpatialTransition::Absolute {
                pos: self.pos,
                dir: self.dir,
            },
            code: None,
        }
    }
    fn render(&self, doc: Document, _start: &Skater, _opts: &mut RenderOptions) -> Document {
        doc
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        Some(Bounds {
            top_left: self.pos,
            bottom_right: self.pos,
        })
    }
}
