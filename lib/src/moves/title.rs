//! Pseudo-move definition for diagram title.

use crate::{
    moves::{self, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    parser, Bounds, Group, Move, MoveParam, Position, RenderOptions, Skater, SvgId, TextPosition,
};
use std::borrow::Cow;
use svg::{node::element::Text, Document};

pub struct Title {
    text_pos: TextPosition,
    text: String,
    pos: Position,
    font_size: Option<u32>,
}

impl Title {
    pub const MOVE: &'static str = "Title";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        summary: "Diagram title",
        example: "Title[text=\"Waltz\"]",
        visible: false,
        params: &[
            params::Info {
                name: "text",
                doc: "Text of title",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "x",
                doc: "Horizontal location of title; -1 indicates automatic centering",
                default: Value::Number(-1),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "y",
                doc: "Vertical location of title",
                default: Value::Number(100),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "font-size",
                doc: "Font size for title; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
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
        let font_size = params[3].value.as_i32(input)?;

        Ok(Self {
            text_pos,
            text: params[0].value.as_str(input)?.to_string(),
            pos: Position::from_params(&params[1], &params[2]),
            font_size: if font_size > 0 {
                Some(font_size as u32)
            } else {
                None
            },
        })
    }

    fn font_size(&self, opts: &RenderOptions) -> u32 {
        self.font_size.unwrap_or_else(|| opts.font_size() * 2)
    }
}

impl Move for Title {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Title)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn defs(&self, opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        opts.title.clone_from(&self.text);
        Vec::new()
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        let x = if self.pos.x >= 0 {
            self.pos.x
        } else {
            opts.bounds.midpoint().x
        };
        let mut text = Text::new(self.text.clone())
            .set("x", x)
            .set("y", self.pos.y)
            .set(
                "style",
                format!(
                    "stroke:black; fill:black; font-size: {}pt;",
                    self.font_size(opts)
                ),
            );
        if let Some(pos) = self.text_pos() {
            let unique_id = opts.next_unique_id(pos);
            text = text.set("id", unique_id);
        }
        doc.add(text)
    }
}
