//! Move definition for simple straight edges.

use super::{cross_transition, pre_transition, HW};
use crate::{
    param,
    params::{params_to_string, populate_params},
    Code, Edge, Foot, Input, Label, Move, MoveParam, OwnedInput, ParseError, Position,
    RenderOptions, Rotation, SkatingDirection, Transition,
};
use svg::node::element::{Group, Path};

pub struct StraightEdge {
    input: OwnedInput,
    cross_transition: bool,
    foot: Foot,
    dir: SkatingDirection,
    len: i32,
}

impl StraightEdge {
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
        let (cross_transition, rest) = if let Some(rest) = input.text.strip_prefix("xf-") {
            (true, rest)
        } else if let Some(rest) = input.text.strip_prefix("xb-") {
            (true, rest)
        } else {
            (false, input.text)
        };
        let (foot, rest) = if let Some(rest) = rest.strip_prefix('L') {
            (Foot::Left, rest)
        } else if let Some(rest) = rest.strip_prefix('R') {
            (Foot::Right, rest)
        } else if let Some(rest) = rest.strip_prefix('B') {
            (Foot::Both, rest)
        } else {
            return Err(ParseError {
                pos: input.pos,
                msg: "No foot recognized".to_string(),
            });
        };
        let (dir, rest) = if let Some(rest) = rest.strip_prefix('F') {
            (SkatingDirection::Forward, rest)
        } else if let Some(rest) = rest.strip_prefix('B') {
            (SkatingDirection::Backward, rest)
        } else {
            return Err(ParseError {
                pos: input.pos,
                msg: "No direction recognized".to_string(),
            });
        };

        let len = match rest {
            "" => 100,
            "+" => 125,
            "++" => 150,
            "+++" => 200,
            "-" => 80,
            "--" => 50,
            "---" => 25,
            rest => {
                let mut params = vec![param!(len = 100)];
                populate_params(&mut params, rest).map_err(|msg| ParseError {
                    pos: input.pos,
                    msg,
                })?;
                params[0].value
            }
        };

        Ok(Box::new(Self {
            input: input.owned(),
            cross_transition,
            foot,
            dir,
            len,
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
        let suffix = match self.len {
            100 => "".to_string(),
            125 => "+".to_string(),
            150 => "++".to_string(),
            200 => "+++".to_string(),
            80 => "-".to_string(),
            50 => "--".to_string(),
            25 => "---".to_string(),
            _ => params_to_string(&[param!(self.len)]),
        };
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
            Group::new().add(Path::new().set(
                "d",
                format!("M 0,0 m {HW},0 l 0,{0} m -{HW},-{0} l 0,{0}", self.len),
            ))
        } else {
            Group::new().add(Path::new().set("d", format!("M 0,0 l 0,{}", self.len)))
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
