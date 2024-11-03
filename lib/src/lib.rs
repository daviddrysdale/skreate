//! Skating diagram creator.
#![warn(missing_docs)]

pub use crate::error::ParseError;
pub use crate::params::MoveParam;
pub use crate::types::*;
use log::{debug, info, trace};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use svg::{
    node::element::{Definitions, Description, Group, Rectangle, Style, Text, Title, Use},
    Document,
};

mod error;
pub mod moves;
pub mod params;
mod types;

/// Extra margin to put around calculated bounding box.
const MARGIN: i64 = 50;

/// Common style definitions.
pub const STYLE_DEF: &str =
    "text { text-anchor: middle } path,rect,circle { fill:none; stroke: black; }";

/// Description of current skater state.
#[derive(Debug, Clone, Copy)]
struct Skater {
    pos: Position,
    dir: Direction,
    code: Code,
}

impl Display for Skater {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({},{}) {}° {}",
            self.pos.x, self.pos.y, self.dir.0, self.code
        )
    }
}

impl std::ops::Add<Position> for Skater {
    type Output = Self;
    fn add(self, delta: Position) -> Self {
        // Start position
        let start_x = self.pos.x as f64;
        let start_y = self.pos.y as f64;

        // Delta in coords if we were aligned with `Direction(0)` ...
        let delta_x = delta.x as f64;
        let delta_y = delta.y as f64;

        // ... but we're not, we're moving at an angle:
        let angle = self.dir.0 as f64 * std::f64::consts::PI / 180.0;
        let dx = delta_x * angle.cos() - delta_y * angle.sin();
        let dy = delta_y * angle.cos() + delta_x * angle.sin();
        trace!("  ({delta_x:+.1},{delta_y:+.1}) at {angle} radians => move ({dx:+.1},{dy:+.1})");

        let new_x = start_x + dx;
        let new_y = start_y + dy;

        Skater {
            pos: pos!(new_x as i64, new_y as i64),
            dir: self.dir,
            code: self.code,
        }
    }
}
impl std::ops::Add<Transition> for Skater {
    type Output = Self;
    fn add(self, transition: Transition) -> Self {
        let mut moved = self;
        match transition.spatial {
            SpatialTransition::Relative { delta, rotate } => {
                moved = moved + delta;
                moved.dir = self.dir + rotate;
            }
            SpatialTransition::Absolute { pos, dir } => {
                moved.pos = pos;
                moved.dir = dir;
            }
        }
        if let Some(new_code) = transition.code {
            moved.code = new_code;
        }
        moved
    }
}

/// Options for how to render the diagram.
#[derive(Debug, Clone, Default)]
struct RenderOptions {
    /// Diagram title.
    title: String,
    /// Whether to render start/end markers.
    markers: bool,
    /// Grid size.
    grid: Option<usize>,
    /// Whether to show bounds.
    show_bounds: bool,
    /// Whether to show bounds of individual moves.
    show_move_bounds: bool,
    /// Calculated bounds.
    bounds: Bounds,
    /// Font size; auto-scale to bounds if [`None`].
    font_size: Option<u32>,
    /// Stroke width; auto-scale with bounds if [`None`].
    stroke_width: Option<u32>,
}

impl RenderOptions {
    fn bounds_diag(&self) -> f64 {
        let diag_squared =
            self.bounds.width() * self.bounds.width() + self.bounds.height() * self.bounds.height();
        (diag_squared as f64).sqrt()
    }

