//! `WeightedFramework<A>`: arguments and weighted attack edges.

use crate::error::Error;
use crate::types::{AttackWeight, WeightedAttack};
use std::collections::HashSet;
use std::hash::Hash;

/// A weighted argumentation framework: a set of arguments and a list
/// of weighted attack edges between them.
///
/// Attack weights are validated at insert time via
/// [`AttackWeight::new`]. Duplicate attack edges (same attacker and
/// target) are NOT deduplicated — each `add_weighted_attack` call
/// appends a new edge, even if one already exists. This matches Dunne
/// 2011, which allows multigraphs with distinct-weight parallel edges.
/// Consumers who want deduplication should call
/// [`WeightedFramework::collapse_duplicate_attacks`].
#[derive(Debug, Clone)]
pub struct WeightedFramework<A: Clone + Eq + Hash> {
    arguments: HashSet<A>,
    attacks: Vec<WeightedAttack<A>>,
}

impl<A: Clone + Eq + Hash> WeightedFramework<A> {
    /// Create an empty framework.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashSet::new(),
            attacks: Vec::new(),
        }
    }

    /// Add an argument. Adding an argument that already exists is a no-op.
    pub fn add_argument(&mut self, a: A) {
        self.arguments.insert(a);
    }

    /// Add a weighted attack. Both endpoints are implicitly added to
    /// the framework. Returns [`Error::InvalidWeight`] if the weight
    /// fails validation. Parallel edges with the same endpoints but
    /// different weights are permitted.
    pub fn add_weighted_attack(
        &mut self,
        attacker: A,
        target: A,
        weight: f64,
    ) -> Result<(), Error> {
        let w = AttackWeight::new(weight)?;
        self.arguments.insert(attacker.clone());
        self.arguments.insert(target.clone());
        self.attacks.push(WeightedAttack {
            attacker,
            target,
            weight: w,
        });
        Ok(())
    }

    /// Collapse parallel edges: for each `(attacker, target)` pair,
    /// keep only one edge whose weight is the sum of all parallel
    /// edges' weights. This is one valid aggregation strategy (sum);
    /// Dunne 2011 does not prescribe one. Consumers who want a
    /// different aggregation (max, min, mean) should implement it
    /// externally.
    ///
    /// Returns [`Error::InvalidWeight`] if the summed weight overflows
    /// to infinity (e.g., two edges each with weight `f64::MAX`).
    pub fn collapse_duplicate_attacks(&mut self) -> Result<(), Error> {
        use std::collections::HashMap;
        let mut map: HashMap<(A, A), f64> = HashMap::new();
        for atk in self.attacks.drain(..) {
            let key = (atk.attacker, atk.target);
            *map.entry(key).or_insert(0.0) += atk.weight.value();
        }
        let mut new_attacks = Vec::with_capacity(map.len());
        for ((attacker, target), weight) in map {
            let w = AttackWeight::new(weight)?;
            new_attacks.push(WeightedAttack { attacker, target, weight: w });
        }
        self.attacks = new_attacks;
        Ok(())
    }

    /// Iterate over all arguments.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.arguments.iter()
    }

    /// Iterate over all weighted attacks.
    pub fn attacks(&self) -> impl Iterator<Item = &WeightedAttack<A>> {
        self.attacks.iter()
    }

    /// Number of arguments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    /// Whether the framework has zero arguments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// Number of attack edges (counting parallel edges separately).
    #[must_use]
    pub fn attack_count(&self) -> usize {
        self.attacks.len()
    }

    /// Return all distinct weight values present in the framework,
    /// sorted ascending. Used by the threshold-sweep API: flip points
    /// can only occur at cumulative-sum values of these weights.
    #[must_use]
    pub fn sorted_weights(&self) -> Vec<f64> {
        let mut ws: Vec<f64> = self.attacks.iter().map(|a| a.weight.value()).collect();
        ws.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ws
    }
}

impl<A: Clone + Eq + Hash> Default for WeightedFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

// Compile-time thread-safety guarantee matching the core crate.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<WeightedFramework<String>>();
    assert_sync::<WeightedFramework<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_framework_has_no_arguments() {
        let wf: WeightedFramework<&str> = WeightedFramework::new();
        assert!(wf.is_empty());
        assert_eq!(wf.len(), 0);
        assert_eq!(wf.attack_count(), 0);
    }

    #[test]
    fn add_weighted_attack_registers_both_endpoints() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        assert_eq!(wf.len(), 2);
        assert_eq!(wf.attack_count(), 1);
    }

    #[test]
    fn add_weighted_attack_rejects_invalid_weight() {
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        assert!(wf.add_weighted_attack("a", "b", -0.1).is_err());
        assert!(wf.add_weighted_attack("a", "b", f64::NAN).is_err());
    }

    #[test]
    fn parallel_edges_are_preserved_before_collapse() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("a", "b", 0.4).unwrap();
        assert_eq!(wf.attack_count(), 2);
    }

    #[test]
    fn collapse_duplicate_attacks_sums_weights() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("a", "b", 0.4).unwrap();
        wf.add_weighted_attack("a", "c", 0.5).unwrap();
        wf.collapse_duplicate_attacks().unwrap();
        assert_eq!(wf.attack_count(), 2);
        // Find the (a, b) edge and verify its weight is 0.7.
        let ab = wf
            .attacks()
            .find(|a| a.attacker == "a" && a.target == "b")
            .unwrap();
        assert!((ab.weight.value() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn collapse_duplicate_attacks_returns_err_on_weight_overflow() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", f64::MAX).unwrap();
        wf.add_weighted_attack("a", "b", f64::MAX).unwrap();
        let err = wf.collapse_duplicate_attacks().unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }

    #[test]
    fn sorted_weights_returns_ascending() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("a", "c", 0.2).unwrap();
        wf.add_weighted_attack("a", "d", 0.8).unwrap();
        let ws = wf.sorted_weights();
        assert_eq!(ws, vec![0.2, 0.5, 0.8]);
    }
}
