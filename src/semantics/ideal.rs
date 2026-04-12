//! Ideal extension: the largest admissible set contained in every preferred
//! extension (Dung, Mancarella, Toni 2007).

use super::subset_enum::subset_from_bits;
use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Compute the ideal extension.
    ///
    /// Defined as the largest admissible subset of the intersection of all
    /// preferred extensions. Unique. The grounded extension is always a
    /// subset of the ideal extension.
    ///
    /// Returns [`crate::Error::TooLarge`] for frameworks above the enumeration limit.
    pub fn ideal_extension(&self) -> Result<HashSet<A>, crate::Error> {
        let preferred = self.preferred_extensions()?;
        if preferred.is_empty() {
            return Ok(HashSet::new());
        }
        let mut intersection: HashSet<A> = preferred[0].clone();
        for ext in &preferred[1..] {
            intersection = intersection.intersection(ext).cloned().collect();
        }
        // Power-set over the intersection. Since `preferred_extensions` already
        // succeeded, we know the framework is within the enumeration limit, so
        // the intersection is too. Sort for determinism.
        let mut args: Vec<A> = intersection.into_iter().collect();
        args.sort();
        let n = args.len();
        let mut best: HashSet<A> = HashSet::new();
        for bits in 0u64..(1u64 << n) {
            let s = subset_from_bits(&args, bits);
            if s.len() <= best.len() {
                continue;
            }
            if self.is_admissible(&s) {
                best = s;
            }
        }
        Ok(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_of_figure_1_is_ac() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let ideal = af.ideal_extension().unwrap();
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert_eq!(ideal, expected);
    }

    #[test]
    fn ideal_of_mutual_attack_is_empty() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"a").unwrap();
        assert!(af.ideal_extension().unwrap().is_empty());
    }
}
