// Copyright 2024-2025 David Drysdale

//! Move definition for simple curved edges.

use crate::{
    apply_style, code,
    moves::{self, parse_code, parse_pre_transition, MoveId, SkatingMoveId},
    param, params,
    params::Value,
    path, pos, Bounds, Centimetres, Code, Label, Move, MoveParam, ParseError, Percentage, Position,
    PreTransition, RenderOptions, Rotation, RotationDirection, Skater, SpatialTransition, SvgId,
    TextPosition, Transition,
};
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
    angle: Rotation,
    len: Centimetres,
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

    pub fn construct(input: &str, text_pos: TextPosition) -> Result<Box<dyn Move>, ParseError> {
        let (rest, pre_transition) = parse_pre_transition(input, text_pos)?;
        let (rest, entry_code) = parse_code(rest, text_pos)?;
        let params = params::populate(Self::INFO.params, rest, text_pos)?;
        Ok(Box::new(Self::from_params(
            input,
            text_pos,
            pre_transition,
            entry_code,
            params,
            &mut moves::Context::default(),
        )?))
    }

    pub fn from_params(
        _input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
        ctx: &mut moves::Context,
    ) -> Result<Self, ParseError> {
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
            return Err(ParseError {
                pos: text_pos,
                msg: format!("{entry_code} not supported"),
            });
        }

        assert!(params::compatible(Self::INFO.params, &params));
        let label = params[2].value.as_str(text_pos)?;
        let transition_label = params[4].value.as_str(text_pos)?;

        Ok(Self {
            text_pos,
            pre_transition,
            code: entry_code,
            angle: params[0].value.as_rotation(text_pos)?,
            len: params[1].value.as_cm(text_pos)?,
            label: if label.is_empty() {
                ctx.prev_label = Some(format!("{entry_code}"));
                None
            } else {
                ctx.prev_label = Some(label.to_string());
                Some(label.to_string())
            },
            transition_label: if transition_label.is_empty() {
                None
            } else {
                Some(transition_label.to_string())
            },
            style: params[3].value.as_str(text_pos)?.to_string(),
            label_offset: params[5].value.as_percent(text_pos)?,
        })
    }

    /// Direction of increasing angle.
    fn sign(&self) -> RotationDirection {
        match &self.code {
            code!(LFO) | code!(RFI) | code!(LBI) | code!(RBO) => RotationDirection::AntiClockwise,
            code!(RFO) | code!(LFI) | code!(RBI) | code!(LBO) => RotationDirection::Clockwise,
            _ => unreachable!("sign for {:?} hit despite constructor check", self.code),
        }
    }

    /// Radius of the circle for which this is an arc.
    fn radius(&self) -> f64 {
        radius(self.len, self.angle)
    }

    /// Point of the arc some percentage along the way, starting at 0,0 facing 0.
    fn percent_point(&self, percent: Percentage) -> Position {
        percent_point(self.len, self.angle, self.sign(), percent)
    }

    /// End point of the arc, starting at 0,0 facing 0.
    fn endpoint(&self) -> Position {
        endpoint(self.len, self.angle, self.sign())
    }
}

