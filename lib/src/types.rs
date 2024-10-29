//! Common basic types.

use crate::MoveParam;
use log::trace;
use std::fmt::{self, Display, Formatter};

const DEGREES: i32 = 360;

/// Rotation, in degrees.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl Position {
    /// Create a `Position` from (x, y) move parameters.
    pub fn from_params(x: &MoveParam, y: &MoveParam) -> Self {
        Self {
            x: x.value.as_i32().unwrap() as i64,
            y: y.value.as_i32().unwrap() as i64,
        }
    }
}

impl std::ops::Add<Position> for Position {
    type Output = Self;
    fn add(self, other: Position) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
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
    /// Modify bounds to ensure they encompass the given additional [`Bounds`].
    pub fn encompass_bounds(&mut self, other: &Bounds) {
        self.encompass(&other.top_left);
        self.encompass(&other.bottom_right);
    }
    /// Translate the bounds by the given amounts.
    pub fn translate(&mut self, dx: i64, dy: i64) {
        self.top_left.x += dx;
        self.top_left.y += dy;
        self.bottom_right.x += dx;
        self.bottom_right.y += dy;
    }
    /// Expand bounds by the given `margin`.
    pub fn add_margin(&mut self, margin_x: i64, margin_y: i64) {
        self.top_left.x -= margin_x;
        self.top_left.y -= margin_y;
        self.bottom_right.x += margin_x;
        self.bottom_right.y += margin_y;
    }
    /// Current width of bounds.
    pub fn width(&self) -> i64 {
        self.bottom_right.x - self.top_left.x
    }
    /// Current height of bounds.
    pub fn height(&self) -> i64 {
        self.bottom_right.y - self.top_left.y
    }
    /// Midpoint of bounds.
    pub fn midpoint(&self) -> Position {
        Position {
            x: self.top_left.x + self.width() / 2,
            y: self.top_left.y + self.height() / 2,
        }
    }
}

/// Convenience macro to build [`Bounds`].
#[macro_export]
macro_rules! bounds {
    { $x1:expr, $y1:expr => $x2:expr, $y2:expr } => {
        Bounds {
            top_left: Position { x: $x1, y: $y1 },
            bottom_right: Position { x: $x2, y: $y2 },
        }
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

/// Spatial effect of a move on a skater.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialTransition {
    /// Relative spatial movement and rotation.
    Relative {
        /// Change in position.
        delta: Position,
        /// Change in direction.
        rotate: Rotation,
    },
    /// Absolute transition to new position and direction.
    Absolute {
        /// New position.
        pos: Position,
        /// New direction.
        dir: Direction,
    },
}

/// Effect of a move on a skater.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Spatial effect on position/direction.
    pub spatial: SpatialTransition,
    /// Post-transition starting foot/dir/edge. `None` implies no change of foot/dir/edge.
    pub code: Option<Code>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            spatial: SpatialTransition::Relative {
                delta: Position::default(),
                rotate: Rotation::default(),
            },
            code: None,
        }
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = match self.code {
            Some(code) => format!("{code}"),
            None => "<unchanged>".to_string(),
        };
        match self.spatial {
            SpatialTransition::Relative { delta, rotate } => {
                write!(
                    f,
                    "({:+.1},{:+.1}) {:+.1}° → {code}",
                    delta.x, delta.y, rotate.0,
                )
            }
            SpatialTransition::Absolute { pos, dir } => {
                write!(f, "({:.1},{:.1}) {:.1}° → {code}", pos.x, pos.y, dir.0,)
            }
        }
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
    { BF } =>  { $crate::Code { foot: $crate::Foot::Both,  dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Flat } };
    { BB } =>  { $crate::Code { foot: $crate::Foot::Both,  dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Flat } };
    { LF } =>  { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Flat } };
    { LFO } => { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Outside } };
    { LFI } => { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Inside } };
    { LB } =>  { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Flat } };
    { LBO } => { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Outside } };
    { LBI } => { $crate::Code { foot: $crate::Foot::Left,  dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Inside } };
    { RF } =>  { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Flat } };
    { RFO } => { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Outside } };
    { RFI } => { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Forward,  edge: $crate::Edge::Inside } };
    { RB } =>  { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Flat } };
    { RBO } => { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Outside } };
    { RBI } => { $crate::Code { foot: $crate::Foot::Right, dir: $crate::SkatingDirection::Backward, edge: $crate::Edge::Inside } };
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

    #[test]
    fn test_add_margin() {
        let mut bounds = Bounds {
            top_left: Position { x: 10, y: 20 },
            bottom_right: Position { x: 110, y: 120 },
        };
        assert_eq!(bounds.width(), 100);
        assert_eq!(bounds.height(), 100);

        bounds.add_margin(5, 5);
        assert_eq!(bounds.top_left, Position { x: 5, y: 15 });
        assert_eq!(bounds.bottom_right, Position { x: 115, y: 125 });
        assert_eq!(bounds.width(), 110);
        assert_eq!(bounds.height(), 110);
    }

    #[test]
    fn test_encompass() {
        let mut bounds = bounds!(10,20 => 110,120);

        bounds.encompass(&Position { x: 200, y: 200 });
        assert_eq!(bounds, bounds!(10,20 => 200,200));

        bounds.encompass(&Position { x: 0, y: 0 });
        assert_eq!(bounds, bounds!(0,0 => 200,200));

        bounds.encompass_bounds(&bounds!(150,150 => 250,250));
        assert_eq!(bounds, bounds!(0,0 => 250,250));

        bounds.encompass_bounds(&bounds!(-50,-50 => 350,350));
        assert_eq!(bounds, bounds!(-50,-50 => 350,350));
    }
}
