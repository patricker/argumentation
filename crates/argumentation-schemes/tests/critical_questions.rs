//! UC4: critical questions as follow-up beat candidates. The encounter
//! engine will iterate `instance.critical_questions` and offer each one
//! as a candidate move for the opposing party.

use argumentation::aspic::Literal;
use argumentation_schemes::catalog::epistemic::argument_from_expert_opinion;
use std::collections::HashMap;

fn alice_bindings() -> HashMap<String, String> {
    [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect()
}

#[test]
fn expert_opinion_produces_six_follow_up_candidates() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    assert_eq!(instance.critical_questions.len(), 6);
}

#[test]
fn every_critical_question_text_resolves_at_least_one_binding() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    for cq in &instance.critical_questions {
        let mentions_binding = cq.text.contains("alice")
            || cq.text.contains("military")
            || cq.text.contains("fortify_east");
        assert!(
            mentions_binding,
            "CQ{} text should contain at least one resolved binding, got: {}",
            cq.number, cq.text
        );
    }
}

#[test]
fn every_critical_question_counter_literal_is_negated() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    for cq in &instance.critical_questions {
        assert!(
            matches!(cq.counter_literal, Literal::Neg(_)),
            "CQ{} counter-literal should be negated, got: {:?}",
            cq.number,
            cq.counter_literal
        );
    }
}

#[test]
fn critical_questions_span_multiple_challenge_types() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    let unique_challenges: std::collections::HashSet<_> = instance
        .critical_questions
        .iter()
        .map(|cq| std::mem::discriminant(&cq.challenge))
        .collect();
    assert!(
        unique_challenges.len() >= 2,
        "expert opinion CQs should span multiple challenge types"
    );
}

#[test]
fn premise_truth_counter_literal_is_contrary_of_premise() {
    use argumentation_schemes::types::Challenge;
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();

    // Find a CQ with a PremiseTruth challenge.
    let cq = instance
        .critical_questions
        .iter()
        .find(|cq| matches!(cq.challenge, Challenge::PremiseTruth(_)))
        .expect("expert opinion has at least one PremiseTruth CQ");

    // The counter-literal should be the contrary of one of the premise literals.
    let counter_is_contrary_of_some_premise = instance
        .premises
        .iter()
        .any(|p| cq.counter_literal.is_contrary_of(p));
    assert!(
        counter_is_contrary_of_some_premise,
        "PremiseTruth counter-literal should be the contrary of one of the instance's premise literals"
    );
}
