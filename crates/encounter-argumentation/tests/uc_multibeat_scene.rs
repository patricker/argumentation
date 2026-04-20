//! End-to-end integration: run encounter's MultiBeat protocol with
//! StateActionScorer + StateAcceptanceEval backed by an
//! EncounterArgumentationState. Verifies that (a) the bridge compiles
//! cleanly against real encounter trait signatures, (b) scenes resolve
//! deterministically across a range of β values, (c) the error latch
//! drains cleanly.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};
use std::collections::HashMap;

/// Inner scorer: uniform 1.0 score, binds self to the calling actor.
struct UniformScorer;

impl<P: Clone> ActionScorer<P> for UniformScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available
            .iter()
            .map(|e| {
                let mut bindings = HashMap::new();
                bindings.insert("self".to_string(), actor.to_string());
                bindings.insert("expert".to_string(), actor.to_string());
                bindings.insert("domain".to_string(), "military".to_string());
                let claim = if e.spec.name == "argue_fortify_east" {
                    "fortify_east"
                } else {
                    "abandon_east"
                };
                bindings.insert("claim".to_string(), claim.to_string());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

fn build_scene() -> (
    EncounterArgumentationState,
    Vec<CatalogEntry<String>>,
    PracticeSpec,
) {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".to_string(), "alice".to_string());
    alice_b.insert("domain".to_string(), "military".to_string());
    alice_b.insert("claim".to_string(), "fortify_east".to_string());
    let alice_instance = scheme.instantiate(&alice_b).unwrap();
    let mut bob_b = HashMap::new();
    bob_b.insert("expert".to_string(), "bob".to_string());
    bob_b.insert("domain".to_string(), "logistics".to_string());
    bob_b.insert("claim".to_string(), "abandon_east".to_string());
    let bob_instance = scheme.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".to_string(), "alice".to_string());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice",
        "argue_fortify_east",
        &alice_af,
        alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".to_string(), "bob".to_string());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob",
        "argue_abandon_east",
        &bob_af,
        bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();

    let make_spec = |name: &str| AffordanceSpec {
        name: name.to_string(),
        domain: "persuasion".to_string(),
        bindings: vec![
            "self".to_string(),
            "expert".to_string(),
            "domain".to_string(),
            "claim".to_string(),
        ],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };

    let catalog_vec = vec![
        CatalogEntry {
            spec: make_spec("argue_fortify_east"),
            precondition: String::new(),
        },
        CatalogEntry {
            spec: make_spec("argue_abandon_east"),
            precondition: String::new(),
        },
    ];
    let practice = PracticeSpec {
        name: "debate".to_string(),
        affordances: vec![
            "argue_fortify_east".to_string(),
            "argue_abandon_east".to_string(),
        ],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    (state, catalog_vec, practice)
}

#[test]
fn scene_resolves_cleanly_at_low_intensity() {
    // At β=0.0, bob's 0.4-weight attack on alice's argue_fortify_east
    // binds, so alice's argument is NOT credulously accepted → no boost.
    // Both of alice's affordances score 1.0; the inner scorer's order
    // is preserved by the stable sort, so alice picks argue_abandon_east
    // (second in catalog, but tied → sort preserves inner order — in
    // practice, alice ends up on argue_abandon_east because the boost
    // path degenerates). Bob's argue_abandon_east IS credulous and
    // receives the boost. Bob has no accepted counter to alice's
    // chosen argument (there's no seeded alice→bob attack), so every
    // beat is accepted.
    let (state, catalog_vec, practice) = build_scene();
    state.set_intensity(Budget::new(0.0).unwrap());
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".to_string(), "bob".to_string()];
    let result =
        MultiBeat.resolve(&participants, &practice, &catalog_vec, &scorer, &acceptance);
    assert_eq!(
        result.beats.len(),
        4,
        "round-robin with max_beats=4 and no PracticeExit should produce 4 beats"
    );
    // With no boost applied to alice's seeded argument (attack binds at
    // β=0), alice never picks argue_fortify_east → no beat rejections
    // via bob's counter. This observed outcome would break if the
    // bridge degenerated to no-op (e.g. StateAcceptanceEval always
    // returning false): bob's beats would then also be `accepted=false`.
    assert!(
        result.beats.iter().all(|b| b.accepted),
        "at β=0 all four beats should be accepted (alice falls through \
         to argue_abandon_east, no cross-actor counter fires); got {:?}",
        result.beats
    );
    assert!(state.take_latest_error().is_none());
}

#[test]
fn higher_intensity_changes_acceptance_outcomes() {
    // At β=0.5 > 0.4, the attack on alice's argue_fortify_east is
    // droppable, so alice's argument IS credulously accepted → the
    // scorer applies the +0.5 boost → alice picks argue_fortify_east
    // (score 1.5 > argue_abandon_east's 1.0). When bob responds,
    // has_accepted_counter_by("bob", alice_id) returns true (bob's
    // argue_abandon_east is credulous and attacks alice's argument)
    // → alice's beats are REJECTED. Bob's argue_abandon_east remains
    // unattacked, credulous, and has no counter from alice → accepted.
    let (state, catalog_vec, practice) = build_scene();
    state.set_intensity(Budget::new(0.5).unwrap());
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".to_string(), "bob".to_string()];
    let result =
        MultiBeat.resolve(&participants, &practice, &catalog_vec, &scorer, &acceptance);
    assert_eq!(result.beats.len(), 4);
    // Round-robin starts with alice: beats 0,2 are alice; beats 1,3 are bob.
    for (i, beat) in result.beats.iter().enumerate() {
        if i % 2 == 0 {
            assert_eq!(beat.actor, "alice", "even beats should be alice");
            assert_eq!(
                beat.action, "argue_fortify_east",
                "β=0.5 should boost alice's seeded argument above her unboosted alternative"
            );
            assert!(
                !beat.accepted,
                "bob's credulous counter should reject alice's argument"
            );
        } else {
            assert_eq!(beat.actor, "bob", "odd beats should be bob");
            assert_eq!(beat.action, "argue_abandon_east");
            assert!(
                beat.accepted,
                "bob's argument is unattacked and alice has no counter → accepted"
            );
        }
    }
    assert!(state.take_latest_error().is_none());
}
