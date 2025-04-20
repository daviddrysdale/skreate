// Copyright 2025 David Drysdale

//! Jumps

use super::{compound::Compound, edge::Curve, hop::Hop, shift::Shift, MoveId, SkatingMoveId};
use crate::{
    code, moves, params, params::Value, parser, Code, JumpCount, MoveParam, PreTransition,
    TextPosition,
};
use std::borrow::Cow;

/// Standard parameters for jumps.
const JUMP_PARAMS: [params::Info; 7] = [
    params::Info {
        name: "entry-angle",
        doc: "Angle of rotation for entry edge, in degrees",
        default: Value::Number(30),
        range: params::Range::StrictlyPositive,
        short: Some(params::Abbrev::GreaterLess(params::Detents {
            add1: 45,
            add2: 60,
            add3: 80,
            less1: 25,
            less2: 20,
            less3: 15,
        })),
    },
    params::Info {
        name: "entry-len",
        doc: "Length of entry edge, in centimetres",
        default: Value::Number(600),
        range: params::Range::StrictlyPositive,
        short: Some(params::Abbrev::PlusMinus(params::Detents {
            add1: 700,
            add2: 850,
            add3: 1000,
            less1: 450,
            less2: 300,
            less3: 200,
        })),
    },
    params::Info {
        name: "exit-angle",
        doc: "Angle of rotation for exit edge, in degrees",
        default: Value::Number(40),
        range: params::Range::Any,
        short: None,
    },
    params::Info {
        name: "exit-len",
        doc: "Length of exit edge, in centimetres",
        default: Value::Number(400),
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
        name: "jump-label",
        doc: "Replacement jump label, used if non-empty",
        default: Value::Text(Cow::Borrowed("")),
        range: params::Range::Text,
        short: None,
    },
    params::Info {
        name: "label-offset",
        doc: "Amount to scale label offsets by, as a percentage, or -1 to use global value",
        default: Value::Number(-1),
        range: params::Range::Any,
        short: None,
    },
];

/// Salchow jump.
pub struct Salchow;

impl Salchow {
    /// Jump code.
    pub const JUMP: &str = "S";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Salchow",
        id: MoveId::Skating(SkatingMoveId::Salchow(JumpCount::Single)),
        summary: "Salchow jump",
        example: "LBI-1S",
        visible: true,
        params: &JUMP_PARAMS,
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: JumpCount,
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let regular = match entry_code {
            code!(LBI) => true,
            code!(RBI) => false,
            _ => return Err(parser::fail(input)),
        };

        let entry_angle = params[0].value.as_i32(input)?;
        let entry_len = params[1].value.as_i32(input)?;
        let exit_angle = params[2].value.as_i32(input)?;
        let exit_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let jump_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let prefix = pre_transition.prefix();
        let out_code = if regular { code!(RBO) } else { code!(LBO) };
        let hop_foot = if regular { "L" } else { "R" };

        let entry_len1 = entry_len / 2;
        let entry_len2 = entry_len / 3;
        let entry_len3 = entry_len / 6;
        let entry_len4 = entry_len / 12;

        let entry_angle1 = 2 * entry_angle / 3;
        let entry_angle2 = entry_angle;
        let entry_angle3 = 5 * entry_angle / 3;
        let entry_angle4 = 7 * entry_angle / 3;

        let entry1 = format!("{prefix}{entry_code}[angle={entry_angle1},len={entry_len1},style=\"{style}\",label-offset={label_offset}]");
        let entry2 = format!(
            "{entry_code}[angle={entry_angle2},len={entry_len2},style=\"{style}\",label=\" \"]"
        );
        let entry3 = format!(
            "{entry_code}[angle={entry_angle3},len={entry_len3},style=\"{style}\",label=\" \"]"
        );
        let entry4 = format!(
            "{entry_code}[angle={entry_angle4},len={entry_len4},style=\"{style}\",label=\" \"]"
        );
        let label = if jump_label.is_empty() {
            format!("{count}{}", Self::JUMP)
        } else {
            jump_label.to_string()
        };
        let hop = format!("{hop_foot}B-Hop [label=\"{label}\"]");
        let shift = if regular {
            "Shift[side=-200,fwd=-150,rotate=120]"
        } else {
            "Shift[side=200,fwd=-150,rotate=-120]"
        }
        .to_string();

