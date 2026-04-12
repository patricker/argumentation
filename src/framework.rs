//! Abstract argumentation framework: arguments and attack relations.

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::hash::Hash;

/// A Dung-style argumentation framework over argument type `A`.
#[derive(Debug, Clone)]
pub struct ArgumentationFramework<A: Clone + Eq + Hash> {
    graph: DiGraph<A, ()>,
    index: HashMap<A, NodeIndex>,
}

impl<A: Clone + Eq + Hash> ArgumentationFramework<A> {
    /// Create an empty framework.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    /// Add an argument. Adding an argument that already exists is a no-op.
    pub fn add_argument(&mut self, a: A) {
        if !self.index.contains_key(&a) {
            let idx = self.graph.add_node(a.clone());
            self.index.insert(a, idx);
        }
    }

    /// Add an attack from `attacker` to `target`. Both must already be present.
    ///
    /// Adding the same attack edge twice is a no-op: `ArgumentationFramework`
    /// does not model weighted or multi-edge attacks.
    ///
    /// The `Debug` bound on `A` lets the error message name the offending
    /// argument when one is missing. Arguments are taken by reference to avoid
    /// cloning owned types like `String` or `Arc<T>` on every call.
    pub fn add_attack(&mut self, attacker: &A, target: &A) -> Result<(), crate::Error>
    where
        A: std::fmt::Debug,
    {
        let a = *self
            .index
            .get(attacker)
            .ok_or_else(|| crate::Error::ArgumentNotFound(format!("{:?}", attacker)))?;
        let t = *self
            .index
            .get(target)
            .ok_or_else(|| crate::Error::ArgumentNotFound(format!("{:?}", target)))?;
        // Dedupe: petgraph's DiGraph is a multigraph; we want at most one
        // edge per (attacker, target) pair.
        if self.graph.find_edge(a, t).is_none() {
            self.graph.add_edge(a, t, ());
        }
        Ok(())
    }

    /// Iterate over all arguments in the framework.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.graph.node_weights()
    }

    /// Number of arguments currently in the framework.
    ///
    /// Constant-time. Useful for size-gating exponential enumerators
    /// against [`crate::ENUMERATION_LIMIT`] before calling them.
    #[must_use]
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// Whether the framework contains zero arguments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Return the arguments that attack `a`.
    pub fn attackers(&self, a: &A) -> Vec<&A> {
        let Some(&idx) = self.index.get(a) else {
            return Vec::new();
        };
        self.graph
            .neighbors_directed(idx, Direction::Incoming)
            .map(|n| &self.graph[n])
            .collect()
    }

    /// Return the arguments attacked by `a`.
    pub fn attacked_by(&self, a: &A) -> Vec<&A> {
        let Some(&idx) = self.index.get(a) else {
            return Vec::new();
        };
        self.graph
            .neighbors_directed(idx, Direction::Outgoing)
            .map(|n| &self.graph[n])
            .collect()
    }
}

impl<A: Clone + Eq + Hash> Default for ArgumentationFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

// Compile-time guarantee: the canonical owned-string framework is
// thread-safe. Consumers (e.g. encounter's multi-character resolution)
// rely on being able to ship a framework across threads.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<ArgumentationFramework<String>>();
    assert_sync::<ArgumentationFramework<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_framework_has_no_arguments() {
        let af: ArgumentationFramework<&str> = ArgumentationFramework::new();
        assert_eq!(af.arguments().count(), 0);
    }

    #[test]
    fn add_argument_adds_one() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        assert_eq!(af.arguments().count(), 1);
    }

    #[test]
    fn add_attack_creates_edge() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        assert_eq!(af.attackers(&"b").len(), 1);
        assert!(af.attackers(&"b").iter().any(|x| **x == "a"));
    }

    #[test]
    fn add_attack_is_idempotent() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"a", &"b").unwrap();
        // Duplicate attack must not create a second edge.
        assert_eq!(af.attackers(&"b").len(), 1);
    }

    #[test]
    fn add_attack_on_missing_argument_reports_which() {
        let mut af: ArgumentationFramework<&str> = ArgumentationFramework::new();
        af.add_argument("a");
        let err = af.add_attack(&"a", &"missing").unwrap_err();
        // Error message must include the offending argument, not just the type.
        let msg = err.to_string();
        assert!(
            msg.contains("missing"),
            "error should mention the missing argument, got: {}",
            msg
        );
    }

    #[test]
    fn self_attack_is_allowed() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_attack(&"a", &"a").unwrap();
        assert_eq!(af.attackers(&"a").len(), 1);
    }

    #[test]
    fn add_argument_is_idempotent() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("a");
        assert_eq!(af.arguments().count(), 1);
    }
}
