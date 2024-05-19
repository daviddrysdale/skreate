//! Wasm to Javascript interaction code.
#![warn(missing_docs)]

use log::info;
use wasm_bindgen::prelude::*;

// Javascript functionality invoked from Rust.

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Rust functionality invoked from Javascript.

/// Perform initialization. Safe to be invoked more than once.
#[wasm_bindgen]
pub fn initialize() {
    wasm_logger::init(wasm_logger::Config::default());
    info!("initialize: set panic hook");
    set_panic_hook();
}

// TODO: move this over to inner library
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

/// Generate output.
#[wasm_bindgen]
pub fn generate(input: &str) -> Result<String, JsError> {
    info!("In generate('{input}')");
    let svg = r##"<td class="diagram">
              <svg width="140" height="170"
  xmlns="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <title>Cat</title>
  <desc>Stick Figure of a Cat</desc>

  <circle cx="70" cy="95" r="50" style="stroke: black; fill: none;"/>
  <circle cx="55" cy="80" r="5" stroke="black" fill="#339933"/>
  <circle cx="85" cy="80" r="5" stroke="black" fill="#339933"/>
  <g id="whiskers">
    <line x1="75" y1="95" x2="135" y2="85" style="stroke: black;"/>
    <line x1="75" y1="95" x2="135" y2="105" style="stroke: black;"/>
  </g>
  <use xlink:href="#whiskers" transform="scale(-1 1) translate(-140 0)"/>
  <!-- ears -->
  <polyline points="108 62,  90 10,  70 45,  50, 10,  32, 62"
    style="stroke: black; fill: none;" />
  <!-- mouth -->
  <polyline points="35 110, 45 120, 95 120, 105, 110"
      style="stroke: black; fill: none;" />
</svg>"##;
    Ok(svg.to_string())
}

// Rust interactions with WASM.

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
