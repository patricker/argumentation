//! `EncounterArgumentationState`: the encounter-level state object
//! composing schemes + bipolar + weighted + weighted-bipolar.
//!
//! Consumers build a state via `new(registry)`, optionally configure
//! a weight source and scene intensity via builders, add scheme
//! instances and raw edges, then query acceptance and coalitions.

use crate::arg_id::ArgumentId;
use argumentation_schemes::instance::SchemeInstance;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_weighted::types::Budget;
use argumentation_weighted_bipolar::WeightedBipolarFramework;
use std::collections::HashMap;

/// Encounter-level argumentation state composing schemes (premises +
/// conclusion), bipolar graph structure (attacks + supports), weighted
/// edge strengths, and a configurable scene-intensity budget.
pub struct EncounterArgumentationState {
    /// Scheme catalog used for instantiation + CQ lookup.
    #[allow(dead_code)]
    registry: CatalogRegistry,
    /// The underlying weighted bipolar framework.
    framework: WeightedBipolarFramework<ArgumentId>,
    /// Which actor asserted each argument. Multiple actors may share
    /// an `ArgumentId` (the same conclusion), so stored as a vec.
    #[allow(dead_code)]
    actors_by_argument: HashMap<ArgumentId, Vec<String>>,
    /// Scheme instances backing each argument.
    #[allow(dead_code)]
    instances_by_argument: HashMap<ArgumentId, Vec<SchemeInstance>>,
    /// Current scene intensity. Defaults to zero.
    intensity: Budget,
}

impl EncounterArgumentationState {
    /// Create a new state with the given scheme registry, no weight
    /// source, and zero scene intensity.
    #[must_use]
    pub fn new(registry: CatalogRegistry) -> Self {
        Self {
            registry,
            framework: WeightedBipolarFramework::new(),
            actors_by_argument: HashMap::new(),
            instances_by_argument: HashMap::new(),
            intensity: Budget::zero(),
        }
    }

    /// Read-only access to the current scene intensity.
    #[must_use]
    pub fn intensity(&self) -> Budget {
        self.intensity
    }

    /// Number of argument nodes in the framework.
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.framework.argument_count()
    }

    /// Number of edges (attacks + supports) in the framework.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.framework.edge_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;

    #[test]
    fn new_state_is_empty() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.argument_count(), 0);
        assert_eq!(state.edge_count(), 0);
    }

    #[test]
    fn new_state_has_zero_intensity() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.intensity().value(), 0.0);
    }
}
