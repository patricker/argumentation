//! Value-aware action scoring.
//!
//! [`ValueAwareScorer`] wraps an inner [`ActionScorer`] and adds a per-
//! affordance boost proportional to how strongly the actor's audience
//! prefers the values the affordance's backing scheme promotes.
//!
//! # Composition
//!
//! Designed to wrap [`crate::SchemeActionScorer`] (which itself wraps a
//! baseline scorer):
//!
//! ```ignore
//! let scorer = ValueAwareScorer::new(
//!     SchemeActionScorer::new(knowledge, registry, baseline, 0.3),
//!     state,
//!     0.2,
//! );
//! ```
//!
//! The two boosts compose additively: scheme-strength boost first, then
//! value-preference boost. Both are skipped silently when the actor has
//! no configured audience (i.e., no VAF dimension on this character).

use crate::state::EncounterArgumentationState;
use encounter::affordance::CatalogEntry;
use encounter::scoring::{ActionScorer, ScoredAffordance};

/// An [`ActionScorer`] that boosts affordances by audience-conditioned
/// value preference.
pub struct ValueAwareScorer<'a, S> {
    inner: S,
    state: &'a EncounterArgumentationState,
    max_boost: f64,
}

impl<'a, S> ValueAwareScorer<'a, S> {
    /// Construct a new value-aware scorer.
    ///
    /// # Parameters
    /// - `inner` — the scorer to wrap (typically `SchemeActionScorer`).
    /// - `state` — the encounter state holding per-actor audiences.
    /// - `max_boost` — additive boost when the actor's most-preferred
    ///   value is promoted by the affordance's scheme. Scaled linearly
    ///   downward by audience tier rank.
    pub fn new(inner: S, state: &'a EncounterArgumentationState, max_boost: f64) -> Self {
        Self {
            inner,
            state,
            max_boost,
        }
    }
}

impl<'a, S, P> ActionScorer<P> for ValueAwareScorer<'a, S>
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

        let Some(audience) = self.state.audience_for(actor) else {
            // No audience for this actor → behave as the inner scorer.
            return scored;
        };

        // Apply value boost per affordance.
        for sa in &mut scored {
            sa.score += value_boost_for_affordance(
                &sa.bindings,
                &audience,
                self.max_boost,
            );
        }
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

/// If the affordance's bindings include a `value` slot (as
/// `argument_from_values` schemes do), and that value is ranked in the
/// audience, returns a positive boost scaled by tier rank. Returns 0.0
/// otherwise.
///
/// Scaling: tier 0 (most preferred) → `max_boost`; deeper tiers scale
/// linearly down toward `max_boost / tier_count` at the bottom tier.
/// Unranked values get 0.
fn value_boost_for_affordance(
    bindings: &std::collections::HashMap<String, String>,
    audience: &argumentation_values::Audience,
    max_boost: f64,
) -> f64 {
    let Some(value_str) = bindings.get("value") else {
        return 0.0;
    };
    let value = argumentation_values::Value::new(value_str.clone());
    let tier_count = audience.tier_count();
    if tier_count == 0 {
        return 0.0;
    }
    let Some(rank) = audience.rank(&value) else {
        return 0.0;
    };
    let normalised = (tier_count - rank) as f64 / tier_count as f64;
    max_boost * normalised
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;
    use argumentation_values::{Audience, Value};
    use encounter::affordance::AffordanceSpec;
    use encounter::scoring::ScoredAffordance;
    use std::collections::HashMap;

    /// Test scorer that returns one fixed-score result for every affordance,
    /// with a value binding set to `value_promoted`.
    struct StubScorer {
        value_promoted: String,
    }

    impl<P: Clone> ActionScorer<P> for StubScorer {
        fn score_actions(
            &self,
            actor: &str,
            available: &[CatalogEntry<P>],
            _participants: &[String],
        ) -> Vec<ScoredAffordance<P>> {
            available
                .iter()
                .map(|entry| {
                    let mut bindings = HashMap::new();
                    bindings.insert("self".into(), actor.into());
                    bindings.insert("value".into(), self.value_promoted.clone());
                    ScoredAffordance {
                        entry: entry.clone(),
                        score: 1.0,
                        bindings,
                    }
                })
                .collect()
        }
    }

    fn dummy_entry() -> CatalogEntry<()> {
        CatalogEntry {
            spec: AffordanceSpec {
                name: "test_affordance".into(),
                domain: "test".into(),
                bindings: vec!["self".into(), "value".into()],
                considerations: Vec::new(),
                effects_on_accept: Vec::new(),
                effects_on_reject: Vec::new(),
                drive_alignment: Vec::new(),
            },
            precondition: (),
        }
    }

    #[test]
    fn no_audience_means_no_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        let inner = StubScorer { value_promoted: "honor".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        assert_eq!(scored.len(), 1);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boost_proportional_to_tier_position() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([Value::new("honor"), Value::new("safety")]),
        );
        let inner = StubScorer { value_promoted: "honor".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        // honor is at tier 0 of 2 tiers → normalised = (2-0)/2 = 1.0.
        // boost = 0.5 * 1.0 = 0.5; total = 1.0 + 0.5 = 1.5.
        assert!((scored[0].score - 1.5).abs() < 1e-9, "got {}", scored[0].score);
    }

    #[test]
    fn unranked_value_gets_no_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([Value::new("life")]),
        );
        let inner = StubScorer { value_promoted: "tradition".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn lower_tier_gets_smaller_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([
                Value::new("honor"),
                Value::new("safety"),
                Value::new("comfort"),
            ]),
        );
        let inner = StubScorer { value_promoted: "comfort".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        // comfort at tier 2 of 3 tiers → normalised = (3-2)/3 ≈ 0.333.
        // boost ≈ 0.5 * 0.333 ≈ 0.167; total ≈ 1.167.
        assert!((scored[0].score - 1.1666666666).abs() < 1e-3, "got {}", scored[0].score);
    }
}
