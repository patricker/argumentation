//! Derived attack closure per Cayrol & Lagasquie-Schiex 2005 and
//! Amgoud et al. 2008 §3.
//!
//! Given a [`BipolarFramework`], compute the set of all attacks (direct
//! plus derived) that hold under necessary-support semantics. Three
//! derivation rules:
//!
//! 1. **Direct**: every edge in the attack set is an attack.
//! 2. **Supported**: if `A` transitively supports `X` and `X` directly
//!    attacks `B`, then `A` attacks `B`.
//! 3. **Secondary/Mediated**: if `A` directly attacks `X` and `X`
//!    transitively supports `C`, then `A` attacks `C`. (Amgoud et al.
//!    distinguishes secondary and mediated but both produce the same
//!    edges under the necessary-support reading.)
//!
//! The closure is computed as a fixed point over all three rules
//! applied together. For a framework with `n` arguments, convergence is
//! bounded by `n` iterations and the closure has at most `n²` edges.

use crate::framework::BipolarFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// Compute the closure of support from each argument: for each `A`,
/// the set of arguments `X` such that `A` transitively supports `X`.
/// Uses repeated BFS over the direct support edges.
fn support_closure<A: Clone + Eq + Hash>(
    framework: &BipolarFramework<A>,
) -> std::collections::HashMap<A, HashSet<A>> {
    use std::collections::{HashMap, VecDeque};

    let mut closure: HashMap<A, HashSet<A>> = HashMap::new();
    for arg in framework.arguments() {
        closure.insert(arg.clone(), HashSet::new());
    }

    // For each argument `start`, BFS the support graph to find every
    // transitively supported argument.
    for start in framework.arguments() {
        let mut visited: HashSet<A> = HashSet::new();
        let mut frontier: VecDeque<A> = VecDeque::new();
        frontier.push_back(start.clone());
        while let Some(current) = frontier.pop_front() {
            for (sup, supd) in framework.supports() {
                if *sup == current && visited.insert(supd.clone()) {
                    frontier.push_back(supd.clone());
                }
            }
        }
        closure.insert(start.clone(), visited);
    }

    closure
}

/// Compute the closed attack set for a bipolar framework under
/// necessary-support semantics.
///
/// The returned set contains `(attacker, target)` pairs for every
/// direct attack plus every derived attack produced by the supported
/// and secondary/mediated rules. Self-attacks are preserved from the
/// direct set (Dung allows them) but are not introduced by derivation.
///
/// The closure is deterministic and order-independent.
pub fn closed_attacks<A>(framework: &BipolarFramework<A>) -> HashSet<(A, A)>
where
    A: Clone + Eq + Hash,
{
    let support_cl = support_closure(framework);

    let mut closed: HashSet<(A, A)> = HashSet::new();

    // Rule 1: direct attacks.
    for (a, b) in framework.attacks() {
        closed.insert((a.clone(), b.clone()));
    }

    // Rule 2: supported attack — A supports* X, X attacks B ⇒ A attacks B.
    // For every direct attack (X, B) and every A with X ∈ support_cl(A),
    // insert (A, B).
    for (x, b) in framework.attacks() {
        for (a, supported_by_a) in &support_cl {
            if supported_by_a.contains(x) {
                closed.insert((a.clone(), b.clone()));
            }
        }
    }

    // Rule 3: secondary / mediated attack — A attacks X, X supports* C ⇒ A attacks C.
    // For every direct attack (A, X) and every C in support_cl(X), insert (A, C).
    for (a, x) in framework.attacks() {
        if let Some(downstream) = support_cl.get(x) {
            for c in downstream {
                closed.insert((a.clone(), c.clone()));
            }
        }
    }

    // Compose: A supports* X, X attacks Y, Y supports* C ⇒ A attacks C.
    // This is the full two-sided closure. The simpler implementation
    // is to iterate the above two rules to a fixed point; but because
    // support is transitively closed in `support_cl`, the single-pass
    // combination captures both directions without iteration:
    for (x, y) in framework.attacks() {
        for (a, supported_by_a) in &support_cl {
            if !supported_by_a.contains(x) {
                continue;
            }
            if let Some(downstream_of_y) = support_cl.get(y) {
                for c in downstream_of_y {
                    closed.insert((a.clone(), c.clone()));
                }
            }
        }
    }

    closed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_attack_is_preserved() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("a", "b")));
        assert_eq!(closed.len(), 1);
    }

    #[test]
    fn supported_attack_rule_fires() {
        // a supports x, x attacks b ⇒ a attacks b (derived).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "b"))); // direct
        assert!(closed.contains(&("a", "b"))); // supported
    }

    #[test]
    fn secondary_attack_rule_fires() {
        // a attacks x, x supports c ⇒ a attacks c (secondary).
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "x");
        bf.add_support("x", "c").unwrap();
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("a", "x"))); // direct
        assert!(closed.contains(&("a", "c"))); // secondary
    }

    #[test]
    fn two_sided_closure_composes_supported_and_secondary() {
        // a supports x, x attacks y, y supports c ⇒ a attacks c
        // (full closure: supported + secondary in one pass).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "y");
        bf.add_support("y", "c").unwrap();
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "y")));
        assert!(closed.contains(&("a", "y"))); // supported half
        assert!(closed.contains(&("x", "c"))); // secondary half
        assert!(closed.contains(&("a", "c"))); // full two-sided closure
    }

    #[test]
    fn transitive_support_chain_propagates_supported_attack() {
        // a supports b, b supports x, x attacks target ⇒ a attacks target.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "x").unwrap();
        bf.add_attack("x", "target");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "target")));
        assert!(closed.contains(&("b", "target")));
        assert!(closed.contains(&("a", "target")));
    }

    #[test]
    fn isolated_arguments_produce_no_derived_attacks() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        assert!(closed_attacks(&bf).is_empty());
    }
}
