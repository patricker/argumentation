//! Complete extensions: admissible sets that contain every argument they defend.
//!
//! Enumerated by subset iteration. Exponential in the number of arguments;
//! scales to ~20 arguments before becoming impractical.

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// Upper bound on the number of arguments we enumerate via subset search.
pub(crate) const ENUMERATION_LIMIT: usize = 30;

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Enumerate all complete extensions via subset search.
    ///
    /// **Performance:** `O(2^n)` in the number of arguments. Frameworks with
    /// more than [`ENUMERATION_LIMIT`] arguments are rejected with
    /// [`crate::Error::TooLarge`]; use a SAT-based semantics entry point
    /// (future work) for larger instances.
    pub fn complete_extensions(&self) -> Result<Vec<HashSet<A>>, crate::Error> {
        let args: Vec<A> = {
            let mut v: Vec<A> = self.arguments().cloned().collect();
            v.sort();
            v
        };
        let n = args.len();
        if n > ENUMERATION_LIMIT {
            return Err(crate::Error::TooLarge {
                arguments: n,
                limit: ENUMERATION_LIMIT,
            });
        }
        let mut results = Vec::new();
        for bits in 0u64..(1u64 << n) {
            let s: HashSet<A> = (0..n)
                .filter(|i| bits & (1u64 << i) != 0)
                .map(|i| args[i].clone())
                .collect();
            if !self.is_admissible(&s) {
                continue;
            }
            // Check that s contains every argument it defends (equal to F(s)).
            let defended: HashSet<A> = self
                .arguments()
                .filter(|a| self.defends(&s, *a))
                .cloned()
                .collect();
            if defended == s {
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
    fn complete_of_figure_1_is_singleton_ac() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let ce = af.complete_extensions().unwrap();
        assert_eq!(ce.len(), 1);
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert!(ce.contains(&expected));
    }

    #[test]
    fn complete_of_mutual_attack_is_three_extensions() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"a").unwrap();
        let ce = af.complete_extensions().unwrap();
        assert_eq!(ce.len(), 3);
        let empty: HashSet<&str> = HashSet::new();
        let a_only: HashSet<&str> = ["a"].into_iter().collect();
        let b_only: HashSet<&str> = ["b"].into_iter().collect();
        assert!(ce.contains(&empty));
        assert!(ce.contains(&a_only));
        assert!(ce.contains(&b_only));
    }

    #[test]
    fn complete_always_contains_grounded() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let grounded = af.grounded_extension();
        let complete = af.complete_extensions().unwrap();
        for ext in &complete {
            assert!(grounded.iter().all(|g| ext.contains(g)));
        }
    }

    #[test]
    fn complete_rejects_frameworks_above_limit() {
        let mut af = ArgumentationFramework::new();
        for i in 0..(ENUMERATION_LIMIT + 1) {
            af.add_argument(format!("a{}", i));
        }
        let result = af.complete_extensions();
        assert!(matches!(result, Err(crate::Error::TooLarge { .. })));
    }
}
