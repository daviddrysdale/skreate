//! Pseudo-move definition for rink description.

use crate::{
    param, params, params::Value, path, Bounds, Code, Direction, Edge, Foot, Input, Label, Move,
    MoveParam, OwnedInput, ParseError, Position, RenderOptions, Skater, SkatingDirection,
    Transition,
};
use svg::node::element::{Circle, ClipPath, Group, Rectangle};

pub struct Rink {
    input: OwnedInput,
    width: i32,
    length: i32,
    start: Position,
    start_dir: Direction,
    show_centre_line: bool,
    centre_circle: Option<i32>,
    show_centre_faceoff: bool,
    mid_lines: Option<i32>,
    goal_lines: Option<i32>,
    show_goals: bool,
    show_faceoffs: bool,
}

impl Rink {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "width",
            default: Value::Number(30 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "length",
            default: Value::Number(61 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "start-x",
            default: Value::Number(2 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "start-y",
            default: Value::Number(2 * 100), // in cm
            range: params::Range::StrictlyPositive,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "start-dir",
            default: Value::Number(0),
            range: params::Range::Positive,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "centre-line",
            default: Value::Number(1), // 1=present, 0=absent
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "centre-circle",
            default: Value::Number(900), // diameter in cm, <= 0 to omit
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "centre-faceoff",
            default: Value::Number(0), // 1=present, 0=absent
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "mid-lines",
            default: Value::Number(17660 / 2), // distance from centre in cm, <= 0 to omit
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "goal-lines",
            default: Value::Number(400), // cm from ends, 0=absent
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "goals",
            default: Value::Number(0), // 1=present, 0=absent
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "faceoffs",
            default: Value::Number(0), // 1=present, 0=absent
            range: params::Range::Any,
            short: params::Abbrev::None,
        },
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
        let Some(rest) = input.text.strip_prefix("Rink") else {
            return Err(ParseError {
                pos: input.pos,
                msg: "No Rink prefix".to_string(),
            });
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(|msg| ParseError {
            pos: input.pos,
            msg,
        })?;
        let to_bool = |param: &MoveParam| param.value.as_i32().unwrap() > 0;
        let to_opt_i32 = |param: &MoveParam| {
            let val = param.value.as_i32().unwrap();
            if val > 0 {
                Some(val)
            } else {
                None
            }
        };

        Ok(Box::new(Self {
            input: input.owned(),
            width: params[0].value.as_i32().unwrap(),
            length: params[1].value.as_i32().unwrap(),
            start: Position {
                x: params[2].value.as_i32().unwrap() as i64,
                y: params[3].value.as_i32().unwrap() as i64,
            },
            start_dir: Direction(params[4].value.as_i32().unwrap() as u32),
            show_centre_line: to_bool(&params[5]),
            centre_circle: to_opt_i32(&params[6]),
            show_centre_faceoff: to_bool(&params[7]),
            mid_lines: to_opt_i32(&params[8]),
            goal_lines: to_opt_i32(&params[9]),
            show_goals: to_bool(&params[10]),
            show_faceoffs: to_bool(&params[11]),
        }))
    }
    fn rounding(&self) -> i32 {
        let dim = std::cmp::min(self.width, self.length);
        std::cmp::min(dim / 4, 850)
    }
    fn portrait(&self) -> bool {
        self.width < self.length
    }
    #[allow(dead_code)]
    fn landscape(&self) -> bool {
        self.width > self.length
    }
}

impl Move for Rink {
    fn params(&self) -> Vec<MoveParam> {
        let from_bool = |val| {
            if val {
                1
            } else {
                0
            }
        };
        let from_opt_i32 = |val: Option<i32>| val.unwrap_or(-1);
        vec![
            param!(self.width),
            param!(self.length),
            param!("start-x" = (self.start.x as i32)),
            param!("start-y" = (self.start.y as i32)),
            param!("start_dir" = (self.start_dir.0 as i32)),
            param!("show-centre-line" = from_bool(self.show_centre_line)),
            param!("centre-circle" = from_opt_i32(self.centre_circle)),
            param!("centre-faceoff" = from_bool(self.show_centre_faceoff)),
            param!("mid-lines" = from_opt_i32(self.mid_lines)),
            param!("goal-lines" = from_opt_i32(self.goal_lines)),
            param!("goals" = from_bool(self.show_goals)),
            param!("faceoffs" = from_bool(self.show_faceoffs)),
        ]
    }
    fn start(&self) -> Code {
        Code {
            foot: Foot::Both,
            dir: SkatingDirection::Forward,
            edge: Edge::Flat,
        }
    }
    fn end(&self) -> Code {
        self.start()
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("Rink {params}")
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, _from: Code) -> Transition {
        Transition {
            delta: Default::default(),
            rotate: Default::default(),
            code: self.start(),
        }
    }
    fn transition(&self) -> Transition {
        Transition {
            delta: self.start,
            rotate: crate::Rotation(self.start_dir.0 as i32),
            code: self.start(),
        }
    }
    fn encompass_bounds(&self, skater: &Skater, _include_pre: bool, bounds: &mut Bounds) -> Skater {
        bounds.encompass(&Position { x: 0, y: 0 });
        bounds.encompass(&Position {
            x: self.width as i64,
            y: self.length as i64,
        });
        *skater + self.transition()
    }
    fn def(&self, _opts: &RenderOptions) -> Group {
        let rink_rect = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", self.width)
            .set("height", self.length)
            .set("rx", self.rounding())
            .set("ry", self.rounding());
        let clip_path = ClipPath::new() // TODO fix clip-path
            .set("id", "clip-rink")
            .add(rink_rect.clone());
        let mut grp = Group::new().add(clip_path).add(rink_rect.set(
            "style",
            format!("{} clip-path:url(#clip-rink)", crate::STYLE_DEF),
        ));
        if self.show_centre_line {
            if self.portrait() {
                grp = grp.add(path!("M 0,{0} l {1},0", self.length / 2, self.width));
            } else {
                grp = grp.add(path!("M {0},0 l 0,{1}", self.width / 2, self.length));
            }
        }
        if let Some(radius) = self.centre_circle {
            grp = grp.add(
                Circle::new()
                    .set("cx", self.width / 2)
                    .set("cy", self.length / 2)
                    .set("r", radius),
            )
        }
        if self.show_centre_faceoff {
            grp = grp.add(
                Circle::new()
                    .set("cx", self.width / 2)
                    .set("cy", self.length / 2)
                    .set("r", 2)
                    .set("style", "fill: black;"),
            )
        }
        // TODO: mid and goal lines need to clip to the rounded-rectangle
        if let Some(dist) = self.mid_lines {
            if self.portrait() {
                grp = grp
                    .add(
                        path!("M 0,{0} l {1},0", (self.length / 2) - dist, self.width)
                            .set("style", "stroke: blue;"),
                    )
                    .add(
                        path!("M 0,{0} l {1},0", (self.length / 2) + dist, self.width)
                            .set("style", "stroke: blue;"),
                    );
            } else {
                grp = grp
                    .add(
                        path!("M {0},0 l 0,{1}", (self.width / 2) - dist, self.length)
                            .set("style", "stroke: blue;"),
                    )
                    .add(
                        path!("M {0},0 l 0,{1}", (self.width / 2) + dist, self.length)
                            .set("style", "stroke: blue;"),
                    );
            }
        }
        if let Some(dist) = self.goal_lines {
            if self.portrait() {
                grp = grp
                    .add(path!("M 0,{0} l {1},0", dist, self.width))
                    .add(path!("M 0,{0} l {1},0", self.length - dist, self.width));
            } else {
                grp = grp
                    .add(path!("M {0},0 l 0,{1}", dist, self.length))
                    .add(path!("M {0},0 l 0,{1}", self.width - dist, self.length));
            }
        }
        // TODO: render `show_goals`
        // TODO: render `show_faceoffs`
        grp
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        Vec::new()
    }
}
