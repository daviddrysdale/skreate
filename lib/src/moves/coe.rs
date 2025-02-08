//! Change of Edge

use super::{
    compound::{self, Compound},
    edge::Curve,
    straight::StraightEdge,
    SkatingMoveId,
};
use crate::{moves, params, parser, Code, Edge, MoveParam, PreTransition, TextPosition};

pub struct ChangeOfEdge;

impl ChangeOfEdge {
    /// Move code.
    pub const MOVE: &'static str = "-CoE";
    /// Alternative move code.
    pub const MOVE_ALT: &'static str = "-COE";

    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Change of Edge",
        summary: "Change of edge",
        example: "LFO-CoE",
        visible: true,
        params: &compound::params_flat(
            30, 45, 60, 90, 120, 180, 210, // Angle values
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
        let flat_len = params[6].value.as_i32(input)?;

        let angle2 = angle1 + delta_angle;
        let len2 = len1 + delta_len;

        let prefix = pre_transition.prefix();
        let flat_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir,
            edge: Edge::Flat,
        };
        let out_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir,
            edge: entry_code.edge.opposite(),
        };

        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\",transition-label=\"{transition_label}\"]");
        let flat = format!("{flat_code}[len={flat_len},label=\"COE\",style=\"{style}\"]");
        let exit = format!("{out_code}[angle={angle2},len={len2},style=\"{style}\"]");
        log::debug!("input {input:?} results in {entry};{flat};{exit}");

        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            StraightEdge::construct(&flat, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::ChangeOfEdge,
            moves,
            params,
            text,
        ))
    }
}
