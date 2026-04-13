//! β-reduction: convert a [`WeightedFramework`] at a given budget into
//! an equivalent unweighted [`argumentation::ArgumentationFramework`].
//!
//! v0.1.0 ships the **cumulative-weight threshold** approximation of
//! Dunne et al. 2011's inconsistency-budget semantics:
//!
//! 1. Sort all attacks by weight ascending.
//! 2. Walk the sorted list, maintaining a running `cumulative` total.
//!    While `cumulative + next_weight ≤ β`, include the next attack in
//!    the "tolerated" set `R` and advance `cumulative`.
//! 3. The residual framework contains all arguments plus every attack
//!    NOT in `R`.
//!
//! This matches the formal definition for the common case (smaller
//! attacks are strictly more expendable). It can under-tolerate in
//! pathological cases where skipping a cheap attack to afford a
//! strategically-important expensive one would yield a larger
//! extension set; the full exponential enumeration is deferred to
//! v0.2.0.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::types::Budget;
use argumentation::ArgumentationFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Reduce a weighted framework at budget `β` to an unweighted Dung
/// framework by tolerating the cheapest attacks first until the
/// cumulative tolerated weight would exceed `β`.
///
/// Returns a plain [`argumentation::ArgumentationFramework`] whose
/// attack edges are the **surviving** attacks (those NOT tolerated).
/// Any existing Dung semantics call on the result corresponds to the
/// weighted semantics at that budget under the cumulative-threshold
/// approximation.
pub fn reduce_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<ArgumentationFramework<A>, Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let mut af = ArgumentationFramework::new();
    for arg in framework.arguments() {
        af.add_argument(arg.clone());
    }

    // Sort a view of attack references by weight ascending so we can
    // walk them in order without modifying the framework.
    let mut sorted_attacks: Vec<&crate::types::WeightedAttack<A>> = framework.attacks().collect();
    sorted_attacks.sort_by(|a, b| {
        a.weight
            .value()
            .partial_cmp(&b.weight.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Tolerate the cheapest attacks first.
    let mut cumulative: f64 = 0.0;
    let mut first_surviving = 0;
    for (i, atk) in sorted_attacks.iter().enumerate() {
        if cumulative + atk.weight.value() <= budget.value() {
            cumulative += atk.weight.value();
        } else {
            first_surviving = i;
            break;
        }
        first_surviving = i + 1;
    }

    // Everything from `first_surviving` onward survives — add those
    // attacks to the residual framework.
    for atk in &sorted_attacks[first_surviving..] {
        af.add_attack(&atk.attacker, &atk.target)?;
    }

    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_budget_keeps_all_attacks() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.8).unwrap();
        let af = reduce_at_budget(&wf, Budget::zero()).unwrap();
        assert_eq!(af.len(), 4);
        assert_eq!(af.attackers(&"b").len(), 1);
        assert_eq!(af.attackers(&"d").len(), 1);
    }

    #[test]
    fn large_budget_tolerates_all_attacks() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.8).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(10.0).unwrap()).unwrap();
        assert_eq!(af.len(), 4);
        assert!(af.attackers(&"b").is_empty());
        assert!(af.attackers(&"d").is_empty());
    }

    #[test]
    fn budget_tolerates_cheapest_attacks_first() {
        // Weights: 0.2, 0.3, 0.5. Budget 0.5 tolerates the 0.2 and
        // 0.3 (cumulative 0.5) but not the 0.5 (would exceed).
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        wf.add_weighted_attack("a3", "target", 0.5).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(0.5).unwrap()).unwrap();
        // Only a3 should still attack target.
        let attackers: Vec<&&str> = af.attackers(&"target").into_iter().collect();
        assert_eq!(attackers.len(), 1);
        assert_eq!(*attackers[0], "a3");
    }

    #[test]
    fn budget_exactly_at_cumulative_tolerates_that_attack() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        // Budget exactly 0.5 — the boundary case. Both should fit.
        let af = reduce_at_budget(&wf, Budget::new(0.5).unwrap()).unwrap();
        assert!(af.attackers(&"target").is_empty());
    }

    #[test]
    fn budget_one_below_cumulative_does_not_tolerate() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(0.499).unwrap()).unwrap();
        // 0.499 < 0.2 + 0.3 = 0.5, so a2 cannot be tolerated; only a1.
        let attackers: Vec<&&str> = af.attackers(&"target").into_iter().collect();
        assert_eq!(attackers.len(), 1);
        assert_eq!(*attackers[0], "a2");
    }

    #[test]
    fn isolated_arguments_preserved_through_reduction() {
        let mut wf = WeightedFramework::new();
        wf.add_argument("isolated");
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let af = reduce_at_budget(&wf, Budget::zero()).unwrap();
        assert_eq!(af.len(), 3);
    }
}
