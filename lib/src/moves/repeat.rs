// Copyright 2025 David Drysdale

//! Pseudo-move definition for repeat markers.

use crate::{
    moves::{self, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    parser, Bounds, Document, Move, MoveParam, RenderOptions, Skater, SvgId, TextPosition,
};

#[derive(Debug, Clone)]
pub struct RepeatStart {
    text_pos: TextPosition,
}

impl RepeatStart {
    pub const MOVE: &'static str = "RepeatStart";
    /// Allow a short code to mark repeat start, inspired by music repeat.
    pub const ALT_MOVE: &'static str = "|:";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Info),
        summary: "Mark start of repeating section",
        example: Self::ALT_MOVE,
        visible: false,
        params: &[],
    };

    pub fn from_params(
        _input: &str,
        text_pos: TextPosition,
        params: Vec<MoveParam>,
    ) -> Result<Self, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        Ok(Self { text_pos })
    }
}

impl Move for RepeatStart {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::RepeatStart)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![]
    }
    fn text(&self) -> String {
        // Always output the short form.
        Self::ALT_MOVE.to_string()
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        _opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        doc
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        self.box_clone(repeat)
    }
    fn box_clone(&self, repeat: Option<usize>) -> Box<dyn Move> {
        let mut copy = self.clone();
        copy.text_pos = self.text_pos.at_repeat(repeat);
        Box::new(copy)
    }
}

#[derive(Debug, Clone)]
pub struct RepeatEnd {
    text_pos: TextPosition,
    pub count: u32,
    pub alternate: bool,
}

impl RepeatEnd {
    pub const MOVE: &'static str = "RepeatEnd";
    /// Allow a short code to mark repeat end, inspired by music repeat.
    pub const ALT_MOVE_SAME: &'static str = ":|";
    pub const ALT_MOVE_OTHER: &'static str = "!|";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Info),
        summary: "Mark end of repeating section",
        example: "RepeatEnd [count=4,alternate=true]",
        visible: false,
        params: &[
            params::Info {
                name: "count",
                doc: "Number of repeats",
                default: Value::Number(2),
                range: params::Range::StrictlyPositive,
                short: None,
            },
            params::Info {
                name: "alternate",
                doc: "Whether to alternate opposite feet",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
        ],
    };

    pub fn from_params(
        input: &str,
        text_pos: TextPosition,
        params: Vec<MoveParam>,
    ) -> Result<Self, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let count = params[0].value.as_i32(input)? as u32;
        let alternate = params[1].value.as_bool(input)?;
        Ok(Self::new(text_pos, count, alternate))
    }

    pub fn new(text_pos: TextPosition, count: u32, alternate: bool) -> Self {
        Self {
            text_pos,
            count,
            alternate,
        }
    }
}

impl Move for RepeatEnd {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::RepeatEnd)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!("count" = (self.count as i32)),
            param!(self.alternate),
        ]
    }
    fn text(&self) -> String {
        let prefix = if self.alternate {
            Self::ALT_MOVE_OTHER
        } else {
            Self::ALT_MOVE_SAME
        };
        let suffix = if self.count == 2 {
            "".to_string()
        } else {
            format!("x{}", self.count)
        };
        format!("{prefix}{suffix}")
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        _opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        doc
    }
    fn opposite(&self, repeat: Option<usize>) -> Box<dyn Move> {
        self.box_clone(repeat)
    }
    fn box_clone(&self, repeat: Option<usize>) -> Box<dyn Move> {
        let mut copy = self.clone();
        copy.text_pos = self.text_pos.at_repeat(repeat);
        Box::new(copy)
    }
    fn as_repeat_end(&self) -> Option<&RepeatEnd> {
        Some(self)
    }
}
