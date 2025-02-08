//! Pseudo-move definition for diagram info.

use crate::{
    moves::{self, MoveId, PseudoMoveId},
    param, params,
    params::Value,
    parser, path, Bounds, Count, Document, Move, MoveParam, Position, RenderOptions, Skater, SvgId,
    TextPosition,
};
use svg::node::element::Group;

pub struct Info {
    text_pos: TextPosition,
    markers: bool,
    bounds: bool,
    grid: Option<i32>,
    margin: Position,
    move_bounds: bool,
    font_size: Option<u32>,
    stroke_width: Option<u32>,
    auto_count: bool,
    count_from: i32,
}

impl Info {
    pub const MOVE: &'static str = "Info";
    /// Static move information.
    pub const INFO: moves::Info = moves::Info {
        name: Self::MOVE,
        summary: "Set diagram rendering information",
        example: "Info[markers=true,grid=100,move-bounds=true]",
        visible: false,
        params: &[
            params::Info {
                name: "markers",
                doc: "Whether to show begin/end move markers",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "bounds",
                doc: "Whether to show overall bounds",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "grid",
                doc: "Grid size to display, 0 for no grid",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "margin-x",
                doc: "Horizontal margin",
                default: Value::Number(crate::MARGIN as i32),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "margin-y",
                doc: "Vertical margin",
                default: Value::Number(crate::MARGIN as i32),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "move-bounds",
                doc: "Whether to show bounds of each move",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "font-size",
                doc: "Font size for labels; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "stroke-width",
                doc: "Stroke width; 0 for auto-scaling",
                default: Value::Number(0),
                range: params::Range::Positive,
                short: None,
            },
            params::Info {
                name: "auto-count",
                doc: "Whether to automatically count moves",
                default: Value::Boolean(false),
                range: params::Range::Boolean,
                short: None,
            },
            params::Info {
                name: "count-from",
                doc: "First number to count from when auto-counting",
                default: Value::Number(1),
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
        let grid = params[2].value.as_i32(input)?;
        let font_size = params[6].value.as_i32(input)?;
        let stroke_width = params[7].value.as_i32(input)?;

        Ok(Self {
            text_pos,
            markers: params[0].value.as_bool(input)?,
            bounds: params[1].value.as_bool(input)?,
            grid: if grid > 0 { Some(grid) } else { None },
            margin: Position::from_params(&params[3], &params[4]),
            move_bounds: params[5].value.as_bool(input)?,
            font_size: if font_size > 0 {
                Some(font_size as u32)
            } else {
                None
            },
            stroke_width: if stroke_width > 0 {
                Some(stroke_width as u32)
            } else {
                None
            },
            auto_count: params[8].value.as_bool(input)?,
            count_from: params[9].value.as_i32(input)?,
        })
    }
}

impl Move for Info {
    fn id(&self) -> MoveId {
        MoveId::Pseudo(PseudoMoveId::Info)
    }
    fn params(&self) -> Vec<MoveParam> {
        vec![
            param!(self.markers),
            param!(self.bounds),
            param!("grid" = (self.grid.unwrap_or(0))),
            param!("label-x" = (self.margin.x as i32)),
            param!("label-y" = (self.margin.y as i32)),
            param!("move-bounds" = self.move_bounds),
            param!("font-size" = (self.font_size.unwrap_or(0) as i32)),
            param!("stroke-width" = (self.stroke_width.unwrap_or(0) as i32)),
            param!("auto-count" = self.auto_count),
            param!("count-from" = self.count_from),
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
        None
    }
    fn defs(&self, opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        // Change some options once and for all in the prelude.
        opts.show_bounds = self.bounds;
        opts.grid = self.grid.map(|g| g as usize);
        opts.show_move_bounds = self.move_bounds;

        let mut grp = Group::new();
        if self.markers {
            grp = grp.add(
                path!(
                    "M 0,0 l 10,0 l -20,0 l 10,0 l 0,20 l 8,-8 l -8,8 l-8,-8 l 8,8 l 0,-30 l 0,10",
                )
                .set("style", "stroke:red;")
                .set("id", "end-mark"),
            );
            grp = grp.add(
                path!(
                    "M 0,0 l 10,0 l -20,0 l 10,0 l 0,20 l 8,-8 l -8,8 l-8,-8 l 8,8 l 0,-30 l 0,10",
                )
                .set("style", "stroke:green;")
                .set("id", "start-mark"),
            );
        }
        vec![(SvgId(self.text()), grp)]
    }
    fn render(
        &self,
        doc: Document,
        _start: &Skater,
        opts: &mut RenderOptions,
        _ns: Option<&SvgId>,
    ) -> Document {
        // Some options can be toggled on/off as we go along.
        opts.markers = self.markers;
        opts.font_size = self.font_size;
        opts.stroke_width = self.stroke_width;
        if self.auto_count {
            opts.auto_count = Some(Count(self.count_from));
        }

        doc
    }
}
