//! Move definition for simple curved edges.

use super::Error;
use crate::{
    bounds, code, moves, param, params, params::Value, parse_code, path, pos, Bounds, Code, Input,
    Label, Move, MoveParam, OwnedInput, Position, PreTransition, RenderOptions, Rotation, Skater,
    SpatialTransition, Transition,
};
use std::f64::consts::PI;
use svg::node::element::Group;

pub struct Curve {
    input: OwnedInput,
    pre_transition: PreTransition,
    code: Code,
    angle: i32,
    len: i32,
}

impl Curve {
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        summary: "Curving edge",
        name: "Edge",
        example: "LFO",
        visible: true,
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation from start to finish, in degrees",
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
        ],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (pre_transition, rest) = PreTransition::parse(input.text);
        let (code, rest) = parse_code(rest).map_err(|_msg| Error::Unrecognized)?;

        let params =
            params::populate(Self::INFO.params, rest).map_err(|_msg| Error::Unrecognized)?;

        Ok(Box::new(Self {
            input: input.owned(),
            pre_transition,
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

    /// Point of the arc some percentage along the way, starting at 0,0 facing 0.
    fn percent_point(&self, percent: i32) -> Position {
        let r = self.radius();
        let theta = self.angle as f64 * PI / 180.0; // radians
        let theta = (percent as f64 / 100.0) * theta;
        let (x, y) = if self.sign() == 1 {
            (r * theta.cos() - r, r * theta.sin())
        } else {
            (r - r * theta.cos(), r * theta.sin())
        };
        pos!(x as i64, y as i64)
    }

    /// End point of the arc, starting at 0,0 facing 0.
    fn endpoint(&self) -> Position {
        let r = self.radius();
        let theta = self.angle as f64 * PI / 180.0; // radians
        let (x, y) = if self.sign() == 1 {
            (r * theta.cos() - r, r * theta.sin())
        } else {
            (r - r * theta.cos(), r * theta.sin())
        };
        pos!(x as i64, y as i64)
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
        let prefix = self.pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &self.params());
        format!("{prefix}{}{suffix}", self.code)
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, from: Code) -> Transition {
        if let Some(start) = self.start() {
            self.pre_transition.perform(from, start)
        } else {
            Transition::default()
        }
    }
    fn transition(&self) -> Transition {
        Transition {
            spatial: SpatialTransition::Relative {
                delta: self.endpoint(),
                rotate: Rotation(self.angle * self.sign()),
            },
            code: Some(self.code),
        }
    }
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        let mut bounds = bounds!(before.pos.x, before.pos.y => before.pos.x, before.pos.y);

        // Calculate 100 points on the curve and ensure they're all included in the bounds.
        // TODO: replace this with some cunning trigonometry.
        for percent in 0..=100 {
            // Figure a point some way along the curve starting from 0,0 direction 0.
            let curve_pt = self.percent_point(percent);

            // Translate and rotate relative to the actual start point.
            let mid = *before + curve_pt;
            bounds.encompass(&mid.pos);
        }

        Some(bounds)
    }
    fn def(&self, _opts: &mut RenderOptions) -> Option<Group> {
        let r = self.radius() as i64;
        let big = if self.angle >= 180 { 1 } else { 0 };
        let sweep = if self.sign() == -1 { 0 } else { 1 };
        let Position { x, y } = self.endpoint();

        Some(Group::new().add(path!("M 0,0 a {r},{r} 0 {big} {sweep} {x},{y}")))
    }
    fn labels(&self, opts: &RenderOptions) -> Vec<Label> {
        let font_size = opts.font_size() as i64;

        let mid_pt = self.percent_point(50);
        let half_theta = (self.sign() * self.angle) as f64 * PI / (2.0 * 180.0); // radians
        let distance = (-3 * font_size) as f64 * self.sign() as f64;
        let mut labels = vec![Label {
            text: format!("{}", self.code),
            pos: mid_pt
                + pos!(
                    (distance * half_theta.cos()) as i64,
                    (distance * half_theta.sin()) as i64
                ),
        }];

        if let Some(transition) = self.pre_transition.label() {
            // Assume that 5% along the curve is still pretty much vertical,
            // so the pre-transition label can just be inset horizontally.
            let early_pt = self.percent_point(5);
            labels.push(Label {
                text: transition.to_string(),
                pos: early_pt + pos!(self.sign() as i64 * 2 * font_size, 0),
            });
        }
        labels
    }
}
