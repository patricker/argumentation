//! β-acceptance under Dunne 2011 inconsistency-budget semantics.
//!
//! All entry points iterate every β-inconsistent residual produced by
//! [`crate::reduce::dunne_residuals`] and aggregate across them:
//! **credulous** queries take an OR (exists-residual), **skeptical**
//! queries take an AND (forall-residual), and extension queries return
//! the set-union of all per-residual extensions.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::reduce::dunne_residuals;
use crate::types::Budget;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Union of grounded extensions across all β-inconsistent residuals.
/// Matches Dunne 2011's credulous reading for the grounded semantics.
pub fn grounded_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<HashSet<A>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut union: HashSet<A> = HashSet::new();
    for af in dunne_residuals(framework, budget)? {
        union.extend(af.grounded_extension());
    }
    Ok(union)
}

/// Union of complete extensions across all β-inconsistent residuals.
pub fn complete_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut out: Vec<HashSet<A>> = Vec::new();
    for af in dunne_residuals(framework, budget)? {
        for ext in af.complete_extensions()? {
            if !out.contains(&ext) {
                out.push(ext);
            }
        }
    }
    Ok(out)
}

/// Union of preferred extensions across all β-inconsistent residuals.
pub fn preferred_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut out: Vec<HashSet<A>> = Vec::new();
    for af in dunne_residuals(framework, budget)? {
        for ext in af.preferred_extensions()? {
            if !out.contains(&ext) {
                out.push(ext);
            }
        }
    }
    Ok(out)
}

/// Union of stable extensions across all β-inconsistent residuals. A
/// residual may have no stable extensions; those contribute nothing.
pub fn stable_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut out: Vec<HashSet<A>> = Vec::new();
    for af in dunne_residuals(framework, budget)? {
        for ext in af.stable_extensions()? {
            if !out.contains(&ext) {
                out.push(ext);
            }
        }
    }
    Ok(out)
}

/// β-credulous acceptance: `target` appears in some preferred extension
/// of some β-inconsistent residual.
pub fn is_credulously_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    for af in dunne_residuals(framework, budget)? {
        if af.preferred_extensions()?.iter().any(|e| e.contains(target)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// β-skeptical acceptance: `target` appears in every preferred
/// extension of every β-inconsistent residual. Returns `false` for
/// frameworks with no preferred extensions in any residual.
pub fn is_skeptically_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    // dunne_residuals always yields at least the empty-subset
    // residual (cost 0 ≤ any non-negative β), so the list is never
    // empty in practice. We still guard against `exts.is_empty()`
    // per residual because a Dung framework with certain cyclic
    // attacks can have no preferred extensions.
    for af in dunne_residuals(framework, budget)? {
        let exts = af.preferred_extensions()?;
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
    fn grounded_at_zero_budget_matches_dung() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("b", "c", 0.5).unwrap();
        let grounded = grounded_at_budget(&wf, Budget::zero()).unwrap();
        assert!(grounded.contains(&"a"));
        assert!(grounded.contains(&"c"));
        assert!(!grounded.contains(&"b"));
    }

    #[test]
    fn grounded_union_widens_as_budget_grows() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let g0 = grounded_at_budget(&wf, Budget::zero()).unwrap();
        let g1 = grounded_at_budget(&wf, Budget::new(1.0).unwrap()).unwrap();
        // At β=0: grounded = {a}. At β=1 (both residuals {} and {a→b}):
        // union = {a, b}.
        assert!(g0.is_subset(&g1));
        assert!(g1.contains(&"b"));
    }

    #[test]
    fn credulous_acceptance_monotone_in_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("b", "c", 0.7).unwrap();
        // c should flip from true (at β=0, defended by a) to stay true
        // (at β=0.3, still defended), and b should flip from false to
        // true at β=0.3 (a→b can be tolerated).
        let at0 = is_credulously_accepted_at(&wf, &"b", Budget::zero()).unwrap();
        let at03 = is_credulously_accepted_at(&wf, &"b", Budget::new(0.3).unwrap()).unwrap();
        assert!(!at0);
        assert!(at03);
    }

    #[test]
    fn skeptical_true_for_grounded_singleton() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        // β = 0: unique preferred extension = {a}. Skeptical: a ∈ every
        // extension of every residual (only residual is {a→b}).
        assert!(is_skeptically_accepted_at(&wf, &"a", Budget::zero()).unwrap());
        assert!(!is_skeptically_accepted_at(&wf, &"b", Budget::zero()).unwrap());
    }

    #[test]
    fn preferred_at_budget_is_union_across_residuals() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("b", "c", 0.4).unwrap();
        let at0 = preferred_at_budget(&wf, Budget::zero()).unwrap();
        assert!(at0.iter().any(|e| e.contains("a") && e.contains("c")));

        let at02 = preferred_at_budget(&wf, Budget::new(0.2).unwrap()).unwrap();
        let union: std::collections::HashSet<&str> =
            at02.iter().flat_map(|e| e.iter().copied()).collect();
        assert!(union.contains("b"), "b should be reachable at β=0.2");
        assert!(union.contains("c"));
    }

    #[test]
    fn preferred_at_budget_large_enough_accepts_all() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let at_big = preferred_at_budget(&wf, Budget::new(10.0).unwrap()).unwrap();
        let union: std::collections::HashSet<&str> =
            at_big.iter().flat_map(|e| e.iter().copied()).collect();
        assert!(union.contains("a"));
        assert!(union.contains("b"));
    }
}
