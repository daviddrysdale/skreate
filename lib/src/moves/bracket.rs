// Copyright 2024-2025 David Drysdale

//! Bracket

use super::{
    compound::{self, Compound},
    edge::Curve,
    label::Label,
    shift::Shift,
    straight::StraightEdge,
    MoveId, SkatingMoveId,
};
use crate::{code, moves, params, parser, Code, Edge, MoveParam, PreTransition, TextPosition};

pub struct Bracket;

impl Bracket {
    /// Move code.
    pub const MOVE: &'static str = "-Br";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Bracket",
        id: MoveId::Skating(SkatingMoveId::Bracket),
        summary: "Bracket turn",
        example: "LFO-Br",
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
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let sign = match entry_code {
            // Clockwise
            code!(LFI) | code!(RFO) | code!(RBI) | code!(LBO) => "",
            // Widdershins
            code!(RFI) | code!(LFO) | code!(LBI) | code!(RBO) => "-",
            _ => return Err(parser::fail(input)),
        };

        let angle1 = params[0].value.as_i32(input)?;
        let len1 = params[1].value.as_i32(input)?;
        let delta_angle = params[2].value.as_i32(input)?;
        let delta_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let transition_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let angle2 = angle1 + delta_angle;
        let len2 = len1 + delta_len;

        let prefix = pre_transition.prefix();

        let entry_flat = Code {
            foot: entry_code.foot,
            dir: entry_code.dir,
            edge: Edge::Flat,
        };
        let entry_rev = Code {
            foot: entry_code.foot,
            dir: entry_code.dir,
            edge: entry_code.edge.opposite(),
        };
        let out_rev = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_rev.edge.opposite(),
        };
        let out_flat = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: Edge::Flat,
        };
        let out_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge.opposite(),
        };

        let len1a = len1 * 75 / 100;
        let len1b = 40;
        let len1c = len1 - len1a - len1b;
        let angle1c = 80;

        let len2a = len2 * 75 / 100;
        let len2b = 40;
        let len2c = len2 - len2a - len2b;
        let angle2c = angle1c;

        let entry1 = format!("{prefix}{entry_code}[angle={angle1},len={len1a},style=\"{style}\",label=\"{entry_code}\",transition-label=\"{transition_label}\",label-offset={label_offset}]");
        let entry2 = format!("{entry_flat}[len={len1b},style=\"{style}\",label=\" \"]");
        let entry3 =
            format!("{entry_rev}[angle={angle1c},len={len1c},style=\"{style}\",label=\" \"]");
        let label = "Label[text=\"Br\",fwd=40]".to_string();
        let shift = format!("Shift[rotate={sign}135,code=\"{out_rev}\"]");
        let exit3 = format!("{out_rev}[angle={angle2c},len={len2c},style=\"{style}\",label=\" \"]");
        let exit2 = format!("{out_flat}[len={len2b},style=\"{style}\",label=\" \"]");
        let exit1 = format!(
            "{out_code}[angle={angle2},len={len2a},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry1};{entry2};{shift};{exit2};{exit1}");
        let moves = vec![
            Curve::construct(&entry1, text_pos).unwrap(),
            StraightEdge::construct(&entry2, text_pos).unwrap(),
            Curve::construct(&entry3, text_pos).unwrap(),
            Label::construct(&label, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit3, text_pos).unwrap(),
            StraightEdge::construct(&exit2, text_pos).unwrap(),
            Curve::construct(&exit1, text_pos).unwrap(),
        ];

        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Bracket,
            moves,
            params,
            text,
        ))
    }
}
