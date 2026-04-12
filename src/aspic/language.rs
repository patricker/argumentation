//! Propositional language for ASPIC+.
//!
//! A `Literal` is either a positive atom (`Atom("bird")`) or its negation
//! (`Neg("bird")`). Contrariness is symmetric literal negation.

use std::fmt;

/// A propositional literal: an atom or its negation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Literal {
    /// Positive atom.
    Atom(String),
    /// Negated atom.
    Neg(String),
}

impl Literal {
    /// Construct a positive literal.
    ///
    /// # Panics (debug builds)
    ///
    /// Panics if `name` starts with the reserved prefix `__applicable_`,
    /// which is used internally by [`crate::aspic::StructuredSystem::add_undercut_rule`]
    /// to encode undercut attacks. Use `add_undercut_rule` to target a
    /// defeasible rule rather than constructing the literal directly.
    pub fn atom(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(
            !name.starts_with("__applicable_"),
            "`__applicable_` prefix is reserved for ASPIC+ undercut encoding; use StructuredSystem::add_undercut_rule"
        );
        Literal::Atom(name)
    }

    /// Construct a negated literal.
    ///
    /// # Panics (debug builds)
    ///
    /// Panics if `name` starts with the reserved prefix `__applicable_`.
    /// See [`Self::atom`] for context.
    pub fn neg(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(
            !name.starts_with("__applicable_"),
            "`__applicable_` prefix is reserved for ASPIC+ undercut encoding; use StructuredSystem::add_undercut_rule"
        );
        Literal::Neg(name)
    }

    /// Construct the reserved undercut-marker literal for a given rule id.
    ///
    /// This is the single sanctioned constructor for the `__applicable_<id>`
    /// reserved namespace and is used internally by
    /// [`crate::aspic::StructuredSystem::add_undercut_rule`] and
    /// [`crate::aspic::compute_attacks`]. Consumers should never call this
    /// directly — use `add_undercut_rule` instead.
    pub(crate) fn undercut_marker(rule_id: usize) -> Literal {
        Literal::Neg(format!("__applicable_{}", rule_id))
    }

    /// Return the contrary of this literal (negation).
    pub fn contrary(&self) -> Literal {
        match self {
            Literal::Atom(n) => Literal::Neg(n.clone()),
            Literal::Neg(n) => Literal::Atom(n.clone()),
        }
    }

    /// Check whether two literals are contraries of each other.
    pub fn is_contrary_of(&self, other: &Literal) -> bool {
        &self.contrary() == other
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Atom(n) => write!(f, "{}", n),
            Literal::Neg(n) => write!(f, "¬{}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrary_of_atom_is_neg() {
        let p = Literal::atom("p");
        assert_eq!(p.contrary(), Literal::neg("p"));
    }

    #[test]
    fn contrary_is_symmetric() {
        let p = Literal::atom("p");
        let np = Literal::neg("p");
        assert!(p.is_contrary_of(&np));
        assert!(np.is_contrary_of(&p));
    }

    #[test]
    fn different_atoms_are_not_contraries() {
        let p = Literal::atom("p");
        let q = Literal::atom("q");
        assert!(!p.is_contrary_of(&q));
    }

    #[test]
    #[should_panic(expected = "__applicable_")]
    fn atom_rejects_reserved_prefix_in_debug() {
        let _ = Literal::atom("__applicable_42");
    }

    #[test]
    #[should_panic(expected = "__applicable_")]
    fn neg_rejects_reserved_prefix_in_debug() {
        let _ = Literal::neg("__applicable_42");
    }

    #[test]
    fn undercut_marker_bypasses_the_check() {
        // The sanctioned constructor is allowed to use the reserved prefix.
        let m = Literal::undercut_marker(42);
        assert_eq!(m, Literal::Neg("__applicable_42".to_string()));
    }
}
