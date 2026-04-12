//! Semi-stable extensions: complete extensions with maximal range.
//!
//! Range of a set S = S ∪ {a | a is attacked by some member of S}.
//! Semi-stable extensions are complete extensions with subset-maximal range
//! (Caminada 2006).

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Enumerate all semi-stable extensions.
    ///
    /// Returns [`crate::Error::TooLarge`] for frameworks above the enumeration limit.
    pub fn semi_stable_extensions(&self) -> Result<Vec<HashSet<A>>, crate::Error> {
        let complete = self.complete_extensions()?;
        if complete.is_empty() {
            return Ok(Vec::new());
        }
        let ranges: Vec<(HashSet<A>, HashSet<A>)> = complete
            .into_iter()
            .map(|ext| {
                let mut range = ext.clone();
                for a in &ext {
                    for target in self.attacked_by(a) {
                        range.insert(target.clone());
                    }
                }
                (ext, range)
            })
            .collect();
        Ok(ranges
            .iter()
            .filter(|(_, r)| {
                !ranges
                    .iter()
                    .any(|(_, other)| other.len() > r.len() && r.is_subset(other))
            })
            .map(|(ext, _)| ext.clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semi_stable_of_figure_1_is_ac() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let sse = af.semi_stable_extensions().unwrap();
        assert_eq!(sse.len(), 1);
        let expected: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert!(sse.contains(&expected));
    }

    #[test]
    fn semi_stable_exists_even_without_stable() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        af.add_attack(&"c", &"a").unwrap();
        let sse = af.semi_stable_extensions().unwrap();
        assert!(!sse.is_empty());
    }
}
