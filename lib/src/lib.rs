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

/// Error in parsing input.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Row of input with error.
    pub row: usize,
    /// Column of input with error.
    pub col: usize,
    /// Error information.
    pub msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.row + 1, self.col + 1, self.msg)
    }
}
impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError {
            row: 0,
            col: 0,
            msg: format!("{err}"),
        }
    }
}
impl From<std::string::FromUtf8Error> for ParseError {
    fn from(err: std::string::FromUtf8Error) -> ParseError {
        ParseError {
            row: 0,
            col: 0,
            msg: format!("{err}"),
        }
    }
}

mod direction;

/// Position, in metres.
struct Position {
    x: f64,
    y: f64,
}

struct Move {}

/// Generate SVG for the given input.
pub fn generate(input: &str) -> Result<String, ParseError> {
    let lines = input.split('\n').collect::<Vec<_>>();
    if lines.len() > 3 {
        return Err(ParseError {
            row: 2,
            col: 3,
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

#[cfg(test)]
mod tests {
    use super::*;

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
