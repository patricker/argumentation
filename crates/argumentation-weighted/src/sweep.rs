//! Threshold-sweep API: compute acceptance trajectories for one
//! argument across the full budget range.
//!
//! Under Dunne 2011 semantics, acceptance can change at any distinct
//! subset-sum of attack weights (up to `2^|attacks|` values). The
//! sweep probes all such breakpoints to guarantee no flip is missed.
//!
//! ## Monotonicity
//!
//! Under Dunne 2011 semantics, credulous acceptance is monotone
//! non-decreasing in β: if `x` is credulously accepted at some `β`, it
//! is credulously accepted at every larger budget. [`min_budget_for_credulous`]
//! is therefore well-defined and returns the infimum.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::semantics::{is_credulously_accepted_at, is_skeptically_accepted_at};
use crate::types::Budget;
use std::fmt::Debug;
use std::hash::Hash;

/// One point in a threshold sweep: the budget at which this point
/// applies, and whether the target is accepted at that budget.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPoint {
    /// The budget value at which this point was evaluated.
    pub budget: f64,
    /// Whether the target was accepted at that budget.
    pub accepted: bool,
}

/// Which acceptance notion to use for the sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptanceMode {
    /// Credulous: in at least one preferred extension.
    Credulous,
    /// Skeptical: in every preferred extension.
    Skeptical,
}

/// Compute the sorted list of budget breakpoints at which acceptance
/// can change under Dunne 2011 semantics.
///
/// Under exact subset-enumeration semantics, acceptance can flip at
/// any distinct subset sum of attack weights — up to `2^|attacks|`
/// distinct values. The v0.1.0 approximation only probed cumulative
/// sums (`m+1` values); that under-samples β and causes
/// `min_budget_for_credulous` / `flip_points` to miss flip points that
/// fall at non-cumulative subset sums.
///
/// If the framework has more than [`crate::reduce::ATTACK_ENUMERATION_LIMIT`]
/// attacks the enumeration is impractical; in that case we fall back to
/// `[0.0]` so callers get a `TooManyAttacks` error from the underlying
/// semantics call rather than a silent wrong answer.
fn breakpoints<A: Clone + Eq + Hash>(framework: &WeightedFramework<A>) -> Vec<f64> {
    let weights: Vec<f64> = framework.attacks().map(|a| a.weight.value()).collect();
    let m = weights.len();
    if m > crate::reduce::ATTACK_ENUMERATION_LIMIT {
        // Fallback: only probe β=0; caller will get TooManyAttacks from semantics fn.
        return vec![0.0];
    }
    let total = 1u64 << m;
    let mut sums: Vec<f64> = Vec::with_capacity(total as usize);
    for bits in 0..total {
        let mut s = 0.0_f64;
        for (i, w) in weights.iter().enumerate() {
            if bits & (1u64 << i) != 0 {
                s += *w;
            }
        }
        sums.push(s);
    }
    sums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sums.dedup_by(|a, b| {
        let diff = (*a - *b).abs();
        let scale = a.abs().max(b.abs());
        // Use a relative epsilon when values are non-negligible; fall back
        // to an absolute epsilon only for values that are both effectively
        // zero. This prevents collapsing tiny-but-distinct sub-picosecond
        // weights (e.g. 0.0 vs 1e-13) while still deduplicating
        // floating-point rounding noise at larger magnitudes.
        if scale > 1e-100 {
            diff < 1e-9 * scale
        } else {
            diff < 1e-100
        }
    });
    sums
}

/// Compute the full acceptance trajectory for `target` across the
/// framework's budget range, returning one `SweepPoint` at every
/// breakpoint.
///
/// The returned vector is sorted by `budget` ascending and starts at
/// `budget = 0`. Use [`flip_points`] if you only want the budgets at
/// which acceptance changes, not every breakpoint.
pub fn acceptance_trajectory<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    mode: AcceptanceMode,
) -> Result<Vec<SweepPoint>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut out = Vec::new();
    for bp in breakpoints(framework) {
        let budget = Budget::new(bp)?;
        let accepted = match mode {
            AcceptanceMode::Credulous => is_credulously_accepted_at(framework, target, budget)?,
            AcceptanceMode::Skeptical => is_skeptically_accepted_at(framework, target, budget)?,
        };
        out.push(SweepPoint {
            budget: bp,
            accepted,
        });
    }
    Ok(out)
}

/// Return only the budgets at which `target`'s acceptance changes as
/// β increases. Useful for the drama-manager flip-point query.
pub fn flip_points<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    mode: AcceptanceMode,
) -> Result<Vec<f64>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let trajectory = acceptance_trajectory(framework, target, mode)?;
    let mut flips = Vec::new();
    let mut last_accepted: Option<bool> = None;
    for point in trajectory {
        if last_accepted != Some(point.accepted) {
            if last_accepted.is_some() {
                flips.push(point.budget);
            }
            last_accepted = Some(point.accepted);
        }
    }
    Ok(flips)
}

