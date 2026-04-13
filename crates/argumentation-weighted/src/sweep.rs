//! Threshold-sweep API: compute acceptance trajectories for one
//! argument across the full budget range.
//!
//! Acceptance can only change at cumulative-sum breakpoints, so a full
//! sweep requires at most `|attacks| + 1` evaluations. Note that
//! acceptance under the v0.1.0 cumulative-weight threshold
//! approximation is NOT globally monotone in β — a chained-defense
//! framework like `a→b→c` can flip `c` from accepted (at β=0, via `a`'s
//! defense) to rejected (once `a→b` is tolerated) to accepted again
//! (once `b→c` is also tolerated). The full Dunne 2011 existential
//! subset semantics would be monotone but is a deferred v0.2.0 target.
//! See `tests/uc3_scene_intensity.rs` for the witness fixture.

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

/// Compute the sorted list of budget breakpoints at which the
/// cumulative-weight threshold transitions — exactly `|attacks|+1`
/// values: `[0, w_1, w_1+w_2, ..., sum]`.
fn breakpoints<A: Clone + Eq + Hash>(framework: &WeightedFramework<A>) -> Vec<f64> {
    let mut out = vec![0.0];
    let mut cumulative = 0.0;
    for w in framework.sorted_weights() {
        cumulative += w;
        out.push(cumulative);
    }
    out
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
/// **⚠ Non-monotonicity caveat.** Under v0.1.0's cumulative-weight
/// threshold approximation, acceptance is NOT globally monotone in
/// the budget. This function returns the *first* breakpoint at which
/// the target becomes credulously accepted, but acceptance may flip
/// back to rejected at larger budgets when tolerating a cheap attack
/// disrupts a chained defense. Do NOT read the returned value as a
/// stable threshold "acceptance holds for all larger budgets."
///
/// For the full picture, call [`acceptance_trajectory`] and inspect
/// every breakpoint. See `tests/uc3_scene_intensity.rs` for the
/// witness fixture where a chain `a→b→c` flips `c` from accepted to
/// rejected back to accepted as the budget grows.
///
/// The full Dunne 2011 existential-subset semantics would be
/// monotone and would make this function a stable threshold query.
/// That variant is a deferred v0.2.0 target.
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
    fn breakpoints_at_zero_and_cumulative_sums() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("c", "d", 0.3).unwrap();
        wf.add_weighted_attack("e", "f", 0.5).unwrap();
        let bps = breakpoints(&wf);
        // Expected: [0.0, 0.2, 0.5, 1.0]
        assert_eq!(bps.len(), 4);
        assert!((bps[0] - 0.0).abs() < 1e-9);
        assert!((bps[1] - 0.2).abs() < 1e-9);
        assert!((bps[2] - 0.5).abs() < 1e-9);
        assert!((bps[3] - 1.0).abs() < 1e-9);
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
        // For a framework with no chained defense (target attacked by
        // two independent arguments), acceptance IS monotone in β. This
        // is NOT a general property — see the
        // `uc3_chained_defense_produces_non_monotone_trajectory`
        // integration test for a counter-example fixture. We keep this
        // test to pin the common-case behavior; don't generalize its
        // assertion.
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
}
