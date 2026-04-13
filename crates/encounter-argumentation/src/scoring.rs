//! Scheme-strength action scoring.
//!
//! [`SchemeActionScorer`] wraps an inner [`ActionScorer`] and boosts scores
//! for actions that have scheme backing in the supplied [`ArgumentKnowledge`].
//! The boost scales with the scheme's [`SchemeStrength`]:
//!
//! | Strength | Multiplier |
//! |----------|-----------|
//! | Strong   | 1.00      |
//! | Moderate | 0.67      |
//! | Weak     | 0.33      |
//!
//! The best (highest) boost across all argument positions for an action is
//! applied; positions without a matching registry entry are skipped.

use crate::knowledge::{ArgumentKnowledge, ArgumentPosition};
use argumentation_schemes::CatalogRegistry;
use argumentation_schemes::types::SchemeStrength;
use encounter::affordance::CatalogEntry;
use encounter::scoring::{ActionScorer, ScoredAffordance};

/// An [`ActionScorer`] that boosts scores for scheme-backed actions.
///
/// Wraps an inner scorer and, for each scored affordance, looks up argument
/// positions from [`ArgumentKnowledge`]. If any position maps to a registered
/// scheme, the affordance score is increased by:
///
/// ```text
/// max_boost * strength_multiplier * preference_weight
/// ```
///
/// The highest boost across all positions wins. The returned list is sorted
/// descending by final score.
pub struct SchemeActionScorer<K, S> {
    knowledge: K,
    registry: CatalogRegistry,
    inner: S,
    max_boost: f64,
}

impl<K: ArgumentKnowledge, S> SchemeActionScorer<K, S> {
    /// Create a new `SchemeActionScorer`.
    ///
    /// # Parameters
    /// - `knowledge` – provides per-character argument positions.
    /// - `registry` – the scheme catalog used to resolve scheme strengths.
    /// - `inner` – the base scorer whose results are boosted.
    /// - `max_boost` – the maximum additive score increase for a Strong scheme
    ///   with `preference_weight = 1.0`.
    pub fn new(knowledge: K, registry: CatalogRegistry, inner: S, max_boost: f64) -> Self {
        Self {
            knowledge,
            registry,
            inner,
            max_boost,
        }
    }
}

impl<K, S, P> ActionScorer<P> for SchemeActionScorer<K, S>
where
    K: ArgumentKnowledge,
    S: ActionScorer<P>,
    P: Clone,
{
    /// Score available actions, boosting those with scheme backing.
    ///
    /// Delegates to the inner scorer, then for each result looks up argument
    /// positions from the knowledge base and applies the best scheme boost.
    /// Returns results sorted descending by final score.
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        let mut scored = self.inner.score_actions(actor, available, participants);
        for sa in &mut scored {
            let positions =
                self.knowledge
                    .arguments_for_action(actor, &sa.entry.spec.name, &sa.bindings);
            let boost = best_scheme_boost(&positions, &self.registry, self.max_boost);
            sa.score += boost;
        }
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

/// Returns the multiplier for a given scheme strength.
fn strength_multiplier(strength: SchemeStrength) -> f64 {
    match strength {
        SchemeStrength::Strong => 1.0,
        SchemeStrength::Moderate => 0.67,
        SchemeStrength::Weak => 0.33,
    }
}

/// Computes the best (highest) scheme boost across all argument positions.
///
/// Positions whose `scheme_key` does not match any entry in the registry are
/// silently skipped.
fn best_scheme_boost(
    positions: &[ArgumentPosition],
    registry: &CatalogRegistry,
    max_boost: f64,
) -> f64 {
    positions
        .iter()
        .filter_map(|pos| {
            let scheme = registry.by_key(&pos.scheme_key)?;
            let mult = strength_multiplier(scheme.metadata.strength);
            Some(max_boost * mult * pos.preference_weight)
        })
        .fold(0.0_f64, f64::max)
}
