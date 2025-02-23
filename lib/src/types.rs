//! Common basic types.

use crate::{
    moves::{cross_transition, pre_transition, wide_transition},
    MoveParam,
};
use log::trace;
use std::fmt::{self, Display, Formatter};
use svg::node::element::Text as SvgText;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextPosition {
    /// Row of input with error, zero-indexed.
    pub row: usize,
    /// Column of input with error, zero-indexed.
    pub col: usize,
    /// Count of chars.
    pub count: usize,
}

impl TextPosition {
    /// Determine position from current location in input stream, subtracting any trailing whitespace.
    pub fn new(start: &str, cur: &str, end: &str) -> Self {
        let mut row = 0;
        let mut col = 0;
        let mut pos = start;
        // Calculate the (row, col) of `cur` relative to `start`.
        while pos.as_ptr() < cur.as_ptr() {
            if pos.as_bytes().first() == Some(&b'\n') {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
            pos = &pos[1..]
        }
        // To calculate the length of the matched chunk of input we want to subtract off any trailing whitespace.
        let count = end.as_ptr() as usize - cur.as_ptr() as usize;
        let chunk = &cur[..std::cmp::min(count, cur.len())];
        TextPosition {
            row,
            col,
            count: chunk.trim_end().len(),
        }
    }
    /// Convert the position into an ID string.
    pub fn unique_id(&self) -> String {
        format!("r_{}_c_{}_{}", self.row, self.col, self.col + self.count)
    }
}

/// Helper macro to create [`Position`] instance.
#[macro_export]
macro_rules! pos {
    { $x:expr, $y:expr} => {
        Position { x: $x, y: $y }
    }
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
            x: x.value.as_i32("<internal>").unwrap() as i64,
            y: y.value.as_i32("<internal>").unwrap() as i64,
        }
    }

    /// Add the `delta` to a `Position`, but rotated by `dir`.
    pub fn add_rotated(self, dir: Direction, delta: Position) -> Self {
        // Start position
        let start_x = self.x as f64;
        let start_y = self.y as f64;

        // Delta in coords if we were aligned with `Direction(0)` ...
        let delta_x = delta.x as f64;
        let delta_y = delta.y as f64;

        // ... but we're not, we're moving at an angle:
        let angle = dir.0 as f64 * std::f64::consts::PI / 180.0;
        let dx = delta_x * angle.cos() - delta_y * angle.sin();
        let dy = delta_y * angle.cos() + delta_x * angle.sin();
        trace!("  ({delta_x:+.1},{delta_y:+.1}) at {angle} radians => move ({dx:+.1},{dy:+.1})");

        let new_x = start_x + dx;
        let new_y = start_y + dy;

        pos!(new_x as i64, new_y as i64)
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
    /// Create bounds that (roughly) encompass `text` of the given `font_size, centred at `pos` but oriented according
    /// to `dir`.
    pub fn for_text_at(text: &str, pos: Position, font_size: i64, dir: Direction) -> Bounds {
        // Make sure the bounds include the centre of the text.
        let mut bounds = Bounds {
            top_left: pos,
            bottom_right: pos,
        };
        // And also the full height of the text.
        let top = pos.add_rotated(dir, pos!(0, -font_size));
        bounds.encompass(&top);

        // Make a guess at x-extent for the text.
        let hw = text.len() as i64 * font_size / 3;
        let left = pos.add_rotated(dir, pos!(-hw, 0));
        bounds.encompass(&left);
        let right = pos.add_rotated(dir, pos!(hw, 0));
        bounds.encompass(&right);
        bounds
    }
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
#[derive(Debug)]
pub struct Label {
    /// Whether to display the label.
    pub display: bool,
    /// Text to display
    pub text: SvgText,
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

impl Foot {
    /// Return opposite `Foot`.
    pub fn opposite(&self) -> Self {
        match self {
            Foot::Left => Foot::Right,
            Foot::Right => Foot::Left,
            Foot::Both => Foot::Both,
        }
    }
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

impl SkatingDirection {
    /// Return opposite `SkatingDirection`.
    pub fn opposite(&self) -> Self {
        match self {
            SkatingDirection::Forward => SkatingDirection::Backward,
            SkatingDirection::Backward => SkatingDirection::Forward,
        }
    }
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

impl Edge {
    /// Return opposite `Edge`.
    pub fn opposite(&self) -> Self {
        match self {
            Edge::Outside => Edge::Inside,
            Edge::Inside => Edge::Outside,
            Edge::Flat => Edge::Flat,
        }
    }
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

/// Transition to new skating foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreTransition {
    /// Normal transition.
    Normal,
    /// Transition with new foot crossing in front.
    CrossFront,
    /// Transition with new foot crossing behind.
    CrossBehind,
    /// Normal transition but with a wide step.
    Wide,
}

impl PreTransition {
    /// Return the prefix associated with this pre-transition.
    pub fn prefix(&self) -> &'static str {
        match self {
            PreTransition::Normal => "",
            PreTransition::CrossFront => "xf-",
            PreTransition::CrossBehind => "xb-",
            PreTransition::Wide => "wd-",
        }
    }

    /// Perform a pre-transition, moving from `from` to the `start` position for a move.
    pub fn perform(&self, from: Code, start: Code) -> Transition {
        match self {
            PreTransition::CrossFront | PreTransition::CrossBehind => cross_transition(from, start),
            PreTransition::Normal => pre_transition(from, start),
            PreTransition::Wide => wide_transition(from, start),
        }
    }

    /// Return label text.
    pub fn label(&self) -> Option<&'static str> {
        match self {
            PreTransition::Normal => None,
            PreTransition::CrossFront => Some("XF"),
            PreTransition::CrossBehind => Some("XB"),
            PreTransition::Wide => Some("Wd"),
        }
    }
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

/// Identifier for an SVG element.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgId(pub String);

impl Display for SvgId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SvgId {
    /// Return a new identifier that has the same ID but within the given namespace.
    pub fn in_ns(&self, ns: &SvgId) -> Self {
        Self(format!("{ns}::{self}"))
    }
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
            top_left: pos!(10, 20),
            bottom_right: pos!(110, 120),
        };
        assert_eq!(bounds.width(), 100);
        assert_eq!(bounds.height(), 100);

        bounds.add_margin(5, 5);
        assert_eq!(bounds.top_left, pos!(5, 15));
        assert_eq!(bounds.bottom_right, pos!(115, 125));
        assert_eq!(bounds.width(), 110);
        assert_eq!(bounds.height(), 110);
    }

    #[test]
    fn test_encompass() {
        let mut bounds = bounds!(10,20 => 110,120);

        bounds.encompass(&pos!(200, 200));
        assert_eq!(bounds, bounds!(10,20 => 200,200));

        bounds.encompass(&pos!(0, 0));
        assert_eq!(bounds, bounds!(0,0 => 200,200));

        bounds.encompass_bounds(&bounds!(150,150 => 250,250));
        assert_eq!(bounds, bounds!(0,0 => 250,250));

        bounds.encompass_bounds(&bounds!(-50,-50 => 350,350));
        assert_eq!(bounds, bounds!(-50,-50 => 350,350));
    }

    #[test]
    fn test_text_position() {
        let text = "abc\nDEF\nXYZ";
        let tests = [
            (
                'a',
                TextPosition {
                    row: 0,
                    col: 0,
                    count: 0,
                },
            ),
            (
                'b',
                TextPosition {
                    row: 0,
                    col: 1,
                    count: 0,
                },
            ),
            (
                'c',
                TextPosition {
                    row: 0,
                    col: 2,
                    count: 0,
                },
            ),
            (
                'D',
                TextPosition {
                    row: 1,
                    col: 0,
                    count: 0,
                },
            ),
            (
                'X',
                TextPosition {
                    row: 2,
                    col: 0,
                    count: 0,
                },
            ),
            (
                'Z',
                TextPosition {
                    row: 2,
                    col: 2,
                    count: 0,
                },
            ),
        ];
        for (needle, want) in tests {
            let cur = text.find(needle).unwrap();
            let end = &text[cur..];
            let got = TextPosition::new(text, end, end);
            assert_eq!(got, want, "for position of '{needle}' in '{text}'");
        }
    }
}
