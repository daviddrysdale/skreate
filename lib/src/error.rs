//! Error type.

use crate::{Input, TextPosition};
use std::fmt::{self, Display, Formatter};

/// Error in parsing input.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Position of the error.
    pub pos: TextPosition,
    /// Error information.
    pub msg: String,
}

impl ParseError {
    /// Create a parse error for the given [`Input`]
    pub(crate) fn from_input(input: &Input, msg: &str) -> Self {
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
