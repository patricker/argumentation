use argumentation_schemes::catalog::default_catalog;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter::types::Effect;
use encounter_argumentation::acceptance::ArgumentAcceptanceEval;
use encounter_argumentation::critical_moves::critical_question_beats;
use encounter_argumentation::knowledge::{ArgumentPosition, StaticKnowledge};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

fn alliance_entry() -> CatalogEntry<String> {
    CatalogEntry {
        spec: AffordanceSpec {
            name: "propose_alliance".into(),
            domain: "political".into(),
            bindings: vec!["self".into(), "target".into()],
            considerations: vec![],
            effects_on_accept: vec![Effect::RelationshipDelta {
                axis: "trust".into(),
                from: "target".into(),
                to: "self".into(),
                delta: 0.15,
            }],
            effects_on_reject: vec![Effect::RelationshipDelta {
                axis: "trust".into(),
                from: "self".into(),
                to: "target".into(),
                delta: -0.1,
            }],
            drive_alignment: vec![],
        },
        precondition: String::new(),
    }
}

fn threaten_entry() -> CatalogEntry<String> {
    CatalogEntry {
        spec: AffordanceSpec {
            name: "threaten".into(),
            domain: "political".into(),
            bindings: vec!["self".into(), "target".into()],
            considerations: vec![],
            effects_on_accept: vec![Effect::RelationshipDelta {
                axis: "trust".into(),
                from: "target".into(),
                to: "self".into(),
                delta: -0.2,
            }],
            effects_on_reject: vec![Effect::RelationshipDelta {
                axis: "trust".into(),
                from: "self".into(),
                to: "target".into(),
                delta: -0.3,
            }],
            drive_alignment: vec![],
        },
        precondition: String::new(),
    }
}

struct PoliticalScorer;

impl ActionScorer<String> for PoliticalScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<String>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<String>> {
        available
            .iter()
            .enumerate()
            .map(|(i, e)| ScoredAffordance {
                entry: e.clone(),
                score: 0.5 + (i as f64 * 0.1),
                bindings: [
                    ("self".into(), actor.into()),
                    ("target".into(), "opponent".into()),
                ]
                .into_iter()
                .collect(),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Test 1: Full multi-beat encounter with argument acceptance
// ---------------------------------------------------------------------------

/// Runs a MultiBeat encounter between alice and bob with two catalog entries
/// (`propose_alliance` and `threaten`). Alice argues for each via weak/weak
/// schemes while bob counters `threaten` with a strong scheme. The test
/// verifies that the encounter produces beats and respects `max_beats`.
#[test]
fn multi_beat_encounter_with_argument_acceptance() {
    let mut knowledge = StaticKnowledge::new();

    // Alice argues for "propose_alliance" via popular opinion (Weak).
    knowledge.add_arguments(
        "alice",
        "propose_alliance",
        vec![ArgumentPosition {
            scheme_key: "argument_from_popular_opinion".into(),
            bindings: [
                ("claim".into(), "propose_alliance".into()),
                ("population".into(), "the_council".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.4,
        }],
    );

    // Alice argues for "threaten" via argument from threat (Weak).
    knowledge.add_arguments(
        "alice",
        "threaten",
        vec![ArgumentPosition {
            scheme_key: "argument_from_threat".into(),
            bindings: [
                ("threatener".into(), "alice".into()),
                ("threat".into(), "military_action".into()),
                ("demand".into(), "surrender".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.5,
        }],
    );

    // Bob counters "threaten" with established rule (Strong) — beats alice's Weak.
    knowledge.add_counter_arguments(
        "bob",
        "threaten",
        vec![ArgumentPosition {
            scheme_key: "argument_from_established_rule".into(),
            bindings: [
                ("rule".into(), "no_threats_allowed".into()),
                ("case".into(), "threaten".into()),
            ]
            .into_iter()
            .collect(),
            preference_weight: 0.9,
        }],
    );

    let registry = default_catalog();
    let eval = ArgumentAcceptanceEval::new(knowledge, registry);

    let practice = PracticeSpec {
        name: "political_dialogue".into(),
        affordances: vec!["propose_alliance".into(), "threaten".into()],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };

    let catalog = vec![alliance_entry(), threaten_entry()];
    let participants = vec!["alice".to_string(), "bob".to_string()];

    let result = MultiBeat.resolve(&participants, &practice, &catalog, &PoliticalScorer, &eval);

    // The encounter must produce at least one beat.
    assert!(
        !result.beats.is_empty(),
        "encounter must produce at least one beat"
    );

    // The encounter must not exceed max_beats.
    assert!(
        result.beats.len() <= 4,
        "encounter must not exceed max_beats=4; got {} beats",
        result.beats.len()
    );

    // The practice name must be recorded on the result.
    assert_eq!(result.practice.as_deref(), Some("political_dialogue"));

    // Both participants must be listed.
    assert_eq!(result.participants.len(), 2);
    assert!(result.participants.contains(&"alice".to_string()));
    assert!(result.participants.contains(&"bob".to_string()));
}

// ---------------------------------------------------------------------------
// Test 2: Critical questions generate follow-up beat candidates
// ---------------------------------------------------------------------------

/// Instantiates `argument_from_expert_opinion` with concrete bindings,
/// calls `critical_question_beats`, and verifies:
/// - exactly 6 beats are produced (one per critical question)
/// - every beat carries the correct challenger as actor
/// - the resolved binding text is present in the beat action strings
#[test]
fn critical_questions_generate_follow_up_candidates() {
    let catalog = default_catalog();
    let scheme = catalog
        .by_key("argument_from_expert_opinion")
        .expect("argument_from_expert_opinion must be in the default catalog");

    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();

    let instance = scheme
        .instantiate(&bindings)
        .expect("instantiation with full bindings must succeed");

    let beats = critical_question_beats(&instance, "bob");

    // Expert opinion has exactly 6 critical questions.
    assert_eq!(
        beats.len(),
        6,
        "argument_from_expert_opinion must yield 6 CQ beats"
    );

    // Every beat must have "bob" as the actor (the challenger).
    for beat in &beats {
        assert_eq!(
            beat.actor, "bob",
            "every CQ beat must carry the challenger as actor"
        );
    }

    // The resolved binding text for "alice" (the expert) must appear somewhere
    // in the beat actions. CQ1 text references ?expert which resolves to "alice".
    let first_beat = &beats[0];
    assert!(
        first_beat.action.contains("alice"),
        "first CQ beat action must contain the resolved expert name 'alice'; got: {}",
        first_beat.action
    );

    // Each beat action must start with the "cqN:" prefix.
    for (i, beat) in beats.iter().enumerate() {
        let expected_prefix = format!("cq{}:", i + 1);
        assert!(
            beat.action.starts_with(&expected_prefix),
            "beat {} action must start with '{}'; got: {}",
            i,
            expected_prefix,
            beat.action
        );
    }

    // CQ beats are adversarial — none should be accepted.
    for beat in &beats {
        assert!(!beat.accepted, "CQ beats must not be marked as accepted");
    }
}
