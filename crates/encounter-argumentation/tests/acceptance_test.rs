use argumentation_schemes::catalog::default_catalog;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::scoring::AcceptanceEval;
use encounter::scoring::ScoredAffordance;
use encounter_argumentation::acceptance::ArgumentAcceptanceEval;
use encounter_argumentation::knowledge::{ArgumentPosition, StaticKnowledge};

fn scored_action(name: &str) -> ScoredAffordance<String> {
    let spec = AffordanceSpec {
        name: name.into(),
        domain: "test".into(),
        bindings: vec!["self".into(), "target".into()],
        considerations: vec![],
        effects_on_accept: vec![],
        effects_on_reject: vec![],
        drive_alignment: vec![],
    };
    ScoredAffordance {
        entry: CatalogEntry {
            spec,
            precondition: String::new(),
        },
        score: 0.8,
        bindings: [
            ("self".into(), "alice".into()),
            ("target".into(), "bob".into()),
        ]
        .into_iter()
        .collect(),
    }
}

/// Test 1: No arguments at all → accept by default.
#[test]
fn accepts_when_responder_has_no_counter_arguments() {
    let knowledge = StaticKnowledge::new();
    let registry = default_catalog();
    let eval = ArgumentAcceptanceEval::new(knowledge, registry);

    let action = scored_action("negotiate");
    assert!(
        eval.evaluate("bob", &action),
        "should accept when there are no proposer arguments registered"
    );
}

/// Test 2: Alice argues weakly (popular opinion), Bob counters strongly
/// (established rule) → reject.
#[test]
fn rejects_when_responder_has_stronger_counter() {
    let mut knowledge = StaticKnowledge::new();

    // Alice proposes with Weak scheme.
    knowledge.add_arguments(
        "alice",
        "negotiate",
        vec![ArgumentPosition {
            scheme_key: "argument_from_popular_opinion".into(),
            bindings: [
                ("claim".into(), "negotiate".into()),
                ("population".into(), "the_council".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.3,
        }],
    );

    // Bob counters with Strong scheme.
    knowledge.add_counter_arguments(
        "bob",
        "negotiate",
        vec![ArgumentPosition {
            scheme_key: "argument_from_established_rule".into(),
            bindings: [
                ("rule".into(), "no_negotiation_rule".into()),
                ("case".into(), "negotiate".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.9,
        }],
    );

    let registry = default_catalog();
    let eval = ArgumentAcceptanceEval::new(knowledge, registry);

    let action = scored_action("negotiate");
    assert!(
        !eval.evaluate("bob", &action),
        "should reject when bob's Strong counter beats alice's Weak argument"
    );
}

/// Test 3: Alice argues moderately (expert opinion), Bob counters weakly
/// (ad hominem) → accept.
#[test]
fn accepts_when_proposer_has_stronger_argument() {
    let mut knowledge = StaticKnowledge::new();

    // Alice proposes with Moderate scheme.
    knowledge.add_arguments(
        "alice",
        "negotiate",
        vec![ArgumentPosition {
            scheme_key: "argument_from_expert_opinion".into(),
            bindings: [
                ("expert".into(), "alice".into()),
                ("domain".into(), "diplomacy".into()),
                ("claim".into(), "negotiate".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.7,
        }],
    );

    // Bob counters with Weak scheme.
    knowledge.add_counter_arguments(
        "bob",
        "negotiate",
        vec![ArgumentPosition {
            scheme_key: "ad_hominem".into(),
            bindings: [
                ("target".into(), "alice".into()),
                ("flaw".into(), "naivety".into()),
                ("claim".into(), "negotiate".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.2,
        }],
    );

    let registry = default_catalog();
    let eval = ArgumentAcceptanceEval::new(knowledge, registry);

    let action = scored_action("negotiate");
    assert!(
        eval.evaluate("bob", &action),
        "should accept when alice's Moderate argument beats bob's Weak counter"
    );
}
