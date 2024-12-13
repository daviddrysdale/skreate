//! Mohawk.

use super::{compound::Compound, edge::Curve, label::Label, shift::Shift, Error};
use crate::{code, moves, params, params::Value, parse_code, Code, Input, Move, PreTransition};
use std::borrow::Cow;

pub struct OpenMohawk;

impl OpenMohawk {
    const MOVE: &'static str = "-OpMo";
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
        ],
    };

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (pre_transition, rest) = PreTransition::parse(input.text);
        let (entry_code, rest) = parse_code(rest).map_err(|_msg| Error::Unrecognized)?;
        let sign = match entry_code {
            code!(LFI) => "-",
            code!(RFI) => "",
            _ => return Err(Error::Unrecognized),
        };

        let Some(rest) = rest.strip_prefix(Self::MOVE) else {
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

        let out_code = Code {
            foot: entry_code.foot.opposite(),
            dir: entry_code.dir.opposite(),
            edge: entry_code.edge,
        };

        let pos = input.pos;
        let entry = format!("{prefix}{entry_code}[angle={angle1},len={len1},style=\"{style}\"]");
        let label = format!("Label[fwd=30,side={sign}70,text=\"OpMo\"]");
        let shift = format!("Shift[side={sign}80,fwd=-65,rotate={sign}90,code=\"{out_code}\"]");
        let exit = format!("{out_code}[angle={angle2},len={len2},style=\"{style}\"]");

        log::info!("input {input:?} results in {entry};{label};{shift};{exit}");
        let moves = vec![
            Curve::construct(&Input { pos, text: &entry }).unwrap(),
            Label::construct(&Input { pos, text: &label }).unwrap(),
            Shift::construct(&Input { pos, text: &shift }).unwrap(),
            Curve::construct(&Input { pos, text: &exit }).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}{}{suffix}", Self::MOVE);

        Ok(Box::new(Compound::new(input, moves, params, text)))
    }
}