        let exit = format!(
            "{out_code}[angle={exit_angle},len={exit_len},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!(
            "input {input:?} results in {entry1};{entry2};{entry3};{entry4};{hop};{shift};{exit}"
        );
        let moves = vec![
            Curve::construct(&entry1, text_pos).unwrap(),
            Curve::construct(&entry2, text_pos).unwrap(),
            Curve::construct(&entry3, text_pos).unwrap(),
            Curve::construct(&entry4, text_pos).unwrap(),
            Hop::construct(&hop, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}-{}{}{suffix}", count, Self::JUMP);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Salchow(count),
            moves,
            params,
            text,
        ))
    }
}

/// Loop jump.
pub struct Loop;

impl Loop {
    /// Jump code.
    pub const JUMP: &str = "Lo";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Loop",
        id: MoveId::Skating(SkatingMoveId::LoopJump(JumpCount::Single)),
        summary: "Loop jump",
        example: "RBO-1Lo",
        visible: true,
        params: &JUMP_PARAMS,
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: JumpCount,
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let regular = match entry_code {
            code!(RBO) => true,
            code!(LBO) => false,
            _ => return Err(parser::fail(input)),
        };

        let entry_angle = params[0].value.as_i32(input)?;
        let entry_len = params[1].value.as_i32(input)?;
        let exit_angle = params[2].value.as_i32(input)?;
        let exit_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let jump_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let prefix = pre_transition.prefix();
        let out_code = if regular { code!(RBO) } else { code!(LBO) };
        let hop_foot = if regular { "R" } else { "L" };

        let entry_len1 = entry_len / 2;
        let entry_len2 = entry_len / 3;
        let entry_len3 = entry_len / 6;
        let entry_len4 = entry_len / 12;

        let entry_angle1 = 2 * entry_angle / 3;
        let entry_angle2 = entry_angle;
        let entry_angle3 = 5 * entry_angle / 3;
        let entry_angle4 = 7 * entry_angle / 3;

        let entry1 = format!("{prefix}{entry_code}[angle={entry_angle1},len={entry_len1},style=\"{style}\",label-offset={label_offset}]");
        let entry2 = format!(
            "{entry_code}[angle={entry_angle2},len={entry_len2},style=\"{style}\",label=\" \"]"
        );
        let entry3 = format!(
            "{entry_code}[angle={entry_angle3},len={entry_len3},style=\"{style}\",label=\" \"]"
        );
        let entry4 = format!(
            "{entry_code}[angle={entry_angle4},len={entry_len4},style=\"{style}\",label=\" \"]"
        );
        let label = if jump_label.is_empty() {
            format!("{count}{}", Self::JUMP)
        } else {
            jump_label.to_string()
        };
        let hop = format!("{hop_foot}B-Hop [label=\"{label}\"]");
        let shift = if regular {
            "Shift[side=-200,fwd=-150,rotate=120]"
        } else {
            "Shift[side=200,fwd=-150,rotate=-120]"
        }
        .to_string();

        let exit = format!(
            "{out_code}[angle={exit_angle},len={exit_len},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!(
            "input {input:?} results in {entry1};{entry2};{entry3};{entry4};{hop};{shift};{exit}"
        );
        let moves = vec![
            Curve::construct(&entry1, text_pos).unwrap(),
            Curve::construct(&entry2, text_pos).unwrap(),
            Curve::construct(&entry3, text_pos).unwrap(),
            Curve::construct(&entry4, text_pos).unwrap(),
            Hop::construct(&hop, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}-{}{}{suffix}", count, Self::JUMP);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::LoopJump(count),
            moves,
            params,
            text,
        ))
    }
}

