use argumentation_schemes::catalog::default_catalog;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter_argumentation::knowledge::{ArgumentPosition, StaticKnowledge};
use encounter_argumentation::scoring::SchemeActionScorer;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_entry(name: &str) -> CatalogEntry<String> {
    CatalogEntry {
        spec: AffordanceSpec {
            name: name.into(),
            domain: "test".into(),
            bindings: vec!["self".into(), "target".into()],
            considerations: vec![],
            effects_on_accept: vec![],
            effects_on_reject: vec![],
            drive_alignment: vec![],
        },
        precondition: String::new(),
    }
}

struct FixedScorer(f64);

impl ActionScorer<String> for FixedScorer {
    fn score_actions(
        &self,
        _actor: &str,
        available: &[CatalogEntry<String>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<String>> {
        available
            .iter()
            .map(|e| ScoredAffordance {
                entry: e.clone(),
                score: self.0,
                bindings: [
                    ("self".into(), "alice".into()),
                    ("target".into(), "bob".into()),
                ]
                .into_iter()
                .collect(),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Test 1: scheme-backed action scores higher than an unbacked one
// ---------------------------------------------------------------------------

/// "threaten" is backed by `argument_from_threat`; "gossip" has no backing.
/// The scorer must rank "threaten" above "gossip".
#[test]
fn scheme_backed_action_scores_higher_than_unbacked() {
    let mut knowledge = StaticKnowledge::new();

    knowledge.add_arguments(
        "alice",
        "threaten",
        vec![ArgumentPosition {
            scheme_key: "argument_from_threat".into(),
            bindings: HashMap::new(),
            preference_weight: 1.0,
        }],
    );
    // "gossip" has no registered arguments.

    let registry = default_catalog();
    let scorer = SchemeActionScorer::new(knowledge, registry, FixedScorer(0.5), 0.3);

    let entries = vec![test_entry("threaten"), test_entry("gossip")];
    let scored = scorer.score_actions("alice", &entries, &[]);

    assert_eq!(scored.len(), 2, "should score both actions");

    let threaten = scored
        .iter()
        .find(|s| s.entry.spec.name == "threaten")
        .unwrap();
    let gossip = scored
        .iter()
        .find(|s| s.entry.spec.name == "gossip")
        .unwrap();

    assert!(
        threaten.score > gossip.score,
        "threaten (scheme-backed) should score higher than gossip (unbacked); \
         threaten={}, gossip={}",
        threaten.score,
        gossip.score,
    );
}

// ---------------------------------------------------------------------------
// Test 2: boost scales with scheme strength
// ---------------------------------------------------------------------------

/// "demand" is backed by `argument_from_cause_to_effect` (Moderate, mult=0.67).
/// "hint" is backed by `argument_from_popular_opinion` (Weak, mult=0.33).
/// With the same preference_weight and max_boost, "demand" must score higher.
#[test]
fn boost_scales_with_scheme_strength() {
    let mut knowledge = StaticKnowledge::new();

    knowledge.add_arguments(
        "alice",
        "demand",
        vec![ArgumentPosition {
            scheme_key: "argument_from_cause_to_effect".into(),
            bindings: HashMap::new(),
            preference_weight: 1.0,
        }],
    );

    knowledge.add_arguments(
        "alice",
        "hint",
        vec![ArgumentPosition {
            scheme_key: "argument_from_popular_opinion".into(),
            bindings: HashMap::new(),
            preference_weight: 1.0,
        }],
    );

    let registry = default_catalog();
    let scorer = SchemeActionScorer::new(knowledge, registry, FixedScorer(0.5), 0.3);

    let entries = vec![test_entry("demand"), test_entry("hint")];
    let scored = scorer.score_actions("alice", &entries, &[]);

    assert_eq!(scored.len(), 2, "should score both actions");

    let demand = scored
        .iter()
        .find(|s| s.entry.spec.name == "demand")
        .unwrap();
    let hint = scored.iter().find(|s| s.entry.spec.name == "hint").unwrap();

    assert!(
        demand.score > hint.score,
        "demand (Moderate backing) should score higher than hint (Weak backing); \
         demand={}, hint={}",
        demand.score,
        hint.score,
    );
}
