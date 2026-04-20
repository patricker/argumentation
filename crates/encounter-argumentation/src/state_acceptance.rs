//! `StateAcceptanceEval`: encounter's `AcceptanceEval` impl backed by
//! an [`EncounterArgumentationState`]'s current β-intensity.
//!
//! The eval rejects a proposed action iff the responder asserts a
//! credulously-accepted attacker of the proposed argument at the
//! current scene intensity. Otherwise it accepts.
//!
//! Internal errors (e.g. weighted-bipolar edge limit exceeded) cause
//! the eval to default to *accept* and append the error to the state's
//! buffer; consumers call `state.drain_errors()` to drain.
//!
//! ## Proposer-slot convention
//!
//! `StateAcceptanceEval` looks up the proposer's argument using the
//! binding slot `"self"`. If your catalog uses a different convention
//! (e.g., `"speaker"` or `"initiator"`), the eval will silently default
//! to *accept* for those affordances. This is intentional —
//! affordances without a seeded argument have nothing to adjudicate.
//! Consumers who use a different slot name should seed their
//! affordances with `"self"` bindings pointing at the proposer, or
//! wrap this eval with a custom version that remaps the slot.

use crate::affordance_key::AffordanceKey;
use crate::state::EncounterArgumentationState;
use encounter::scoring::{AcceptanceEval, ScoredAffordance};

/// An [`AcceptanceEval<P>`] backed by a shared reference to a live
/// [`EncounterArgumentationState`].
///
/// The eval uses [`EncounterArgumentationState::has_accepted_counter_by`]
/// to decide whether the responder has a credulously-accepted counter
/// to the proposed action's argument at current β. If the action has
/// no seeded argument in the state, the eval accepts (falls back to
/// permissive — there's no argumentation claim against which to
/// adjudicate).
pub struct StateAcceptanceEval<'a> {
    state: &'a EncounterArgumentationState,
}

impl<'a> StateAcceptanceEval<'a> {
    /// Construct an acceptance eval borrowing the given state.
    #[must_use]
    pub fn new(state: &'a EncounterArgumentationState) -> Self {
        Self { state }
    }

    /// Build the key to look up the PROPOSER's argument from a scored
    /// action. By convention the proposer binding slot is `"self"`.
    fn proposer_key<P>(&self, action: &ScoredAffordance<P>) -> Option<AffordanceKey> {
        let proposer = action.bindings.get("self")?;
        Some(AffordanceKey::new(
            proposer,
            &action.entry.spec.name,
            &action.bindings,
        ))
    }
}

impl<P> AcceptanceEval<P> for StateAcceptanceEval<'_> {
    fn evaluate(&self, responder: &str, action: &ScoredAffordance<P>) -> bool {
        let Some(key) = self.proposer_key(action) else {
            self.state.record_error(crate::error::Error::MissingProposerBinding {
                affordance_name: action.entry.spec.name.clone(),
            });
            return true;
        };
        let Some(target) = self.state.argument_id_for(&key) else {
            return true;
        };
        match self.state.has_accepted_counter_by(responder, &target) {
            Ok(true) => false,
            Ok(false) => true,
            Err(e) => {
                self.state.record_error(e);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;
    use encounter::affordance::{AffordanceSpec, CatalogEntry};
    use encounter::scoring::ScoredAffordance;
    use std::collections::HashMap;

    fn make_affordance(
        name: &str,
        self_actor: &str,
        expert: &str,
        domain: &str,
        claim: &str,
    ) -> ScoredAffordance<String> {
        let spec = AffordanceSpec {
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
        let entry = CatalogEntry { spec, precondition: String::new() };
        let mut bindings = HashMap::new();
        bindings.insert("self".to_string(), self_actor.to_string());
        bindings.insert("expert".to_string(), expert.to_string());
        bindings.insert("domain".to_string(), domain.to_string());
        bindings.insert("claim".to_string(), claim.to_string());
        ScoredAffordance { entry, score: 1.0, bindings }
    }

    #[test]
    fn accepts_when_no_argument_is_seeded_for_the_affordance() {
        let state = EncounterArgumentationState::new(default_catalog());
        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance("unseeded_action", "alice", "alice", "military", "x");
        assert!(eval.evaluate("bob", &action));
    }

    #[test]
    fn accepts_when_responder_has_no_counter() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut proposer_bindings = HashMap::new();
        proposer_bindings.insert("expert".to_string(), "alice".to_string());
        proposer_bindings.insert("domain".to_string(), "military".to_string());
        proposer_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&proposer_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let mut affordance_bindings = proposer_bindings.clone();
        affordance_bindings.insert("self".to_string(), "alice".to_string());
        state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &affordance_bindings,
            instance,
        );
        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance(
            "argue_fortify_east",
            "alice",
            "alice",
            "military",
            "fortify_east",
        );
        assert!(eval.evaluate("bob", &action));
    }

    #[test]
    fn rejects_when_responder_asserts_an_accepted_counter() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

        let mut alice_bindings = HashMap::new();
        alice_bindings.insert("expert".to_string(), "alice".to_string());
        alice_bindings.insert("domain".to_string(), "military".to_string());
        alice_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let alice_instance = scheme.instantiate(&alice_bindings).unwrap();

        let mut bob_bindings = HashMap::new();
        bob_bindings.insert("expert".to_string(), "bob".to_string());
        bob_bindings.insert("domain".to_string(), "logistics".to_string());
        bob_bindings.insert("claim".to_string(), "abandon_east".to_string());
        let bob_instance = scheme.instantiate(&bob_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let mut alice_af_bindings = alice_bindings.clone();
        alice_af_bindings.insert("self".to_string(), "alice".to_string());
        let alice_id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &alice_af_bindings,
            alice_instance,
        );
        let bob_id = state.add_scheme_instance("bob", bob_instance);
        state.add_weighted_attack(&bob_id, &alice_id, 0.5).unwrap();

        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance(
            "argue_fortify_east",
            "alice",
            "alice",
            "military",
            "fortify_east",
        );
        assert!(!eval.evaluate("bob", &action), "bob should reject alice's claim");
    }

    #[test]
    fn default_permissive_on_missing_self_binding() {
        let state = EncounterArgumentationState::new(default_catalog());
        let eval = StateAcceptanceEval::new(&state);
        let spec = AffordanceSpec {
            name: "anon".to_string(),
            domain: "x".to_string(),
            bindings: vec![],
            considerations: Vec::new(),
            effects_on_accept: Vec::new(),
            effects_on_reject: Vec::new(),
            drive_alignment: Vec::new(),
        };
        let entry = CatalogEntry { spec, precondition: String::new() };
        let action = ScoredAffordance {
            entry,
            score: 1.0,
            bindings: HashMap::new(),
        };
        assert!(eval.evaluate("anyone", &action));
        let errors = state.drain_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            crate::error::Error::MissingProposerBinding { affordance_name }
                if affordance_name == "anon"
        ));
    }
}
