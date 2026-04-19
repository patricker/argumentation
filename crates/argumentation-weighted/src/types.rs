//! Foundational types for weighted argumentation.
//!
//! - [`AttackWeight`] — validated non-negative finite f64 wrapper.
//! - [`Budget`] — validated non-negative finite f64 wrapper for
//!   inconsistency-budget values.
//! - [`WeightedAttack`] — a directed attack edge carrying a weight.

use crate::error::Error;

/// A non-negative finite attack weight. Constructed via [`Self::new`],
/// which rejects NaN, infinity, and negative values.
///
/// Implements `Copy`, `Clone`, `Debug`, `PartialEq`, and `PartialOrd`
/// but NOT `Eq` or `Hash` — `f64` does not satisfy those by default.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AttackWeight(f64);

impl AttackWeight {
    /// Construct a weight, rejecting NaN, infinity, and negative values.
    pub fn new(value: f64) -> Result<Self, Error> {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::InvalidWeight { weight: value });
        }
        Ok(Self(value))
    }

    /// The underlying `f64` value. Always non-negative and finite by
    /// construction.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// A non-negative finite inconsistency budget. Semantics: attacks whose
/// cumulative weight is at most this value may be tolerated for the
/// purposes of Dung semantics.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Budget(f64);

impl Budget {
    /// Construct a budget, rejecting NaN, infinity, and negative values.
    pub fn new(value: f64) -> Result<Self, Error> {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::InvalidBudget { budget: value });
        }
        Ok(Self(value))
    }

    /// The underlying `f64` value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// A zero budget — equivalent to running standard Dung semantics
    /// (no attacks are tolerated).
    #[must_use]
    pub fn zero() -> Self {
        Self(0.0)
    }
}

/// A weighted directed attack edge: `attacker` attacks `target` with
/// the given `weight`.
///
/// Generic over argument type `A` to match the core crate's convention.
// Eq is not derived because AttackWeight wraps f64, which violates
// Eq's reflexivity requirement (NaN ≠ NaN). All constructed weights
// are finite non-NaN by AttackWeight::new validation, but the trait
// bound is unavailable.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAttack<A: Clone + Eq> {
    /// The attacking argument.
    pub attacker: A,
    /// The target argument.
    pub target: A,
    /// The attack weight. Higher weights are harder to tolerate.
    pub weight: AttackWeight,
}

impl<A: Clone + Eq> WeightedAttack<A> {
    /// Convenience constructor.
    pub fn new(attacker: A, target: A, weight: f64) -> Result<Self, Error> {
        Ok(Self {
            attacker,
            target,
            weight: AttackWeight::new(weight)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_weight_accepts_valid_values() {
        assert!(AttackWeight::new(0.0).is_ok());
        assert!(AttackWeight::new(0.5).is_ok());
        assert!(AttackWeight::new(100.0).is_ok());
    }

    #[test]
    fn attack_weight_rejects_nan() {
        let err = AttackWeight::new(f64::NAN).unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }

    #[test]
    fn attack_weight_rejects_infinity() {
        assert!(AttackWeight::new(f64::INFINITY).is_err());
        assert!(AttackWeight::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn attack_weight_rejects_negative() {
        assert!(AttackWeight::new(-0.1).is_err());
    }

    #[test]
    fn budget_zero_is_valid() {
        assert_eq!(Budget::zero().value(), 0.0);
        assert!(Budget::new(0.0).is_ok());
    }

    #[test]
    fn budget_rejects_invalid_values() {
        assert!(Budget::new(-1.0).is_err());
        assert!(Budget::new(f64::NAN).is_err());
        assert!(Budget::new(f64::INFINITY).is_err());
    }

    #[test]
    fn weighted_attack_new_validates_weight() {
        assert!(WeightedAttack::new("a", "b", 0.5).is_ok());
        assert!(WeightedAttack::new("a", "b", -0.5).is_err());
    }
}
