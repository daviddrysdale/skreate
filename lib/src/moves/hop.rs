// Copyright 2024-2025 David Drysdale

//! Move definition for hop.

use super::{MoveId, SkatingMoveId};
use crate::{
    moves::{self, parse_code, parse_pre_transition},
    param,
    params::{self, Value},
    parser, pos, Code, Edge, Foot, Label, Move, MoveParam, ParseError, Percentage, Position,
    PreTransition, RenderOptions, Rotation, SkatingDirection, SpatialTransition, SvgId,
    TextPosition, Transition,
};
use nom::bytes::complete::tag;
use std::borrow::Cow;
use svg::node::element::Text as SvgText;
use svg::node::element::{Circle, Group};

#[derive(Debug, Clone)]
pub struct Hop {
    text_pos: TextPosition,
    pre_transition: PreTransition,
    foot: Foot,
    dir: SkatingDirection,
    size: i32,
    label: Option<String>,
    label_offset: Percentage,
}

impl Hop {
    /// Move code.
    pub const MOVE: &'static str = "-Hop";
    pub const INFO: moves::Info = moves::Info {
        name: "Hop",
        id: MoveId::Skating(SkatingMoveId::Hop),
        summary: "Hop",
        example: "RB-Hop",
        visible: true,
        params: &[
            params::Info {
                name: "size",
                doc: "Circle size in centimetres",
                default: Value::Number(5), // in cm
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "label",
                doc: "Replacement label, used if non-empty",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "label-offset",
                doc: "Amount to scale label offsets by, as a percentage, or -1 to use global value",
                default: Value::Number(-1),
                range: params::Range::Any,
                short: None,
            },
        ],
    };

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, ParseError> {
        let (rest, pre_transition) = parse_pre_transition(input, text_pos)?;
        let (rest, entry_code) = parse_code(rest, text_pos)?;
        if entry_code.edge != Edge::Flat {
            return Err(ParseError {
                pos: text_pos,
                msg: format!("Entry edge {entry_code} not supported"),
            });
        }
        let (rest, _move) = tag(Self::MOVE)(rest).map_err(|_e: parser::Error| ParseError {
            pos: text_pos,
            msg: format!("Missing expected '{}'", Self::MOVE),
        })?;

        let params = params::populate(Self::INFO.params, rest, text_pos)?;
        Ok(Box::new(Self::from_params(
            text_pos,
            pre_transition,
            entry_code,
            params,
        )?))
    }

    pub fn from_params(
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
    ) -> Result<Self, ParseError> {
        assert!(params::compatible(Self::INFO.params, &params));
        let label = params[1].value.as_str(text_pos)?;

        Ok(Self {
            text_pos,
            pre_transition,
            foot: entry_code.foot,
            dir: entry_code.dir,
            size: params[0].value.as_i32(text_pos)?,
            label: if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            },
            label_offset: Percentage(params[2].value.as_i32(text_pos)?),
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

impl Move for Hop {
    fn id(&self) -> MoveId {
        MoveId::Skating(SkatingMoveId::Hop)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.size),
            param!("label" = (self.label.clone().unwrap_or("".to_string()))),
            param!("label-offset" = self.label_offset.0),
        ]
    }
    fn start(&self) -> Option<Code> {
        Some(self.code())
    }
    fn text(&self) -> String {
        let suffix = params::to_string(Self::INFO.params, &self.params());
        format!("{}{}-Hop{suffix}", self.foot, self.dir)
    }
    fn expanded_text(&self) -> String {
        let suffix = params::to_expanded(Self::INFO.params, &self.params());
        format!("{}{}-Hop{suffix}", self.foot, self.dir)
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
                delta: Position::default(),
                rotate: Rotation::default(),
            },
            code: Some(self.code()),
        }
    }
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        let grp = Group::new().add(
            Circle::new()
                .set("r", self.size)
                .set("style", "fill: black;"),
        );
        vec![(SvgId(self.text()), grp)]
    }
    fn labels(&self, opts: &RenderOptions) -> Vec<Label> {
        let text = match &self.label {
            Some(label) => label.clone(),
            None => "Hop".to_string(),
        };
        let label_offset = self.label_offset.for_opts(opts);
        let dist = (30.0 * label_offset) as i64;
        vec![Label {
            display: !text.trim().is_empty(),
            text: SvgText::new(text),
            pos: pos!(dist, 0),
        }]
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        Box::new(Self {
            foot: self.foot.opposite(),
            text_pos: self.text_pos.at_repeat(repeat),
            ..self.clone()
        })
    }
    fn box_clone(&self, repeat: Option<usize>) -> Box<dyn Move> {
        let mut copy = self.clone();
        copy.text_pos = self.text_pos.at_repeat(repeat);
        Box::new(copy)
    }
}
