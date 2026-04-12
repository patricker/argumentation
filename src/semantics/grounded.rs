//! Grounded extension: least fixed point of the characteristic function.
//!
//! Per Dung 1995 §3: the grounded extension is the unique least complete
//! extension. It is computed by iterating the characteristic function
//! `F(S) = {a | S defends a}` starting from the empty set until fixed.

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash> ArgumentationFramework<A> {
    /// Compute the grounded extension.
    ///
    /// Returns the unique least complete extension (always exists, always unique).
    pub fn grounded_extension(&self) -> HashSet<A> {
        let mut current: HashSet<A> = HashSet::new();
        loop {
            let next: HashSet<A> = self
                .arguments()
                .filter(|a| self.defends(&current, *a))
                .cloned()
                .collect();
            if next == current {
                return current;
            }
            current = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_of_empty_is_empty() {
        let af: ArgumentationFramework<&str> = ArgumentationFramework::new();
        assert_eq!(af.grounded_extension(), HashSet::new());
    }

    #[test]
    fn grounded_includes_unattacked_arguments() {
        // a is unattacked -> a is in grounded
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        let g = af.grounded_extension();
        let expected: HashSet<&str> = ["a"].into_iter().collect();
        assert_eq!(g, expected);
    }

    #[test]
    fn grounded_of_figure_1_is_a_and_c() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let g = af.grounded_extension();
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert_eq!(g, expected);
    }

    #[test]
    fn grounded_of_odd_cycle_is_empty() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        af.add_attack(&"c", &"a").unwrap();
        assert_eq!(af.grounded_extension(), HashSet::new());
    }

    #[test]
    fn grounded_of_even_cycle_is_empty() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"a").unwrap();
        assert_eq!(af.grounded_extension(), HashSet::new());
    }
}
