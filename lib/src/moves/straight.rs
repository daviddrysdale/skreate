//! Move definition for simple straight edges.

use super::{Error, HW};
use crate::{
    apply_style, moves, param, params, params::Value, parse_foot_dir, path, pos, Code, Edge, Foot,
    Input, Label, Move, MoveParam, OwnedInput, Position, PreTransition, RenderOptions, Rotation,
    SkatingDirection, SpatialTransition, SvgId, Transition,
};
use std::borrow::Cow;
use svg::node::element::Group;

pub struct StraightEdge {
    input: OwnedInput,
    pre_transition: PreTransition,
    foot: Foot,
    dir: SkatingDirection,
    len: i32,
    label: Option<String>,
    style: String,
}

impl StraightEdge {
    pub const INFO: moves::Info = moves::Info {
        name: "Straight Edge",
        summary: "Straight edge",
        example: "LF",
        visible: true,
        params: &[
            params::Info {
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
            },
            params::Info {
                name: "label",
                doc: "Replacement label, used if non-empty",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "style",
                doc: "Style of line",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
        ],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (pre_transition, rest) = PreTransition::parse(input.text);
        let (foot, dir, rest) = parse_foot_dir(rest).map_err(|_msg| Error::Unrecognized)?;

        let params =
            params::populate(Self::INFO.params, rest).map_err(|_msg| Error::Unrecognized)?;
        let label = params[1].value.as_str().unwrap();

        Ok(Box::new(Self {
            input: input.owned(),
            pre_transition,
            foot,
            dir,
            len: params[0].value.as_i32().unwrap(),
            label: if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            },
            style: params[2].value.as_str().unwrap().to_string(),
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
        vec![
            param!(self.len),
            param!("label" = (self.label.clone().unwrap_or("".to_string()))),
            param!(self.style),
        ]
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
                delta: pos!(0, self.len as i64),
                rotate: Rotation(0),
            },
            code: self.end(),
        }
    }
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        let len = self.len;
        let mut path = if self.foot == Foot::Both {
            path!("M 0,0 m {HW},0 l 0,{len} m -{HW},-{len} l 0,{len}")
        } else {
            path!("M 0,0 l 0,{len}")
        };
        path = apply_style(path, &self.style);
        vec![(SvgId(self.text()), Group::new().add(path))]
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        if self.foot == Foot::Both {
            vec![]
        } else {
            vec![Label {
                text: match &self.label {
                    Some(label) => label.clone(),
                    None => format!("{}{}", self.foot, self.dir),
                },
                pos: pos!(30, self.len as i64 / 2),
            }]
        }
    }
}
