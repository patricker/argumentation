//! UC3: same scheme instance, same budget, but different relationship
//! snapshots produce different attack weights and therefore different
//! acceptance thresholds.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use argumentation_weighted::WeightSource;
use encounter_argumentation::{
    ArgumentId, EncounterArgumentationState, RelationshipDims, RelationshipSnapshot,
    RelationshipWeightSource,
};

fn build_state_with_weight(attack_weight: f64, budget: Budget) -> EncounterArgumentationState {
    let mut state = EncounterArgumentationState::new(default_catalog()).at_intensity(budget);
    let alice = ArgumentId::new("alice");
    let bob = ArgumentId::new("bob");
    state.add_weighted_attack(&alice, &bob, attack_weight).unwrap();
    state
}

#[test]
fn high_trust_reduces_effective_attack_weight() {
    // Create two snapshots: neutral, and alice-highly-trusts-bob.
    let neutral = RelationshipSnapshot::new();
    let mut trusting = RelationshipSnapshot::new();
    trusting.set(
        "alice",
        "bob",
        RelationshipDims {
            trust: 1.0,
            fear: 0.0,
            respect: 0.0,
            attraction: 0.0,
            friendship: 0.0,
        },
    );

    let neutral_source = RelationshipWeightSource::new(neutral);
    let trusting_source = RelationshipWeightSource::new(trusting);

    let neutral_w = neutral_source
        .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
        .unwrap();
    let trusting_w = trusting_source
        .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
        .unwrap();

    assert!(
        trusting_w < neutral_w,
        "high-trust weight ({}) should be below neutral ({})",
        trusting_w,
        neutral_w,
    );
}

#[test]
fn same_scenario_flips_acceptance_at_different_budgets_for_different_weights() {
    // Low-trust (neutral) scenario: attack weight ~0.5. At β=0.3 the
    // attack binds; at β=0.6 the residual drops it and b is accepted.
    let low_beta = Budget::new(0.3).unwrap();
    let high_beta = Budget::new(0.6).unwrap();

    let state_low = build_state_with_weight(0.5, low_beta);
    let state_high = build_state_with_weight(0.5, high_beta);

    let bob = ArgumentId::new("bob");
    assert!(!state_low.is_credulously_accepted(&bob).unwrap());
    assert!(state_high.is_credulously_accepted(&bob).unwrap());

    // Now a "trusting" relationship yields a weight of ~0.35 (neutral
    // 0.5 − 0.15 for trust=1.0). At β=0.3 the attack now binds (0.3 <
    // 0.35 still), at β=0.6 it's tolerated.
    let state_trust_low = build_state_with_weight(0.35, low_beta);
    let state_trust_high = build_state_with_weight(0.35, high_beta);

    assert!(!state_trust_low.is_credulously_accepted(&bob).unwrap());
    assert!(state_trust_high.is_credulously_accepted(&bob).unwrap());

    // Sanity: an even-lower-weight attack (0.2) would be tolerated
    // already at β=0.3, demonstrating that relationship modulation can
    // flip acceptance at a fixed β.
    let state_very_trust = build_state_with_weight(0.2, low_beta);
    assert!(state_very_trust.is_credulously_accepted(&bob).unwrap());
}
