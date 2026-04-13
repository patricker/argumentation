use argumentation_schemes::catalog::default_catalog;
use encounter_argumentation::critical_moves::{cq_to_beat, critical_question_beats};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn expert_opinion_instance() -> argumentation_schemes::instance::SchemeInstance {
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
    argumentation_schemes::instantiate(scheme, &bindings)
        .expect("expert opinion instantiation must succeed")
}

fn ad_hominem_instance() -> argumentation_schemes::instance::SchemeInstance {
    let catalog = default_catalog();
    let scheme = catalog
        .by_key("ad_hominem")
        .expect("ad_hominem must be in the default catalog");
    let bindings: HashMap<String, String> = [
        ("target".to_string(), "alice".to_string()),
        ("flaw".to_string(), "cowardice".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    argumentation_schemes::instantiate(scheme, &bindings)
        .expect("ad hominem instantiation must succeed")
}

// ---------------------------------------------------------------------------
// Test 1: Expert opinion scheme produces 6 CQ beats
// ---------------------------------------------------------------------------

/// The expert opinion scheme has 6 critical questions; `critical_question_beats`
/// must return exactly 6 beats, all with the given challenger as actor and
/// `accepted = false`.
#[test]
fn expert_opinion_produces_six_cq_beats() {
    let instance = expert_opinion_instance();
    let beats = critical_question_beats(&instance, "bob");

    assert_eq!(
        beats.len(),
        6,
        "expert opinion scheme must yield 6 CQ beats"
    );

    for beat in &beats {
        assert_eq!(
            beat.actor, "bob",
            "every CQ beat must have the challenger as actor"
        );
        assert!(
            !beat.accepted,
            "CQ beats are adversarial and must not be accepted"
        );
        assert!(
            beat.effects.is_empty(),
            "critical_question_beats produces beats with no effects"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 2: Ad hominem scheme produces 3 CQ beats
// ---------------------------------------------------------------------------

/// The ad hominem scheme has 3 critical questions; `critical_question_beats`
/// must return exactly 3 beats.
#[test]
fn ad_hominem_produces_three_cq_beats() {
    let instance = ad_hominem_instance();
    let beats = critical_question_beats(&instance, "charlie");

    assert_eq!(beats.len(), 3, "ad hominem scheme must yield 3 CQ beats");

    for beat in &beats {
        assert_eq!(beat.actor, "charlie");
        assert!(!beat.accepted);
    }
}

// ---------------------------------------------------------------------------
// Test 3: CQ beat action contains resolved question text
// ---------------------------------------------------------------------------

/// `cq_to_beat` formats the action as `"cq<n>:<resolved_text>"`.
/// After binding `expert = "alice"`, CQ1 of the expert-opinion scheme
/// resolves to text containing "alice". Verify the action string reflects this.
#[test]
fn cq_beat_action_contains_question_text() {
    let instance = expert_opinion_instance();

    // CQ1: "How credible is ?expert as an expert source?" → "How credible is alice …"
    let cq1 = &instance.critical_questions[0];
    let beat = cq_to_beat(cq1, "bob", Vec::new());

    assert!(
        beat.action.starts_with("cq1:"),
        "action must start with the CQ number prefix; got: {}",
        beat.action
    );
    assert!(
        beat.action.contains("alice"),
        "resolved action text must contain the bound expert name 'alice'; got: {}",
        beat.action
    );
}
