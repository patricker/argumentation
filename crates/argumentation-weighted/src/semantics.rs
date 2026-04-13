//! Weighted extensions at a fixed budget.
//!
//! These are thin wrappers that reduce the framework at the given
//! budget and delegate to the core crate's Dung semantics on the
//! residual framework. Every Dung semantics variant
//! (grounded/complete/preferred/stable/semi-stable/ideal) gets a
//! corresponding `*_at_budget` entry point here.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::reduce::reduce_at_budget;
use crate::types::Budget;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// The grounded extension at the given budget.
pub fn grounded_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<HashSet<A>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.grounded_extension())
}

/// All complete extensions at the given budget.
pub fn complete_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.complete_extensions()?)
}

/// All preferred extensions at the given budget.
pub fn preferred_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.preferred_extensions()?)
}

/// All stable extensions at the given budget.
pub fn stable_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.stable_extensions()?)
}

/// Whether `target` is **credulously accepted** at the given budget:
/// does it appear in at least one preferred extension of the residual
/// framework?
pub fn is_credulously_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let prefs = preferred_at_budget(framework, budget)?;
    Ok(prefs.iter().any(|ext| ext.contains(target)))
}

/// Whether `target` is **skeptically accepted** at the given budget:
/// does it appear in every preferred extension of the residual
/// framework?
pub fn is_skeptically_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let prefs = preferred_at_budget(framework, budget)?;
    if prefs.is_empty() {
        return Ok(false);
    }
    Ok(prefs.iter().all(|ext| ext.contains(target)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_at_zero_budget_matches_dung() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("b", "c", 0.5).unwrap();
        // Dung: grounded = {a, c} because a is unattacked and c is
        // attacked by b, which is attacked by a.
        let grounded = grounded_at_budget(&wf, Budget::zero()).unwrap();
        assert!(grounded.contains(&"a"));
        assert!(grounded.contains(&"c"));
        assert!(!grounded.contains(&"b"));
    }

    #[test]
    fn large_budget_grounds_every_argument() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let grounded = grounded_at_budget(&wf, Budget::new(10.0).unwrap()).unwrap();
        // All attacks tolerated; both a and b unattacked.
        assert!(grounded.contains(&"a"));
        assert!(grounded.contains(&"b"));
    }

    #[test]
    fn credulous_and_skeptical_agree_on_grounded_case() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        // Unique preferred extension = {a}. Both credulous and skeptical
        // acceptance of `a` should be true.
        let budget = Budget::zero();
        assert!(is_credulously_accepted_at(&wf, &"a", budget).unwrap());
        assert!(is_skeptically_accepted_at(&wf, &"a", budget).unwrap());
        assert!(!is_credulously_accepted_at(&wf, &"b", budget).unwrap());
        assert!(!is_skeptically_accepted_at(&wf, &"b", budget).unwrap());
    }
}