    /// Return the effective font-size in points.
    pub fn font_size(&self) -> u32 {
        if let Some(font_size) = &self.font_size {
            *font_size
        } else {
            let diagonal = self.bounds_diag();
            let pts = if diagonal < 500.0 {
                10
            } else if diagonal < 800.0 {
                12
            } else if diagonal < 933.0 {
                14
            } else if diagonal < 1067.0 {
                16
            } else if diagonal < 1200.0 {
                18
            } else if diagonal < 1600.0 {
                20
            } else if diagonal < 2400.0 {
                22
            } else {
                24
            };
            debug!("diagonal dimension {diagonal} => {pts}pts text");
            pts
        }
    }
    /// Return the effective stroke-width.
    pub fn stroke_width(&self) -> u32 {
        if let Some(stroke_width) = &self.stroke_width {
            *stroke_width
        } else {
            let diagonal = self.bounds_diag();
            let width = if diagonal < 1000.0 {
                1
            } else if diagonal < 1600.0 {
                2
            } else if diagonal < 2400.0 {
                3
            } else {
                4
            };
            debug!("diagonal dimension {diagonal} => stroke-width: {width}");
            width
        }
    }
}

fn use_at(skater: &Skater, def_id: &str, opts: &RenderOptions) -> Use {
    Use::new()
        .set("xlink:href", format!("#{def_id}"))
        .set(
            "transform",
            format!(
                "translate({} {}) rotate({})",
                skater.pos.x, skater.pos.y, skater.dir.0
            ),
        )
        .set("style", format!("stroke-width:{};", opts.stroke_width()))
}

/// Trait describing the external behavior of a move.
trait Move {
    /// Parameters for the move.
    fn params(&self) -> Vec<MoveParam>;

    /// Start of the move.
    fn start(&self) -> Option<Code> {
        None
    }

    /// End of the move.
    fn end(&self) -> Option<Code> {
        self.start()
    }

    /// Transition needed before starting the move, starting from `Direction(0)`.
    fn pre_transition(&self, _from: Code) -> Transition {
        Transition::default()
    }

    /// Transition as a result of the move, starting from `Direction(0)`, and assuming that [`pre_transition`] has
    /// already happened.
    fn transition(&self) -> Transition {
        Transition::default()
    }

    /// Return a bounding box that encompasses the move, starting from `before`.
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        // The default implementation just encompasses the before and after positions.
        let mut bounds = bounds!(before.pos.x, before.pos.y => before.pos.x,before.pos.y);
        let after = *before + self.transition();
        bounds.encompass(&after.pos);

        Some(bounds)
    }

    /// Emit SVG group definition for the move.
    fn def(&self, _opts: &mut RenderOptions) -> Option<Group> {
        None
    }

    /// Return the labels for this move. Each returned position is relative to (0,0) at 0°.
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        Vec::new()
    }

    /// Render the move into the given SVG document, assuming the existence of groups included in the output from [`defs`].
    fn render(&self, mut doc: Document, start: &Skater, opts: &mut RenderOptions) -> Document {
        // Default implementation uses the definition, suitably translated and rotated.
        let def_id = self.text();
        let mut use_link = use_at(start, &def_id, opts);
        if let Some(input) = self.input() {
            use_link = use_link.set("id", input.unique_id());
        }
        doc = doc.add(use_link);
        for label in self.labels(opts) {
            let loc = *start + label.pos;
            let text = Text::new(label.text)
                .set("x", loc.pos.x)
                .set("y", loc.pos.y)
                .set("style", format!("font-size:{}pt;", opts.font_size()));
            doc = doc.add(text);
        }
        doc
    }

    /// Emit text that describes the move.  Feeding this text into `moves::factory` should result in the
    /// same `Move` (although it may have different `input_text`).
    fn text(&self) -> String;

    /// Emit the input that was used to originally create the move, if available.  This may have different text
    /// (e.g. longer, using alias forms) than the result of [`text`].
    fn input(&self) -> Option<OwnedInput>;
}

/// Generate canonicalized / minimized input.
pub fn canonicalize(input: &str) -> Result<String, ParseError> {
    // Convert the input into a list of move input strings.
    let inputs = split_inputs(input)?;

    let moves = inputs
        .iter()
        .map(|input| moves::factory(input))
        .collect::<Result<Vec<_>, ParseError>>()?;
    let min_inputs = moves.into_iter().map(|m| m.text()).collect::<Vec<_>>();
    Ok(min_inputs.join(";"))
}

