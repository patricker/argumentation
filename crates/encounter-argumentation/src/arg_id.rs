//! `ArgumentId`: the identifier type for argument nodes in the
//! encounter-level weighted bipolar framework.
//!
//! An `ArgumentId` is the stringified rendering of a literal
//! (e.g. `"fortify_east"` for a positive atom, `"¬deny_claim"` for a
//! negated literal). Two scheme instances that share a conclusion
//! literal share the same `ArgumentId`, so both count as supporting
//! the same argument node. This is the correct convergence behaviour.

use argumentation::aspic::Literal;

/// Opaque identifier for an argument node in the weighted bipolar
/// framework. Constructed from a `Literal` via `From`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArgumentId(String);

impl ArgumentId {
    /// Construct an `ArgumentId` from a raw string. Prefer `From<&Literal>`
    /// when converting from scheme conclusions.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The underlying string, e.g. for AIF export or logging.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Literal> for ArgumentId {
    /// **Ambiguity warning.** `Literal::Atom("¬foo")` and
    /// `Literal::Neg("foo")` both render to `"¬foo"` and will
    /// therefore collide on the same `ArgumentId`. Phase A accepts
    /// this because the default scheme catalog never mints atoms whose
    /// names begin with `¬`; consumers minting literals dynamically
    /// should avoid leading `¬` in atom names.
    fn from(lit: &Literal) -> Self {
        Self(lit.to_string())
    }
}

impl From<Literal> for ArgumentId {
    fn from(lit: Literal) -> Self {
        Self(lit.to_string())
    }
}

impl std::fmt::Display for ArgumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wraps_string() {
        let id = ArgumentId::new("fortify_east");
        assert_eq!(id.as_str(), "fortify_east");
    }

    #[test]
    fn from_positive_literal_renders_plain() {
        let lit = Literal::atom("fortify_east");
        let id: ArgumentId = (&lit).into();
        assert_eq!(id.as_str(), "fortify_east");
    }

    #[test]
    fn from_negated_literal_renders_with_prefix() {
        let lit = Literal::neg("deny_claim");
        let id: ArgumentId = (&lit).into();
        assert_eq!(id.as_str(), "¬deny_claim");
    }

    #[test]
    fn two_same_literals_produce_equal_ids() {
        let a = ArgumentId::from(&Literal::atom("x"));
        let b = ArgumentId::from(&Literal::atom("x"));
        assert_eq!(a, b);
    }

    #[test]
    fn display_matches_as_str() {
        let id = ArgumentId::new("foo");
        assert_eq!(format!("{}", id), "foo");
    }
}
