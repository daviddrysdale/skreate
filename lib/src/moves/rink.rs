//! Pseudo-move definition for rink description.

use crate::{
    moves, param, params, params::Value, parser, path, pos, Bounds, Move, MoveParam, Position,
    RenderOptions, Skater, SvgId, TextPosition,
};
use svg::node::element::{Circle, ClipPath, Group, Rectangle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rink {
    text_pos: TextPosition,
    width: i32,
    length: i32,
    show_centre_line: bool,
    centre_circle: Option<i32>,
    show_centre_faceoff: bool,
    mid_lines: Option<i32>,
    goal_lines: Option<i32>,
    show_goals: bool,
    show_faceoffs: bool,
}

impl Rink {
    pub const MOVE: &'static str = "Rink";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        summary: "Rink depiction",
        example: "Rink",
        visible: true,
        params: &[
            params::Info {
                name: "width",
                doc: "Rink width in centimetres",
                default: Value::Number(30 * 100), // in cm
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "length",
                doc: "Rink length in centimetres",
                default: Value::Number(61 * 100), // in cm
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "centre-line",
                doc: "Whether to show the centre line",
                default: Value::Boolean(true),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "centre-circle",
                doc: "Size of the centre circle in centimetre, 0 to omit",
                default: Value::Number(400), // diameter in cm, <= 0 to omit
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "centre-faceoff",
                doc: "Whether to show the centre face-off",
                default: Value::Boolean(true),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "mid-lines",
                doc: "Location of mid-lines in centimetres from the centre line; 0 to omit",
                default: Value::Number(800), // distance from centre in cm, <= 0 to omit
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "goal-lines",
                doc: "Location of goal lines in centimetres from the ends; 0 to omit",
                default: Value::Number(0), // cm from ends, 0=absent
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "goals",
                doc: "Whether to show the goals",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "faceoffs",
                doc: "Whether to show the face-offs",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
        ],
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        params: Vec<MoveParam>,
    ) -> Result<Self, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let to_opt_i32 = |param: &MoveParam| {
            let val = param.value.as_i32(input).unwrap();
            if val > 0 {
                Some(val)
            } else {
                None
            }
        };
        Ok(Self {
            text_pos,
            width: params[0].value.as_i32(input)?,
            length: params[1].value.as_i32(input)?,
            show_centre_line: params[2].value.as_bool(input)?,
            centre_circle: to_opt_i32(&params[3]),
            show_centre_faceoff: params[4].value.as_bool(input)?,
            mid_lines: to_opt_i32(&params[5]),
            goal_lines: to_opt_i32(&params[6]),
            show_goals: params[7].value.as_bool(input)?,
            show_faceoffs: params[8].value.as_bool(input)?,
        })
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
        let from_opt_i32 = |val: Option<i32>| val.unwrap_or(0);
        vec![
            param!(self.width),
            param!(self.length),
            param!("show-centre-line" = self.show_centre_line),
            param!("centre-circle" = from_opt_i32(self.centre_circle)),
            param!("centre-faceoff" = self.show_centre_faceoff),
            param!("mid-lines" = from_opt_i32(self.mid_lines)),
            param!("goal-lines" = from_opt_i32(self.goal_lines)),
            param!("goals" = self.show_goals),
            param!("faceoffs" = self.show_faceoffs),
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
        Some(Bounds {
            top_left: pos!(0, 0),
            bottom_right: pos!(self.width as i64, self.length as i64),
        })
    }
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
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
                grp = grp.add(
                    path!("M 0,{0} l {1},0", self.length / 2, self.width)
                        .set("style", "stroke:red;"),
                );
            } else {
                grp = grp.add(
                    path!("M {0},0 l 0,{1}", self.width / 2, self.length)
                        .set("style", "stroke:red;"),
                );
            }
        }
        if let Some(radius) = self.centre_circle {
            grp = grp.add(
                Circle::new()
                    .set("cx", self.width / 2)
                    .set("cy", self.length / 2)
                    .set("r", radius)
                    .set("style", "stroke:red;"),
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
        vec![(SvgId(self.text()), grp)]
    }
}
