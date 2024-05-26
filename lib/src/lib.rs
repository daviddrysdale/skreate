//! Skating diagram creator.
#![warn(missing_docs)]

use crate::direction::{Direction, Rotation};
use log::{debug, info};
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

/// Position in input text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

// TODO
#[derive(Debug, Clone, Copy)]
struct RenderOptions {}

/// Trait describing the external behavior of a move.
trait Move {
    /// Transition as a result of the move, starting from `Direction(0)`.
    fn transition(&self) -> Transition;

    /// Emit SVG group definition for the move.
    fn def(&self, _opts: &RenderOptions) -> Group;

    /// Emit a unique identifier for the SVG group definition for the move.
    fn def_id(&self) -> &'static str;

    /// Render the move into the given SVG document, assuming the existence of groups included in the output from [`defs`].
    fn render(&self, doc: Document, start: &Skater, _opts: &RenderOptions) -> Document {
        let def_id = self.def_id();
        doc.add(Use::new().set("xlink:href", format!("#{def_id}")).set(
            "transform",
            format!(
                "translate({} {}) rotate({})",
                start.pos.x, start.pos.y, start.dir.0
            ),
        ))
    }

    /// Emit text that describes the move.  Feeding this text into `move_factory` should result in the
    /// same `Move` (although it may have different `input_text`).
    fn text(&self) -> String;

    /// Emit the text that was used to originally create the move, if available.  This may be different
    /// (e.g. longer, using alias forms) than the result of [`text`].
    fn input_text(&self) -> Option<String>;
}

fn move_factory(input: &Input) -> Result<Box<dyn Move>, ParseError> {
    info!("parse '{input:?}' into move");
    Ok(Box::new(moves::Lf::new(input)))
}

/// Generate SVG for the given input.
pub fn generate(input: &str) -> Result<String, ParseError> {
    // Convert the input into a list of move input strings.
    let inputs = split_inputs(input)?;

    let moves = inputs
        .iter()
        .map(|input| move_factory(input))
        .collect::<Result<Vec<_>, ParseError>>()?;

    let mut doc = Document::new()
        .set("width", 400)
        .set("height", 250)
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

    let mut skater = Skater {
        pos: Position { x: 300, y: 0 },
        dir: Direction::new(0),
        foot: Foot::Both,
    };
    for mv in &moves {
        doc = mv.render(doc, &skater, &opts);
        let transition = mv.transition();

        // Start position
        let start_x = skater.pos.x as f64;
        let start_y = skater.pos.y as f64;

        // Delta in coords if we were aligned with `Direction(0)` ...
        let delta_x = transition.delta.x as f64;
        let delta_y = transition.delta.y as f64;

        // ... but we're not, we're moving at an angle:
        let angle = skater.dir.0 as f64 * std::f64::consts::PI / 180.0;
        let dx = -delta_x * angle.cos() - delta_y * angle.sin();
        let dy = delta_y * angle.cos() - delta_x * angle.sin();
        debug!("move ({delta_x:+.1},{delta_y:+.1}) at {angle} radians => move ({dx:+.1},{dy:+.1})");

        let new_x = start_x + dx;
        let new_y = start_y + dy;

        let after = Skater {
            pos: Position {
                x: new_x as i64,
                y: new_y as i64,
            },
            dir: skater.dir + transition.rotate,
            foot: transition.foot,
        };
        info!("{skater} == {transition} ==> {after}");
        skater = after;
    }

    let mut svg = Vec::new();
    svg::write(&mut svg, &doc)?;
    let svg = String::from_utf8(svg)?;
    info!("{svg}");
    Ok(svg)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Input<'a> {
    pos: TextPosition,
    text: &'a str,
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
