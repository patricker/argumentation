//! Hal & Carla — Bench-Capon's canonical value-based example, run as an
//! abstract weighted framework. The current implementation does not yet
//! support values explicitly; this fixture treats the scene as a Dung-style
//! weighted framework so the user can see the symmetric-attack stalemate at
//! low β and the resolution as β rises. The conceptual page links from here
//! to the open-areas / VAF scoping doc that explains what changes once
//! values are wired in.
//!
//! Bench-Capon (2003): Hal, a diabetic, takes Carla's insulin to save his
//! life. Should he be punished?

use crate::trace::{AttackEdge, BeatRecord, SeededArg, Trace};
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

/// (actor, claim, expert-domain).
const PARTIES: &[(&str, &str, &str)] = &[
    ("hal", "life_over_property", "ethics"),
    ("carla", "property_rights", "ethics"),
    ("hal", "too_poor_to_compensate", "economics"),
    ("carla", "my_only_dose", "ethics"),
];

const ATTACKS: &[(usize, usize, f64)] = &[
    // (attacker_idx, target_idx, weight) — indices into PARTIES.
    (1, 0, 0.5), // C1 ↔ H1, mutual
    (0, 1, 0.5),
    (3, 2, 0.6), // C2 dampens H2 (Carla also at risk)
    (2, 1, 0.4), // H2 attacks C1 (cannot compensate)
];

struct HalCarlaScorer;

impl<P: Clone> ActionScorer<P> for HalCarlaScorer {
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
                bindings.insert("self".into(), actor.to_string());
                bindings.insert("expert".into(), actor.to_string());
                let (claim, domain) = match e.spec.name.as_str() {
                    "argue_life_over_property" => ("life_over_property", "ethics"),
                    "argue_property_rights" => ("property_rights", "ethics"),
                    "argue_too_poor_to_compensate" => ("too_poor_to_compensate", "economics"),
                    "argue_my_only_dose" => ("my_only_dose", "ethics"),
                    other => panic!("unexpected affordance: {other}"),
                };
                bindings.insert("claim".into(), claim.into());
                bindings.insert("domain".into(), domain.into());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

pub fn trace(beta: f64) -> Trace {
    let registry = default_catalog();

    // Pre-instantiate all scheme instances while `scheme` is borrowed from
    // `registry`, before moving `registry` into the state (borrow-checker
    // workaround — see siege_council.rs for the same pattern).
    let prepared: Vec<_> = {
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        PARTIES
            .iter()
            .map(|(actor, claim, domain)| {
                let mut b = HashMap::new();
                b.insert("expert".into(), (*actor).into());
                b.insert("domain".into(), (*domain).into());
                b.insert("claim".into(), (*claim).into());
                let instance = scheme.instantiate(&b).unwrap();
                let mut af = b.clone();
                af.insert("self".into(), (*actor).into());
                (*actor, *claim, af, instance)
            })
            .collect()
    };

    let mut state = EncounterArgumentationState::new(registry);
    let mut ids: Vec<_> = Vec::new();
    for (actor, claim, af, instance) in prepared {
        let id = state.add_scheme_instance_for_affordance(
            actor,
            &format!("argue_{claim}"),
            &af,
            instance,
        );
        ids.push(id);
    }

    let mut attack_edges = Vec::new();
    for (a, t, w) in ATTACKS {
        state.add_weighted_attack(&ids[*a], &ids[*t], *w).unwrap();
        attack_edges.push(AttackEdge {
            attacker: PARTIES[*a].1.into(),
            target: PARTIES[*t].1.into(),
            weight: *w,
        });
    }

    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "courtroom".into(),
        bindings: vec![
            "self".into(),
            "expert".into(),
            "domain".into(),
            "claim".into(),
        ],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let aff_names = [
        "argue_life_over_property",
        "argue_property_rights",
        "argue_too_poor_to_compensate",
        "argue_my_only_dose",
    ];
    let catalog: Vec<_> = aff_names
        .iter()
        .map(|n| CatalogEntry {
            spec: make_spec(n),
            precondition: String::new(),
        })
        .collect();
    let practice = PracticeSpec {
        name: "courtroom".into(),
        affordances: aff_names.iter().map(|s| (*s).to_string()).collect(),
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 6 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, HalCarlaScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    // Two unique participants — Hal and Carla — each speaks for two arguments.
    let participants: Vec<String> = vec!["hal".into(), "carla".into()];
    let result = MultiBeat.resolve(
        &participants,
        &practice,
        &catalog,
        &scorer,
        &acceptance,
    );

    let seeded_arguments = PARTIES
        .iter()
        .map(|(actor, claim, _)| SeededArg {
            actor: (*actor).to_string(),
            affordance_name: format!("argue_{claim}"),
            conclusion: (*claim).to_string(),
        })
        .collect();

    Trace {
        scene_name: "hal_carla".into(),
        beta,
        participants,
        seeded_arguments,
        attacks: attack_edges,
        beats: result
            .beats
            .iter()
            .map(|b| BeatRecord {
                actor: b.actor.clone(),
                action: b.action.clone(),
                accepted: b.accepted,
            })
            .collect(),
        errors: state.drain_errors().iter().map(|e| e.to_string()).collect(),
    }
}
