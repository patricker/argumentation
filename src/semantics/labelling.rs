//! Caminada three-valued labellings and their correspondence with extensions.
//!
//! Per Caminada 2006: every argument is labelled IN, OUT, or UNDEC.
//! - IN iff all attackers are OUT
//! - OUT iff some attacker is IN
//! - UNDEC otherwise
//!
//! Complete labellings correspond to complete extensions.

use crate::framework::ArgumentationFramework;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// The label assigned to an argument under a Caminada labelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Label {
    /// Accepted.
    In,
    /// Rejected.
    Out,
    /// Undecided.
    Undec,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::In => write!(f, "in"),
            Label::Out => write!(f, "out"),
            Label::Undec => write!(f, "undec"),
        }
    }
}

/// A complete three-valued labelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labelling<A: Clone + Eq + Hash> {
    labels: HashMap<A, Label>,
}

impl<A: Clone + Eq + Hash> Labelling<A> {
    /// Get the label for an argument, or `None` if not labelled.
    pub fn label_of(&self, a: &A) -> Option<Label> {
        self.labels.get(a).copied()
    }

    /// Get all arguments with the `In` label (= the extension).
    pub fn in_set(&self) -> HashSet<A> {
        self.labels
            .iter()
            .filter(|(_, l)| **l == Label::In)
            .map(|(a, _)| a.clone())
            .collect()
    }
}

impl<A: Clone + Eq + Hash + Ord> ArgumentationFramework<A> {
    /// Compute the labelling corresponding to a given extension.
    ///
    /// An argument is `In` iff in the extension; `Out` iff attacked by
    /// something in the extension; `Undec` otherwise.
    pub fn extension_to_labelling(&self, ext: &HashSet<A>) -> Labelling<A> {
        let mut labels = HashMap::new();
        for a in self.arguments() {
            if ext.contains(a) {
                labels.insert(a.clone(), Label::In);
            } else if self.attackers(a).iter().any(|att| ext.contains(*att)) {
                labels.insert(a.clone(), Label::Out);
            } else {
                labels.insert(a.clone(), Label::Undec);
            }
        }
        Labelling { labels }
    }

    /// Return labellings corresponding to all complete extensions.
    ///
    /// Propagates [`crate::Error::TooLarge`] from `complete_extensions`.
    pub fn complete_labellings(&self) -> Result<Vec<Labelling<A>>, crate::Error> {
        Ok(self
            .complete_extensions()?
            .iter()
            .map(|ext| self.extension_to_labelling(ext))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labelling_of_figure_1_is_in_out_in() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        let ext: HashSet<&str> = ["a", "c"].into_iter().collect();
        let lab = af.extension_to_labelling(&ext);
        assert_eq!(lab.label_of(&"a"), Some(Label::In));
        assert_eq!(lab.label_of(&"b"), Some(Label::Out));
        assert_eq!(lab.label_of(&"c"), Some(Label::In));
    }

    #[test]
    fn labelling_in_set_matches_extension() {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_attack(&"a", &"b").unwrap();
        let ext: HashSet<&str> = ["a"].into_iter().collect();
        let lab = af.extension_to_labelling(&ext);
        assert_eq!(lab.in_set(), ext);
    }

    #[test]
    fn label_displays_as_lowercase_word() {
        assert_eq!(format!("{}", Label::In), "in");
        assert_eq!(format!("{}", Label::Out), "out");
        assert_eq!(format!("{}", Label::Undec), "undec");
    }
}
