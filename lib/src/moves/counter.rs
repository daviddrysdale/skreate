//! Counter

use super::{
    compound::{self, Compound},
    edge::Curve,
    label::Label,
    shift::Shift,
    straight::StraightEdge,
    MoveId, SkatingMoveId,
};
use crate::{code, moves, params, parser, Code, Edge, MoveParam, PreTransition, TextPosition};

pub struct Counter;

impl Counter {
    /// Move code.
    pub const MOVE: &'static str = "-Ctr";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Counter",
        id: MoveId::Skating(SkatingMoveId::Counter),
        summary: "Counter turn",
        example: "LFO-Ctr",
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
        let out_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge,
        };

        let len1a = len1 * 75 / 100;
        let len1b = 20;
        let len1c = len1 - len1a - len1b;
        let angle1c = 80;

        let len2a = len2 * 85 / 100;
        let len2b = len2 - len2a;
        let angle2b = 80;
        let angle2a = angle2;

        let entry1 = format!("{prefix}{entry_code}[angle={angle1},len={len1a},style=\"{style}\",label=\"{entry_code}\",transition-label=\"{transition_label}\"]");
        let entry2 = format!("{entry_flat}[len={len1b},style=\"{style}\",label=\" \"]");
        let entry3 =
            format!("{entry_rev}[angle={angle1c},len={len1c},style=\"{style}\",label=\" \"]");
        let label = "Label[text=\"Ctr\",fwd=40]".to_string();
        let shift = format!("Shift[rotate={sign}135,code=\"{out_code}\"]");
        let exit2 =
            format!("{out_code}[angle={angle2b},len={len2b},style=\"{style}\",label=\" \"]");
        let exit1 = format!("{out_code}[angle={angle2a},len={len2a},style=\"{style}\"]");

        log::info!("input {input:?} results in {entry1};{entry2};{shift};{exit2};{exit1}");
        let moves = vec![
            Curve::construct(&entry1, text_pos).unwrap(),
            StraightEdge::construct(&entry2, text_pos).unwrap(),
            Curve::construct(&entry3, text_pos).unwrap(),
            Label::construct(&label, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit2, text_pos).unwrap(),
            Curve::construct(&exit1, text_pos).unwrap(),
        ];

        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Counter,
            moves,
            params,
            text,
        ))
    }
}
