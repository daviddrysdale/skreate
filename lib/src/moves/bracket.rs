//! Bracket

use super::{compound::Compound, edge::Curve, label::Label, shift::Shift, straight::StraightEdge};
use crate::{code, moves, params, params::Value, parser, Code, Edge, MoveParam, PreTransition};
use std::borrow::Cow;

pub struct Bracket;

impl Bracket {
    /// Move code.
    pub const MOVE: &'static str = "-Br";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Bracket",
        summary: "Bracket turn",
        example: "LFO-Br",
        visible: true,
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation for each curved part, in degrees",
                default: Value::Number(90),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::GreaterLess(params::Detents {
                    add1: 100,
                    add2: 120,
                    add3: 140,
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

        let entry1 = format!("{prefix}{entry_code}[angle={angle1},len={len1a},style=\"{style}\",label=\"{entry_code}{}\",transition-label=\"{transition_label}\"]", Self::MOVE);
        let entry2 = format!("{entry_flat}[len={len1b},style=\"{style}\",label=\" \"]");
        let entry3 =
            format!("{entry_rev}[angle={angle1c},len={len1c},style=\"{style}\",label=\" \"]");
        let label = "Label[text=\"Br\",fwd=40]".to_string();
        let shift = format!("Shift[rotate={sign}135,code=\"{out_rev}\"]");
        let exit3 = format!("{out_rev}[angle={angle2c},len={len2c},style=\"{style}\",label=\" \"]");
        let exit2 = format!("{out_flat}[len={len2b},style=\"{style}\",label=\" \"]");
        let exit1 = format!("{out_code}[angle={angle2},len={len2a},style=\"{style}\"]");

        log::info!("input {input:?} results in {entry1};{entry2};{shift};{exit2};{exit1}");
        let moves = vec![
            Curve::construct(&entry1).unwrap(),
            StraightEdge::construct(&entry2).unwrap(),
            Curve::construct(&entry3).unwrap(),
            Label::construct(&label).unwrap(),
            Shift::construct(&shift).unwrap(),
            Curve::construct(&exit3).unwrap(),
            StraightEdge::construct(&exit2).unwrap(),
            Curve::construct(&exit1).unwrap(),
        ];

        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Compound::new(input, moves, params, text))
    }
}
