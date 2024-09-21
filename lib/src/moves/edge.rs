//! Move definition for simple curved edges.

use super::{cross_transition, pre_transition, Error};
use crate::{
    code, param, params, params::Value, parse_code, parse_transition_prefix, path, Code, Edge,
    Foot, Input, Label, Move, MoveParam, OwnedInput, Position, RenderOptions, Rotation,
    SkatingDirection, Transition,
};
use std::f64::consts::PI;
use svg::node::element::Group;

pub struct Curve {
    input: OwnedInput,
    cross_transition: bool,
    code: Code,
    angle: i32,
    len: i32,
}

impl Curve {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "angle",
            default: Value::Number(20),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::GreaterLess(params::Detents {
                add1: 60,
                add2: 110,
                add3: 180,
                less1: 15,
                less2: 10,
                less3: 5,
            })),
        },
        params::Info {
            name: "len",
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
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (cross_transition, rest) = parse_transition_prefix(input.text);
        let (code, rest) = parse_code(rest).map_err(|_msg| Error::Unrecognized)?;

        let params =
            params::populate(Self::PARAMS_INFO, rest).map_err(|_msg| Error::Unrecognized)?;

        Ok(Box::new(Self {
            input: input.owned(),
            cross_transition,
            code,
            angle: params[0].value.as_i32().unwrap(),
            len: params[1].value.as_i32().unwrap(),
        }))
    }

    /// Direction of increasing angle.
    fn sign(&self) -> i32 {
        match &self.code {
            code!(LFO) | code!(RFI) | code!(LBI) | code!(RBO) => -1,
            code!(RFO) | code!(LFI) | code!(RBI) | code!(LBO) => 1,
            _ => panic!("sign for {:?} ?", self.code),
        }
    }

    /// Radius of the circle for which this is an arc.
    fn radius(&self) -> f64 {
        // If `angle` were 360, arc `len` would be 2πr.
        // For general angle, arc `len` == (angle/360)*2πr.
        // Therefore r == len * 360  / (angle * 2π).
        self.len as f64 * 180.0 / (self.angle as f64 * PI)
    }

    /// End point of the arc
    fn endpoint(&self) -> Position {
        let r = self.radius();
        let theta = self.angle as f64 * PI / 180.0; // radians
        let (x, y) = if self.sign() == 1 {
            (r * theta.cos() - r, r * theta.sin())
        } else {
            (r - r * theta.cos(), r * theta.sin())
        };
        Position {
            x: x as i64,
            y: y as i64,
        }
    }
}

impl Move for Curve {
    fn params(&self) -> Vec<MoveParam> {
        vec![param!(self.angle), param!(self.len)]
    }
    fn start(&self) -> Option<Code> {
        Some(self.code)
    }
    fn text(&self) -> String {
        let prefix = match (self.cross_transition, self.code.dir) {
            (false, _) => "",
            (true, SkatingDirection::Forward) => "xf-",
            (true, SkatingDirection::Backward) => "xb-",
        };
        let suffix = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{prefix}{}{suffix}", self.code)
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, from: Code) -> Transition {
        if let Some(start) = self.start() {
            if self.cross_transition {
                cross_transition(from, start)
            } else {
                pre_transition(from, start)
            }
        } else {
            Transition::default()
        }
    }
    fn transition(&self) -> Transition {
        Transition {
            delta: self.endpoint(),
            code: Some(self.code),
            rotate: Rotation(self.angle * self.sign()),
        }
    }
    fn def(&self, _opts: &RenderOptions) -> Option<Group> {
        let r = self.radius() as i64;
        let big = if self.angle >= 180 { 1 } else { 0 };
        let sweep = 0;
        let Position { x, y } = self.endpoint();
        Some(Group::new().add(path!("M 0,0 a {r},{r} 0 {big} {sweep} {x},{y}")))
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        let Position { x, y } = self.endpoint();
        vec![Label {
            text: format!("{}", self.code),
            pos: Position {
                x: x / 2 + 30,
                y: y / 2,
            },
        }]
    }
}
