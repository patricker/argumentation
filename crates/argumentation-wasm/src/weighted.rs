//! Weighted-attack framework wrapper for the β-playground demo.
//!
//! "Live attacks at β" uses a single-edge visualisation rule: an
//! attack is *droppable* (visually faded) iff its individual weight
//! is ≤ β. This is a per-edge approximation of the full β-residual
//! semantics — sufficient to make β legible at-a-glance without
//! solving subset-sum on every drag tick. The `is_credulous` query
//! uses the real semantics from `argumentation_weighted_bipolar`.

use argumentation_weighted_bipolar::semantics::is_credulously_accepted_at;
use argumentation_weighted_bipolar::{Budget, WeightedBipolarFramework};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWeightedFramework {
    inner: WeightedBipolarFramework<String>,
    beta: f64,
    /// Edges recorded for the live-attacks visualisation. The upstream
    /// framework exposes `attacks()` but the iteration order isn't
    /// guaranteed; recording in insertion order makes JS output stable.
    attacks: Vec<(String, String, f64)>,
}

impl Default for WasmWeightedFramework {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmWeightedFramework {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WeightedBipolarFramework::new(),
            beta: 0.0,
            attacks: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, id: &str) {
        self.inner.add_argument(id.to_string());
    }

    /// Record an attack with a non-negative weight. Invalid weights
    /// (NaN, infinity, negative) are silently ignored — the playground
    /// should never surface these to the user.
    pub fn add_weighted_attack(&mut self, from: &str, to: &str, weight: f64) {
        if self
            .inner
            .add_weighted_attack(from.to_string(), to.to_string(), weight)
            .is_ok()
        {
            self.attacks.push((from.to_string(), to.to_string(), weight));
        }
    }

    /// Set β, clamped to [0, 1].
    pub fn set_intensity(&mut self, beta: f64) {
        self.beta = beta.clamp(0.0, 1.0);
    }

    pub fn current_intensity(&self) -> f64 {
        self.beta
    }

    /// Returns `"from->to"` strings for attacks that BIND at the current
    /// β (those whose weight strictly exceeds β). Edges absent from this
    /// list are droppable; the UI fades them.
    pub fn live_attacks_at_current_beta(&self) -> Vec<String> {
        self.attacks
            .iter()
            .filter(|(_, _, w)| *w > self.beta)
            .map(|(f, t, _)| format!("{f}->{t}"))
            .collect()
    }

    /// Credulous acceptance at the current β, using the real semantics.
    /// Returns `false` on internal errors (e.g. framework too large).
    pub fn is_credulous(&self, arg: &str) -> bool {
        let budget = Budget::new(self.beta).unwrap_or_else(|_| Budget::zero());
        let target = arg.to_string();
        is_credulously_accepted_at(&self.inner, &target, budget).unwrap_or(false)
    }
}
