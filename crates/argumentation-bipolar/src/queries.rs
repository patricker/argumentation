//! Transitive queries over a bipolar framework.
//!
//! - [`transitive_supporters`] — all arguments that directly or
//!   indirectly support `a` via the support graph.
//! - [`transitive_attackers`] — all arguments that attack `a` under the
//!   closed attack relation (direct + derived).
//! - [`coalitioned_with`] — the members of `a`'s coalition per
//!   [`crate::coalition::detect_coalitions`].

use crate::coalition::detect_coalitions;
use crate::derived::closed_attacks;
use crate::framework::BipolarFramework;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

/// All arguments that directly or transitively support `a` in the
/// support graph. Does not include `a` itself.
#[must_use]
pub fn transitive_supporters<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    let mut visited: HashSet<A> = HashSet::new();
    let mut frontier: VecDeque<A> = VecDeque::new();
    frontier.push_back(a.clone());

    while let Some(current) = frontier.pop_front() {
        for (supporter, supported) in framework.supports() {
            if *supported == current && visited.insert(supporter.clone()) {
                frontier.push_back(supporter.clone());
            }
        }
    }
    visited.remove(a);
    visited
}

/// All arguments that attack `a` under the closed attack relation
/// (direct attacks plus derived attacks from support closure).
#[must_use]
pub fn transitive_attackers<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    closed_attacks(framework)
        .into_iter()
        .filter_map(|(att, tgt)| if tgt == *a { Some(att) } else { None })
        .collect()
}

/// The members of `a`'s coalition, excluding `a` itself. If `a` is in
/// a singleton coalition, returns an empty set.
#[must_use]
pub fn coalitioned_with<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    let coalitions = detect_coalitions(framework);
    for coalition in coalitions {
        if coalition.members.contains(a) {
            return coalition.members.into_iter().filter(|m| m != a).collect();
        }
    }
    HashSet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitive_supporters_walks_support_chain() {
        // a supports b, b supports c. Transitive supporters of c: {a, b}.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        let sups = transitive_supporters(&bf, &"c");
        assert_eq!(sups.len(), 2);
        assert!(sups.contains(&"a"));
        assert!(sups.contains(&"b"));
    }

    #[test]
    fn transitive_attackers_includes_derived_edges() {
        // a supports x, x attacks b ⇒ a attacks b (derived).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let atts = transitive_attackers(&bf, &"b");
        assert!(atts.contains(&"a"), "derived attacker should be present");
        assert!(atts.contains(&"x"));
    }

    #[test]
    fn coalitioned_with_returns_siblings() {
        let mut bf = BipolarFramework::new();
        bf.add_support("alice", "bob").unwrap();
        bf.add_support("bob", "alice").unwrap();
        let allies = coalitioned_with(&bf, &"alice");
        assert_eq!(allies.len(), 1);
        assert!(allies.contains(&"bob"));
    }

    #[test]
    fn coalitioned_with_returns_empty_for_singleton() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("alice");
        let allies = coalitioned_with(&bf, &"alice");
        assert!(allies.is_empty());
    }
}
