// Copyright 2024-2025 David Drysdale

//! Wasm to Javascript interaction code.
#![warn(missing_docs)]

use log::{info, trace};
use wasm_bindgen::prelude::*;

// Rust functionality invoked from Javascript.

/// Perform initialization. Safe to be invoked more than once.
#[wasm_bindgen]
pub fn initialize() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    #[cfg(feature = "console_error_panic_hook")]
    {
        info!("initialize: set panic hook");
        console_error_panic_hook::set_once();
    }
}

/// Generate output.
#[wasm_bindgen]
pub fn generate(input: &str) -> Result<String, ParseError> {
    trace!("In generate('{input}')");
    Ok(skreate::generate(input)?)
}

/// Generated SVG and move positions.
#[wasm_bindgen]
pub struct GeneratedSvgPositions {
    /// Generated SVG
    svg: String,
    /// List of move positions in the form "r_<row>_c_<col>_<count>".
    positions: Vec<String>,
}

#[wasm_bindgen]
impl GeneratedSvgPositions {
    /// Retrieve the SVG.
    #[wasm_bindgen(getter)]
    pub fn svg(&self) -> String {
        self.svg.clone()
    }
    /// Retrieve the move positions.
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<String> {
        self.positions.clone()
    }
}

/// Generate output with positions.
#[wasm_bindgen]
pub fn generate_with_positions(input: &str) -> Result<GeneratedSvgPositions, ParseError> {
    trace!("In generate_with_positions('{input}')");
    let (svg, positions) = skreate::generate_with_positions(input)?;
    Ok(GeneratedSvgPositions { svg, positions })
}

/// Generate canonical input.
#[wasm_bindgen]
pub fn canonicalize(input: &str) -> Result<String, ParseError> {
    trace!("In canonicalize('{input}')");
    Ok(skreate::canonicalize(input)?)
}

/// Generate canonical input displayed vertically.
#[wasm_bindgen]
pub fn canonicalize_vert(input: &str) -> Result<String, ParseError> {
    trace!("In canonicalize('{input}')");
    Ok(skreate::canonicalize_vert(input)?)
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
    pub fn set_msg(&mut self, msg: String) {
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
