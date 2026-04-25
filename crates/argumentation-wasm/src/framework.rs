//! Dung-only framework wrapper for the docs-site playground.
//!
//! Wraps `argumentation::ArgumentationFramework<String>` and exposes
//! a `&str`-friendly API for wasm-bindgen consumers.

use argumentation::ArgumentationFramework;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmFramework {
    inner: ArgumentationFramework<String>,
}

impl Default for WasmFramework {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmFramework {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: ArgumentationFramework::new() }
    }

    pub fn add_argument(&mut self, id: &str) {
        self.inner.add_argument(id.to_string());
    }

    /// Add an attack from `from` to `to`. The upstream method returns
    /// `Result<(), Error>` (errors when the args are unknown); since
    /// our wrapper's contract is "call add_argument first," any error
    /// would be a programmer error — we panic via `expect`.
    pub fn add_attack(&mut self, from: &str, to: &str) {
        self.inner
            .add_attack(&from.to_string(), &to.to_string())
            .expect("add_attack: arguments must be added first");
    }

    /// Grounded extension as a sorted Vec<String>. wasm-bindgen converts
    /// this to a JS Array<string>. Sorting makes the JS-side output
    /// deterministic for tests; HashSet iteration order otherwise varies.
    pub fn grounded_extension(&self) -> Vec<String> {
        let mut v: Vec<String> = self.inner.grounded_extension().into_iter().collect();
        v.sort();
        v
    }
}
