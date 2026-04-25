//! WebAssembly bindings for the argumentation crates.
//!
//! This crate is consumed only by the docs site (`website/`). It is not
//! published to crates.io and its JS API may change without notice.

use wasm_bindgen::prelude::*;

mod framework;
mod weighted;

pub use framework::WasmFramework;
pub use weighted::WasmWeightedFramework;

/// Install a panic hook that pipes Rust panics to `console.error`.
/// Called once on first construction; idempotent.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
