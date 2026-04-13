use argumentation_schemes::catalog::default_catalog;
use argumentation_schemes::instantiate;
use encounter_argumentation::resolver::{ArgumentOutcome, resolve_argument};
use std::collections::HashMap;

fn expert_bindings() -> HashMap<String, String> {
    [
        ("expert".into(), "alice".into()),
        ("domain".into(), "military".into()),
        ("claim".into(), "fortify_east".into()),
    ]
    .into_iter()
    .collect()
}

fn adhominem_bindings() -> HashMap<String, String> {
    [
        ("target".into(), "alice".into()),
        ("flaw".into(), "cowardice".into()),
        ("claim".into(), "fortify_east".into()),
    ]
    .into_iter()
    .collect()
}

#[test]
fn proposer_wins_when_no_counter_arguments() {
    let catalog = default_catalog();
    let expert_scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
    let expert_instance = instantiate(expert_scheme, &expert_bindings()).unwrap();

    let outcome = resolve_argument(&[expert_instance], &[], &catalog);
    assert!(
        matches!(outcome, ArgumentOutcome::ProposerWins { .. }),
        "expected ProposerWins, got {:?}",
        outcome
    );
}

#[test]
fn proposer_wins_when_stronger_than_counter() {
    let catalog = default_catalog();
    let expert_scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
    let adhominem_scheme = catalog.by_key("ad_hominem").unwrap();

    let expert_instance = instantiate(expert_scheme, &expert_bindings()).unwrap();
    let adhominem_instance = instantiate(adhominem_scheme, &adhominem_bindings()).unwrap();

    // Expert opinion is Moderate, Ad Hominem is Weak — proposer should win.
    let outcome = resolve_argument(&[expert_instance], &[adhominem_instance], &catalog);
    assert!(
        matches!(outcome, ArgumentOutcome::ProposerWins { .. }),
        "expected ProposerWins (Moderate > Weak), got {:?}",
        outcome
    );
}

#[test]
fn undecided_when_equal_strength_no_preference() {
    // Two moderate schemes with non-conflicting conclusions.
    // Both survive so the outcome is ProposerWins or Undecided.
    let catalog = default_catalog();
    let expert_scheme = catalog.by_key("argument_from_expert_opinion").unwrap();

    let bindings_a: HashMap<String, String> = [
        ("expert".into(), "alice".into()),
        ("domain".into(), "military".into()),
        ("claim".into(), "fortify_east".into()),
    ]
    .into_iter()
    .collect();

    let bindings_b: HashMap<String, String> = [
        ("expert".into(), "bob".into()),
        ("domain".into(), "economics".into()),
        ("claim".into(), "fund_navy".into()),
    ]
    .into_iter()
    .collect();

    let instance_a = instantiate(expert_scheme, &bindings_a).unwrap();
    let instance_b = instantiate(expert_scheme, &bindings_b).unwrap();

    // Both proposer and responder use Moderate-strength schemes on different conclusions.
    let outcome = resolve_argument(&[instance_a], &[instance_b], &catalog);
    assert!(
        matches!(
            outcome,
            ArgumentOutcome::ProposerWins { .. } | ArgumentOutcome::Undecided
        ),
        "expected ProposerWins or Undecided for equal-strength non-conflicting, got {:?}",
        outcome
    );
}
