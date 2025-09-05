// Copyright 2024-2025 David Drysdale

//! Move definition for simple curved edges.

use crate::{
    apply_style, bounds, code,
    moves::{self, MoveId, SkatingMoveId},
    param, params,
    params::Value,
    parser,
    parser::types::{parse_code, parse_pre_transition},
    path, pos, Bounds, Code, Edge, Label, Move, MoveParam, Percentage, Position, PreTransition,
    RenderOptions, Rotation, Skater, SpatialTransition, SvgId, TextPosition, Transition,
};
use log::error;
use std::borrow::Cow;
use std::f64::consts::PI;
use svg::node::element::Group;
use svg::node::element::TSpan as SvgTSpan;
use svg::node::element::Text as SvgText;
use svg::node::Text as NodeText;

#[derive(Debug, Clone)]
pub struct Curve {
    text_pos: TextPosition,
    pre_transition: PreTransition,
    code: Code,
    angle: i32,
    len: i32,
    label: Option<String>,
    transition_label: Option<String>,
    style: String,
    label_offset: Percentage,
}

impl Curve {
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Edge",
        id: MoveId::Skating(SkatingMoveId::Curve),
        summary: "Curving edge",
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
            params::Info {
                name: "label",
                doc: "Replacement label, used if non-empty",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "style",
                doc: "Style of line",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "transition-label",
                doc: "Replacement transition label, used if non-empty",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "label-offset",
                doc: "Amount to scale label offsets by, as a percentage, or -1 to use global value",
                default: Value::Number(-1),
                range: params::Range::Any,
                short: None,
            },
        ],
    };

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, parser::Error> {
        let (rest, pre_transition) = parse_pre_transition(input)?;
        let (rest, entry_code) = parse_code(rest)?;
        if entry_code.edge == Edge::Flat {
            return Err(parser::fail(input));
        }

        let params = params::populate(Self::INFO.params, rest)?;
        Ok(Box::new(Self::from_params(
            input,
            text_pos,
            pre_transition,
            entry_code,
            params,
        )?))
    }

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
    ) -> Result<Self, parser::Error> {
        // Reject invalid entry codes.
        if !matches!(
            entry_code,
            code!(LFO)
                | code!(RFI)
                | code!(LBI)
                | code!(RBO)
                | code!(RFO)
                | code!(LFI)
                | code!(RBI)
                | code!(LBO)
        ) {
            error!("{entry_code} not supported");
            return Err(parser::fail(input));
        }

        assert!(params::compatible(Self::INFO.params, &params));
        let label = params[2].value.as_str(input)?;
        let transition_label = params[4].value.as_str(input)?;

        Ok(Self {
            text_pos,
            pre_transition,
            code: entry_code,
            angle: params[0].value.as_i32(input)?,
            len: params[1].value.as_i32(input)?,
            label: if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            },
            transition_label: if transition_label.is_empty() {
                None
            } else {
                Some(transition_label.to_string())
            },
            style: params[3].value.as_str(input)?.to_string(),
            label_offset: Percentage(params[5].value.as_i32(input)?),
        })
    }

    /// Direction of increasing angle.
    fn sign(&self) -> i32 {
        match &self.code {
            code!(LFO) | code!(RFI) | code!(LBI) | code!(RBO) => -1,
            code!(RFO) | code!(LFI) | code!(RBI) | code!(LBO) => 1,
            _ => unreachable!("sign for {:?} hit despite constructor check", self.code),
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
    fn id(&self) -> MoveId {
        MoveId::Skating(SkatingMoveId::Curve)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.angle),
            param!(self.len),
            param!("label" = (self.label.clone().unwrap_or("".to_string()))),
            param!(self.style),
            param!("transition-label" = (self.transition_label.clone().unwrap_or("".to_string()))),
            param!("label-offset" = self.label_offset.0),
        ]
    }
    fn start(&self) -> Option<Code> {
        Some(self.code)
    }
    fn text(&self) -> String {
        let prefix = self.pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &self.params());
        format!("{prefix}{}{suffix}", self.code)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
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
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        let r = self.radius() as i64;
        let big = if self.angle >= 180 { 1 } else { 0 };
        let sweep = if self.sign() == -1 { 0 } else { 1 };
        let Position { x, y } = self.endpoint();

        let mut path = path!("M 0,0 a {r},{r} 0 {big} {sweep} {x},{y}");
        path = apply_style(path, &self.style);
        vec![(SvgId(self.text()), Group::new().add(path))]
    }
    fn labels(&self, opts: &RenderOptions) -> Vec<Label> {
        let font_size = opts.font_size() as i64;

        let mid_pt = self.percent_point(50);
        let half_theta = (self.sign() * self.angle) as f64 * PI / (2.0 * 180.0); // radians
        let distance = (-3 * font_size) as f64 * self.sign() as f64;

        let text = match &self.label {
            Some(label) => label.to_string(),
            None => format!("{}", self.code),
        };
        let display = opts.count.is_some() || !text.trim().is_empty();

        let svg_text = if let Some(count) = opts.count {
            timing_text(count.0).add(NodeText::new(text))
        } else {
            SvgText::new(text)
        };
        let label_offset = self.label_offset.for_opts(opts);

        let mut labels = vec![Label {
            display,
            text: svg_text,
            pos: mid_pt
                + pos!(
                    (distance * label_offset * half_theta.cos()) as i64,
                    (distance * label_offset * half_theta.sin()) as i64
                ),
        }];
        if let Some(duration) = opts.duration {
            labels.push(Label {
                display: true,
                text: timing_text(duration.0),
                pos: mid_pt
                    + pos!(
                        (-distance * label_offset * half_theta.cos()) as i64,
                        (-distance * label_offset * half_theta.sin()) as i64
                    ),
            });
        }

        let transition: Option<&str> = if self.transition_label.is_some() {
            self.transition_label.as_deref()
        } else {
            self.pre_transition.label()
        };

        if let Some(transition) = transition {
            // Assume that 5% along the curve is still pretty much vertical,
            // so the pre-transition label can just be inset horizontally.
            let early_pt = self.percent_point(5);
            let text = transition.to_string();
            labels.push(Label {
                display: !text.trim().is_empty(),
                text: SvgText::new(text),
                pos: early_pt + pos!(self.sign() as i64 * 2 * font_size, 0),
            });
        }
        labels
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        Box::new(Self {
            code: self.code.opposite(),
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

fn timing_text(val: i32) -> SvgText {
    SvgText::new("").add(
        SvgTSpan::new(format!("{val}"))
            .set("font-weight", "bolder")
            .set("fill", "purple")
            .set("stroke", "purple"),
    )
}
