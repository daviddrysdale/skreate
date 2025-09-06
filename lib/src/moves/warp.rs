// Copyright 2024-2025 David Drysdale

//! Pseudo-move definition for moving skater to new position.

use crate::{
    moves::{self, parse_code, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    Bounds, Code, Direction, Document, Move, MoveParam, ParseError, Position, RenderOptions,
    Skater, SpatialTransition, SvgId, TextPosition, Transition,
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    text_pos: TextPosition,
    pos: Position,
    dir: Direction,
    code: Option<Code>,
}

impl Warp {
    pub const MOVE: &'static str = "Warp";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Warp),
        summary: "Move skater to new location/position",
        example: "Warp[x=100,y=100,dir=270]",
        visible: false,
        params: &[
            params::Info {
                name: "x",
                doc: "Horizontal coordinate to warp to",
                default: Value::Number(0), // in cm
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "y",
                doc: "Vertical coordinate to warp to",
                default: Value::Number(0), // in cm
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "dir",
                doc: "Direction to be facing after warp, in degrees (0 is facing down)",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "code",
                doc: "Foot code to start with after warp",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
        ],
    };

    pub fn from_params(text_pos: TextPosition, params: Vec<MoveParam>) -> Result<Self, ParseError> {
        assert!(params::compatible(Self::INFO.params, &params));
        let code_str = params[3].value.as_str(text_pos)?;
        let code = if code_str.is_empty() {
            None
        } else {
            let (_rest, code) = parse_code(code_str, text_pos)?;
            Some(code)
        };

        Ok(Self {
            text_pos,
            pos: Position::from_params(&params[0], &params[1], text_pos)?,
            dir: Direction(params[2].value.as_i32(text_pos)? as u32),
            code,
        })
    }
}

impl Move for Warp {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Warp)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
            param!("dir" = (self.dir.0 as i32)),
            param!(
                "code" = (match &self.code {
                    Some(code) => format!("{code}"),
                    None => String::new(),
                })
            ),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn transition(&self) -> Transition {
        Transition {
            spatial: SpatialTransition::Absolute {
                pos: self.pos,
                dir: self.dir,
            },
            code: self.code,
        }
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        _opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        doc
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        Some(Bounds {
            top_left: self.pos,
            bottom_right: self.pos,
        })
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        Box::new(Self {
            code: self.code.map(|code| code.opposite()),
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
