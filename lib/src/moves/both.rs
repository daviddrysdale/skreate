// Copyright 2024-2026 David Drysdale

//! Move definition for two-footed curved edges.

use crate::{
    apply_style, code,
    moves::{
        self,
        edge::{endpoint, percent_point, radius, timing_text},
        MoveId, SkatingMoveId, HW,
    },
    param, params,
    params::Value,
    path, pos, Bounds, Centimetres, Code, Label, Move, MoveParam, ParseError, Percentage, Position,
    PreTransition, RenderOptions, Rotation, RotationDirection, Skater, SpatialTransition, SvgId,
    TextPosition, Transition,
};
use std::borrow::Cow;
use svg::node::element::Group;
use svg::node::element::Text as SvgText;
use svg::node::Text as NodeText;

#[derive(Debug, Clone)]
pub struct CurveBoth {
    text_pos: TextPosition,
    pre_transition: PreTransition,
    code: Code,
    angle: Rotation,
    len: Centimetres,
    left_label: Option<String>,
    right_label: Option<String>,
    transition_label: Option<String>,
    style: String,
    label_offset: Percentage,
}

impl CurveBoth {
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Both",
        id: MoveId::Skating(SkatingMoveId::CurveBoth),
        summary: "Curving two-footed edges",
        example: "BFL",
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
                name: "left-label",
                doc: "Replacement label for left foot, used if non-empty",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "right-label",
                doc: "Replacement label for left foot, used if non-empty",
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

    pub fn from_params(
        _input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        params: Vec<MoveParam>,
        _ctx: &mut moves::Context,
    ) -> Result<Self, ParseError> {
        // Reject invalid entry codes.
        if !matches!(
            entry_code,
            code!(BFL) | code!(BFR) | code!(BBL) | code!(BBR)
        ) {
            return Err(ParseError {
                pos: text_pos,
                msg: format!("{entry_code} not supported"),
            });
        }

        assert!(params::compatible(Self::INFO.params, &params));
        let left_label = params[2].value.as_str(text_pos)?;
        let right_label = params[3].value.as_str(text_pos)?;
        let transition_label = params[5].value.as_str(text_pos)?;

        Ok(Self {
            text_pos,
            pre_transition,
            code: entry_code,
            angle: params[0].value.as_rotation(text_pos)?,
            len: params[1].value.as_cm(text_pos)?,
            left_label: if left_label.is_empty() {
                None
            } else {
                Some(left_label.to_string())
            },
            right_label: if right_label.is_empty() {
                None
            } else {
                Some(right_label.to_string())
            },
            transition_label: if transition_label.is_empty() {
                None
            } else {
                Some(transition_label.to_string())
            },
            style: params[4].value.as_str(text_pos)?.to_string(),
            label_offset: params[6].value.as_percent(text_pos)?,
        })
    }

