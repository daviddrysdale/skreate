//! Move definition for simple straight edges.

use super::{Error, HW};
use crate::{
    moves, param, params, params::Value, parse_foot_dir, path, Code, Edge, Foot, Input, Label,
    Move, MoveParam, OwnedInput, Position, PreTransition, RenderOptions, Rotation,
    SkatingDirection, SpatialTransition, Transition,
};
use svg::node::element::Group;

pub struct StraightEdge {
    input: OwnedInput,
    pre_transition: PreTransition,
    foot: Foot,
    dir: SkatingDirection,
    len: i32,
}

impl StraightEdge {
    pub const INFO: moves::Info = moves::Info {
        name: "Straight edge",
        summary: "Straight edge",
        example: "LF",
        params: &[params::Info {
            name: "len",
            doc: "Length in centimetres",
            default: Value::Number(450),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::PlusMinus(params::Detents {
                add1: 600,
                add2: 850,
                add3: 1000,
                less1: 300,
                less2: 240,
                less3: 100,
            })),
        }],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (pre_transition, rest) = PreTransition::parse(input.text);
        let (foot, dir, rest) = parse_foot_dir(rest).map_err(|_msg| Error::Unrecognized)?;

        let params =
            params::populate(Self::INFO.params, rest).map_err(|_msg| Error::Unrecognized)?;

        Ok(Box::new(Self {
            input: input.owned(),
            pre_transition,
            foot,
            dir,
            len: params[0].value.as_i32().unwrap(),
        }))
    }
    fn code(&self) -> Code {
        Code {
            foot: self.foot,
            dir: self.dir,
            edge: Edge::Flat,
        }
    }
}

impl Move for StraightEdge {
    fn params(&self) -> Vec<MoveParam> {
        vec![param!(self.len)]
    }
    fn start(&self) -> Option<Code> {
        Some(self.code())
    }
    fn text(&self) -> String {
        let prefix = self.pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &self.params());
        format!("{prefix}{}{}{suffix}", self.foot, self.dir)
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, from: Code) -> Transition {
        self.pre_transition.perform(from, self.code())
    }
    fn transition(&self) -> Transition {
        Transition {
            spatial: SpatialTransition::Relative {
                delta: Position {
                    x: 0,
                    y: self.len as i64,
                },
                rotate: Rotation(0),
            },
            code: self.end(),
        }
    }
    fn def(&self, _opts: &mut RenderOptions) -> Option<Group> {
        let grp = if self.foot == Foot::Both {
            Group::new().add(path!(
                "M 0,0 m {HW},0 l 0,{0} m -{HW},-{0} l 0,{0}",
                self.len
            ))
        } else {
            Group::new().add(path!("M 0,0 l 0,{}", self.len))
        };
        Some(grp)
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        if self.foot == Foot::Both {
            vec![]
        } else {
            vec![Label {
                text: format!("{}{}", self.foot, self.dir),
                pos: Position {
                    x: 30,
                    y: self.len as i64 / 2,
                },
            }]
        }
    }
}
