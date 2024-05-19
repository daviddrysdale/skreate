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

/// Generate output.
#[wasm_bindgen]
pub fn generate(input: &str) -> Result<String, JsError> {
    info!("In generate('{input}')");
    Ok(skreate::generate(input).expect("TODO: map error"))
}

// Rust interactions with WASM.

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
