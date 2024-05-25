//! Skating diagram creator.
#![warn(missing_docs)]

use log::info;
use svg::{
    node::{
        element::{Circle, Description, Group, Line, Polyline, Title, Use},
        Comment, Text,
    },
    Document,
};

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

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

mod direction;

/// Position, in metres.
struct _Position {
    x: f64,
    y: f64,
}

struct _Move {}

/// Generate SVG for the given input.
pub fn generate(input: &str) -> Result<String, ParseError> {
    // Convert the input into a list of move inputs
    let inputs = split_inputs(input)?;
    if inputs.len() > 3 {
        return Err(ParseError {
            pos: TextPosition { row: 2, col: 3 },
            msg: "fake error".to_string(),
        });
    }
    let doc = Document::new()
        .set("width", 140)
        .set("height", 170)
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .add(Title::new("Cat"))
        .add(Description::new().add(Text::new("Stick Figure of a Cat")))
        .add(
            Circle::new()
                .set("cx", 70)
                .set("cy", 95)
                .set("r", 50)
                .set("style", "stroke: black; fill: none;"),
        )
        .add(
            Circle::new()
                .set("cx", 55)
                .set("cy", 80)
                .set("r", 5)
                .set("stroke", "blue")
                .set("fill", "#339933"),
        )
        .add(
            Circle::new()
                .set("cx", 85)
                .set("cy", 80)
                .set("r", 5)
                .set("stroke", "black")
                .set("fill", "#339933"),
        )
        .add(
            Group::new()
                .set("id", "whiskers")
                .add(
                    Line::new()
                        .set("x1", 75)
                        .set("y1", 95)
                        .set("x2", 135)
                        .set("y2", 85)
                        .set("style", "stroke: black;"),
                )
                .add(
                    Line::new()
                        .set("x1", 75)
                        .set("y1", 95)
                        .set("x2", 135)
                        .set("y2", 105)
                        .set("style", "stroke: black;"),
                ),
        )
        .add(
            Use::new()
                .set("xlink:href", "#whiskers")
                .set("transform", "scale(-1 1) translate(-140 0)"),
        )
        .add(Comment::new("ears"))
        .add(
            Polyline::new()
                .set("points", "108 62,  90 10,  70 45,  50, 10,  32, 62")
                .set("style", "stroke: black; fill: none;"),
        )
        .add(Comment::new("mouth"))
        .add(
            Polyline::new()
                .set("points", "35 110, 45 120, 95 120, 105, 110")
                .set("style", "stroke: black; fill: none;"),
        );

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

    #[test]
    fn test_generate() {
        let want = r##"<svg height="170" width="140" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
<title>Cat</title>
<desc>Stick Figure of a Cat</desc>
<circle cx="70" cy="95" r="50" style="stroke: black; fill: none;"/>
<circle cx="55" cy="80" fill="#339933" r="5" stroke="blue"/>
<circle cx="85" cy="80" fill="#339933" r="5" stroke="black"/>
<g id="whiskers">
<line style="stroke: black;" x1="75" x2="135" y1="95" y2="85"/>
<line style="stroke: black;" x1="75" x2="135" y1="95" y2="105"/>
</g>
<use transform="scale(-1 1) translate(-140 0)" xlink:href="#whiskers"/>
<!-- ears -->
<polyline points="108 62,  90 10,  70 45,  50, 10,  32, 62" style="stroke: black; fill: none;"/>
<!-- mouth -->
<polyline points="35 110, 45 120, 95 120, 105, 110" style="stroke: black; fill: none;"/>
</svg>"##;

        let got = generate("xyzzy").unwrap();
        assert_eq!(want, got);
    }
}