/// Axel jump.
pub struct Axel;

impl Axel {
    /// Jump code.
    pub const JUMP: &str = "A";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Axel",
        id: MoveId::Skating(SkatingMoveId::Axel(JumpCount::Single)),
        summary: "Axel jump",
        example: "LFO-1A",
        visible: true,
        params: &JUMP_PARAMS,
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: JumpCount,
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let regular = match entry_code {
            code!(LFO) => true,
            code!(RFO) => false,
            _ => return Err(parser::fail(input)),
        };

        let entry_angle = params[0].value.as_i32(input)?;
        let entry_len = params[1].value.as_i32(input)?;
        let exit_angle = params[2].value.as_i32(input)?;
        let exit_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let jump_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let prefix = pre_transition.prefix();
        let out_code = if regular { code!(RBO) } else { code!(LBO) };
        let hop_foot = if regular { "R" } else { "L" };

        let entry_len1 = entry_len / 2;
        let entry_len2 = entry_len / 3;
        let entry_len3 = entry_len / 6;

        let entry_angle1 = 2 * entry_angle / 3;
        let entry_angle2 = entry_angle;
        let entry_angle3 = 5 * entry_angle / 3;

        let entry1 = format!("{prefix}{entry_code}[angle={entry_angle1},len={entry_len1},style=\"{style}\",label-offset={label_offset}]");
        let entry2 = format!(
            "{entry_code}[angle={entry_angle2},len={entry_len2},style=\"{style}\",label=\" \"]"
        );
        let entry3 = format!(
            "{entry_code}[angle={entry_angle3},len={entry_len3},style=\"{style}\",label=\" \"]"
        );
        let label = if jump_label.is_empty() {
            format!("{count}{}", Self::JUMP)
        } else {
            jump_label.to_string()
        };
        let hop = format!("{hop_foot}B-Hop [label=\"{label}\"]");
        let shift = if regular {
            "Shift[side=-200,fwd=-150,rotate=120]"
        } else {
            "Shift[side=200,fwd=-150,rotate=-120]"
        }
        .to_string();

        let exit = format!(
            "{out_code}[angle={exit_angle},len={exit_len},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry1};{entry2};{entry3};{hop};{shift};{exit}");
        let moves = vec![
            Curve::construct(&entry1, text_pos).unwrap(),
            Curve::construct(&entry2, text_pos).unwrap(),
            Curve::construct(&entry3, text_pos).unwrap(),
            Hop::construct(&hop, text_pos).unwrap(),
            Shift::construct(&shift, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}-{}{}{suffix}", count, Self::JUMP);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Axel(count),
            moves,
            params,
            text,
        ))
    }
}

/// ToeLoop jump.
pub struct ToeLoop;

impl ToeLoop {
    /// Jump code.
    pub const JUMP: &str = "T";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Toe Loop",
        id: MoveId::Skating(SkatingMoveId::ToeLoop(JumpCount::Single)),
        summary: "Toe Loop jump",
        example: "RBO-1T",
        visible: true,
        params: &JUMP_PARAMS,
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: JumpCount,
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let regular = match entry_code {
            code!(RBO) => true,
            code!(LBO) => false,
            _ => return Err(parser::fail(input)),
        };

        let entry_angle = params[0].value.as_i32(input)?;
        let entry_len = params[1].value.as_i32(input)?;
        let exit_angle = params[2].value.as_i32(input)?;
        let exit_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let jump_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let prefix = pre_transition.prefix();
        let out_code = if regular { code!(RBO) } else { code!(LBO) };
        let hop_foot = if regular { "R" } else { "L" };

