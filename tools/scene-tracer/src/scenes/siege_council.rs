//! Siege council — four officers debate the response to a frontier siege.
//!
//! Two relationship climates: `Cold` (officers bickering, full attack weights)
//! and `Warm` (officers cooperating, weights halved). The attack graph itself
//! is identical — only the weights differ. This is faithful to how
//! `SocietasRelationshipSource` modulates attack weights via relationship
//! state in production wiring.

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

#[derive(Clone, Copy)]
pub enum Climate {
    Cold,
    Warm,
}

impl Climate {
    fn weight_multiplier(self) -> f64 {
        match self {
            Climate::Cold => 1.0,
            Climate::Warm => 0.5,
        }
    }

    fn scene_name(self) -> &'static str {
        match self {
            Climate::Cold => "siege_council_cold",
            Climate::Warm => "siege_council_warm",
        }
    }
}

/// (actor, claim, expert-domain) for each officer.
const OFFICERS: &[(&str, &str, &str)] = &[
    ("aleric", "fortify", "military"),
    ("maren", "abandon", "logistics"),
    ("drust", "sortie", "intelligence"),
    ("liss", "evacuate_first", "civilian"),
];

/// (attacker_actor, target_actor, base_weight) — base weights are scaled by
/// climate. Edges:
///   - maren->aleric: "we have no supplies for a siege"
///   - aleric->maren: "retreat exposes the civilians we are sworn to protect"
///   - drust->maren: "the enemy is thinner than we feared; retreat is unnecessary"
///   - liss->aleric: "civilians need time to evacuate before fortification"
///   - drust->liss:  "there is no time for a full evacuation"
const ATTACKS: &[(&str, &str, f64)] = &[
    ("maren", "aleric", 0.5),
    ("aleric", "maren", 0.4),
    ("drust", "maren", 0.6),
    ("liss", "aleric", 0.3),
    ("drust", "liss", 0.5),
];

struct SiegeScorer;

impl<P: Clone> ActionScorer<P> for SiegeScorer {
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
                    "argue_fortify" => ("fortify", "military"),
                    "argue_abandon" => ("abandon", "logistics"),
                    "argue_sortie" => ("sortie", "intelligence"),
                    "argue_evacuate_first" => ("evacuate_first", "civilian"),
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

pub fn trace(beta: f64, climate: Climate) -> Trace {
    let registry = default_catalog();

    // Pre-instantiate schemes for each officer before moving the registry into state.
    let prepared: Vec<_> = {
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        OFFICERS
            .iter()
            .map(|(actor, claim, domain)| {
                let mut b = HashMap::new();
                b.insert("expert".into(), (*actor).to_string());
                b.insert("domain".into(), (*domain).to_string());
                b.insert("claim".into(), (*claim).to_string());
                let instance = scheme.instantiate(&b).unwrap();
                let mut af = b.clone();
                af.insert("self".into(), (*actor).to_string());
                (*actor, *claim, af, instance)
            })
            .collect()
    };

    let mut state = EncounterArgumentationState::new(registry);
    let mut ids: HashMap<&str, _> = HashMap::new();

    for (actor, claim, af, instance) in prepared {
        let id = state.add_scheme_instance_for_affordance(
            actor,
            &format!("argue_{claim}"),
            &af,
            instance,
        );
        ids.insert(actor, id);
    }

    let mult = climate.weight_multiplier();
    let mut attack_edges = Vec::new();
    for (atk, tgt, base_w) in ATTACKS {
        let w = (base_w * mult).min(1.0);
        state.add_weighted_attack(&ids[atk], &ids[tgt], w).unwrap();
        // Track the conclusion-level attack edge for the trace JSON.
        let atk_claim = OFFICERS.iter().find(|(a, _, _)| a == atk).unwrap().1;
        let tgt_claim = OFFICERS.iter().find(|(a, _, _)| a == tgt).unwrap().1;
        attack_edges.push(AttackEdge {
            attacker: atk_claim.into(),
            target: tgt_claim.into(),
            weight: w,
        });
    }

    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "council".into(),
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
        "argue_fortify",
        "argue_abandon",
        "argue_sortie",
        "argue_evacuate_first",
    ];
    let catalog: Vec<_> = aff_names
        .iter()
        .map(|n| CatalogEntry {
            spec: make_spec(n),
            precondition: String::new(),
        })
        .collect();
    let practice = PracticeSpec {
        name: "council".into(),
        affordances: aff_names.iter().map(|s| (*s).to_string()).collect(),
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 8 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, SiegeScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants: Vec<String> =
        OFFICERS.iter().map(|(a, _, _)| (*a).to_string()).collect();
    let result = MultiBeat.resolve(
        &participants,
        &practice,
        &catalog,
        &scorer,
        &acceptance,
    );

    let seeded_arguments = OFFICERS
        .iter()
        .map(|(actor, claim, _)| SeededArg {
            actor: (*actor).to_string(),
            affordance_name: format!("argue_{claim}"),
            conclusion: (*claim).to_string(),
        })
        .collect();

    Trace {
        scene_name: climate.scene_name().to_string(),
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
