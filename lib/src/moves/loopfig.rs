//! Loop

use super::{
    compound::{self, Compound},
    edge::Curve,
    SkatingMoveId,
};
use crate::{moves, params, parser, Code, MoveParam, PreTransition, TextPosition};

pub struct Loop;

impl Loop {
    /// Move code.
    pub const MOVE: &'static str = "-Loop";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Loop",
        summary: "Loop figure",
        example: "RBI-Loop",
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

        let angle1 = params[0].value.as_i32(input)?;
        let len1 = params[1].value.as_i32(input)?;
        let delta_angle = params[2].value.as_i32(input)?;
        let delta_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let transition_label = params[5].value.as_str(input)?;
        let angle2 = angle1 + delta_angle;
        let len2 = len1 + delta_len;

        let prefix = pre_transition.prefix();

        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\",transition-label=\"{transition_label}\"]");

        // The loop itself is a fixed size and net rotates 330 degrees.
        let start = format!("{entry_code}[angle=100,len=80,style=\"{style}\",label=\" \"]");
        let corner = format!("{entry_code}[angle=130,len=80,style=\"{style}\",label=\" \"]");
        let end = format!("{entry_code}[angle=100,len=80,style=\"{style}\",label=\" \"]");

        let exit = format!("{entry_code}[angle={angle2},len={len2},style=\"{style}\",label=\" \"]");

        log::info!("input {input:?} results in {entry};{start};{corner};{end};{exit}");
        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            Curve::construct(&start, text_pos).unwrap(),
            Curve::construct(&corner, text_pos).unwrap(),
            Curve::construct(&end, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Loop,
            moves,
            params,
            text,
        ))
    }
}
