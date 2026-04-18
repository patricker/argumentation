//! `WeightedBipolarFramework<A>`: arguments, weighted attacks, weighted supports.

use crate::error::Error;
use crate::types::{AttackWeight, WeightedAttack, WeightedSupport};
use std::collections::HashSet;
use std::hash::Hash;

/// A weighted bipolar argumentation framework.
///
/// Stores arguments and two lists of weighted directed edges — attacks
/// and supports — with non-negative finite weights.
#[derive(Debug, Clone)]
pub struct WeightedBipolarFramework<A: Clone + Eq + Hash> {
    arguments: HashSet<A>,
    attacks: Vec<WeightedAttack<A>>,
    supports: Vec<WeightedSupport<A>>,
}

impl<A: Clone + Eq + Hash> Default for WeightedBipolarFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Clone + Eq + Hash> WeightedBipolarFramework<A> {
    /// Create an empty framework.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashSet::new(),
            attacks: Vec::new(),
            supports: Vec::new(),
        }
    }

    /// Add an argument. Adding an existing argument is a no-op.
    pub fn add_argument(&mut self, a: A) {
        self.arguments.insert(a);
    }

    /// Add a weighted attack. Both endpoints are implicitly added.
    pub fn add_weighted_attack(
        &mut self,
        attacker: A,
        target: A,
        weight: f64,
    ) -> Result<(), Error> {
        let w = AttackWeight::new(weight).map_err(|_| Error::InvalidWeight { weight })?;
        self.arguments.insert(attacker.clone());
        self.arguments.insert(target.clone());
        self.attacks.push(WeightedAttack {
            attacker,
            target,
            weight: w,
        });
        Ok(())
    }

    /// Add a weighted support. Both endpoints are implicitly added.
    /// Returns [`Error::IllegalSelfSupport`] if `supporter == supported`.
    pub fn add_weighted_support(
        &mut self,
        supporter: A,
        supported: A,
        weight: f64,
    ) -> Result<(), Error> {
        let support = WeightedSupport::new(supporter.clone(), supported.clone(), weight)?;
        self.arguments.insert(supporter);
        self.arguments.insert(supported);
        self.supports.push(support);
        Ok(())
    }

    /// Iterate arguments.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.arguments.iter()
    }

    /// Iterate weighted attacks.
    pub fn attacks(&self) -> impl Iterator<Item = &WeightedAttack<A>> {
        self.attacks.iter()
    }

    /// Iterate weighted supports.
    pub fn supports(&self) -> impl Iterator<Item = &WeightedSupport<A>> {
        self.supports.iter()
    }

    /// Total edge count (attacks + supports).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.attacks.len() + self.supports.len()
    }

    /// Argument count.
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_framework_is_empty() {
        let wbf: WeightedBipolarFramework<&str> = WeightedBipolarFramework::new();
        assert_eq!(wbf.argument_count(), 0);
        assert_eq!(wbf.edge_count(), 0);
    }

    #[test]
    fn adding_weighted_attack_adds_endpoints() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.5).unwrap();
        assert_eq!(wbf.argument_count(), 2);
        assert_eq!(wbf.edge_count(), 1);
    }

    #[test]
    fn adding_weighted_support_adds_endpoints() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_support("a", "b", 0.5).unwrap();
        assert_eq!(wbf.argument_count(), 2);
        assert_eq!(wbf.edge_count(), 1);
    }

    #[test]
    fn invalid_attack_weight_rejected() {
        let mut wbf = WeightedBipolarFramework::new();
        let err = wbf.add_weighted_attack("a", "b", -0.5).unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }

    #[test]
    fn self_support_rejected() {
        let mut wbf = WeightedBipolarFramework::new();
        let err = wbf.add_weighted_support("a", "a", 0.5).unwrap_err();
        assert!(matches!(err, Error::IllegalSelfSupport));
    }

    #[test]
    fn add_argument_idempotent() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_argument("a");
        wbf.add_argument("a");
        assert_eq!(wbf.argument_count(), 1);
    }
}
