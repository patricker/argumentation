//! Edge types for weighted bipolar frameworks.
//!
//! Re-exports `WeightedAttack` from `argumentation-weighted` and adds
//! `WeightedSupport`, the support-relation counterpart.

use crate::error::Error;
pub use argumentation_weighted::types::{AttackWeight, Budget, WeightedAttack};

/// A weighted directed support edge: `supporter` supports `supported`
/// with the given weight under necessary-support semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSupport<A: Clone + Eq> {
    /// The supporter argument.
    pub supporter: A,
    /// The supported argument.
    pub supported: A,
    /// The support weight. Higher weights are harder to tolerate.
    pub weight: AttackWeight,
}

impl<A: Clone + Eq> WeightedSupport<A> {
    /// Construct a weighted support, rejecting self-support and
    /// invalid weights.
    pub fn new(supporter: A, supported: A, weight: f64) -> Result<Self, Error> {
        if supporter == supported {
            return Err(Error::IllegalSelfSupport);
        }
        let w = AttackWeight::new(weight).map_err(|_| Error::InvalidWeight { weight })?;
        Ok(Self { supporter, supported, weight: w })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_support_new_validates_weight() {
        assert!(WeightedSupport::new("a", "b", 0.5).is_ok());
        assert!(WeightedSupport::new("a", "b", -1.0).is_err());
        assert!(WeightedSupport::new("a", "b", f64::NAN).is_err());
    }

    #[test]
    fn weighted_support_rejects_self_support() {
        let err = WeightedSupport::new("a", "a", 0.5).unwrap_err();
        assert!(matches!(err, Error::IllegalSelfSupport));
    }
}
