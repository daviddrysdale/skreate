//! Skating diagram creator.
#![warn(missing_docs)]

use crate::direction::{Direction, Rotation};
use log::{debug, info, trace};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use svg::{
    node::{
        element::{Definitions, Description, Group, Title, Use},
        Text,
    },
    Document,
};

mod direction;
mod moves;

const MARGIN: i64 = 100;

/// Position in input text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    /// Row of input with error, zero-indexed.
    pub row: usize,
    /// Column of input with error, zero-indexed.
    pub col: usize,
}

/// Error in parsing input.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Position of the error.
    pub pos: TextPosition,
    /// Error information.
    pub msg: String,
}

impl ParseError {
    fn from_input(input: &Input, msg: &str) -> Self {
        Self {
            pos: input.pos,
            msg: msg.to_string(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.pos.row + 1, self.pos.col + 1, self.msg)
    }
}
impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError {
            pos: TextPosition::default(),
            msg: format!("{err}"),
        }
    }
}
impl From<std::string::FromUtf8Error> for ParseError {
    fn from(err: std::string::FromUtf8Error) -> ParseError {
        ParseError {
            pos: TextPosition::default(),
            msg: format!("{err}"),
        }
    }
}

/// Position, in centimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bounds {
    top_left: Position,
    bottom_right: Position,
}

impl Display for Bounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.top_left, self.bottom_right)
    }
}

impl Bounds {
    fn new() -> Self {
        Self {
            top_left: Position { x: 0, y: 0 },
            bottom_right: Position { x: 0, y: 0 },
        }
    }
    fn encompass(&mut self, pos: &Position) {
        if pos.x > self.bottom_right.x {
            self.bottom_right.x = pos.x;
        }
        if pos.x < self.top_left.x {
            self.top_left.x = pos.x;
        }
        if pos.y > self.bottom_right.y {
            self.bottom_right.y = pos.y;
        }
        if pos.y < self.top_left.y {
            self.top_left.y = pos.y;
        }
        trace!("encompass {pos} in bounds => {self}");
    }
    fn add_margin(&mut self, margin: i64) {
        self.top_left.x -= margin;
        self.top_left.y -= margin;
        self.bottom_right.x += margin;
        self.bottom_right.y += margin;
    }
    fn width(&self) -> i64 {
        self.bottom_right.x - self.top_left.x
    }
    fn height(&self) -> i64 {
        self.bottom_right.y - self.top_left.y
    }
}

/// Effect of a move on a skater.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Transition {
    delta: Position,
    rotate: Rotation,
    foot: Foot,
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:+.1},{:+.1}) {:+.1}° →{}",
            self.delta.x, self.delta.y, self.rotate.0, self.foot
        )
    }
}

/// Which foot has weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Foot {
    Left,
    Right,
    Both,
}

impl Display for Foot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Foot::Left => write!(f, "LF"),
            Foot::Right => write!(f, "RF"),
            Foot::Both => write!(f, "BF"),
        }
    }
}

/// Description of current skater state.
#[derive(Debug, Clone, Copy)]
struct Skater {
    pos: Position,
    dir: Direction,
    foot: Foot,
}

impl Display for Skater {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({},{}) {}° {}",
            self.pos.x, self.pos.y, self.dir.0, self.foot
        )
    }
}

impl std::ops::Add<Transition> for Skater {
    type Output = Self;
    fn add(self, transition: Transition) -> Self {
        // Start position
        let start_x = self.pos.x as f64;
        let start_y = self.pos.y as f64;

        // Delta in coords if we were aligned with `Direction(0)` ...
        let delta_x = transition.delta.x as f64;
        let delta_y = transition.delta.y as f64;

        // ... but we're not, we're moving at an angle:
        let angle = self.dir.0 as f64 * std::f64::consts::PI / 180.0;
        let dx = delta_x * angle.cos() - delta_y * angle.sin();
        let dy = delta_y * angle.cos() + delta_x * angle.sin();
        trace!("  ({delta_x:+.1},{delta_y:+.1}) at {angle} radians => move ({dx:+.1},{dy:+.1})");

        let new_x = start_x + dx;
        let new_y = start_y + dy;

        Skater {
            pos: Position {
                x: new_x as i64,
                y: new_y as i64,
            },
            dir: self.dir + transition.rotate,
            foot: transition.foot,
        }
    }
}
// TODO
#[derive(Debug, Clone, Copy)]
struct RenderOptions {}

