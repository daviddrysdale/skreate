//! Rocker

use super::{
    compound::Compound, edge::Curve, label::Label, shift::Shift, straight::StraightEdge, Error,
};
use crate::{
    code, moves, params, params::Value, parse_code, Code, Edge, Input, Move, PreTransition,
};
use std::borrow::Cow;

pub struct Rocker;

impl Rocker {
    const MOVE: &'static str = "-Rk";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Rocker",
        summary: "Rocker turn",
        example: "LFO-Rk",
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
                name: "delta_angle",
                doc: "Difference in angle for second curved part, in degrees",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "delta_len",
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
        ],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (pre_transition, rest) = PreTransition::parse(input.text);
        let (entry_code, rest) = parse_code(rest).map_err(|_msg| Error::Unrecognized)?;
        let sign = match entry_code {
            // Clockwise
            code!(LFI) | code!(RFO) | code!(RBI) | code!(LBO) => "-",
            // Widdershins
            code!(RFI) | code!(LFO) | code!(LBI) | code!(RBO) => "",
            _ => return Err(Error::Unrecognized),
        };

        let Some((_, rest)) = rest.split_once(Self::MOVE) else {
            return Err(Error::Unrecognized);
        };

        let params =
            params::populate(Self::INFO.params, rest).map_err(|_msg| Error::Unrecognized)?;
        let angle1 = params[0].value.as_i32().unwrap();
        let len1 = params[1].value.as_i32().unwrap();
        let delta_angle = params[2].value.as_i32().unwrap();
        let delta_len = params[3].value.as_i32().unwrap();
        let style = params[4].value.as_str().unwrap();

        let angle2 = angle1 + delta_angle;
        let len2 = len1 + delta_len;

        let prefix = pre_transition.prefix();

        let out_rev = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge.opposite(),
        };
        let out_flat = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: Edge::Flat,
        };
        let out_code = Code {
            foot: entry_code.foot,
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge,
        };

        let len1a = len1 * 85 / 100;
        let len1b = len1 - len1a;
        let angle1b = 80;
        let angle1a = angle1;

        let len2a = len2 * 75 / 100;
        let len2b = 20;
        let len2c = len2 - len2a - len2b;
        let angle2c = 80;

        let pos = input.pos;
        let entry1 = format!("{prefix}{entry_code}[angle={angle1a},len={len1a},style=\"{style}\",label=\"{entry_code}{}\"]", Self::MOVE);
        let entry2 =
            format!("{entry_code}[angle={angle1b},len={len1b},style=\"{style}\",label=\" \"]");
        let label = "Label[text=\"Rk\",fwd=40]".to_string();
        let shift = format!("Shift[rotate={sign}135,code=\"{out_rev}\"]");
        let exit3 = format!("{out_rev}[angle={angle2c},len={len2c},style=\"{style}\",label=\" \"]");
        let exit2 = format!("{out_flat}[len={len2b},style=\"{style}\",label=\" \"]");
        let exit1 = format!("{out_code}[angle={angle2},len={len2a},style=\"{style}\"]");

        log::info!("input {input:?} results in {entry1};{entry2};{shift};{exit2};{exit1}");
        let moves = vec![
            Curve::construct(&Input { pos, text: &entry1 }).unwrap(),
            Curve::construct(&Input { pos, text: &entry2 }).unwrap(),
            Label::construct(&Input { pos, text: &label }).unwrap(),
            Shift::construct(&Input { pos, text: &shift }).unwrap(),
            Curve::construct(&Input { pos, text: &exit3 }).unwrap(),
            StraightEdge::construct(&Input { pos, text: &exit2 }).unwrap(),
            Curve::construct(&Input { pos, text: &exit1 }).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{}{}{suffix}", entry_code, Self::MOVE);

        Ok(Box::new(Compound::new(input, moves, params, text)))
    }
}
