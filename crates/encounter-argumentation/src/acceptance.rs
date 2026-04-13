//! Argument-backed acceptance evaluation.

use crate::knowledge::{ArgumentKnowledge, ArgumentPosition};
use crate::resolver::{ArgumentOutcome, resolve_argument};
use argumentation_schemes::CatalogRegistry;
use argumentation_schemes::instance::SchemeInstance;
use encounter::scoring::{AcceptanceEval, ScoredAffordance};

/// Acceptance evaluator that uses argumentation schemes to decide whether
/// a responder accepts a proposed action.
///
/// When the responder has no counter-arguments, the action is accepted.
/// When both sides have arguments, they are resolved via ASPIC+ extension
/// semantics, and the action is accepted if the proposer's arguments survive.
/// When ASPIC+ is inconclusive (non-conflicting conclusions), scheme strength
/// is compared as a fallback — both sides are scoped to the same action by the
/// `ArgumentKnowledge` trait, so the stronger authority basis wins.
pub struct ArgumentAcceptanceEval<K> {
    knowledge: K,
    registry: CatalogRegistry,
}

impl<K: ArgumentKnowledge> ArgumentAcceptanceEval<K> {
    /// Create a new evaluator with the given knowledge provider and scheme registry.
    pub fn new(knowledge: K, registry: CatalogRegistry) -> Self {
        Self {
            knowledge,
            registry,
        }
    }
}

impl<K: ArgumentKnowledge, P> AcceptanceEval<P> for ArgumentAcceptanceEval<K> {
    fn evaluate(&self, responder: &str, action: &ScoredAffordance<P>) -> bool {
        let action_name = &action.entry.spec.name;
        let proposer = action
            .bindings
            .get("self")
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let proposer_positions =
            self.knowledge
                .arguments_for_action(proposer, action_name, &action.bindings);

        if proposer_positions.is_empty() {
            return true; // No formal argument backing → accept by default
        }

        let responder_positions =
            self.knowledge
                .counter_arguments(responder, action_name, &proposer_positions);

        if responder_positions.is_empty() {
            return true;
        }

        let proposer_instances = instantiate_positions(&proposer_positions, &self.registry);
        let responder_instances = instantiate_positions(&responder_positions, &self.registry);

        let outcome = resolve_argument(&proposer_instances, &responder_instances, &self.registry);

        match outcome {
            ArgumentOutcome::ResponderWins { .. } => false,
            ArgumentOutcome::ProposerWins { .. } | ArgumentOutcome::Undecided => {
                // Strength fallback: both sides are scoped to the same action
                // by the ArgumentKnowledge trait, so comparing strength ranks
                // is always relevant — it's "how strong is your basis for
                // supporting/opposing THIS action?" not unrelated topics.
                // When ASPIC+ is inconclusive (non-conflicting conclusions),
                // the stronger authority basis wins.
                let proposer_rank = max_strength_rank(&proposer_positions, &self.registry);
                let responder_rank = max_strength_rank(&responder_positions, &self.registry);
                responder_rank <= proposer_rank
            }
        }
    }
}

/// Return the maximum strength rank across a set of argument positions.
fn max_strength_rank(positions: &[ArgumentPosition], registry: &CatalogRegistry) -> u8 {
    positions
        .iter()
        .filter_map(|pos| {
            registry
                .by_key(&pos.scheme_key)
                .map(|s| crate::strength_rank(s.metadata.strength))
        })
        .max()
        .unwrap_or(0)
}

fn instantiate_positions(
    positions: &[ArgumentPosition],
    registry: &CatalogRegistry,
) -> Vec<SchemeInstance> {
    positions
        .iter()
        .filter_map(|pos| {
            let scheme = registry.by_key(&pos.scheme_key)?;
            scheme.instantiate(&pos.bindings).ok()
        })
        .collect()
}
