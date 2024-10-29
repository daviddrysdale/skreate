//! Pseudo-move definition for moving skater to new position.

use super::Error;
use crate::{
    param, params, params::Value, parse_code, Bounds, Code, Direction, Document, Input, Move,
    MoveParam, OwnedInput, Position, RenderOptions, Skater, SpatialTransition, Transition,
};
use std::borrow::Cow;

const NAME: &str = "Warp";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    input: OwnedInput,
    pos: Position,
    dir: Direction,
    code: Option<Code>,
}

impl Warp {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "x",
            default: Value::Number(0), // in cm
            range: params::Range::StrictlyPositive,
            short: None,
        },
        params::Info {
            name: "y",
            default: Value::Number(0), // in cm
            range: params::Range::StrictlyPositive,
            short: None,
        },
        params::Info {
            name: "dir",
            default: Value::Number(0),
            range: params::Range::Positive,
            short: None,
        },
        params::Info {
            name: "code",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
    ];

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        Ok(Box::new(Self::new(input)?))
    }

    pub fn new(input: &Input) -> Result<Self, Error> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(Error::Unrecognized);
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(Error::Failed)?;
        let code_str = params[3].value.as_str().map_err(Error::Failed)?;
        let code = if code_str.is_empty() {
            None
        } else {
            let (code, _rest) = parse_code(code_str).map_err(Error::Failed)?;
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
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME}{params}")
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
    fn render(&self, doc: Document, _start: &Skater, _opts: &mut RenderOptions) -> Document {
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
    use crate::code;

    #[test]
    fn test_params() {
        let tests = [
            ("Warp", Position { x: 0, y: 0 }, Direction(0), None),
            (
                "Warp[x=20,y=30,dir=90]",
                Position { x: 20, y: 30 },
                Direction(90),
                None,
            ),
            (
                "Warp[x=20,y=30,code=\"LFO\"]",
                Position { x: 20, y: 30 },
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
