// Copyright 2024-2025 David Drysdale

//! Pseudo-move definition for label positioned relative to current position.

use crate::{
    moves::{self, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    Bounds, Move, MoveParam, ParseError, Position, RenderOptions, Rotation, Skater,
    SpatialTransition, SvgId, TextPosition, Transition,
};
use std::borrow::Cow;
use svg::{node::element::Text, Document};

#[derive(Debug, Clone)]
pub struct Label {
    text_pos: TextPosition,
    text: String,
    delta: Position,
    font_size: Option<u32>,
    rotate: i32,
}

impl Label {
    pub const MOVE: &'static str = "Label";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Label),
        summary: "Diagram label relative to current position",
        example: "Label[text=\"CoE\",fwd=50,side=20]",
        visible: false,
        params: &[
            params::Info {
                name: "text",
                doc: "Label to display",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "fwd",
                doc: "Location of text relative to current position, in current direction",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "side",
                doc:
                    "Location of text relative to current position, sideways from current direction",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "font-size",
                doc: "Font size for label; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "rotate",
                doc: "Angle to rotate text by",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
        ],
    };

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, ParseError> {
        let Some(rest) = input.strip_prefix(Self::INFO.name) else {
            return Err(ParseError {
                pos: text_pos,
                msg: format!("Missing expected prefix {}", Self::INFO.name),
            });
        };
        let params = params::populate(Self::INFO.params, rest, text_pos)?;
        Ok(Box::new(Self::from_params(text_pos, params)?))
    }

    pub fn from_params(text_pos: TextPosition, params: Vec<MoveParam>) -> Result<Self, ParseError> {
        assert!(params::compatible(Self::INFO.params, &params));
        let font_size = params[3].value.as_i32(text_pos)?;

        Ok(Self {
            text_pos,
            text: params[0].value.as_str(text_pos)?.to_string(),
            delta: Position::from_params(&params[2], &params[1], text_pos)?,
            font_size: if font_size > 0 {
                Some(font_size as u32)
            } else {
                None
            },
            rotate: params[4].value.as_i32(text_pos)?,
        })
    }

    fn font_size(&self, opts: &RenderOptions) -> u32 {
        self.font_size.unwrap_or_else(|| opts.font_size())
    }
}

impl Move for Label {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Label)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("fwd" = (self.delta.y as i32)),
            param!("side" = (self.delta.x as i32)),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
            param!(self.rotate),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(
        &self,
        doc: Document,
        start: &Skater,
        opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        let delta = Transition {
            spatial: SpatialTransition::Relative {
                delta: self.delta,
                rotate: Rotation(0),
            },
            code: None,
        };
        let pos = *start + delta;
        let mut text = Text::new(self.text.clone())
            .set("x", pos.pos.x)
            .set("y", pos.pos.y)
            .set(
                "style",
                format!(
                    "stroke:black; fill:black; font-size:{}pt;",
                    self.font_size(opts)
                ),
            );
        if self.rotate != 0 {
            text = text.set(
                "transform",
                format!("rotate({},{},{})", self.rotate, pos.pos.x, pos.pos.y),
            )
        }
        if let Some(pos) = self.text_pos() {
            let unique_id = opts.next_unique_id(pos);
            text = text.set("id", unique_id);
        }
        doc.add(text)
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        Box::new(Self {
            delta: Position {
                x: -self.delta.x,
                y: self.delta.y,
            },
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
