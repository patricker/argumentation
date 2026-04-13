//! `WeightSource` trait for computing attack weights from external
//! state (relationship metadata, personality traits, etc.).
//!
//! This is a deliberately minimal abstraction for v0.1.0 — it exists
//! so consumers like the future `encounter` crate can pass a policy
//! closure for deriving weights, without coupling this crate to any
//! specific narrative-stack types. The full `encounter`-specific
//! integration lives in whatever bridge crate the narrative team
//! ships; this trait is just the hook.

use crate::error::Error;
use crate::framework::WeightedFramework;
use std::hash::Hash;

/// A source of attack weights. Given an attacker and a target (and
/// whatever context `Self` carries), produce the weight for the
/// corresponding attack edge.
///
/// Implementations might read participant relationship metadata,
/// personality compatibility, recent interaction history, or any other
/// external state. The trait itself is stateless from this crate's
/// perspective.
pub trait WeightSource<A> {
    /// Compute the weight for an attack from `attacker` to `target`.
    /// Returns `None` if this source has no opinion (i.e., the attack
    /// should not be added). Returns `Some(w)` otherwise.
    fn weight_for(&self, attacker: &A, target: &A) -> Option<f64>;
}

/// A closure-based `WeightSource` that wraps any `Fn(&A, &A) -> Option<f64>`.
pub struct ClosureWeightSource<F>(pub F);

impl<A, F> WeightSource<A> for ClosureWeightSource<F>
where
    F: Fn(&A, &A) -> Option<f64>,
{
    fn weight_for(&self, attacker: &A, target: &A) -> Option<f64> {
        (self.0)(attacker, target)
    }
}

/// Populate a `WeightedFramework` from a list of attack pairs, pulling
/// each weight from the provided `WeightSource`. Pairs for which the
/// source returns `None` are skipped. Pairs for which the source
/// returns an invalid weight propagate an [`Error::InvalidWeight`].
///
/// This is a convenience builder. Consumers that need more control
/// (e.g., different sources for different attack types) should call
/// `add_weighted_attack` directly.
pub fn populate_from_source<A, W, I>(
    framework: &mut WeightedFramework<A>,
    pairs: I,
    source: &W,
) -> Result<(), Error>
where
    A: Clone + Eq + Hash,
    W: WeightSource<A>,
    I: IntoIterator<Item = (A, A)>,
{
    for (attacker, target) in pairs {
        if let Some(weight) = source.weight_for(&attacker, &target) {
            framework.add_weighted_attack(attacker, target, weight)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedSource(f64);

    impl WeightSource<&'static str> for FixedSource {
        fn weight_for(&self, _attacker: &&'static str, _target: &&'static str) -> Option<f64> {
            Some(self.0)
        }
    }

    #[test]
    fn closure_weight_source_returns_closure_output() {
        let src = ClosureWeightSource(|_a: &&str, _b: &&str| Some(0.42));
        assert_eq!(src.weight_for(&"x", &"y"), Some(0.42));
    }

    #[test]
    fn populate_from_source_adds_all_attacks() {
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        let src = FixedSource(0.5);
        populate_from_source(&mut wf, vec![("a", "b"), ("c", "d")], &src).unwrap();
        assert_eq!(wf.attack_count(), 2);
    }

    #[test]
    fn populate_skips_none_weights() {
        let src = ClosureWeightSource(
            |_a: &&str, target: &&str| if *target == "b" { Some(0.5) } else { None },
        );
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        populate_from_source(&mut wf, vec![("x", "b"), ("x", "c")], &src).unwrap();
        assert_eq!(wf.attack_count(), 1);
    }

    #[test]
    fn populate_propagates_invalid_weights() {
        let src = ClosureWeightSource(|_a: &&str, _b: &&str| Some(-1.0));
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        let err = populate_from_source(&mut wf, vec![("x", "y")], &src).unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }
}
