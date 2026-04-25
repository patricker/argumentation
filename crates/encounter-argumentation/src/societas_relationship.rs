//! `SocietasRelationshipSource`: a [`WeightSource<ArgumentId>`] that
//! derives attack weights from live `societas-relations` state.
//!
//! Replaces the Phase A `RelationshipWeightSource` stub. Unlike the
//! stub, this adapter handles the [`ArgumentId`]-vs-actor-name mismatch
//! correctly: it looks up the actors that asserted each argument via
//! [`crate::state::EncounterArgumentationState::actors_by_argument`], resolves their
//! names to `EntityId`s via a pluggable [`NameResolver`], queries the
//! five societas relationship dimensions per actor pair, applies a
//! coefficient recipe (see module-level constants), and returns the
//! arithmetic mean across pairs.
//!
//! A worked example using [`WeightSource::weight_for`] appears in the
//! crate-level docs on `lib.rs` and in
//! `tests/uc_relationship_modulation.rs`.

use crate::arg_id::ArgumentId;
use crate::name_resolver::NameResolver;
use argumentation_weighted::WeightSource;
use societas_core::{EntityId, SocialStore, Tick};
use societas_relations::Dimension;
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
    /// [`crate::state::EncounterArgumentationState::actors_by_argument`].
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

impl<R: NameResolver> WeightSource<ArgumentId> for SocietasRelationshipSource<'_, R> {
    /// Compute the attack weight for `attacker → target` from live
    /// societas relationship state. See the module-level documentation
    /// for the coefficient recipe and aggregation strategy.
    ///
    /// Always returns `Some(w)` — this source has an opinion on every
    /// edge. Unseeded or unresolvable pairs fall back to
    /// [`BASELINE_WEIGHT`].
    fn weight_for(&self, attacker: &ArgumentId, target: &ArgumentId) -> Option<f64> {
        let Some(attacker_actors) = self.actors_by_argument.get(attacker) else {
            return Some(BASELINE_WEIGHT);
        };
        let Some(target_actors) = self.actors_by_argument.get(target) else {
            return Some(BASELINE_WEIGHT);
        };

        let attacker_ids: Vec<EntityId> = attacker_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        let target_ids: Vec<EntityId> = target_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        if attacker_ids.is_empty() || target_ids.is_empty() {
            return Some(BASELINE_WEIGHT);
        }

        let mut sum = 0.0_f64;
        let mut count = 0_u32;
        for &src in &attacker_ids {
            for &tgt in &target_ids {
                sum += self.pairwise_weight(src, tgt);
                count += 1;
            }
        }
        let mean = sum / f64::from(count);
        Some(mean)
    }
}

