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
    pub fn atom(name: impl Into<String>) -> Self {
        Literal::Atom(name.into())
    }

    /// Construct a negated literal.
    pub fn neg(name: impl Into<String>) -> Self {
        Literal::Neg(name.into())
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
}
