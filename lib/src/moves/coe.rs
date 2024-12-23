//! Change of Edge

use super::{compound::Compound, edge::Curve, straight::StraightEdge};
use crate::{
    moves, params, params::Value, parser, Code, Edge, MoveParam, PreTransition, TextPosition,
};
use std::borrow::Cow;

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
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation for each curved part, in degrees",
                default: Value::Number(90),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::GreaterLess(params::Detents {
                    add1: 120,
                    add2: 180,
                    add3: 210,
                    less1: 60,
                    less2: 45,
                    less3: 30,
                })),
            },
            params::Info {
                name: "len",
                doc: "Length of each curved part in centimetres",
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
                name: "flat-len",
                doc: "Length between edges in centimetres",
                default: Value::Number(50),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "delta-angle",
                doc: "Difference in angle for second curved part, in degrees",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "delta-len",
                doc: "Difference in length for second curved part, in centimetres",
                default: Value::Number(0),
                range: params::Range::Any,
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
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let angle1 = params[0].value.as_i32(input)?;
        let len1 = params[1].value.as_i32(input)?;
        let flat_len = params[2].value.as_i32(input)?;
        let delta_angle = params[3].value.as_i32(input)?;
        let delta_len = params[4].value.as_i32(input)?;
        let style = params[5].value.as_str(input)?;
        let transition_label = params[6].value.as_str(input)?;

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

        Ok(Compound::new(input, text_pos, moves, params, text))
    }
}
