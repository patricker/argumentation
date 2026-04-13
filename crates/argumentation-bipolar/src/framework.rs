//! `BipolarFramework<A>`: arguments, attacks, and supports.
//!
//! Stores the framework as two independent directed edge sets over the
//! same node set. The node set is the union of all distinct arguments
//! introduced via [`BipolarFramework::add_argument`] or as an endpoint
//! of a call to [`BipolarFramework::add_attack`] or
//! [`BipolarFramework::add_support`].

use crate::error::Error;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A bipolar argumentation framework over argument type `A`.
///
/// The type is generic over `A` to match the core crate's convention —
/// `A` can be `String`, `&'static str`, a custom `ArgumentId` newtype, etc.
#[derive(Debug, Clone)]
pub struct BipolarFramework<A: Clone + Eq + Hash> {
    arguments: HashSet<A>,
    attacks: HashSet<(A, A)>,
    supports: HashSet<(A, A)>,
}

impl<A: Clone + Eq + Hash> BipolarFramework<A> {
    /// Create an empty framework.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashSet::new(),
            attacks: HashSet::new(),
            supports: HashSet::new(),
        }
    }

    /// Add an argument. Adding an argument that already exists is a no-op.
    pub fn add_argument(&mut self, a: A) {
        self.arguments.insert(a);
    }

    /// Add an attack `attacker → target`. Both arguments are implicitly
    /// added to the framework if not already present. Adding the same
    /// attack twice is a no-op.
    pub fn add_attack(&mut self, attacker: A, target: A) {
        self.arguments.insert(attacker.clone());
        self.arguments.insert(target.clone());
        self.attacks.insert((attacker, target));
    }

    /// Add a support `supporter → supported`. Both arguments are
    /// implicitly added. Adding the same support twice is a no-op.
    /// Returns [`Error::IllegalSelfSupport`] if `supporter == supported`
    /// — an argument cannot be its own necessary supporter.
    pub fn add_support(&mut self, supporter: A, supported: A) -> Result<(), Error>
    where
        A: std::fmt::Debug,
    {
        if supporter == supported {
            return Err(Error::IllegalSelfSupport(format!("{:?}", supporter)));
        }
        self.arguments.insert(supporter.clone());
        self.arguments.insert(supported.clone());
        self.supports.insert((supporter, supported));
        Ok(())
    }

    /// Remove a support edge. Returns true if the edge was present.
    /// Used by consumers modelling betrayal (a support edge is retracted).
    /// Does NOT remove the endpoint arguments from the framework.
    pub fn remove_support(&mut self, supporter: &A, supported: &A) -> bool {
        self.supports
            .remove(&(supporter.clone(), supported.clone()))
    }

    /// Remove an attack edge. Returns true if the edge was present.
    pub fn remove_attack(&mut self, attacker: &A, target: &A) -> bool {
        self.attacks.remove(&(attacker.clone(), target.clone()))
    }

    /// Iterate over all arguments in the framework.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.arguments.iter()
    }

    /// Iterate over all direct attack edges.
    pub fn attacks(&self) -> impl Iterator<Item = (&A, &A)> {
        self.attacks.iter().map(|(a, b)| (a, b))
    }

    /// Iterate over all direct support edges.
    pub fn supports(&self) -> impl Iterator<Item = (&A, &A)> {
        self.supports.iter().map(|(a, b)| (a, b))
    }

    /// Number of arguments in the framework.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    /// Whether the framework has zero arguments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// Direct attackers of `a` (arguments `X` such that `X → a` in the
    /// attack edge set). Does NOT include derived attackers — see
    /// [`crate::derived`] for the closure.
    pub fn direct_attackers(&self, a: &A) -> Vec<&A> {
        self.attacks
            .iter()
            .filter(|(_, target)| target == a)
            .map(|(attacker, _)| attacker)
            .collect()
    }

    /// Direct supporters of `a` (arguments `X` such that `X → a` in the
    /// support edge set).
    pub fn direct_supporters(&self, a: &A) -> Vec<&A> {
        self.supports
            .iter()
            .filter(|(_, target)| target == a)
            .map(|(supporter, _)| supporter)
            .collect()
    }

    /// Map of each argument to its direct necessary supporters.
    ///
    /// Used by the support-closure filter in [`crate::semantics`] and
    /// by [`crate::queries`] for transitive queries.
    pub fn supporter_map(&self) -> HashMap<&A, HashSet<&A>> {
        let mut map: HashMap<&A, HashSet<&A>> =
            self.arguments.iter().map(|a| (a, HashSet::new())).collect();
        for (supporter, supported) in &self.supports {
            map.entry(supported).or_default().insert(supporter);
        }
        map
    }
}

impl<A: Clone + Eq + Hash> Default for BipolarFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

// Compile-time guarantee that the canonical owned-string bipolar
// framework is thread-safe, matching the core crate's guarantee.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<BipolarFramework<String>>();
    assert_sync::<BipolarFramework<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_framework_has_no_arguments() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        assert!(bf.is_empty());
        assert_eq!(bf.len(), 0);
    }

    #[test]
    fn add_argument_is_idempotent() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("a");
        assert_eq!(bf.len(), 1);
    }

    #[test]
    fn add_attack_registers_both_endpoints() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        assert_eq!(bf.len(), 2);
        assert_eq!(bf.direct_attackers(&"b"), vec![&"a"]);
        assert!(bf.direct_attackers(&"a").is_empty());
    }

    #[test]
    fn add_support_registers_both_endpoints() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        assert_eq!(bf.len(), 2);
        assert_eq!(bf.direct_supporters(&"b"), vec![&"a"]);
    }

    #[test]
    fn self_support_is_rejected() {
        let mut bf: BipolarFramework<&str> = BipolarFramework::new();
        let err = bf.add_support("a", "a").unwrap_err();
        assert!(matches!(err, Error::IllegalSelfSupport(_)));
    }

    #[test]
    fn remove_support_returns_whether_edge_was_present() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        assert!(bf.remove_support(&"a", &"b"));
        assert!(!bf.remove_support(&"a", &"b"));
        // Arguments stay in the framework even after the edge is removed.
        assert_eq!(bf.len(), 2);
    }

    #[test]
    fn supporter_map_includes_all_arguments_even_unsupported_ones() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_support("b", "c").unwrap();
        let map = bf.supporter_map();
        assert_eq!(map.len(), 3);
        assert!(map[&"a"].is_empty());
        assert!(map[&"b"].is_empty());
        assert_eq!(map[&"c"].len(), 1);
        assert!(map[&"c"].contains(&"b"));
    }
}
