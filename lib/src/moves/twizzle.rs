//! Twizzle.

use super::{compound::Compound, edge::Curve, label::Label, shift::Shift, Error};
use crate::{
    code, moves, params,
    params::Value,
    parser::types::{parse_code, parse_pre_transition},
    Code, Input, Move,
};
use regex::Regex;
use std::borrow::Cow;

pub struct Twizzle;

impl Twizzle {
    const MOVE: &'static str = "-Tw";
    const PATTERN: &'static str = r#"(?P<n>[0-9])(?P<half>\.5)?(?P<rest>.*)"#;
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Twizzle",
        summary: "Twizzle",
        example: "LFI-Tw1.5",
        visible: true,
        params: &[
            params::Info {
                name: "angle",
                doc: "Angle of rotation for each curved part, in degrees",
                default: Value::Number(60),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::GreaterLess(params::Detents {
                    add1: 70,
                    add2: 80,
                    add3: 90,
                    less1: 50,
                    less2: 40,
                    less3: 30,
                })),
            },
            params::Info {
                name: "len",
                doc: "Length of each curved part in centimetres",
                default: Value::Number(200),
                range: params::Range::StrictlyPositive,
                short: Some(params::Abbrev::PlusMinus(params::Detents {
                    add1: 300,
                    add2: 450,
                    add3: 600,
                    less1: 180,
                    less2: 100,
                    less3: 80,
                })),
            },
            params::Info {
                name: "pre-len",
                doc: "Length of entry curve in centimetres",
                default: Value::Number(100),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "pre-angle",
                doc: "Angle of entry curve in degrees",
                default: Value::Number(45),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "post-len",
                doc: "Length of exit curve in centimetres",
                default: Value::Number(100),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "post-angle",
                doc: "Angle of exit curve in degrees",
                default: Value::Number(45),
                range: params::Range::StrictlyPositive,
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

    pub fn construct(input: &Input) -> Result<Box<dyn Move>, Error> {
        let (rest, pre_transition) = parse_pre_transition(input.text)?;
        let (rest, entry_code) = parse_code(rest)?;
        let sign = match entry_code {
            // Clockwise
            code!(LFI) | code!(RFO) | code!(RBI) | code!(LBO) => "-",
            // Widdershins
            code!(RFI) | code!(LFO) | code!(LBI) | code!(RBO) => "",
            _ => return Err(Error::Unrecognized),
        };

        let Some(rest) = rest.strip_prefix(Self::MOVE) else {
            return Err(Error::Unrecognized);
        };
        if rest.is_empty() {
            return Err(Error::Unrecognized);
        }
        let re = Regex::new(Self::PATTERN).unwrap();
        let Some(captures) = re.captures(rest) else {
            return Err(Error::Unrecognized);
        };
        let Some(n) = captures.name("n") else {
            return Err(Error::Unrecognized);
        };
        let Ok(n) = n.as_str().parse::<u32>() else {
            return Err(Error::Unrecognized);
        };
        let Some(rest) = captures.name("rest") else {
            return Err(Error::Unrecognized);
        };
        let rest = rest.as_str();
        let half = captures.name("half").is_some();
        let count = n * 2 + if half { 1 } else { 0 };
        if count < 2 {
            log::warn!("need more than {count} turns in a twizzle");
            return Err(Error::Unrecognized);
        }

        let params =
            params::populate(Self::INFO.params, rest).map_err(|_msg| Error::Unrecognized)?;
        let angle = params[0].value.as_i32().unwrap();
        let len = params[1].value.as_i32().unwrap();
        let pre_len = params[2].value.as_i32().unwrap();
        let pre_angle = params[3].value.as_i32().unwrap();
        let post_len = params[4].value.as_i32().unwrap();
        let post_angle = params[5].value.as_i32().unwrap();
        let style = params[6].value.as_str().unwrap();
        let transition_label = params[7].value.as_str().unwrap();

        let len_a = len * 75 / 100;
        let len_b = len - len_a;
        let angle_b = angle * 60 / 100;
        let angle_a = angle - angle_b;
        let mid_angle = 2 * angle;

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let label_text = format!(
            "{entry_code}{}{}{}",
            Self::MOVE,
            count / 2,
            if count % 2 == 0 { "" } else { ".5" }
        );
        let text = format!("{prefix}{label_text}{suffix}");

        let pos = input.pos;
        let mut code = entry_code;
        let mut moves = Vec::new();

        let pre = format!(
            "{prefix}{code} [len={pre_len},angle={pre_angle},style=\"{style}\",label=\" \",transition-label=\"{transition_label}\"]"
        );
        moves.push(Curve::construct(&Input { pos, text: &pre }).unwrap());
        let mut debug = format!("{pre};");

        for n in 0..count {
            let out_code = Code {
                foot: code.foot,
                dir: code.dir.opposite(),
                edge: code.edge.opposite(),
            };

            let entry1 =
                format!("{code}[angle={angle_a},len={len_a},style=\"{style}\",label=\" \"]");
            let entry2 =
                format!("{code}[angle={angle_b},len={len_b},style=\"{style}\",label=\" \"]");
            let shift = format!("Shift[rotate={sign}{mid_angle},code=\"{out_code}\"]");
            let exit2 =
                format!("{out_code}[angle={angle_b},len={len_b},style=\"{style}\",label=\" \"]");
            let exit1 =
                format!("{out_code}[angle={angle_a},len={len_a},style=\"{style}\",label=\" \"]");

            moves.push(Curve::construct(&Input { pos, text: &entry1 }).unwrap());
            moves.push(Curve::construct(&Input { pos, text: &entry2 }).unwrap());
            if count % 2 == 1 && n == count / 2 {
                let label = format!("Label [fwd=100,side=30,text=\"{label_text}\"]");
                moves.push(Label::construct(&Input { pos, text: &label }).unwrap());
            }

            moves.push(Shift::construct(&Input { pos, text: &shift }).unwrap());
            moves.push(Curve::construct(&Input { pos, text: &exit2 }).unwrap());
            moves.push(Curve::construct(&Input { pos, text: &exit1 }).unwrap());

            if count % 2 == 0 && n == (count - 1) / 2 {
                let label = format!("Label [side=100,text=\"{label_text}\"]");
                moves.push(Label::construct(&Input { pos, text: &label }).unwrap());
            }

            code = out_code;
            debug = format!("{debug}{entry1};{entry2};{shift};{exit2};{exit1};");
        }
        let post =
            format!("{code} [len={post_len},angle={post_angle},style=\"{style}\",label=\" \"]");
        moves.push(Curve::construct(&Input { pos, text: &post }).unwrap());
        debug = format!("{debug}{post}");

        log::info!("input {input:?} results in {debug}");
        Ok(Box::new(Compound::new(input, moves, params, text)))
    }
}
