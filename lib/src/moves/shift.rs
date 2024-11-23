//! Pseudo-move definition for moving skater relative to current location.

use super::Error;
use crate::{
    moves, param, params, params::Value, parse_code, Bounds, Code, Document, Input, Move,
    MoveParam, OwnedInput, Position, RenderOptions, Rotation, Skater, SpatialTransition,
    Transition,
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shift {
    input: OwnedInput,
    delta: Position,
    rotate: i32,
    code: Option<Code>,
}

impl Shift {
    pub const INFO: moves::Info = moves::Info {
        name: "Shift",
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

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        Ok(Box::new(Self::new(input)?))
    }

    pub fn new(input: &Input) -> Result<Self, Error> {
        let Some(rest) = input.text.strip_prefix(Self::INFO.name) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::INFO.params, rest).map_err(Error::Failed)?;
        let code_str = params[3].value.as_str().map_err(Error::Failed)?;
        let code = if code_str.is_empty() {
            None
        } else {
            let (code, _rest) = parse_code(code_str).map_err(Error::Failed)?;
            Some(code)
        };

        Ok(Self {
            input: input.owned(),
            // Note that `fwd` is first, and is in (relative) y-direction.
            delta: Position::from_params(&params[1], &params[0]),
            rotate: params[2].value.as_i32().unwrap(),
            code,
        })
    }
}

impl Move for Shift {
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
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
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
    fn render(&self, doc: Document, _start: &Skater, _opts: &mut RenderOptions) -> Document {
        doc
    }
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        let after = *before + self.delta;
        Some(Bounds {
            top_left: after.pos,
            bottom_right: after.pos,
        })
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

        for (text, delta, rotate, code) in tests {
            let input = Input {
                pos: crate::TextPosition::default(),
                text,
            };
            let want = Shift {
                input: input.owned(),
                delta,
                rotate,
                code,
            };
            let got = Shift::new(&input).unwrap();
            assert_eq!(got, want, "for input '{text}'");
            let regen = got.text();
            assert_eq!(text, regen);
        }
    }
}
