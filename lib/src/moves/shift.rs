// Copyright 2024-2025 David Drysdale

//! Pseudo-move definition for moving skater relative to current location.

use crate::{
    moves::{self, parse_code, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    Bounds, Code, Document, Move, MoveParam, ParseError, Position, RenderOptions, Rotation, Skater,
    SpatialTransition, SvgId, TextPosition, Transition,
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shift {
    text_pos: TextPosition,
    delta: Position,
    rotate: i32,
    code: Option<Code>,
}

impl Shift {
    pub const MOVE: &'static str = "Shift";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Shift),
        summary: "Move skater relative to current location and direction",
        example: "Shift[fwd=50,side=50,rotate=270]",
        visible: false,
        params: &[
            params::Info {
                name: "fwd",
                doc: "Distance to shift forward in direction of travel",
                default: Value::Number(0), // in cm
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "side",
                doc: "Distance to shift sidewards relative to direction of travel",
                default: Value::Number(0), // in cm
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "rotate",
                doc: "Rotation (clockwise) to perform, in degrees",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "code",
                doc: "Foot code to start with after shift",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
        ],
    };

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, ParseError> {
        Ok(Box::new(Self::new(input, text_pos)?))
    }

    pub fn new(input: &str, text_pos: TextPosition) -> Result<Self, ParseError> {
        let Some(rest) = input.strip_prefix(Self::INFO.name) else {
            return Err(ParseError {
                pos: text_pos,
                msg: format!("Missing expected prefix {}", Self::INFO.name),
            });
        };
        let params = params::populate(Self::INFO.params, rest, text_pos)?;
        Self::from_params(text_pos, params)
    }

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
            // Note that `fwd` is first, and is in (relative) y-direction.
            delta: Position::from_params(&params[1], &params[0], text_pos)?,
            rotate: params[2].value.as_i32(text_pos)?,
            code,
        })
    }
}

impl Move for Shift {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Shift)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("fwd" = (self.delta.y as i32)),
            param!("side" = (self.delta.x as i32)),
            param!(self.rotate),
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
    fn expanded_text(&self) -> String {
        let params = params::to_expanded(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn transition(&self) -> Transition {
        Transition {
            spatial: SpatialTransition::Relative {
                delta: self.delta,
                rotate: Rotation(self.rotate),
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
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        let after = *before + self.delta;
        Some(Bounds {
            top_left: after.pos,
            bottom_right: after.pos,
        })
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        Box::new(Self {
            delta: Position {
                x: -self.delta.x,
                y: self.delta.y,
            },
            rotate: -self.rotate,
            code: self.code.map(|code| code.opposite()),
            text_pos: self.text_pos.at_repeat(repeat),
        })
    }
    fn box_clone(&self, repeat: Option<usize>) -> Box<dyn Move> {
        let mut copy = self.clone();
        copy.text_pos = self.text_pos.at_repeat(repeat);
        Box::new(copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{code, pos};

    #[test]
    fn test_params() {
        let tests = [
            ("Shift", pos!(0, 0), 0, None),
            ("Shift[fwd=20,side=30,rotate=90]", pos!(30, 20), 90, None),
            (
                "Shift[fwd=20,side=30,code=\"LFO\"]",
                pos!(30, 20),
                0,
                Some(code!(LFO)),
            ),
        ];
        let text_pos = TextPosition::default();

        for (text, delta, rotate, code) in tests {
            let want = Shift {
                text_pos,
                delta,
                rotate,
                code,
            };
            let got = Shift::new(text, text_pos).unwrap();
            assert_eq!(got, want, "for input '{text}'");
            let regen = got.text();
            assert_eq!(text, regen);
        }
    }
}
