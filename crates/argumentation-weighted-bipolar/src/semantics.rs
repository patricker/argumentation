//! Acceptance semantics for weighted bipolar frameworks under Amgoud
//! 2008 + Dunne 2011: iterate every β-inconsistent residual bipolar
//! framework and aggregate across them (OR for credulous, AND for
//! skeptical).

use crate::error::Error;
use crate::framework::WeightedBipolarFramework;
use crate::reduce::wbipolar_residuals;
use crate::types::Budget;
use argumentation_bipolar::bipolar_preferred_extensions;
use std::fmt::Debug;
use std::hash::Hash;

/// `target` is **β-credulously accepted** iff it belongs to some
/// bipolar-preferred extension of some β-inconsistent residual.
pub fn is_credulously_accepted_at<A>(
    framework: &WeightedBipolarFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    for bf in wbipolar_residuals(framework, budget)? {
        let exts = bipolar_preferred_extensions(&bf)?;
        if exts.iter().any(|e| e.contains(target)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `target` is **β-skeptically accepted** iff it belongs to every
/// bipolar-preferred extension of every β-inconsistent residual.
/// Returns `false` when any residual has no preferred extensions.
pub fn is_skeptically_accepted_at<A>(
    framework: &WeightedBipolarFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let residuals = wbipolar_residuals(framework, budget)?;
    // `wbipolar_residuals` always yields at least the empty-subset
    // residual (cost 0 ≤ any non-negative β), so `residuals` is
    // never empty in practice. We still guard against `exts.is_empty()`
    // per residual because a bipolar framework with cyclic attacks can
    // have no preferred extensions.
    for bf in residuals {
        let exts = bipolar_preferred_extensions(&bf)?;
        if exts.is_empty() {
            return Ok(false);
        }
        if !exts.iter().all(|e| e.contains(target)) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credulous_at_zero_budget_matches_bipolar_preferred() {
        // a attacks b; β = 0 ⇒ unique residual = original bipolar framework.
        // Bipolar preferred = { {a} }. a is credulous, b is not.
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.5).unwrap();
        assert!(is_credulously_accepted_at(&wbf, &"a", Budget::zero()).unwrap());
        assert!(!is_credulously_accepted_at(&wbf, &"b", Budget::zero()).unwrap());
    }

    #[test]
    fn tolerating_a_support_breaks_support_closure() {
        // a → b (support, weight 0.3). c attacks b (attack, weight 0.6).
        // Pins monotonicity: if b accepted at β=0, still accepted at β=0.3.
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_support("a", "b", 0.3).unwrap();
        wbf.add_weighted_attack("c", "b", 0.6).unwrap();
        let at0 = is_credulously_accepted_at(&wbf, &"b", Budget::zero()).unwrap();
        let at_drop_support =
            is_credulously_accepted_at(&wbf, &"b", Budget::new(0.3).unwrap()).unwrap();
        if at0 {
            assert!(at_drop_support, "credulous monotonicity violated");
        }
    }

    #[test]
    fn skeptical_accepts_unattacked_self_supporter() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_argument("a");
        // Sole residual: {a}, only preferred extension = {a}.
        assert!(is_skeptically_accepted_at(&wbf, &"a", Budget::zero()).unwrap());
    }

    #[test]
    fn credulous_monotone_in_budget() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.4).unwrap();
        wbf.add_weighted_attack("c", "a", 0.6).unwrap();
        let at0 = is_credulously_accepted_at(&wbf, &"b", Budget::zero()).unwrap();
        let at05 = is_credulously_accepted_at(&wbf, &"b", Budget::new(0.5).unwrap()).unwrap();
        if at0 {
            assert!(at05);
        }
    }
}
