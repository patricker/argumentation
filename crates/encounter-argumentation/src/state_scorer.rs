//! `StateActionScorer`: encounter's `ActionScorer` impl backed by an
//! [`EncounterArgumentationState`].
//!
//! The scorer composes:
//! 1. An inner `ActionScorer<P>` that produces base scores.
//! 2. A reference to a live `EncounterArgumentationState`.
//!
//! After the inner scorer runs, each scored affordance is amplified
//! by `boost * state.is_credulously_accepted(arg_id)` where the
//! argument id is looked up from the affordance's forward-index key.
//! Affordances with no seeded argument receive no boost (inner score
//! unchanged).
//!
//! This gives the *proposer* bias: actions whose own argument is
//! credulously acceptable at the scene's current β get boosted,
//! making the scene self-consistent with the argumentation state.
//! The per-responder gate is orthogonal and lives in
//! [`crate::state_acceptance::StateAcceptanceEval`].

use crate::affordance_key::AffordanceKey;
use crate::state::EncounterArgumentationState;
use encounter::affordance::CatalogEntry;
use encounter::scoring::{ActionScorer, ScoredAffordance};

/// An [`ActionScorer<P>`] composing an inner scorer with a shared-ref
/// view of an [`EncounterArgumentationState`].
pub struct StateActionScorer<'a, S> {
    state: &'a EncounterArgumentationState,
    inner: S,
    boost: f64,
}

impl<'a, S> StateActionScorer<'a, S> {
    /// Construct a state-aware scorer wrapping `inner`. `boost` is
    /// the additive score added to any affordance whose argument is
    /// credulously accepted at the current β. Typical values: 0.3–1.0.
    #[must_use]
    pub fn new(state: &'a EncounterArgumentationState, inner: S, boost: f64) -> Self {
        Self { state, inner, boost }
    }
}

impl<S, P> ActionScorer<P> for StateActionScorer<'_, S>
where
    S: ActionScorer<P>,
    P: Clone,
{
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        let mut scored = self.inner.score_actions(actor, available, participants);
        for sa in &mut scored {
            let key = AffordanceKey::new(actor, &sa.entry.spec.name, &sa.bindings);
            let Some(id) = self.state.argument_id_for(&key) else {
                continue;
            };
            match self.state.is_credulously_accepted(&id) {
                Ok(true) => sa.score += self.boost,
                Ok(false) => {}
                Err(e) => {
                    self.state.record_error(e);
                }
            }
        }
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;
    use encounter::affordance::AffordanceSpec;
    use std::collections::HashMap;

    struct FlatInner;
    impl<P: Clone> ActionScorer<P> for FlatInner {
        fn score_actions(
            &self,
            _actor: &str,
            available: &[CatalogEntry<P>],
            _participants: &[String],
        ) -> Vec<ScoredAffordance<P>> {
            available
                .iter()
                .map(|e| {
                    let mut bindings = HashMap::new();
                    bindings.insert("self".to_string(), "alice".to_string());
                    bindings.insert("expert".to_string(), "alice".to_string());
                    bindings.insert("domain".to_string(), "military".to_string());
                    bindings.insert("claim".to_string(), "fortify_east".to_string());
                    ScoredAffordance {
                        entry: e.clone(),
                        score: 1.0,
                        bindings,
                    }
                })
                .collect()
        }
    }

    fn catalog() -> Vec<CatalogEntry<String>> {
        let spec = AffordanceSpec {
            name: "argue_fortify_east".to_string(),
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
        vec![CatalogEntry { spec, precondition: String::new() }]
    }

    #[test]
    fn unboosted_when_no_argument_is_seeded() {
        let state = EncounterArgumentationState::new(default_catalog());
        let scorer = StateActionScorer::new(&state, FlatInner, 0.5);
        let catalog_vec = catalog();
        let scored = scorer.score_actions("alice", &catalog_vec, &["alice".to_string()]);
        assert_eq!(scored.len(), 1);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boosted_when_argument_is_credulously_accepted() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut b = HashMap::new();
        b.insert("expert".to_string(), "alice".to_string());
        b.insert("domain".to_string(), "military".to_string());
        b.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&b).unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let mut affordance_b = b.clone();
        affordance_b.insert("self".to_string(), "alice".to_string());
        state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &affordance_b,
            instance,
        );

        let scorer = StateActionScorer::new(&state, FlatInner, 0.5);
        let catalog_vec = catalog();
        let scored = scorer.score_actions("alice", &catalog_vec, &["alice".to_string()]);
        assert_eq!(scored.len(), 1);
        // inner gave 1.0; argument is unattacked → credulously accepted → +0.5.
        assert!(
            (scored[0].score - 1.5).abs() < 1e-9,
            "expected 1.5, got {}",
            scored[0].score
        );
    }

    #[test]
    fn not_boosted_when_argument_is_attacked_above_budget() {
        // Seed an argument, attach an attacker heavier than the current
        // budget, and verify: (a) the argument is NOT credulously
        // accepted, so (b) the scorer does not apply the boost.
        use crate::arg_id::ArgumentId;

        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut b = HashMap::new();
        b.insert("expert".to_string(), "alice".to_string());
        b.insert("domain".to_string(), "military".to_string());
        b.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&b).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let mut affordance_b = b.clone();
        affordance_b.insert("self".to_string(), "alice".to_string());
        let alice_id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &affordance_b,
            instance,
        );
        // Attacker weight 0.8 > default β=0 → attack binds, argument not credulous.
        state
            .add_weighted_attack(&ArgumentId::new("attacker"), &alice_id, 0.8)
            .unwrap();

        let scorer = StateActionScorer::new(&state, FlatInner, 0.5);
        let catalog_vec = catalog();
        let scored = scorer.score_actions("alice", &catalog_vec, &["alice".to_string()]);
        assert_eq!(scored.len(), 1);
        // Attack binds at β=0 → argument NOT credulously accepted → score stays at 1.0.
        assert!(
            (scored[0].score - 1.0).abs() < 1e-9,
            "expected 1.0 (no boost), got {}",
            scored[0].score
        );
    }
}