        let entry = format!("{prefix}{entry_code}[angle={entry_angle},len={entry_len},style=\"{style}\",label-offset={label_offset}]");
        let shift1 = if regular {
            "Shift[side=100,fwd=50]"
        } else {
            "Shift[side=-100,fwd=50]"
        }
        .to_string();
        let label = if jump_label.is_empty() {
            format!("{count}{}", Self::JUMP)
        } else {
            jump_label.to_string()
        };
        let hop = format!("{hop_foot}B-Hop [label=\"{label}\"]");
        let shift2 = if regular {
            "Shift[side=-50,fwd=150]"
        } else {
            "Shift[side=50,fwd=150]"
        }
        .to_string();

        let exit = format!(
            "{out_code}[angle={exit_angle},len={exit_len},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry};{shift1};{hop};{shift2};{exit}");

        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            Shift::construct(&shift1, text_pos).unwrap(),
            Hop::construct(&hop, text_pos).unwrap(),
            Shift::construct(&shift2, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}-{}{}{suffix}", count, Self::JUMP);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::ToeLoop(count),
            moves,
            params,
            text,
        ))
    }
}

/// Flip jump.
pub struct Flip;

impl Flip {
    /// Jump code.
    pub const JUMP: &str = "F";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: "Flip",
        id: MoveId::Skating(SkatingMoveId::Flip(JumpCount::Single)),
        summary: "Flip jump",
        example: "LBI-1F",
        visible: true,
        params: &JUMP_PARAMS,
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        pre_transition: PreTransition,
        entry_code: Code,
        count: JumpCount,
        params: Vec<MoveParam>,
    ) -> Result<Compound, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let regular = match entry_code {
            code!(LBI) => true,
            code!(RBI) => false,
            _ => return Err(parser::fail(input)),
        };

        let entry_angle = params[0].value.as_i32(input)?;
        let entry_len = params[1].value.as_i32(input)?;
        let exit_angle = params[2].value.as_i32(input)?;
        let exit_len = params[3].value.as_i32(input)?;
        let style = params[4].value.as_str(input)?;
        let jump_label = params[5].value.as_str(input)?;
        let label_offset = params[6].value.as_i32(input)?;

        let prefix = pre_transition.prefix();
        let out_code = if regular { code!(RBO) } else { code!(LBO) };
        let hop_foot = if regular { "R" } else { "L" };

        let entry = format!("{prefix}{entry_code}[angle={entry_angle},len={entry_len},style=\"{style}\",label-offset={label_offset}]");
        let shift1 = if regular {
            "Shift[side=100,fwd=50]"
        } else {
            "Shift[side=-100,fwd=50]"
        }
        .to_string();
        let label = if jump_label.is_empty() {
            format!("{count}{}", Self::JUMP)
        } else {
            jump_label.to_string()
        };
        let hop = format!("{hop_foot}B-Hop [label=\"{label}\"]");
        let shift2 = if regular {
            "Shift[side=-50,fwd=150]"
        } else {
            "Shift[side=50,fwd=150]"
        }
        .to_string();

        let exit = format!(
            "{out_code}[angle={exit_angle},len={exit_len},style=\"{style}\",label-offset={label_offset}]"
        );

        log::info!("input {input:?} results in {entry};{shift1};{hop};{shift2};{exit}");

        let moves = vec![
            Curve::construct(&entry, text_pos).unwrap(),
            Shift::construct(&shift1, text_pos).unwrap(),
            Hop::construct(&hop, text_pos).unwrap(),
            Shift::construct(&shift2, text_pos).unwrap(),
            Curve::construct(&exit, text_pos).unwrap(),
        ];

        let prefix = pre_transition.prefix();
        let suffix = params::to_string(Self::INFO.params, &params);
        let text = format!("{prefix}{entry_code}-{}{}{suffix}", count, Self::JUMP);

        Ok(Compound::new(
            input,
            text_pos,
            SkatingMoveId::Flip(count),
            moves,
            params,
            text,
        ))
    }
}

/*

Lutz

LBO [len=600,angle=45]
Shift [fwd=50,side=50]
LB-Hop [label="1Lz"]
Shift [fwd=100,side=-50]
RBO [len=400,angle=40]

*/
