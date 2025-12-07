// Copyright 2024-2025 David Drysdale

//! Three-turn

use super::{
    compound::{self, add_angle, add_len, map_errs, Compound},
    edge::Curve,
    edge_err,
    shift::Shift,
    MoveId, ParseError, SkatingMoveId,
};
use crate::{code, moves, params, Code, MoveParam, PreTransition, TextPosition};

pub struct ThreeTurn;

impl ThreeTurn {
    /// Move code.
    pub const MOVE: &'static str = "3";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Three Turn",
        id: MoveId::Skating(SkatingMoveId::ThreeTurn),
        summary: "Three turn",
        example: "LFO3",
        visible: true,
        params: &compound::params(
            60, 70, 80, 90, 100, 120, 140, // Angle values
            100, 240, 300, 450, 600, 850, 1000, // Length values
        ),
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
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

        let angle1 = params[0].value.as_i32(text_pos)?;
        let len1 = params[1].value.as_i32(text_pos)?;
        let delta_angle = params[2].value.as_i32(text_pos)?;
        let delta_len = params[3].value.as_i32(text_pos)?;
        let style = params[4].value.as_str(text_pos)?;
        let transition_label = params[5].value.as_str(text_pos)?;
        let label1 = params[6].value.as_str(text_pos)?;
        let label2 = params[7].value.as_str(text_pos)?;
        let label_offset = params[8].value.as_i32(text_pos)?;

        let angle2 = add_angle(angle1, delta_angle, text_pos)?;
        let len2 = add_len(len1, delta_len, text_pos)?;

        let prefix = pre_transition.prefix();

        let out_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge.opposite(),
        };

        let len1a = len1 * 75 / 100;
        let len1b = len1 - len1a;
        let angle1b = angle1 * 60 / 100;
        let angle1a = angle1 - angle1b;

        let len2a = len2 * 75 / 100;
        let len2b = len2 - len2a;
        let angle2b = angle2 * 60 / 100;
        let angle2a = angle2 - angle2b;

        let entry_label = if label1.is_empty() {
            // TODO: this label does not get mirrored by Compound::opposite()
            format!(",label=\"{entry_code}{}\"", Self::MOVE)
        } else {
            format!(",label=\"{label1}\"")
        };
        let exit_label = if label2.is_empty() {
            "".to_string()
        } else {
            format!(",label=\"{label2}\"")
        };

        let entry1 = format!("{prefix}{entry_code}[angle={angle1a},len={len1a},style=\"{style}\",transition-label=\"{transition_label}\",label-offset={label_offset}{entry_label}]");
        let entry2 =
            format!("{entry_code}[angle={angle1b},len={len1b},style=\"{style}\",label=\" \"]");
        let shift = format!("Shift[rotate={sign}135,code=\"{out_code}\"]");
        let exit2 =
            format!("{out_code}[angle={angle2b},len={len2b},style=\"{style}\",label=\" \"]");
        let exit1 = format!(
            "{out_code}[angle={angle2a},len={len2a},style=\"{style}\",label-offset={label_offset}{exit_label}]"
        );

        log::info!("input {input:?} results in {entry1};{entry2};{shift};{exit2};{exit1}");
        let moves = vec![
            Curve::construct(&entry1, text_pos),
            Curve::construct(&entry2, text_pos),
            Shift::construct(&shift, text_pos),
            Curve::construct(&exit2, text_pos),
            Curve::construct(&exit1, text_pos),
        ];

        let text_prefix = format!("{prefix}{entry_code}{}", Self::MOVE);

        Ok(Compound::new(
            text_pos,
            SkatingMoveId::ThreeTurn,
            map_errs(moves)?,
            Self::INFO.params,
            params,
            text_prefix,
        ))
    }
}
