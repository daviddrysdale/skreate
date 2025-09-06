// Copyright 2024-2025 David Drysdale

//! Twizzle.

use super::{
    compound::Compound, edge::Curve, edge_err, label::Label, shift::Shift, MoveId, SkatingMoveId,
};
use crate::{
    code, moves, params, params::Value, Code, MoveParam, ParseError, PreTransition, TextPosition,
};
use std::borrow::Cow;

pub struct Twizzle;

impl Twizzle {
    /// Move code.
    pub const MOVE: &'static str = "-Tw";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Twizzle",
        id: MoveId::Skating(SkatingMoveId::Twizzle(2)),
        summary: "Twizzle",
        example: "LFI-Tw1.5",
        visible: true,
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation for each curved part, in degrees",
                default: Value::Number(60),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::GreaterLess(params::Detents {
                    add1: 70,
                    add2: 80,
                    add3: 90,
                    less1: 50,
                    less2: 40,
                    less3: 30,
                })),
            },
            params::Info {
                name: "len",
                doc: "Length of each curved part in centimetres",
                default: Value::Number(200),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::PlusMinus(params::Detents {
                    add1: 300,
                    add2: 450,
                    add3: 600,
                    less1: 180,
                    less2: 100,
                    less3: 80,
                })),
            },
            params::Info {
                name: "pre-len",
                doc: "Length of entry curve in centimetres",
                default: Value::Number(100),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "pre-angle",
                doc: "Angle of entry curve in degrees",
                default: Value::Number(45),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "post-len",
                doc: "Length of exit curve in centimetres",
                default: Value::Number(100),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "post-angle",
                doc: "Angle of exit curve in degrees",
                default: Value::Number(45),
                range: params::Range::StrictlyPositive,
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
        ],
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: u32,
        params: Vec<MoveParam>,
    ) -> Result<Compound, ParseError> {
        assert!(params::compatible(Self::INFO.params, &params));
        let sign = match entry_code {
            // Clockwise
            code!(LFI) | code!(RFO) | code!(RBI) | code!(LBO) => "-",
            // Widdershins
            code!(RFI) | code!(LFO) | code!(LBI) | code!(RBO) => "",
            _ => return Err(edge_err(text_pos, entry_code, Self::INFO)),
        };

        let angle = params[0].value.as_i32(text_pos)?;
        let len = params[1].value.as_i32(text_pos)?;
        let pre_len = params[2].value.as_i32(text_pos)?;
        let pre_angle = params[3].value.as_i32(text_pos)?;
        let post_len = params[4].value.as_i32(text_pos)?;
        let post_angle = params[5].value.as_i32(text_pos)?;
        let style = params[6].value.as_str(text_pos)?;
        let transition_label = params[7].value.as_str(text_pos)?;

        let len_a = len * 75 / 100;
        let len_b = len - len_a;
        let angle_b = angle * 60 / 100;
        let angle_a = angle - angle_b;
        let mid_angle = 2 * angle;

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let label_text = format!(
            "{entry_code}{}{}{}",
            Self::MOVE,
            count / 2,
            if count % 2 == 0 { "" } else { ".5" }
        );
        let text = format!("{prefix}{label_text}{suffix}");

        let mut code = entry_code;
        let mut moves = Vec::new();

        let pre = format!(
            "{prefix}{code} [len={pre_len},angle={pre_angle},style=\"{style}\",label=\" \",transition-label=\"{transition_label}\"]"
        );
        moves.push(Curve::construct(&pre, text_pos)?);
        let mut debug = format!("{pre};");

        for n in 0..count {
            let out_code = Code {
                foot: code.foot,
                dir: code.dir.opposite(),
                edge: code.edge.opposite(),
            };

            let entry1 =
                format!("{code}[angle={angle_a},len={len_a},style=\"{style}\",label=\" \"]");
            let entry2 =
                format!("{code}[angle={angle_b},len={len_b},style=\"{style}\",label=\" \"]");
            let shift = format!("Shift[rotate={sign}{mid_angle},code=\"{out_code}\"]");
            let exit2 =
                format!("{out_code}[angle={angle_b},len={len_b},style=\"{style}\",label=\" \"]");
            let exit1 =
                format!("{out_code}[angle={angle_a},len={len_a},style=\"{style}\",label=\" \"]");

            moves.push(Curve::construct(&entry1, text_pos)?);
            moves.push(Curve::construct(&entry2, text_pos)?);
            if count % 2 == 1 && n == count / 2 {
                let label = format!("Label [fwd=100,side=30,text=\"{label_text}\"]");
                moves.push(Label::construct(&label, text_pos)?);
            }

            moves.push(Shift::construct(&shift, text_pos)?);
            moves.push(Curve::construct(&exit2, text_pos)?);
            moves.push(Curve::construct(&exit1, text_pos)?);

            if count % 2 == 0 && n == (count - 1) / 2 {
                let label = format!("Label [side=100,text=\"{label_text}\"]");
                moves.push(Label::construct(&label, text_pos)?);
            }

            code = out_code;
            debug = format!("{debug}{entry1};{entry2};{shift};{exit2};{exit1};");
        }
        let post =
            format!("{code} [len={post_len},angle={post_angle},style=\"{style}\",label=\" \"]");
        moves.push(Curve::construct(&post, text_pos)?);
        debug = format!("{debug}{post}");

        log::info!("input {input:?} results in {debug}");
        Ok(Compound::new_with_count_idx(
            text_pos,
            SkatingMoveId::Twizzle(count),
            moves,
            params,
            text,
            None,
        ))
    }
}