impl Move for Curve {
    fn id(&self) -> MoveId {
        MoveId::Skating(SkatingMoveId::Curve)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("angle" = self.angle.0),
            param!("len" = self.len.0 as i32),
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
    fn expanded_text(&self) -> String {
        let prefix = self.pre_transition.prefix();
        let suffix = params::to_expanded(Self::INFO.params, &self.params());
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
                rotate: self.angle * self.sign(),
            },
            code: Some(self.code),
        }
    }
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        let mut bounds = Bounds {
            top_left: before.pos,
            bottom_right: before.pos,
        };

        // Calculate 100 points on the curve and ensure they're all included in the bounds.
        // TODO: replace this with some cunning trigonometry.
        for percent in 0..=100 {
            // Figure a point some way along the curve starting from 0,0 direction 0.
            let curve_pt = self.percent_point(Percentage(percent));

            // Translate and rotate relative to the actual start point.
            let mid = *before + curve_pt;
            bounds.encompass(&mid.pos);
        }

        Some(bounds)
    }
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        let r = self.radius() as i64;
        let big = if self.angle.0 >= 180 { 1 } else { 0 };
        let sweep = if self.sign() == RotationDirection::AntiClockwise {
            0
        } else {
            1
        };
        let Position { x, y } = self.endpoint();

        let mut path = path!("M 0,0 a {r},{r} 0 {big} {sweep} {x},{y}");
        path = apply_style(path, &self.style);
        vec![(SvgId(self.text()), Group::new().add(path))]
    }
    fn labels(&self, opts: &RenderOptions) -> Vec<Label> {
        let font_size = opts.font_size().0 as i64;

        // Calculate the position and direction of the mid-point of the edge.
        let mid_pt = self.percent_point(Percentage(50));
        let half_theta = (self.angle * self.sign()).radians() / 2.0;

        // How far to the side should the label be?
        let distance = match self.sign() {
            RotationDirection::AntiClockwise => 3 * font_size,
            RotationDirection::Clockwise => -3 * font_size,
        } as f64;
        let label_offset_fraction = self.label_offset.for_opts(opts);
        let distance = label_offset_fraction * distance;

        // Calculate the label offset from the mid-point.
        let label_offset = pos!(
            (distance * half_theta.cos()) as i64,
            (distance * half_theta.sin()) as i64
        );

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

        let mut labels = vec![Label {
            display,
            text: svg_text,
            pos: mid_pt + label_offset,
        }];
        if let Some(duration) = opts.duration {
            // Put the duration label on the opposite side to the main label.
            labels.push(Label {
                display: true,
                text: timing_text(duration.0),
                pos: mid_pt - label_offset,
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
            let early_pt = self.percent_point(Percentage(5));
            let text = transition.to_string();
            let x_offset = match self.sign() {
                RotationDirection::Clockwise => 2 * font_size,
                RotationDirection::AntiClockwise => -2 * font_size,
            };
            labels.push(Label {
                display: !text.trim().is_empty(),
                text: SvgText::new(text),
                pos: early_pt + pos!(x_offset, 0),
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

/// Radius of the circle for which this is an arc, in centimetres.
pub(crate) fn radius(len: Centimetres, angle: Rotation) -> f64 {
    // If `angle` were 360, arc `len` would be 2πr.
    // For general angle, arc `len` == (angle/360)*2πr.
    // Therefore r == len * 360  / (angle * 2π).
    len.0 as f64 * 180.0 / (angle.0 as f64 * PI)
}

/// Point of the arc some percentage along the way, starting at 0,0 facing 0.
pub(crate) fn percent_point(
    len: Centimetres,
    angle: Rotation,
    sign: RotationDirection,
    percent: Percentage,
) -> Position {
    let r = radius(len, angle);
    let theta = percent.as_f64() * angle.radians();
    let (x, y) = if sign == RotationDirection::Clockwise {
        // Centre of arc is at (-r,0)
        (r * theta.cos() - r, r * theta.sin())
    } else {
        // Centre of arc is at (+r,0)
        (r - r * theta.cos(), r * theta.sin())
    };
    pos!(x as i64, y as i64)
}

/// End point of the arc, starting at 0,0 facing 0.
pub(crate) fn endpoint(len: Centimetres, angle: Rotation, sign: RotationDirection) -> Position {
    let r = radius(len, angle);
    let theta = angle.radians();
    let (x, y) = if sign == RotationDirection::Clockwise {
        // Centre of arc is at (-r,0)
        (r * theta.cos() - r, r * theta.sin())
    } else {
        // Centre of arc is at (+r,0)
        (r - r * theta.cos(), r * theta.sin())
    };
    pos!(x as i64, y as i64)
}
pub(crate) fn timing_text(val: i32) -> SvgText {
    SvgText::new("").add(
        SvgTSpan::new(format!("{val}"))
            .set("font-weight", "bolder")
            .set("fill", "purple")
            .set("stroke", "purple"),
    )
}
