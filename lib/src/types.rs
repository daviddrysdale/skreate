//! Common basic types.

use log::trace;
use std::fmt::{self, Display, Formatter};

const DEGREES: i32 = 360;

/// Rotation, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rotation(pub i32);

/// Direction, in degrees.
///
/// Invariant: value in [0, DEGREES).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Direction(pub u32);

impl Direction {
    /// Construct a `Direction`, clamping to [0, 360).
    pub fn new(mut dir: i32) -> Self {
        while dir < 0 {
            dir += DEGREES;
        }
        Self((dir % DEGREES) as u32)
    }
}

impl std::ops::Add<Rotation> for Direction {
    type Output = Self;
    fn add(self, other: Rotation) -> Self {
        Self::new(self.0 as i32 + other.0)
    }
}

impl std::ops::AddAssign<Rotation> for Direction {
    fn add_assign(&mut self, other: Rotation) {
        self.0 = Direction::new(self.0 as i32 + other.0).0;
    }
}

impl std::ops::Sub<Direction> for Direction {
    type Output = Rotation;
    fn sub(self, other: Direction) -> Rotation {
        Rotation(self.0 as i32 - other.0 as i32)
    }
}

/// Position in input text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    /// Row of input with error, zero-indexed.
    pub row: usize,
    /// Column of input with error, zero-indexed.
    pub col: usize,
}

/// Position, in centimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

/// Rectangular boundary in canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bounds {
    /// Top-left of bounds.
    pub top_left: Position,
    /// Bottom-right of bounds.
    pub bottom_right: Position,
}

impl Display for Bounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.top_left, self.bottom_right)
    }
}

impl Bounds {
    /// Modify bounds to ensure they encompass the given [`Position`].
    pub fn encompass(&mut self, pos: &Position) {
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
    /// Expand bounds by the given `margin`.
    pub fn add_margin(&mut self, margin: i64) {
        self.top_left.x -= margin;
        self.top_left.y -= margin;
        self.bottom_right.x += margin;
        self.bottom_right.y += margin;
    }
    /// Current width of bounds.
    pub fn width(&self) -> i64 {
        self.bottom_right.x - self.top_left.x
    }
    /// Current height of bounds.
    pub fn height(&self) -> i64 {
        self.bottom_right.y - self.top_left.y
    }
}

/// Label for parts of a move.
pub struct Label {
    /// Text to display
    pub text: String,
    /// Where to centre the text.
    pub pos: Position,
}

/// Convenience macro to build a [`Label`].
#[macro_export]
macro_rules! label {
    { $text:literal @ $x:literal, $y:literal } => {
        Label {
            text: $text.to_string(),
            pos: Position {
                x: $x,
                y: $y,
            }
        }
    }
}

/// Effect of a move on a skater.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Spatial movement in the transition.
    pub delta: Position,
    /// Rotation in the transition.
    pub rotate: Rotation,
    /// Post-transition starting foot/dir/edge.
    pub code: Code,
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:+.1},{:+.1}) {:+.1}° →{}",
            self.delta.x, self.delta.y, self.rotate.0, self.code
        )
    }
}

/// Which foot has weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foot {
    /// L
    Left,
    /// R
    Right,
    /// B
    Both,
}

impl Display for Foot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Foot::Left => write!(f, "L"),
            Foot::Right => write!(f, "R"),
            Foot::Both => write!(f, "B"),
        }
    }
}

/// Direction of skating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkatingDirection {
    /// Skating forwards.
    Forward,
    /// Skating backwards.
    Backward,
}

impl Display for SkatingDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SkatingDirection::Forward => write!(f, "F"),
            SkatingDirection::Backward => write!(f, "B"),
        }
    }
}

/// Blade edge in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// Outside of the blade.
    Outside,
    /// Inside of the blade.
    Inside,
    /// Flat
    Flat,
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Edge::Outside => write!(f, "O"),
            Edge::Inside => write!(f, "I"),
            Edge::Flat => write!(f, ""),
        }
    }
}

/// Code describing current foot/direction/edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Code {
    /// Foot
    pub foot: Foot,
    /// Direction
    pub dir: SkatingDirection,
    /// Edge
    pub edge: Edge,
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.foot, self.dir, self.edge)
    }
}