/// Generate SVG for the given input.
pub fn generate(input: &str) -> Result<String, ParseError> {
    // Convert the input into a list of move input strings.
    let inputs = split_inputs(input)?;

    let moves = inputs
        .iter()
        .map(|input| moves::factory(input))
        .collect::<Result<Vec<_>, ParseError>>()?;

    let mut doc = Document::new().set("xmlns:xlink", "http://www.w3.org/1999/xlink");
    let mut opts = RenderOptions {
        title: "Skating Diagram".to_string(),
        ..Default::default()
    };

    // First pass: emit definitions for all moves in use, and get global option updates.
    info!("========= emit definitions ===========");
    let style = Style::new(STYLE_DEF);
    let mut seen = HashSet::new();
    let mut defs = Definitions::new().add(style);
    for mv in &moves {
        let id = mv.text();
        if seen.contains(&id) {
            continue;
        }
        if let Some(group) = mv.def(&mut opts) {
            seen.insert(id.clone());
            defs = defs.add(group.set("id", id));
        }
    }
    doc = doc
        .add(Title::new(opts.title.clone()))
        .add(Description::new().add(Text::new(opts.title.clone())));
    doc = doc.add(defs);

    // Second pass: figure out a bounding box, starting at (0,0) facing 0.
    info!("========= determine bounding box ===========");
    let mut bounds: Option<Bounds> = None;
    let mut skater = Skater {
        pos: Position::default(),
        dir: Direction::new(0),
        code: code!(BF),
    };
    let mut first = true;
    for mv in &moves {
        if first {
            // Don't apply pre-transition for first move.
            if let Some(start_code) = mv.start() {
                skater.code = start_code;
            }
            debug!("start: {skater}");
        } else {
            let pre_transition = mv.pre_transition(skater.code);
            skater = skater + pre_transition;
            debug!("pre:  add {pre_transition} ==> {skater}");
        };

        let move_bounds = mv.bounds(&skater);

        if let Some(move_bounds) = move_bounds {
            match &mut bounds {
                Some(bounds) => bounds.encompass_bounds(&move_bounds),
                None => bounds = Some(move_bounds),
            }
            debug!("bounds.encompass({move_bounds})");
            if opts.show_move_bounds {
                doc = doc.add(
                    Rectangle::new()
                        .set("width", move_bounds.width())
                        .set("height", move_bounds.height())
                        .set("x", move_bounds.top_left.x)
                        .set("y", move_bounds.top_left.y)
                        .set("stroke-dasharray", "2,2")
                        .set(
                            "style",
                            format!("stroke:blue; stroke-width:{};", 2 * opts.stroke_width()),
                        ),
                );
            }
        }

        let transition = mv.transition();
        let after = skater + transition;
        debug!("post: {skater} + {transition} ==> {after}");

        skater = after;
        first = false;
    }
    let bounds = bounds.unwrap_or_default();
    opts.bounds = bounds;
    info!("calculated bounds {bounds}");

    // Add a margin.
    let mut outer_bounds = bounds;
    outer_bounds.add_margin(MARGIN, MARGIN);
    doc = doc
        .set("width", outer_bounds.width())
        .set("height", outer_bounds.height());
    info!("add {MARGIN} to get {outer_bounds}");

    // Third pass: render all the moves.
    info!("========= render ===========");
    let mut skater = Skater {
        pos: Position::default(),
        dir: Direction::new(0),
        code: code!(BF),
    };
    let mut first = true;
    for mv in &moves {
        if first {
            // Don't apply pre-transition for first move.
            if let Some(start_code) = mv.start() {
                skater.code = start_code;
            }
            debug!("start: {skater}");
        } else {
            let pre_transition = mv.pre_transition(skater.code);
            skater = skater + pre_transition;
            debug!("pre:  add {pre_transition} ==> {skater}");
        };

        info!("{:?} => {:?}", mv.start(), mv.end());
        debug!("perform: {}", mv.text());
        if opts.markers {
            doc = doc.add(use_at(&skater, "start-mark", &opts));
        }
        let show_marker = opts.markers;
        doc = mv.render(doc, &skater, &mut opts);

        let transition = mv.transition();
        let after = skater + transition;
        debug!("post: {skater} + {transition} ==> {after}");
        if show_marker {
            doc = doc.add(use_at(&after, "end-mark", &opts));
        }

        skater = after;
        first = false;
    }

    if let Some(grid) = opts.grid {
        let grid = grid as i64;
        let n = (bounds.top_left.x + grid - 1) / grid;
        let mut x = grid * n;
        while x < bounds.bottom_right.x {
            let y1 = bounds.top_left.y;
            let y2 = bounds.bottom_right.y;
            let stroke = if x == 0 {
                "stroke:gray; stroke-width:2;"
            } else {
                "stroke:lightgray"
            };
            doc = doc.add(path!("M {x},{y1} L {x},{y2}").set("style", stroke));
            x += grid;
        }
        let n = (bounds.top_left.y + grid - 1) / grid;
        let mut y = grid * n;
        while y < bounds.bottom_right.y {
            let x1 = bounds.top_left.x;
            let x2 = bounds.bottom_right.x;
            let stroke = if y == 0 {
                "stroke:gray; stroke-width:2;"
            } else {
                "stroke:lightgray"
            };
            doc = doc.add(path!("M {x1},{y} L {x2},{y}").set("style", stroke));
            y += grid;
        }
    }
    if opts.show_bounds {
        doc = doc.add(
            Rectangle::new()
                .set("width", outer_bounds.width())
                .set("height", outer_bounds.height())
                .set("x", outer_bounds.top_left.x)
                .set("y", outer_bounds.top_left.y)
                .set("stroke-dasharray", "5,5")
                .set(
                    "style",
                    format!("stroke:red; stroke-width:{};", 3 * opts.stroke_width()),
                ),
        );
        doc = doc.add(
            Rectangle::new()
                .set("width", bounds.width())
                .set("height", bounds.height())
                .set("x", bounds.top_left.x)
                .set("y", bounds.top_left.y)
                .set("stroke-dasharray", "5,5")
                .set(
                    "style",
                    format!("stroke:green; stroke-width:{};", 3 * opts.stroke_width()),
                ),
        );
    }

    // Set the viewBox to the outer bounds.
    doc = doc.set(
        "viewBox",
        format!(
            "{} {} {} {}",
            outer_bounds.top_left.x,
            outer_bounds.top_left.y,
            outer_bounds.width(),
            outer_bounds.height()
        ),
    );

    let mut svg = Vec::new();
    svg::write(&mut svg, &doc)?;
    let svg = String::from_utf8(svg)?;
    trace!("emit SVG:\n{svg}");
    Ok(svg)
}

