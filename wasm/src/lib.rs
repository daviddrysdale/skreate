//! Wasm to Javascript interaction code.

#![warn(missing_docs)]

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
    alert("Calling set_panic_hook");
    set_panic_hook();
}

/// Generate output.
#[wasm_bindgen]
pub fn generate(input: &str) -> String {
    alert(&format!("In generate('{input}')"));
    format!("output from {input}")
}

// Rust interactions with WASM.

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
