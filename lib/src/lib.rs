//! Skating diagram creator.
#![warn(missing_docs)]

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

mod direction;

/// Position, in metres.
struct Position {
    x: f64,
    y: f64,
}

struct Move {}
