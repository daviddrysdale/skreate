// Copyright 2024-2025 David Drysdale

//! Choctaw.

use super::{
    compound::{self, Compound},
    edge::Curve,
    label::Label,
    shift::Shift,
    MoveId, SkatingMoveId,
};
use crate::{code, moves, params, parser, Code, MoveParam, PreTransition, TextPosition};

pub struct OpenChoctaw;

impl OpenChoctaw {
    /// Move code.
    pub const MOVE: &'static str = "-OpCho";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Open Choctaw",
        id: MoveId::Skating(SkatingMoveId::OpenChoctaw),
        summary: "Open choctaw",
        example: "LFI-OpCho",
        visible: true,
        params: &compound::params(
            60, 70, 80, 90, 110, 130, 150, // Angle values
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
            code!(LFI) => "-",
            code!(RFI) => "",
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

        let out_code = Code {
            foot: entry_code.foot.opposite(),
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge.opposite(),
        };

        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\",transition-label=\"{transition_label}\",label-offset={label_offset}]");
        let label = format!("Label[fwd=10,side={sign}80,text=\"OpCho\"]");
        let shift = format!("Shift[side={sign}40,code=\"{out_code}\"]");
        let exit = format!(
            "{out_code}[angle={angle2},len={len2},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry};{label};{shift};{exit}");
        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            Label::construct(&label, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::OpenChoctaw,
            moves,
            params,
            text,
        ))
    }
}

pub struct ClosedChoctaw;

impl ClosedChoctaw {
    /// Move code.
    pub const MOVE: &'static str = "-ClCho";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Closed Choctaw",
        id: MoveId::Skating(SkatingMoveId::ClosedChoctaw),
        summary: "Closed choctaw",
        example: "RBO-ClCho",
        visible: true,
        params: &compound::params(
            60, 70, 80, 90, 110, 130, 150, // Angle values
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
            code!(LBO) => "-",
            code!(RBO) => "",
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

        let out_code = Code {
            foot: entry_code.foot.opposite(),
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge.opposite(),
        };

        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\",transition-label=\"{transition_label}\",label-offset={label_offset}]");
        let label = format!("Label[side={sign}60,text=\"ClCho\"]");
        let shift = format!("Shift[side={sign}30,fwd=-30,code=\"{out_code}\"]");
        let exit = format!(
            "{out_code}[angle={angle2},len={len2},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry};{label};{shift};{exit}");
        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            Label::construct(&label, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::ClosedChoctaw,
            moves,
            params,
            text,
        ))
    }
}
