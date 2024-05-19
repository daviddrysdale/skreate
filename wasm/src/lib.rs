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
pub fn generate(input: &str) -> Result<String, JsError> {
    info!("In generate('{input}')");
    Ok(skreate::generate(input).expect("TODO: map error"))
}