/// User input.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Input<'a> {
    pos: TextPosition,
    text: &'a str,
}

impl<'a> Input<'a> {
    fn owned(&self) -> OwnedInput {
        OwnedInput {
            pos: self.pos,
            text: self.text.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedInput {
    pos: TextPosition,
    text: String,
}

impl OwnedInput {
    fn unique_id(&self) -> String {
        format!(
            "r_{}_c_{}_{}",
            self.pos.row,
            self.pos.col,
            self.pos.col + self.text.chars().count()
        )
    }
}

fn split_inputs(input: &str) -> Result<Vec<Input>, ParseError> {
    // TODO: deal with quoted strings
    Ok(input
        .split('\n')
        .map(strip_comment) // stripping trailing comments doesn't affect column numbers
        .enumerate()
        .map(|(row, l)| Input {
            pos: TextPosition { row, col: 0 },
            text: l,
        })
        .flat_map(split_inputs_in_line)
        .map(trim_whitespace)
        .filter(|input| !input.text.is_empty())
        .collect::<Vec<_>>())
}

fn split_inputs_in_line(input: Input) -> Vec<Input> {
    let Input {
        pos: TextPosition { row, col },
        text,
    } = input;

    let mut results = Vec::new();
    let mut offset = 0; // in bytes
    let mut col = col; // in characters
    loop {
        match text[offset..].find(';') {
            Some(delta) => {
                let text = &text[offset..offset + delta];
                let num_chars = text.chars().count();
                results.push(Input {
                    pos: TextPosition { row, col },
                    text,
                });
                offset += delta + 1;
                col += num_chars + 1;
            }
            None => {
                results.push(Input {
                    pos: TextPosition { row, col },
                    text: &text[offset..],
                });
                return results;
            }
        }
    }
}

fn trim_whitespace(input: Input) -> Input {
    let Input {
        pos: TextPosition { row, col },
        text,
    } = input;
    // Trimming whitespace off the end doesn't affect columns.
    let text = text.trim_end();
    let nonws_offset = text.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let col = col + text[..nonws_offset].chars().count();
    Input {
        pos: TextPosition { row, col },
        text: &text[nonws_offset..],
    }
}

fn strip_comment(l: &str) -> &str {
    match l.split_once('#') {
        Some((before, _after)) => before,
        None => l,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_whitespace() {
        let tests = [
            (
                Input {
                    pos: TextPosition { row: 0, col: 0 },
                    text: "text",
                },
                Input {
                    pos: TextPosition { row: 0, col: 0 },
                    text: "text",
                },
            ),
            (
                Input {
                    pos: TextPosition { row: 0, col: 0 },
                    text: "  text  ",
                },
                Input {
                    pos: TextPosition { row: 0, col: 2 },
                    text: "text",
                },
            ),
            (
                Input {
                    pos: TextPosition { row: 10, col: 10 },
                    text: "  text with",
                },
                Input {
                    pos: TextPosition { row: 10, col: 12 },
                    text: "text with",
                },
            ),
        ];
        for (input, want) in tests {
            let got = trim_whitespace(input.clone());
            assert_eq!(got, want, "for input: {input:?}");
        }
    }

    #[test]
    fn test_split_inputs() {
        let tests = [
            ("", vec![]),
            ("abc", vec![("abc", 0, 0)]),
            ("a\nb\nc", vec![("a", 0, 0), ("b", 1, 0), ("c", 2, 0)]),
            ("  a\nb  \n c ", vec![("a", 0, 2), ("b", 1, 0), ("c", 2, 1)]),
            (
                "  a #comment\nb  \n #just comment\n c ",
                vec![("a", 0, 2), ("b", 1, 0), ("c", 3, 1)],
            ),
            (
                "a;b\nc; d\n¥;e",
                vec![
                    ("a", 0, 0),
                    ("b", 0, 2),
                    ("c", 1, 0),
                    ("d", 1, 3),
                    ("¥", 2, 0),
                    ("e", 2, 2), // Note: character offset not byte offset.
                ],
            ),
        ];
        for (input, want) in tests {
            let want = want
                .iter()
                .map(|(text, row, col)| Input {
                    text,
                    pos: TextPosition {
                        row: *row,
                        col: *col,
                    },
                })
                .collect::<Vec<_>>();
            let got = split_inputs(input).unwrap();
            assert_eq!(got, want, "for input: {input}");
        }
    }

    #[test]
    fn test_strip_comment() {
        let tests = [
            ("", ""),
            ("text", "text"),
            (" text ", " text "),
            (" text # comment", " text "),
            (" text # comment # comment", " text "),
            ("# comment", ""),
        ];
        for (line, want) in tests {
            let got = strip_comment(line);
            assert_eq!(got, want, "for input: {line}");
        }
    }

    #[test]
    fn test_code() {
        assert_eq!(
            code!(LFO),
            Code {
                foot: Foot::Left,
                dir: SkatingDirection::Forward,
                edge: Edge::Outside
            }
        );
    }
}