/// Return the smallest budget at which `target` is credulously
/// accepted, or `None` if it is never accepted across the framework's
/// full budget range.
///
/// Under Dunne 2011 semantics, credulous acceptance is monotone in β,
/// so the returned value is a stable threshold: once the target is
/// accepted, it remains accepted for all larger budgets.
pub fn min_budget_for_credulous<A>(
    framework: &WeightedFramework<A>,
    target: &A,
) -> Result<Option<f64>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let trajectory = acceptance_trajectory(framework, target, AcceptanceMode::Credulous)?;
    Ok(trajectory
        .into_iter()
        .find(|p| p.accepted)
        .map(|p| p.budget))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_enumerates_all_distinct_subset_sums() {
        // Three attacks with weights 0.2, 0.3, 0.5.
        // All subset sums: {0.0, 0.2, 0.3, 0.5, 0.5, 0.7, 0.8, 1.0}
        // After dedup (0.5 appears twice): [0.0, 0.2, 0.3, 0.5, 0.7, 0.8, 1.0] — 7 values.
        // Old cumulative approach only produced [0.0, 0.2, 0.5, 1.0] — 4 values.
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("c", "d", 0.3).unwrap();
        wf.add_weighted_attack("e", "f", 0.5).unwrap();
        let bps = breakpoints(&wf);
        // Must include the non-cumulative subset sums 0.3, 0.7, 0.8 that
        // the v0.1.0 cumulative approach missed.
        assert_eq!(bps.len(), 7);
        assert!((bps[0] - 0.0).abs() < 1e-9);
        assert!((bps[1] - 0.2).abs() < 1e-9);
        assert!((bps[2] - 0.3).abs() < 1e-9);
        assert!((bps[3] - 0.5).abs() < 1e-9);
        assert!((bps[4] - 0.7).abs() < 1e-9);
        assert!((bps[5] - 0.8).abs() < 1e-9);
        assert!((bps[6] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unattacked_argument_is_accepted_at_every_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_argument("unattacked");
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let trajectory =
            acceptance_trajectory(&wf, &"unattacked", AcceptanceMode::Credulous).unwrap();
        assert!(trajectory.iter().all(|p| p.accepted));
    }

    #[test]
    fn singly_attacked_argument_flips_at_attack_weight() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("attacker", "target", 0.5).unwrap();
        // At β=0: attacker defeats target (not accepted).
        // At β=0.5: attack tolerated, target accepted.
        let flips = flip_points(&wf, &"target", AcceptanceMode::Credulous).unwrap();
        assert_eq!(flips.len(), 1);
        assert!((flips[0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn min_budget_for_credulous_finds_smallest_accepting_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "target", 0.3).unwrap();
        wf.add_weighted_attack("b", "target", 0.7).unwrap();
        // Target accepted only once both attacks are tolerated (β ≥ 1.0).
        let min = min_budget_for_credulous(&wf, &"target").unwrap();
        assert_eq!(min, Some(1.0));
    }

    #[test]
    fn min_budget_returns_none_for_self_attack() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "a", 0.5).unwrap();
        // Self-attacking argument is never accepted under any budget
        // (tolerating the attack leaves an isolated unattacked node,
        // so it IS accepted at β ≥ 0.5). Let's verify the correct answer.
        let min = min_budget_for_credulous(&wf, &"a").unwrap();
        assert_eq!(min, Some(0.5));
    }

    #[test]
    fn trajectory_for_independent_attackers_is_monotone() {
        // Sanity check: with two independent attackers, acceptance is
        // monotone in β. Under Dunne 2011 this holds in general (see
        // uc3_chained_defense_is_monotone_under_dunne_semantics for the
        // chained case); this test pins the simplest instance.
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.3).unwrap();
        wf.add_weighted_attack("a2", "target", 0.5).unwrap();
        let trajectory = acceptance_trajectory(&wf, &"target", AcceptanceMode::Credulous).unwrap();
        let mut seen_accepted = false;
        for p in trajectory {
            if p.accepted {
                seen_accepted = true;
            } else {
                assert!(
                    !seen_accepted,
                    "acceptance should be monotone non-decreasing in budget"
                );
            }
        }
    }

    #[test]
    fn credulous_trajectory_is_monotone_in_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.4).unwrap();
        wf.add_weighted_attack("b", "c", 0.6).unwrap();
        let budgets: Vec<Budget> = [0.0, 0.4, 1.0, 1.5]
            .into_iter()
            .map(|b| Budget::new(b).unwrap())
            .collect();
        let mut traj = Vec::new();
        for budget in &budgets {
            let accepted = is_credulously_accepted_at(&wf, &"c", *budget).unwrap();
            traj.push(SweepPoint {
                budget: budget.value(),
                accepted,
            });
        }
        // Monotone: once true at some β, remains true for all β' > β.
        let first_true = traj.iter().position(|p| p.accepted);
        if let Some(i) = first_true {
            for p in &traj[i..] {
                assert!(p.accepted, "credulous trajectory regressed at β={}", p.budget);
            }
        }
    }

    #[test]
    fn min_budget_for_credulous_handles_sub_picosecond_weights() {
        // Witness: weights far below 1e-12 must not be silently
        // merged into the 0.0 breakpoint.
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 1e-13).unwrap();
        let min = min_budget_for_credulous(&wf, &"b").unwrap();
        assert_eq!(min, Some(1e-13));
    }

    #[test]
    fn min_budget_captures_non_cumulative_subset_sum_flip() {
        // Witness: a↔x mutual attack + y attacking a.
        // Dropping only y→a (β=0.5) leaves a↔x and makes a credulous.
        // Under v0.1.0 cumulative-threshold: breakpoints only probe
        // cumulative sums {0, 0.3, 0.6, 1.1}, so the flip at 0.5 is
        // missed and min_budget returns 0.6.
        // Under v0.2.0 Dunne: breakpoints enumerate all subset sums
        // including 0.5, so min_budget correctly returns 0.5.
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "x", 0.3).unwrap();
        wf.add_weighted_attack("x", "a", 0.3).unwrap();
        wf.add_weighted_attack("y", "a", 0.5).unwrap();
        let min = min_budget_for_credulous(&wf, &"a").unwrap();
        assert!(min.is_some(), "a should be credulously accepted at some finite budget");
        assert!((min.unwrap() - 0.5).abs() < 1e-9, "flip should occur at β=0.5, got {:?}", min);
    }
}
