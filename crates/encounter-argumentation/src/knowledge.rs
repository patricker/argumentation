//! Trait for providing per-character argumentation capabilities.

use std::collections::HashMap;

/// A character's position in an argument: which scheme they invoke and with what bindings.
#[derive(Debug, Clone)]
pub struct ArgumentPosition {
    /// Snake-case scheme key (e.g., "argument_from_expert_opinion").
    pub scheme_key: String,
    /// Slot bindings for the scheme (e.g., {"expert": "alice", "domain": "military"}).
    pub bindings: HashMap<String, String>,
    /// Relative preference weight. Higher = stronger conviction. Range: [0.0, 1.0].
    pub preference_weight: f64,
}

/// Provides per-character argumentation capabilities.
pub trait ArgumentKnowledge: Send + Sync {
    /// What arguments can `actor` make in support of performing `action`?
    fn arguments_for_action(
        &self,
        actor: &str,
        action_name: &str,
        action_bindings: &HashMap<String, String>,
    ) -> Vec<ArgumentPosition>;

    /// What counter-arguments can `actor` make against `action`?
    fn counter_arguments(
        &self,
        actor: &str,
        action_name: &str,
        proposer_arguments: &[ArgumentPosition],
    ) -> Vec<ArgumentPosition>;
}

/// Test helper: returns pre-configured argument positions.
#[derive(Debug, Default)]
pub struct StaticKnowledge {
    entries: Vec<StaticEntry>,
}

#[derive(Debug)]
struct StaticEntry {
    actor: String,
    action: String,
    is_counter: bool,
    positions: Vec<ArgumentPosition>,
}

impl StaticKnowledge {
    /// Create an empty knowledge base.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register arguments for an actor performing an action.
    pub fn add_arguments(&mut self, actor: &str, action: &str, positions: Vec<ArgumentPosition>) {
        self.entries.push(StaticEntry {
            actor: actor.into(),
            action: action.into(),
            is_counter: false,
            positions,
        });
    }

    /// Register counter-arguments for an actor against an action.
    pub fn add_counter_arguments(
        &mut self,
        actor: &str,
        action: &str,
        positions: Vec<ArgumentPosition>,
    ) {
        self.entries.push(StaticEntry {
            actor: actor.into(),
            action: action.into(),
            is_counter: true,
            positions,
        });
    }
}

impl ArgumentKnowledge for StaticKnowledge {
    fn arguments_for_action(
        &self,
        actor: &str,
        action_name: &str,
        _action_bindings: &HashMap<String, String>,
    ) -> Vec<ArgumentPosition> {
        self.entries
            .iter()
            .filter(|e| e.actor == actor && e.action == action_name && !e.is_counter)
            .flat_map(|e| e.positions.clone())
            .collect()
    }

    fn counter_arguments(
        &self,
        actor: &str,
        action_name: &str,
        _proposer_arguments: &[ArgumentPosition],
    ) -> Vec<ArgumentPosition> {
        self.entries
            .iter()
            .filter(|e| e.actor == actor && e.action == action_name && e.is_counter)
            .flat_map(|e| e.positions.clone())
            .collect()
    }
}