/// Parse a possible transition prefix ("xf-", "xb-") from `text`.
pub fn parse_transition_prefix(text: &str) -> (bool, &str) {
    if let Some(rest) = text.strip_prefix("xf-") {
        (true, rest)
    } else if let Some(rest) = text.strip_prefix("xb-") {
        (true, rest)
    } else {
        (false, text)
    }
}

/// Parse a foot and direction from `text`.
pub fn parse_foot_dir(text: &str) -> Result<(Foot, SkatingDirection, &str), String> {
    let (foot, rest) = if let Some(rest) = text.strip_prefix('L') {
        (Foot::Left, rest)
    } else if let Some(rest) = text.strip_prefix('R') {
        (Foot::Right, rest)
    } else if let Some(rest) = text.strip_prefix('B') {
        (Foot::Both, rest)
    } else {
        return Err("No foot recognized".to_string());
    };
    let (dir, rest) = if let Some(rest) = rest.strip_prefix('F') {
        (SkatingDirection::Forward, rest)
    } else if let Some(rest) = rest.strip_prefix('B') {
        (SkatingDirection::Backward, rest)
    } else {
        return Err("No direction recognized".to_string());
    };
    Ok((foot, dir, rest))
}

/// Parse an edge code from `text`.
pub fn parse_code(text: &str) -> Result<(Code, &str), String> {
    let (foot, dir, rest) = parse_foot_dir(text)?;
    let (edge, rest) = if let Some(rest) = rest.strip_prefix('O') {
        (Edge::Outside, rest)
    } else if let Some(rest) = rest.strip_prefix('I') {
        (Edge::Inside, rest)
    } else {
        return Err("No edge recognized".to_string());
    };
    Ok((Code { foot, dir, edge }, rest))
}

/// Create a [`Code`] instance from a short code.
#[macro_export]
macro_rules! code {
    { BF } => { Code { foot: Foot::Both, dir: SkatingDirection::Forward, edge: Edge::Flat } };
    { BB } => { Code { foot: Foot::Both, dir: SkatingDirection::Backward, edge: Edge::Flat } };
    { LF } => { Code { foot: Foot::Left, dir: SkatingDirection::Forward, edge: Edge::Flat } };
    { LFO } => { Code { foot: Foot::Left, dir: SkatingDirection::Forward, edge: Edge::Outside } };
    { LFI } => { Code { foot: Foot::Left, dir: SkatingDirection::Forward, edge: Edge::Inside } };
    { LB } => { Code { foot: Foot::Left, dir: SkatingDirection::Backward, edge: Edge::Flat } };
    { LBO } => { Code { foot: Foot::Left, dir: SkatingDirection::Backward, edge: Edge::Outside } };
    { LBI } => { Code { foot: Foot::Left, dir: SkatingDirection::Backward, edge: Edge::Inside } };
    { RF } => { Code { foot: Foot::Right, dir: SkatingDirection::Forward, edge: Edge::Flat } };
    { RFO } => { Code { foot: Foot::Right, dir: SkatingDirection::Forward, edge: Edge::Outside } };
    { RFI } => { Code { foot: Foot::Right, dir: SkatingDirection::Forward, edge: Edge::Inside } };
    { RB } => { Code { foot: Foot::Right, dir: SkatingDirection::Backward, edge: Edge::Flat } };
    { RBO } => { Code { foot: Foot::Right, dir: SkatingDirection::Backward, edge: Edge::Outside } };
    { RBI } => { Code { foot: Foot::Right, dir: SkatingDirection::Backward, edge: Edge::Inside } };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let values = [
            (10, 10, 20),
            (10, -10, 0),
            (350, 10, 0),
            (350, -10, 340),
            (350, 20, 10),
            (350, 380, 10),
        ];
        for (start, delta, want) in values {
            let start = Direction::new(start);
            let delta = Rotation(delta);
            let want = Direction::new(want);
            let got = start + delta;
            assert_eq!(got, want, "{start:?} + {delta:?} should be {want:?}");
        }
    }
}
