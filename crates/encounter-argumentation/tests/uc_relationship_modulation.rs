//! UC3 relationship modulation — Phase C rewrite.
//!
//! Phase A used a snapshot stub that mapped actor-name strings → dims
//! directly, which was unsound: `WeightSource::weight_for` receives
//! `ArgumentId` (a conclusion literal), not an actor name. Phase C
//! replaces the stub with `SocietasRelationshipSource`, which uses the
//! bridge's `actors_by_argument` map to resolve an `ArgumentId` to the
//! actors who asserted that conclusion, then reads relationship
//! dimensions from a live `societas-relations` registry + store.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::{types::Budget, WeightSource};
use encounter_argumentation::{
    ArgumentId, EncounterArgumentationState, SocietasRelationshipSource, TRUST_COEF,
};
use societas_core::{EntityId, ModifierSource, Tick};
use societas_memory::MemStore;
use societas_relations::{Dimension, RelationshipRegistry};
use std::collections::HashMap;

fn seed_state_with_pairwise_debate() -> (
    EncounterArgumentationState,
    ArgumentId,
    ArgumentId,
    HashMap<String, EntityId>,
) {
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let alice_instance = expert
        .instantiate(
            &[
                ("expert".to_string(), "alice".to_string()),
                ("domain".to_string(), "military".to_string()),
                ("claim".to_string(), "fortify_east".to_string()),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();
    let bob_instance = expert
        .instantiate(
            &[
                ("expert".to_string(), "bob".to_string()),
                ("domain".to_string(), "logistics".to_string()),
                ("claim".to_string(), "abandon_east".to_string()),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let alice_id = state.add_scheme_instance("alice", alice_instance);
    let bob_id = state.add_scheme_instance("bob", bob_instance);

    let mut resolver: HashMap<String, EntityId> = HashMap::new();
    resolver.insert("alice".to_string(), EntityId::from_u64(1));
    resolver.insert("bob".to_string(), EntityId::from_u64(2));

    (state, alice_id, bob_id, resolver)
}

#[test]
fn high_trust_reduces_effective_attack_weight() {
    // Alice asserts fortify_east; Bob asserts abandon_east.
    // Bob has HIGH trust of Alice (bob→alice Trust = 1.0). This means
    // Bob's attack on Alice's conclusion should be DAMPENED — from the
    // baseline 0.5 down to 0.5 + TRUST_COEF = 0.35.
    let (state, alice_id, bob_id, resolver) = seed_state_with_pairwise_debate();
    let mut store = MemStore::new();
    let registry = RelationshipRegistry::new();
    registry.add_modifier(
        &mut store,
        EntityId::from_u64(2), // bob
        EntityId::from_u64(1), // alice
        Dimension::Trust,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );

    let source = SocietasRelationshipSource::new(
        &registry,
        &store,
        &resolver,
        state.actors_by_argument(),
        Tick(0),
    );

    let w = source.weight_for(&bob_id, &alice_id).unwrap();
    let expected = (0.5_f64 + TRUST_COEF).clamp(0.0, 1.0);
    assert!(
        (w - expected).abs() < 1e-9,
        "bob→alice with Trust=1.0 should produce 0.5 + TRUST_COEF = {expected}, got {w}"
    );

    // Reverse direction: alice has no trust recorded of bob → baseline.
    let w_reverse = source.weight_for(&alice_id, &bob_id).unwrap();
    assert!(
        (w_reverse - 0.5).abs() < 1e-9,
        "alice→bob with no recorded trust should sit at baseline 0.5, got {w_reverse}"
    );
}

#[test]
fn same_scenario_flips_acceptance_at_different_budgets_for_different_weights() {
    // Construct two parallel states: one where bob has high trust of
    // alice (attack weight < 0.5), one where bob has high fear
    // (attack weight > 0.5). At an intensity β that sits between the
    // two, only the high-trust state accepts alice credulously.
    let (state_trust, alice_trust_id, bob_trust_id, resolver_trust) =
        seed_state_with_pairwise_debate();
    let (state_fear, alice_fear_id, bob_fear_id, resolver_fear) =
        seed_state_with_pairwise_debate();

    let mut store_trust = MemStore::new();
    let registry_trust = RelationshipRegistry::new();
    registry_trust.add_modifier(
        &mut store_trust,
        EntityId::from_u64(2),
        EntityId::from_u64(1),
        Dimension::Trust,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );
    let source_trust = SocietasRelationshipSource::new(
        &registry_trust,
        &store_trust,
        &resolver_trust,
        state_trust.actors_by_argument(),
        Tick(0),
    );

    let mut store_fear = MemStore::new();
    let registry_fear = RelationshipRegistry::new();
    registry_fear.add_modifier(
        &mut store_fear,
        EntityId::from_u64(2),
        EntityId::from_u64(1),
        Dimension::Fear,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );
    let source_fear = SocietasRelationshipSource::new(
        &registry_fear,
        &store_fear,
        &resolver_fear,
        state_fear.actors_by_argument(),
        Tick(0),
    );

    let w_trust = source_trust
        .weight_for(&bob_trust_id, &alice_trust_id)
        .unwrap();
    let w_fear = source_fear.weight_for(&bob_fear_id, &alice_fear_id).unwrap();
    assert!(
        w_trust < w_fear,
        "trust-based weight ({w_trust}) should be below fear-based weight ({w_fear})"
    );

    // Wire the weights into their respective states.
    let mut state_trust_mut = state_trust;
    state_trust_mut
        .add_weighted_attack(&bob_trust_id, &alice_trust_id, w_trust)
        .unwrap();
    let mut state_fear_mut = state_fear;
    state_fear_mut
        .add_weighted_attack(&bob_fear_id, &alice_fear_id, w_fear)
        .unwrap();

    // Pick a β strictly between the two weights. Alice survives in the
    // trust state (β > w_trust → residual drops the attack), but NOT in
    // the fear state (β < w_fear → residual retains the attack).
    let mid = (w_trust + w_fear) / 2.0;
    let beta = Budget::new(mid).unwrap();
    let trust_acceptance = state_trust_mut
        .at_intensity(beta)
        .is_credulously_accepted(&alice_trust_id)
        .unwrap();
    let fear_acceptance = state_fear_mut
        .at_intensity(beta)
        .is_credulously_accepted(&alice_fear_id)
        .unwrap();
    assert!(
        trust_acceptance,
        "at β between the two weights, trust-dampened attack should be dropped → alice credulous"
    );
    assert!(
        !fear_acceptance,
        "at β between the two weights, fear-amplified attack should bind → alice not credulous"
    );
}