    /// Direction of increasing angle.
    fn sign(&self) -> RotationDirection {
        match &self.code {
            code!(BFL) | code!(BBL) => RotationDirection::AntiClockwise,
            code!(BFR) | code!(BBR) => RotationDirection::Clockwise,
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

    /// Move code for the left foot.
    fn left_code(&self) -> Code {
        match &self.code {
            code!(BFL) => code!(LFO),
            code!(BFR) => code!(LFI),
            code!(BBL) => code!(LBI),
            code!(BBR) => code!(LBO),
            _ => unreachable!("unexpected code {:?} despite constructor check", self.code),
        }
    }

    /// Move code for the right foot.
    fn right_code(&self) -> Code {
        match &self.code {
            code!(BFL) => code!(RFI),
            code!(BFR) => code!(RFO),
            code!(BBL) => code!(RBO),
            code!(BBR) => code!(RBI),
            _ => unreachable!("unexpected code {:?} despite constructor check", self.code),
        }
    }
}

impl Move for CurveBoth {
    fn id(&self) -> MoveId {
        MoveId::Skating(SkatingMoveId::Curve)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("angle" = self.angle.0),
            param!("len" = self.len.0 as i32),
            param!("left-label" = (self.left_label.clone().unwrap_or("".to_string()))),
            param!("right-label" = (self.right_label.clone().unwrap_or("".to_string()))),
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
        for percent in 0..=100 {
            // Figure a point some way along the curve starting from 0,0 direction 0.
            let curve_pt = self.percent_point(Percentage(percent));

            // Translate and rotate relative to the actual start point.
            let mid = *before + curve_pt;
            bounds.encompass(&mid.pos);
        }
        // TODO: expand the bounds slightly to allow for (-HW, HW)

        Some(bounds)
    }
    fn defs(&self, _opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        // Calculate parameters for a mid-curve (not rendered).
        let mid_r = Centimetres(self.radius() as i64);
        let big = if self.angle.0 >= 180 { 1 } else { 0 };
        let sweep = if self.sign() == RotationDirection::AntiClockwise {
            0
        } else {
            1
        };
        let mid_endpt = self.endpoint();

        let rf_start_x = -HW;
        let lf_start_x = HW;

        let inner_r = mid_r - HW;
        let outer_r = mid_r + HW;
        let (rf_r, lf_r) = if self.sign() == RotationDirection::Clockwise {
            (inner_r, outer_r)
        } else {
            (outer_r, inner_r)
        };

        // Calculate the direction at the endpoint in radians clockwise from vertically down.
        let theta = (self.angle * self.sign()).radians();

        // Calculate the perpendicular offset from the mid endpoint.
        let hw = HW.0 as f64;
        let perp_offset = pos!((hw * theta.cos()) as i64, (hw * theta.sin()) as i64);
        let rf_endpt = mid_endpt - perp_offset;
        let lf_endpt = mid_endpt + perp_offset;
        let Position { x: lf_x, y: lf_y } = lf_endpt;
        let Position { x: rf_x, y: rf_y } = rf_endpt;
        let mut path = path!(
            "M {rf_start_x},0 A {rf_r},{rf_r} 0 {big} {sweep} {rf_x},{rf_y} M {lf_start_x},0 A {lf_r},{lf_r} 0 {big} {sweep} {lf_x},{lf_y}",
        );

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
            RotationDirection::AntiClockwise => 4 * font_size,
            RotationDirection::Clockwise => -4 * font_size,
        } as f64;
        let label_offset_fraction = self.label_offset.for_opts(opts);
        let distance = label_offset_fraction * distance;

        // Calculate the label offset from the mid-point.
        let label_offset = pos!(
            (distance * half_theta.cos()) as i64,
            (distance * half_theta.sin()) as i64
        );

        let lf_text = match &self.left_label {
            Some(label) => label.to_string(),
            None => format!("{}", self.left_code()),
        };
        let rf_text = match &self.right_label {
            Some(label) => label.to_string(),
            None => format!("{}", self.right_code()),
        };
        let display =
            opts.count.is_some() || !lf_text.trim().is_empty() || !rf_text.trim().is_empty();

        let svg_lf_text = if let Some(count) = opts.count {
            timing_text(count.0).add(NodeText::new(lf_text))
        } else {
            SvgText::new(lf_text)
        };
        let svg_rf_text = if let Some(count) = opts.count {
            timing_text(count.0).add(NodeText::new(rf_text))
        } else {
            SvgText::new(rf_text)
        };
        let (lf_label_offset, rf_label_offset) = match &self.code {
            code!(BFL) => (label_offset, -label_offset),
            code!(BFR) => (-label_offset, label_offset),
            code!(BBL) => (-label_offset, label_offset),
            code!(BBR) => (label_offset, -label_offset),
            _ => unreachable!("for {:?}", self.code),
        };

        let mut labels = vec![
            Label {
                display,
                text: svg_lf_text,
                pos: mid_pt + lf_label_offset,
            },
            Label {
                display,
                text: svg_rf_text,
                pos: mid_pt + rf_label_offset,
            },
        ];
        if let Some(duration) = opts.duration {
            // Assume that 10% along the curve is still pretty much vertical,
            // so the count label can just be inset horizontally.
            let early_pt = self.percent_point(Percentage(10));
            let x_offset = match self.sign() {
                RotationDirection::Clockwise => 2 * font_size,
                RotationDirection::AntiClockwise => -2 * font_size,
            };
            labels.push(Label {
                display: true,
                text: timing_text(duration.0),
                pos: early_pt + pos!(x_offset, 0),
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