/// Trait describing the external behavior of a move.
trait Move {
    /// Foot that the move starts on.
    fn start_foot(&self) -> Foot;

    /// Foot that the move ends on.
    fn end_foot(&self) -> Foot;

    /// Transition needed before starting the move, starting from `Direction(0)`.
    fn pre_transition(&self, from: Foot) -> Transition;

    /// Transition as a result of the move, starting from `Direction(0)`, and assuming that [`pre_transition`] has
    /// already happened.
    fn transition(&self) -> Transition;

    /// Emit SVG group definition for the move.
    fn def(&self, _opts: &RenderOptions) -> Group;

    /// Emit a unique identifier for the SVG group definition for the move.
    fn def_id(&self) -> &'static str;

    /// Render the move into the given SVG document, assuming the existence of groups included in the output from [`defs`].
    fn render(&self, doc: Document, start: &Skater, _opts: &RenderOptions) -> Document {
        // Default implementation uses the definition, suitable translated and rotated.
        let def_id = self.def_id();
        let mut use_link = Use::new().set("xlink:href", format!("#{def_id}")).set(
            "transform",
            format!(
                "translate({} {}) rotate({})",
                start.pos.x, start.pos.y, start.dir.0
            ),
        );
        if let Some(input) = self.input() {
            use_link = use_link.set("id", input.unique_id());
        }
        doc.add(use_link)
    }

    /// Emit text that describes the move.  Feeding this text into `moves::factory` should result in the
    /// same `Move` (although it may have different `input_text`).
    fn text(&self) -> String;

    /// Emit the input that was used to originally create the move, if available.  This may have different text
    /// (e.g. longer, using alias forms) than the result of [`text`].
    fn input(&self) -> Option<OwnedInput>;
}

/// Generate SVG for the given input.
pub fn generate(input: &str) -> Result<String, ParseError> {
    // Convert the input into a list of move input strings.
    let inputs = split_inputs(input)?;

    let moves = inputs
        .iter()
        .map(|input| moves::factory(input))
        .collect::<Result<Vec<_>, ParseError>>()?;

    let mut doc = Document::new()
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .add(Title::new("Skating Diagram"))
        .add(Description::new().add(Text::new("Skating Diagram")));
    let opts = RenderOptions {};

    // First pass: emit definitions for all moves in use.
    let mut seen = HashSet::new();
    let mut defs = Definitions::new();
    for mv in &moves {
        let id = mv.def_id();
        if seen.contains(id) {
            continue;
        }
        seen.insert(id);
        let group = mv.def(&opts).set("id", id);
        defs = defs.add(group);
    }
    doc = doc.add(defs);

    // Second pass: figure out a bounding box.
    let mut skater = Skater {
        pos: Position { x: 0, y: 0 },
        dir: Direction::new(0),
        foot: Foot::Both,
    };
    let mut bounds = Bounds::new();
    for mv in &moves {
        let pre_transition = mv.pre_transition(skater.foot);
        let before = skater + pre_transition;
        bounds.encompass(&before.pos);
        let transition = mv.transition();
        let after = before + transition;
        bounds.encompass(&after.pos);
        skater = after;
    }
    let mut outer_bounds = bounds;
    outer_bounds.add_margin(MARGIN);
    doc = doc
        .set("width", outer_bounds.width())
        .set("height", outer_bounds.height());
    info!("inner bounds {bounds}, add {MARGIN} to get {outer_bounds}");

    // Third pass: render all the moves.
    let start_pos = Position {
        x: MARGIN - bounds.top_left.x,
        y: MARGIN - bounds.top_left.y,
    };
    info!("start at {start_pos}");
    let mut skater = Skater {
        pos: start_pos,
        dir: Direction::new(0),
        foot: Foot::Both,
    };
    for mv in &moves {
        let pre_transition = mv.pre_transition(skater.foot);
        let before = skater + pre_transition;
        debug!("pre:  {skater} == {pre_transition} ==> {before}");
        doc = mv.render(doc, &before, &opts);
        let transition = mv.transition();
        let after = before + transition;
        debug!("post: {before} == {transition} ==> {after}");
        skater = after;
    }

    let mut svg = Vec::new();
    svg::write(&mut svg, &doc)?;
    let svg = String::from_utf8(svg)?;
    info!("emit SVG:\n{svg}");
    Ok(svg)
}

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
}
