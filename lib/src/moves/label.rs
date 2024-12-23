//! Pseudo-move definition for label positioned relative to current position.

use crate::{
    moves, param, params, params::Value, parser, Bounds, Move, MoveParam, Position, RenderOptions,
    Rotation, Skater, SpatialTransition, SvgId, Transition,
};
use std::borrow::Cow;
use svg::{node::element::Text, Document};

pub struct Label {
    text: String,
    delta: Position,
    font_size: Option<u32>,
    rotate: i32,
}

impl Label {
    pub const MOVE: &'static str = "Label";
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        summary: "Diagram label relative to current position",
        example: "Label[text=\"CoE\",fwd=50,side=20]",
        visible: false,
        params: &[
            params::Info {
                name: "text",
                doc: "Label to display",
                default: Value::Text(Cow::Borrowed("")),
                range: params::Range::Text,
                short: None,
            },
            params::Info {
                name: "fwd",
                doc: "Location of text relative to current position, in current direction",
                default: Value::Number(0),
                range: params::Range::Any,
                short: None,
            },
            params::Info {
                name: "side",
                doc:
                    "Location of text relative to current position, sideways from current direction",
                default: Value::Number(0),
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

    pub fn construct(input: &str) -> Result<Box<dyn Move>, parser::Error> {
        let Some(rest) = input.strip_prefix(Self::INFO.name) else {
            return Err(parser::fail(input));
        };
        let params = params::populate(Self::INFO.params, rest)?;
        Ok(Box::new(Self::from_params(input, params)?))
    }

    pub fn from_params(input: &str, params: Vec<MoveParam>) -> Result<Self, parser::Error> {
        assert!(params::compatible(Self::INFO.params, &params));
        let font_size = params[3].value.as_i32(input)?;

        Ok(Self {
            text: params[0].value.as_str(input)?.to_string(),
            delta: Position::from_params(&params[2], &params[1]),
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

impl Move for Label {
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.text),
            param!("fwd" = (self.delta.y as i32)),
            param!("side" = (self.delta.x as i32)),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
            param!(self.rotate),
        ]
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::INFO.params, &self.params());
        format!("{}{params}", Self::INFO.name)
    }
    fn bounds(&self, _before: &Skater) -> Option<Bounds> {
        None
    }
    fn render(
        &self,
        doc: Document,
        start: &Skater,
        opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        let delta = Transition {
            spatial: SpatialTransition::Relative {
                delta: self.delta,
                rotate: Rotation(0),
            },
            code: None,
        };
        let pos = *start + delta;
        let mut text = Text::new(self.text.clone())
            .set("x", pos.pos.x)
            .set("y", pos.pos.y)
            .set("style", format!("font-size:{}pt;", self.font_size(opts)));
        if self.rotate != 0 {
            text = text.set(
                "transform",
                format!("rotate({},{},{})", self.rotate, pos.pos.x, pos.pos.y),
            )
        }
        doc.add(text)
    }
}
