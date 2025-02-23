//! Pseudo-move definition for diagram text.

use crate::{
    moves::{self, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    parser, Bounds, Direction, Move, MoveParam, Position, RenderOptions, Skater, SvgId,
    TextPosition,
};
use std::borrow::Cow;
use svg::{node::element::Text as SvgText, Document};

pub struct Text {
    text_pos: TextPosition,
    text: String,
    pos: Position,
    font_size: Option<u32>,
    rotate: i32,
}

impl Text {
    pub const MOVE: &'static str = "Text";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        id: MoveId::Pseudo(PseudoMoveId::Text),
        summary: "Diagram text",
        example: "Text[text=\"Start\",x=500,y=200]",
        visible: false,
        params: &[
            params::Info {
                name: "text",
                doc: "Text to display",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "x",
                doc: "Horizontal location of text",
                default: Value::Number(100),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "y",
                doc: "Vertical location of text (increasing down)",
                default: Value::Number(100),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "font-size",
                doc: "Font size for label; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "rotate",
                doc: "Angle to rotate text by",
                default: Value::Number(0),
                range: params::Range::Any,
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
            rotate: params[4].value.as_i32(input)?,
        })
    }

    fn font_size(&self, opts: &RenderOptions) -> u32 {
        self.font_size.unwrap_or_else(|| opts.font_size())
    }
}

impl Move for Text {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Text)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("x" = (self.pos.x as i32)),
            param!("y" = (self.pos.y as i32)),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
            param!(self.rotate),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        // TODO: cope with a `RenderOptions`-specified font size (rather than just guessing 10).
        let font_size = self.font_size.unwrap_or(10) as i64;
        let dir = Direction::new(self.rotate);
        Some(Bounds::for_text_at(&self.text, self.pos, font_size, dir))
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        let mut text = SvgText::new(self.text.clone())
            .set("x", self.pos.x)
            .set("y", self.pos.y)
            .set(
                "style",
                format!(
                    "stroke:black; fill:black; font-size:{}pt;",
                    self.font_size(opts)
                ),
            );
        if self.rotate != 0 {
            text = text.set(
                "transform",
                format!("rotate({},{},{})", self.rotate, self.pos.x, self.pos.y),
            )
        }
        if let Some(pos) = self.text_pos() {
            let unique_id = opts.next_unique_id(pos);
            text = text.set("id", unique_id);
        }
        doc.add(text)
    }
}
