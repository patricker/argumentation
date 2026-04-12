//! Stable extensions: conflict-free sets that attack every argument outside them.

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Enumerate all stable extensions.
    ///
    /// A stable extension is a conflict-free set `S` such that every argument
    /// not in `S` is attacked by some member of `S`. Stable extensions may not
    /// exist (e.g., in odd cycles).
    ///
    /// Returns [`crate::Error::TooLarge`] for frameworks above the enumeration limit.
    pub fn stable_extensions(&self) -> Result<Vec<HashSet<A>>, crate::Error> {
        let args: Vec<A> = {
            let mut v: Vec<A> = self.arguments().cloned().collect();
            v.sort();
            v
        };
        let n = args.len();
        if n > super::complete::ENUMERATION_LIMIT {
            return Err(crate::Error::TooLarge {
                arguments: n,
                limit: super::complete::ENUMERATION_LIMIT,
            });
        }
        let mut results = Vec::new();
        for bits in 0u64..(1u64 << n) {
            let s: HashSet<A> = (0..n)
                .filter(|i| bits & (1u64 << i) != 0)
                .map(|i| args[i].clone())
                .collect();
            if !self.is_conflict_free(&s) {
                continue;
            }
            let attacks_all_outside = self
                .arguments()
                .filter(|a| !s.contains(*a))
                .all(|a| self.attackers(a).iter().any(|att| s.contains(*att)));
            if attacks_all_outside {
                results.push(s);
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_of_figure_1_is_singleton_ac() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let se = af.stable_extensions().unwrap();
        assert_eq!(se.len(), 1);
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert!(se.contains(&expected));
    }

    #[test]
    fn stable_of_odd_cycle_is_empty() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        af.add_attack(&"c", &"a").unwrap();
        let se = af.stable_extensions().unwrap();
        assert!(se.is_empty());
    }

    #[test]
    fn stable_subset_of_preferred() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"a").unwrap();
        let se = af.stable_extensions().unwrap();
        let pe = af.preferred_extensions().unwrap();
        for s in &se {
            assert!(pe.contains(s));
        }
    }
}
