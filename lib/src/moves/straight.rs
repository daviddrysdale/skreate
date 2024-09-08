//! Move definition for simple straight edges.

use super::{cross_transition, pre_transition, HW};
use crate::{
    param, params, params::Value, parse_foot_dir, parse_transition_prefix, path, Code, Edge, Foot,
    Input, Label, Move, MoveParam, OwnedInput, ParseError, Position, RenderOptions, Rotation,
    SkatingDirection, Transition,
};
use svg::node::element::Group;

pub struct StraightEdge {
    input: OwnedInput,
    cross_transition: bool,
    foot: Foot,
    dir: SkatingDirection,
    len: i32,
}

impl StraightEdge {
    const PARAMS_INFO: &'static [params::Info] = &[params::Info {
        name: "len",
        default: Value::Number(450),
        range: params::Range::StrictlyPositive,
        short: params::Abbrev::PlusMinus(params::Detents {
            add1: 600,
            add2: 850,
            add3: 1000,
            less1: 300,
            less2: 240,
            less3: 100,
        }),
    }];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
        let (cross_transition, rest) = parse_transition_prefix(input.text);
        let (foot, dir, rest) = parse_foot_dir(rest).map_err(|msg| ParseError {
            pos: input.pos,
            msg,
        })?;

        let params = params::populate(Self::PARAMS_INFO, rest).map_err(|msg| ParseError {
            pos: input.pos,
            msg,
        })?;

        Ok(Box::new(Self {
            input: input.owned(),
            cross_transition,
            foot,
            dir,
            len: params[0].value.as_i32().unwrap(),
        }))
    }
}

impl Move for StraightEdge {
    fn params(&self) -> Vec<MoveParam> {
        vec![param!(self.len)]
    }
    fn start(&self) -> Code {
        Code {
            foot: self.foot,
            dir: self.dir,
            edge: Edge::Flat,
        }
    }
    fn end(&self) -> Code {
        self.start()
    }
    fn text(&self) -> String {
        let prefix = match (self.cross_transition, self.dir) {
            (false, _) => "",
            (true, SkatingDirection::Forward) => "xf-",
            (true, SkatingDirection::Backward) => "xb-",
        };
        let suffix = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{prefix}{}{}{suffix}", self.foot, self.dir)
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, from: Code) -> Transition {
        if self.cross_transition {
            cross_transition(from, self.start())
        } else {
            pre_transition(from, self.start())
        }
    }
    fn transition(&self) -> Transition {
        Transition {
            delta: Position {
                x: 0,
                y: self.len as i64,
            },
            code: self.end(),
            rotate: Rotation(0),
        }
    }
    fn def(&self, _opts: &RenderOptions) -> Group {
        if self.foot == Foot::Both {
            Group::new().add(path!(
                "M 0,0 m {HW},0 l 0,{0} m -{HW},-{0} l 0,{0}",
                self.len
            ))
        } else {
            Group::new().add(path!("M 0,0 l 0,{}", self.len))
        }
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
