//! `SocietasRelationshipSource`: a [`WeightSource<ArgumentId>`] that
//! derives attack weights from live `societas-relations` state.
//!
//! Replaces the Phase A `RelationshipWeightSource` stub. Unlike the
//! stub, this adapter handles the [`ArgumentId`]-vs-actor-name mismatch
//! correctly: it looks up the actors that asserted each argument via
//! `EncounterArgumentationState::actors_by_argument`, resolves their
//! names to `EntityId`s via a pluggable [`NameResolver`], queries the
//! five societas relationship dimensions per actor pair, applies a
//! coefficient recipe (see module-level constants), and returns the
//! arithmetic mean across pairs.
//!
//! A worked example using `WeightSource::weight_for` appears in the
//! crate-level docs on `lib.rs` and in
//! `tests/uc_relationship_modulation.rs`.

use crate::arg_id::ArgumentId;
use crate::name_resolver::NameResolver;
use societas_core::{SocialStore, Tick};
use societas_relations::RelationshipRegistry;
use std::collections::HashMap;

/// Neutral-relationship baseline weight used when no actor pair has
/// resolvable relationship data.
pub const BASELINE_WEIGHT: f64 = 0.5;
/// Coefficient on `Dimension::Trust`. Higher trust → lower attack weight.
pub const TRUST_COEF: f64 = -0.15;
/// Coefficient on `Dimension::Fear`. Higher fear → higher attack weight.
pub const FEAR_COEF: f64 = 0.25;
/// Coefficient on `Dimension::Respect`. Higher respect → lower attack weight.
pub const RESPECT_COEF: f64 = -0.05;
/// Coefficient on `Dimension::Attraction`. Higher attraction → lower attack weight.
pub const ATTRACTION_COEF: f64 = -0.05;
/// Coefficient on `Dimension::Friendship`. Higher friendship → lower attack weight.
pub const FRIENDSHIP_COEF: f64 = -0.10;

/// A [`WeightSource<ArgumentId>`] that reads relationship dimensions
/// from a live `societas-relations` registry + store.
///
/// See the module-level documentation for the coefficient recipe and
/// the aggregation strategy for multi-actor arguments.
// Fields will be read by WeightSource impl in Tasks 5-7.
#[allow(dead_code)]
pub struct SocietasRelationshipSource<'a, R> {
    registry: &'a RelationshipRegistry,
    store: &'a dyn SocialStore,
    resolver: &'a R,
    actors_by_argument: &'a HashMap<ArgumentId, Vec<String>>,
    tick: Tick,
}

impl<'a, R: NameResolver> SocietasRelationshipSource<'a, R> {
    /// Construct a new source bound to the given registry, store,
    /// resolver, actor map, and evaluation tick.
    ///
    /// All references are borrowed for the adapter's lifetime `'a`.
    /// `tick` is owned and fixed at construction time — consumers
    /// wanting a new tick should build a fresh adapter.
    ///
    /// `actors_by_argument` is typically obtained by calling
    /// [`EncounterArgumentationState::actors_by_argument`].
    #[must_use]
    pub fn new(
        registry: &'a RelationshipRegistry,
        store: &'a dyn SocialStore,
        resolver: &'a R,
        actors_by_argument: &'a HashMap<ArgumentId, Vec<String>>,
        tick: Tick,
    ) -> Self {
        Self {
            registry,
            store,
            resolver,
            actors_by_argument,
            tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use societas_core::EntityId;
    use societas_memory::MemStore;

    #[test]
    fn constants_match_phase_a_stub() {
        // Locks in the exact coefficient values shipped in Phase A.
        // Calibration changes should be conscious version bumps with
        // CHANGELOG entries — not drift.
        assert_eq!(BASELINE_WEIGHT, 0.5);
        assert_eq!(TRUST_COEF, -0.15);
        assert_eq!(FEAR_COEF, 0.25);
        assert_eq!(RESPECT_COEF, -0.05);
        assert_eq!(ATTRACTION_COEF, -0.05);
        assert_eq!(FRIENDSHIP_COEF, -0.10);
    }

    #[test]
    fn new_source_constructs() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        // Compile-time check: the constructor accepts these types.
        let _source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
    }
}
