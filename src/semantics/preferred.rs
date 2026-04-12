//! Preferred extensions: maximal (subset-maximal) admissible sets.

use super::subset_enum::{sorted_args_or_too_large, subset_from_bits};
use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Enumerate all preferred extensions.
    ///
    /// A preferred extension is a subset-maximal admissible set.
    /// Every argumentation framework has at least one preferred extension.
    ///
    /// Returns [`crate::Error::TooLarge`] for frameworks above the
    /// enumeration limit.
    pub fn preferred_extensions(&self) -> Result<Vec<HashSet<A>>, crate::Error> {
        let args = sorted_args_or_too_large(self)?;
        let n = args.len();
        let mut admissible: Vec<HashSet<A>> = Vec::new();
        for bits in 0u64..(1u64 << n) {
            let s = subset_from_bits(&args, bits);
            if self.is_admissible(&s) {
                admissible.push(s);
            }
        }
        Ok(admissible
            .iter()
            .filter(|s| {
                !admissible
                    .iter()
                    .any(|t| t.len() > s.len() && s.is_subset(t))
            })
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_of_figure_1_is_singleton_ac() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let pe = af.preferred_extensions().unwrap();
        assert_eq!(pe.len(), 1);
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert!(pe.contains(&expected));
    }

    #[test]
    fn preferred_of_mutual_attack_is_two() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"a").unwrap();
        let pe = af.preferred_extensions().unwrap();
        assert_eq!(pe.len(), 2);
    }

    #[test]
    fn every_framework_has_a_preferred_extension() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        af.add_attack(&"c", &"a").unwrap();
        let pe = af.preferred_extensions().unwrap();
        assert!(!pe.is_empty());
    }
}
