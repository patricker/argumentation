//! Conflict-freeness, defence, and admissibility (Dung 1995 §2 definitions).

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

impl<A: Clone + Eq + Hash> ArgumentationFramework<A> {
    /// A set `s` is conflict-free iff no argument in `s` attacks another in `s`.
    pub fn is_conflict_free(&self, s: &HashSet<A>) -> bool {
        for a in s {
            for target in self.attacked_by(a) {
                if s.contains(target) {
                    return false;
                }
            }
        }
        true
    }

    /// `s` defends `a` iff every attacker of `a` is itself attacked by some member of `s`.
    pub fn defends(&self, s: &HashSet<A>, a: &A) -> bool {
        for attacker in self.attackers(a) {
            let counter = self
                .attackers(attacker)
                .iter()
                .any(|defender| s.contains(*defender));
            if !counter {
                return false;
            }
        }
        true
    }

    /// `s` is admissible iff it is conflict-free and defends all its members.
    pub fn is_admissible(&self, s: &HashSet<A>) -> bool {
        if !self.is_conflict_free(s) {
            return false;
        }
        s.iter().all(|a| self.defends(s, a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dung 1995 Figure 1: a -> b -> c (a attacks b, b attacks c).
    fn figure_1() -> ArgumentationFramework<&'static str> {
        let mut af = ArgumentationFramework::new();
        af.add_argument("a");
        af.add_argument("b");
        af.add_argument("c");
        af.add_attack(&"a", &"b").unwrap();
        af.add_attack(&"b", &"c").unwrap();
        af
    }

    #[test]
    fn empty_set_is_conflict_free() {
        let af = figure_1();
        assert!(af.is_conflict_free(&HashSet::new()));
    }

    #[test]
    fn singleton_a_is_conflict_free() {
        let af = figure_1();
        let s: HashSet<&str> = ["a"].into_iter().collect();
        assert!(af.is_conflict_free(&s));
    }

    #[test]
    fn a_and_b_together_is_not_conflict_free() {
        let af = figure_1();
        let s: HashSet<&str> = ["a", "b"].into_iter().collect();
        assert!(!af.is_conflict_free(&s));
    }

    #[test]
    fn a_defends_c() {
        let af = figure_1();
        let s: HashSet<&str> = ["a"].into_iter().collect();
        assert!(af.defends(&s, &"c"));
    }

    #[test]
    fn a_and_c_is_admissible() {
        let af = figure_1();
        let s: HashSet<&str> = ["a", "c"].into_iter().collect();
        assert!(af.is_admissible(&s));
    }

    #[test]
    fn empty_set_is_admissible() {
        let af = figure_1();
        assert!(af.is_admissible(&HashSet::new()));
    }
}
