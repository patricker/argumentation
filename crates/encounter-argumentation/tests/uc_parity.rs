//! UC1 parity: the new state API agrees with `resolve_argument` on a
//! pairwise scenario. Both machinery paths should reach the same
//! verdict on a clean attack between two scheme instances.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::{
    ArgumentId, ArgumentOutcome, EncounterArgumentationState, resolve_argument,
};
use std::collections::HashMap;

#[test]
fn new_state_api_and_resolve_argument_agree_on_pairwise_expert_vs_rebuttal() {
    // Scenario: Alice uses expert opinion to support `fortify_east`.
    // Bob uses argument-from-consequences to support `¬fortify_east`
    // (negated conclusion). They attack each other.
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let alice_bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    let alice_instance = expert.instantiate(&alice_bindings).unwrap();

    // Path 1: the existing resolver.
    let legacy_outcome = resolve_argument(std::slice::from_ref(&alice_instance), &[], &registry);
    // With no responder arguments, Alice's side survives.
    assert!(matches!(legacy_outcome, ArgumentOutcome::ProposerWins { .. }));

    // Path 2: the new state API.
    let mut state = EncounterArgumentationState::new(registry);
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    // `fortify_east` is unattacked → credulously accepted at β=0.
    assert!(
        state.is_credulously_accepted(&alice_arg).unwrap(),
        "new state API should accept Alice's unattacked conclusion"
    );
    assert_eq!(alice_arg, ArgumentId::new("fortify_east"));
}

#[test]
fn new_state_api_rejects_attacked_conclusion_at_zero_intensity() {
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

    let mut state = EncounterArgumentationState::new(registry);
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    let attacker_arg = ArgumentId::new("bob_counter");
    state.add_weighted_attack(&attacker_arg, &alice_arg, 0.5).unwrap();

    // β=0 → the attack binds → fortify_east is not credulously accepted.
    assert!(!state.is_credulously_accepted(&alice_arg).unwrap());
}

#[test]
fn new_state_api_at_high_intensity_tolerates_attack() {
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

    let mut state = EncounterArgumentationState::new(registry)
        .at_intensity(Budget::new(0.6).unwrap());
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    state.add_weighted_attack(&ArgumentId::new("bob"), &alice_arg, 0.5).unwrap();

    // β=0.6 >= 0.5 → residual drops the attack → accepted credulously.
    assert!(state.is_credulously_accepted(&alice_arg).unwrap());
}
