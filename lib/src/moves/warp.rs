//! Pseudo-move definition for moving skater to new position.

use super::Error;
use crate::{
    moves, param, params, params::Value, parser::types::parse_code, Bounds, Code, Direction,
    Document, Input, Move, MoveParam, OwnedInput, Position, RenderOptions, Skater,
    SpatialTransition, SvgId, Transition,
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    input: OwnedInput,
    pos: Position,
    dir: Direction,
    code: Option<Code>,
}

impl Warp {
    pub const INFO: moves::Info = moves::Info {
        name: "Warp",
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
            let (_rest, code) = parse_code(code_str)?;
            Some(code)
        };

        Ok(Self {
            input: input.owned(),
            pos: Position::from_params(&params[0], &params[1]),
            dir: Direction(params[2].value.as_i32().unwrap() as u32),
            code,
        })
    }
}

impl Move for Warp {
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
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{code, pos};

    #[test]
    fn test_params() {
        let tests = [
            ("Warp", pos!(0, 0), Direction(0), None),
            ("Warp[x=20,y=30,dir=90]", pos!(20, 30), Direction(90), None),
            (
                "Warp[x=20,y=30,code=\"LFO\"]",
                pos!(20, 30),
                Direction(0),
                Some(code!(LFO)),
            ),
        ];

        for (text, pos, dir, code) in tests {
            let input = Input {
                pos: crate::TextPosition::default(),
                text,
            };
            let want = Warp {
                input: input.owned(),
                pos,
                dir,
                code,
            };
            let got = Warp::new(&input).unwrap();
            assert_eq!(got, want, "for input '{text}'");
            let regen = got.text();
            assert_eq!(text, regen);
        }
    }
}
