//! scene-tracer: pre-renders argumentation scenes to JSON for the website.
//!
//! Usage: `cargo run -p scene-tracer -- east-wall 0.5 website/static/traces/east-wall-b05.json`

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Serialize)]
struct Trace {
    scene_name: String,
    beta: f64,
    participants: Vec<String>,
    seeded_arguments: Vec<SeededArg>,
    attacks: Vec<AttackEdge>,
    beats: Vec<BeatRecord>,
    errors: Vec<String>,
}

#[derive(Serialize)]
struct SeededArg {
    actor: String,
    affordance_name: String,
    conclusion: String,
}

#[derive(Serialize)]
struct AttackEdge {
    attacker: String,
    target: String,
    weight: f64,
}

#[derive(Serialize)]
struct BeatRecord {
    actor: String,
    action: String,
    accepted: bool,
}

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
                bindings.insert("self".into(), actor.to_string());
                // Pick claim by affordance name so the AffordanceKey lookup
                // matches the seeded forward-index entry.
                let claim = if e.spec.name == "argue_fortify_east" {
                    "fortify_east"
                } else {
                    "abandon_east"
                };
                bindings.insert("claim".into(), claim.into());
                bindings.insert("expert".into(), actor.to_string());
                bindings.insert("domain".into(), "military".into());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

fn trace_east_wall(beta: f64) -> Trace {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".into(), "alice".into());
    alice_b.insert("domain".into(), "military".into());
    alice_b.insert("claim".into(), "fortify_east".into());
    let alice_instance = scheme.instantiate(&alice_b).unwrap();

    let mut bob_b = HashMap::new();
    bob_b.insert("expert".into(), "bob".into());
    bob_b.insert("domain".into(), "logistics".into());
    bob_b.insert("claim".into(), "abandon_east".into());
    let bob_instance = scheme.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".into(), "alice".into());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice",
        "argue_fortify_east",
        &alice_af,
        alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".into(), "bob".into());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob",
        "argue_abandon_east",
        &bob_af,
        bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "persuasion".into(),
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
    let catalog = vec![
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
        name: "debate".into(),
        affordances: vec![
            "argue_fortify_east".into(),
            "argue_abandon_east".into(),
        ],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    // 0.5 is the scorer boost magnitude, independent of scene β (set via set_intensity above).
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".into(), "bob".into()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

    Trace {
        scene_name: "east_wall".into(),
        beta,
        participants,
        seeded_arguments: vec![
            SeededArg {
                actor: "alice".into(),
                affordance_name: "argue_fortify_east".into(),
                conclusion: "fortify_east".into(),
            },
            SeededArg {
                actor: "bob".into(),
                affordance_name: "argue_abandon_east".into(),
                conclusion: "abandon_east".into(),
            },
        ],
        attacks: vec![AttackEdge {
            attacker: "abandon_east".into(),
            target: "fortify_east".into(),
            weight: 0.4,
        }],
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: scene-tracer <scene> <beta> <out-path>");
        std::process::exit(2);
    }
    let beta: f64 = args[2].parse().expect("beta must be a float");
    let trace = match args[1].as_str() {
        "east-wall" => trace_east_wall(beta),
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
    let json = serde_json::to_string_pretty(&trace).unwrap();
    fs::write(&args[3], json).expect("write failed");
    println!("wrote {}", args[3]);
}
