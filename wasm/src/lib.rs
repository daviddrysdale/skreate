//! Wasm to Javascript interaction code.
#![warn(missing_docs)]

use log::info;
use wasm_bindgen::prelude::*;

// Rust functionality invoked from Javascript.

/// Perform initialization. Safe to be invoked more than once.
#[wasm_bindgen]
pub fn initialize() {
    wasm_logger::init(wasm_logger::Config::default());
    #[cfg(feature = "console_error_panic_hook")]
    {
        info!("initialize: set panic hook");
        console_error_panic_hook::set_once();
    }
}

/// Generate output.
#[wasm_bindgen]
pub fn generate(input: &str) -> Result<String, ParseError> {
    info!("In generate('{input}')");
    Ok(skreate::generate(input)?)
}

/// Error in parsing input.  Direct equivalent of [`skreate::ParseError`], mirrored here to allow conversion to
/// Javascript.
#[wasm_bindgen]
pub struct ParseError {
    /// Row of input with error.
    pub row: usize,
    /// Column of input with error.
    pub col: usize,
    /// Error information.
    msg: String,
}

#[wasm_bindgen]
impl ParseError {
    /// Retrieve the message associated with the error.
    #[wasm_bindgen(getter)]
    pub fn msg(&self) -> String {
        self.msg.clone()
    }

    /// Set the message associated with the error.
    #[wasm_bindgen(setter)]
    pub fn set_field(&mut self, msg: String) {
        self.msg = msg;
    }
}

impl From<skreate::ParseError> for ParseError {
    fn from(err: skreate::ParseError) -> ParseError {
        ParseError {
            row: err.pos.row,
            col: err.pos.col,
            msg: err.msg,
        }
    }
}
