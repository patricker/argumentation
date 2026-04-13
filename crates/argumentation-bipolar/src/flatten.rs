//! Flattening: convert a [`BipolarFramework`] into an equivalent
//! [`argumentation::ArgumentationFramework`] whose attack relation is
//! the closed attack set from [`crate::derived::closed_attacks`].
//!
//! The flattened framework has the same node set as the bipolar
//! framework. Every direct attack and every derived attack (supported,
//! secondary, mediated) appears as a single edge in the flattened
//! framework's attack relation. Support edges are NOT represented in
//! the flattened framework — they are handled at the semantics layer
//! via the support-closure filter.
//!
//! This is the abstraction that lets the rest of the crate reuse the
//! core Dung semantics without re-implementing fixed-point equations.

use crate::derived::closed_attacks;
use crate::framework::BipolarFramework;
use argumentation::ArgumentationFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Build a [`argumentation::ArgumentationFramework`] from a
/// [`BipolarFramework`] whose attack relation is the closed attack set.
///
/// Propagates [`argumentation::Error`] from `add_attack` calls, but in
/// practice this only fires if the argument universe is inconsistent
/// (an edge references an argument that wasn't registered), which
/// cannot happen here because `closed_attacks` only produces edges
/// between arguments already in the framework.
pub fn flatten<A>(
    framework: &BipolarFramework<A>,
) -> Result<ArgumentationFramework<A>, argumentation::Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let mut af = ArgumentationFramework::new();
    for arg in framework.arguments() {
        af.add_argument(arg.clone());
    }
    for (attacker, target) in closed_attacks(framework) {
        af.add_attack(&attacker, &target)?;
    }
    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bipolar_flattens_to_empty_dung() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 0);
    }

    #[test]
    fn direct_attack_survives_flattening() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 2);
        assert_eq!(af.attackers(&"b").len(), 1);
    }

    #[test]
    fn supported_attack_becomes_direct_in_flat_framework() {
        // a supports x, x attacks b. Flattened: a → b and x → b.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let af = flatten(&bf).unwrap();
        let attackers_of_b: Vec<&&str> = af.attackers(&"b").into_iter().collect();
        assert_eq!(attackers_of_b.len(), 2);
        assert!(attackers_of_b.contains(&&"a"));
        assert!(attackers_of_b.contains(&&"x"));
    }

    #[test]
    fn unrelated_arguments_appear_in_flattened_framework() {
        // Arguments with no edges still appear as isolated nodes.
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 2);
        assert!(af.attackers(&"a").is_empty());
        assert!(af.attackers(&"b").is_empty());
    }
}
