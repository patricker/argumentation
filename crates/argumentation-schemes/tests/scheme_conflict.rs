//! UC2: Alice uses argument from expert opinion. Bob uses ad hominem against
//! Alice. Both schemes use the same `claim` slot binding (`fortify_east`).
//! The expert opinion scheme has a positive conclusion template, so Alice's
//! instance concludes `Literal::atom("fortify_east")`. The ad hominem scheme
//! has a negated conclusion template, so Bob's instance concludes
//! `Literal::neg("fortify_east")`. These are direct contraries — ASPIC+
//! detects them as rebutting each other and the AF is fully populated.
//!
//! With Alice's rule preferred, Alice wins. With Bob's rule preferred, Bob
//! wins. With neither preferred, both arguments survive in different
//! preferred extensions.

use argumentation::aspic::{Literal, StructuredSystem};
use argumentation_schemes::aspic::add_scheme_to_system;
use argumentation_schemes::catalog::epistemic::argument_from_expert_opinion;
use argumentation_schemes::catalog::source::ad_hominem;
use std::collections::HashMap;

fn alice_instance() -> argumentation_schemes::instance::SchemeInstance {
    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    argument_from_expert_opinion().instantiate(&bindings).unwrap()
}

fn bob_instance() -> argumentation_schemes::instance::SchemeInstance {
    let bindings: HashMap<String, String> = [
        ("target".to_string(), "alice".to_string()),
        ("flaw".to_string(), "cowardice".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    ad_hominem().instantiate(&bindings).unwrap()
}

#[test]
fn ad_hominem_concludes_negated_claim_directly() {
    let bob = bob_instance();
    assert_eq!(
        bob.conclusion,
        Literal::neg("fortify_east"),
        "ad hominem must conclude ¬claim, not claim"
    );
}

#[test]
fn alice_and_bob_rebut_each_other_in_the_af() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    add_scheme_to_system(&alice, &mut system);
    add_scheme_to_system(&bob, &mut system);

    let built = system.build_framework().unwrap();
    let alice_args = built.arguments_with_conclusion(&Literal::atom("fortify_east"));
    let bob_args = built.arguments_with_conclusion(&Literal::neg("fortify_east"));
    assert!(!alice_args.is_empty(), "Alice's argument should be constructed");
    assert!(!bob_args.is_empty(), "Bob's argument should be constructed");

    // With no preferences set, both should survive in some preferred extension
    // (one each), since rebut is mutual and neither strictly defeats the other.
    let preferred = built.framework.preferred_extensions().unwrap();
    let alice_in_some = alice_args
        .iter()
        .any(|a| preferred.iter().any(|ext| ext.contains(&a.id)));
    let bob_in_some = bob_args
        .iter()
        .any(|a| preferred.iter().any(|ext| ext.contains(&a.id)));
    assert!(alice_in_some, "Alice's argument must appear in some preferred extension");
    assert!(bob_in_some, "Bob's argument must appear in some preferred extension");
}

#[test]
fn alice_wins_when_her_rule_is_preferred() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    let alice_rule = add_scheme_to_system(&alice, &mut system);
    let bob_rule = add_scheme_to_system(&bob, &mut system);
    system.prefer_rule(alice_rule, bob_rule).unwrap();

    let built = system.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 1, "with strict preference, expect a unique preferred extension");

    let alice_wins = built
        .arguments_with_conclusion(&Literal::atom("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    let bob_wins = built
        .arguments_with_conclusion(&Literal::neg("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    assert!(alice_wins, "Alice should win when her rule is preferred");
    assert!(!bob_wins, "Bob should be defeated when Alice's rule is preferred");
}

#[test]
fn bob_wins_when_his_rule_is_preferred() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    let alice_rule = add_scheme_to_system(&alice, &mut system);
    let bob_rule = add_scheme_to_system(&bob, &mut system);
    system.prefer_rule(bob_rule, alice_rule).unwrap();

    let built = system.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 1);

    let alice_wins = built
        .arguments_with_conclusion(&Literal::atom("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    let bob_wins = built
        .arguments_with_conclusion(&Literal::neg("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    assert!(!alice_wins, "Alice should be defeated when Bob's rule is preferred");
    assert!(bob_wins, "Bob should win when his rule is preferred");
}
