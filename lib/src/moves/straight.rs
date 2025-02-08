//! Move definition for simple straight edges.

use super::{MoveId, SkatingMoveId, HW};
use crate::{
    apply_style, moves, param, params,
    params::Value,
    parser,
    parser::types::{parse_code, parse_pre_transition},
    path, pos, Code, Edge, Foot, Label, Move, MoveParam, Position, PreTransition, RenderOptions,
    Rotation, SkatingDirection, SpatialTransition, SvgId, TextPosition, Transition,
};
use std::borrow::Cow;
use svg::node::element::Group;
use svg::node::element::Text as SvgText;

pub struct StraightEdge {
    text_pos: TextPosition,
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
        id: MoveId::Skating(SkatingMoveId::StraightEdge),
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

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, parser::Error> {
        let (rest, pre_transition) = parse_pre_transition(input)?;
        let (rest, entry_code) = parse_code(rest)?;
        if entry_code.edge != Edge::Flat {
            return Err(parser::fail(input));
        }

        let params = params::populate(Self::INFO.params, rest)?;
        Ok(Box::new(Self::from_params(
            input,
            text_pos,
            pre_transition,
            entry_code,
            params,
        )?))
    }

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
    ) -> Result<Self, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let label = params[1].value.as_str(input)?;

        Ok(Self {
            text_pos,
            pre_transition,
            foot: entry_code.foot,
            dir: entry_code.dir,
            len: params[0].value.as_i32(input)?,
            label: if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            },
            style: params[2].value.as_str(input)?.to_string(),
        })
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
    fn id(&self) -> MoveId {
        MoveId::Skating(SkatingMoveId::StraightEdge)
    }
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
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
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
            let text = match &self.label {
                Some(label) => label.clone(),
                None => format!("{}{}", self.foot, self.dir),
            };
            vec![Label {
                display: !text.trim().is_empty(),
                text: SvgText::new(text),
                pos: pos!(30, self.len as i64 / 2),
            }]
        }
    }
}
