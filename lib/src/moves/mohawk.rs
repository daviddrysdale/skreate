//! Mohawk.

use super::{compound::Compound, edge::Curve, label::Label, shift::Shift};
use crate::{
    code, moves, params, params::Value, parser, Code, MoveParam, PreTransition, TextPosition,
};
use std::borrow::Cow;

pub struct OpenMohawk;

impl OpenMohawk {
    /// Move code.
    pub const MOVE: &'static str = "-OpMo";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Open Mohawk",
        summary: "Open mohawk",
        example: "LFI-OpMo",
        visible: true,
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation for each curved part, in degrees",
                default: Value::Number(90),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::GreaterLess(params::Detents {
                    add1: 110,
                    add2: 130,
                    add3: 150,
                    less1: 80,
                    less2: 70,
                    less3: 60,
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

        let angle2 = angle1 + delta_angle;
        let len2 = len1 + delta_len;

        let prefix = pre_transition.prefix();

        let out_code = Code {
            foot: entry_code.foot.opposite(),
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge,
        };

        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\",transition-label=\"{transition_label}\"]");
        let label = format!("Label[fwd=30,side={sign}70,text=\"OpMo\"]");
        let shift = format!("Shift[side={sign}80,fwd=-65,rotate={sign}90,code=\"{out_code}\"]");
        let exit = format!("{out_code}[angle={angle2},len={len2},style=\"{style}\"]");

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

        Ok(Compound::new(input, text_pos, moves, params, text))
    }
}