impl<R: NameResolver> SocietasRelationshipSource<'_, R> {
    /// Compute the per-pair weight for a single (source, target)
    /// `EntityId` pair by summing the coefficient-weighted dimension
    /// scores and clamping to the unit interval.
    fn pairwise_weight(&self, source: EntityId, target: EntityId) -> f64 {
        let trust =
            self.registry
                .score(self.store, source, target, Dimension::Trust, self.tick);
        let fear =
            self.registry
                .score(self.store, source, target, Dimension::Fear, self.tick);
        let respect =
            self.registry
                .score(self.store, source, target, Dimension::Respect, self.tick);
        let attraction = self.registry.score(
            self.store,
            source,
            target,
            Dimension::Attraction,
            self.tick,
        );
        let friendship = self.registry.score(
            self.store,
            source,
            target,
            Dimension::Friendship,
            self.tick,
        );
        let raw = BASELINE_WEIGHT
            + TRUST_COEF * trust
            + FEAR_COEF * fear
            + RESPECT_COEF * respect
            + ATTRACTION_COEF * attraction
            + FRIENDSHIP_COEF * friendship;
        raw.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use societas_core::ModifierSource;
    use societas_memory::MemStore;
    use societas_relations::Dimension;

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

    #[test]
    fn baseline_weight_when_attacker_has_no_actors() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    #[test]
    fn baseline_weight_when_target_has_no_actors() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    #[test]
    fn baseline_weight_when_neither_name_resolves() {
        // Actors are recorded, but the resolver has no EntityId for either.
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        actors.insert(ArgumentId::new("y"), vec!["bob".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    /// Helper: build a minimal single-pair scene. Alice is the attacker
    /// actor (EntityId 1), Bob the target (EntityId 2).
    fn single_pair_fixture() -> (
        RelationshipRegistry,
        MemStore,
        HashMap<String, EntityId>,
        HashMap<ArgumentId, Vec<String>>,
    ) {
        let registry = RelationshipRegistry::new();
        let store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("bob".to_string(), EntityId::from_u64(2));
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("alice_arg"), vec!["alice".to_string()]);
        actors.insert(ArgumentId::new("bob_arg"), vec!["bob".to_string()]);
        (registry, store, resolver, actors)
    }

    #[test]
    fn neutral_societas_state_yields_baseline_weight() {
        let (registry, store, resolver, actors) = single_pair_fixture();
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        assert!(
            (w - BASELINE_WEIGHT).abs() < 1e-9,
            "neutral-state pair should produce baseline weight, got {w}"
        );
    }

    #[test]
    fn high_trust_lowers_attack_weight() {
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(2),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "high trust should produce baseline + TRUST_COEF = {expected}, got {w}"
        );
    }

    #[test]
    fn high_fear_raises_attack_weight() {
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(2),
            Dimension::Fear,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + FEAR_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "high fear should produce baseline + FEAR_COEF = {expected}, got {w}"
        );
    }

    #[test]
    fn weight_is_clamped_to_unit_interval_on_extreme_values() {
        // Simultaneously max out every dimension in both directions.
        // The raw linear combination can exceed [0, 1]; verify we clamp.
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        for dim in [
            Dimension::Trust,
            Dimension::Fear,
            Dimension::Friendship,
            Dimension::Respect,
            Dimension::Attraction,
        ] {
            registry.add_modifier(
                &mut store,
                EntityId::from_u64(1),
                EntityId::from_u64(2),
                dim,
                1.0,
                0.0,
                ModifierSource::Personality,
                Tick(0),
            );
        }
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        assert!(
            (0.0..=1.0).contains(&w),
            "weight should be clamped to [0, 1], got {w}"
        );
    }

    #[test]
    fn asymmetric_relationship_produces_asymmetric_weights() {
        // alice's view of bob has high trust; bob's view of alice is neutral.
        // alice → bob attack gets dampened (trust reduces weight);
        // bob → alice attack stays at baseline.
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1), // alice
            EntityId::from_u64(2), // bob
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w_alice_on_bob = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let w_bob_on_alice = source
            .weight_for(&ArgumentId::new("bob_arg"), &ArgumentId::new("alice_arg"))
            .unwrap();
        assert!(
            w_alice_on_bob < w_bob_on_alice,
            "alice→bob with high trust should weigh less than bob→alice baseline; got {w_alice_on_bob} vs {w_bob_on_alice}"
        );
    }

    #[test]
    fn multi_attacker_averages_per_pair_weights() {
        // Two attackers, one target. Alice has high trust of target;
        // Bob has neutral. The result should be the arithmetic mean of
        // the two per-pair weights, not a min/max/first-wins.
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("bob".to_string(), EntityId::from_u64(2));
        resolver.insert("target".to_string(), EntityId::from_u64(3));
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "bob".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["target".to_string()],
        );

        // Alice trusts the target strongly; Bob is neutral.
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(3),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );

        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("duo_attack"), &ArgumentId::new("target_arg"))
            .unwrap();

        let alice_pair = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        let bob_pair = BASELINE_WEIGHT;
        let expected = (alice_pair + bob_pair) / 2.0;
        assert!(
            (w - expected).abs() < 1e-9,
            "two-attacker case should mean per-pair weights: expected {expected}, got {w}"
        );
    }

    #[test]
    fn unresolvable_actors_are_skipped_not_treated_as_neutral_pair() {
        // Two attackers: alice (resolvable), eve (NOT in resolver).
        // Expected behavior: eve is skipped entirely; result equals
        // the alice-only weight, NOT mean(alice_pair, neutral_baseline).
        // This makes the adapter robust to partial registries.
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("target".to_string(), EntityId::from_u64(3));
        // NOTE: "eve" is deliberately NOT in the resolver.
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "eve".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["target".to_string()],
        );
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(3),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("duo_attack"), &ArgumentId::new("target_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "unresolvable eve should be filtered out; result should match alice-only pair = {expected}, got {w}"
        );
    }
}
