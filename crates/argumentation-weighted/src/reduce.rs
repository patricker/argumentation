//! Dunne 2011 β-inconsistent residual enumeration.
//!
//! Given a weighted framework `WF` and budget `β`, [`dunne_residuals`]
//! returns the plain Dung framework obtained by dropping attacks in
//! each subset `S ⊆ attacks(WF)` whose cumulative weight is at most
//! `β`. The acceptance predicates in [`crate::semantics`] iterate these
//! residuals to compute β-credulous and β-skeptical acceptance.
//!
//! ## Complexity
//!
//! Enumeration is O(2^m · f(n)) where `m = |attacks(WF)|`, `n =
//! |arguments(WF)|`, and `f(n)` is the Dung semantics cost on the
//! residual. [`ATTACK_ENUMERATION_LIMIT`] caps `m` at 24 to keep the
//! factor manageable; larger frameworks return
//! [`crate::Error::TooManyAttacks`].
//!
//! ## v0.1.0 → v0.2.0 migration note
//!
//! v0.1.0 exposed `reduce_at_budget(wf, β) -> ArgumentationFramework`,
//! a cumulative-threshold *approximation* that returned a single
//! residual. That function is removed in v0.2.0: there is no canonical
//! "the" residual under Dunne 2011, so the semantics layer iterates
//! all residuals internally and callers should use
//! [`crate::semantics`] acceptance predicates instead.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::types::Budget;
use argumentation::ArgumentationFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Upper bound on attack count for exact Dunne 2011 subset enumeration.
///
/// At `n = 24` the power-set iteration visits `~16.8M` subsets; in
/// release builds with the straight-line Dung enumerator on the
/// residual this stays under ~2 seconds on commodity hardware. Larger
/// frameworks hit [`crate::Error::TooManyAttacks`].
///
/// The core crate enforces a separate limit on arguments for its own
/// subset enumerators (22, see `argumentation::semantics::subset_enum`);
/// the two limits are independent because they count different things.
pub const ATTACK_ENUMERATION_LIMIT: usize = 24;

/// Enumerate the Dung residuals of `framework` at budget `β`.
///
/// A residual is `WF \ S` for some β-inconsistent `S` — i.e., the
/// plain Dung framework with the attacks in `S` omitted. Every argument
/// is preserved in every residual; only attack edges differ.
///
/// Returns one [`ArgumentationFramework`] per β-inconsistent subset.
/// With `m` attacks, the maximum residual count is `2^m`; the budget
/// typically prunes this substantially. Residuals are returned in bit-
/// mask order (subset 0 = no attacks dropped; subset `2^m - 1` = all
/// attacks dropped).
///
/// Fails with [`Error::TooManyAttacks`] if the framework has more
/// than [`ATTACK_ENUMERATION_LIMIT`] attacks.
pub fn dunne_residuals<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<ArgumentationFramework<A>>, Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let attacks: Vec<&crate::types::WeightedAttack<A>> = framework.attacks().collect();
    let m = attacks.len();

    if m > ATTACK_ENUMERATION_LIMIT {
        return Err(Error::TooManyAttacks {
            attacks: m,
            limit: ATTACK_ENUMERATION_LIMIT,
        });
    }

    let args: Vec<A> = framework.arguments().cloned().collect();
    let total = 1u64 << m;
    let mut residuals = Vec::new();

    for bits in 0..total {
        // Compute the cumulative weight of the dropped set S (bits
        // where the corresponding attack is tolerated, i.e., removed).
        let mut cost = 0.0_f64;
        for (i, atk) in attacks.iter().enumerate() {
            if bits & (1u64 << i) != 0 {
                cost += atk.weight.value();
            }
        }
        if cost > budget.value() {
            continue;
        }

        // Build the residual: all arguments, and all attacks NOT in S.
        let mut af: ArgumentationFramework<A> = ArgumentationFramework::new();
        for a in &args {
            af.add_argument(a.clone());
        }
        for (i, atk) in attacks.iter().enumerate() {
            if bits & (1u64 << i) == 0 {
                af.add_attack(&atk.attacker, &atk.target)?;
            }
        }
        residuals.push(af);
    }

    Ok(residuals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_enumeration_limit_is_24() {
        assert_eq!(super::ATTACK_ENUMERATION_LIMIT, 24);
    }

    #[test]
    fn dunne_residuals_zero_budget_yields_single_residual_with_all_attacks() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.8).unwrap();
        let residuals = dunne_residuals(&wf, Budget::zero()).unwrap();
        assert_eq!(residuals.len(), 1);
        assert_eq!(residuals[0].attackers(&"b").len(), 1);
        assert_eq!(residuals[0].attackers(&"d").len(), 1);
    }

    #[test]
    fn dunne_residuals_budget_covers_cheapest_attack_yields_two_residuals() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("c", "d", 0.9).unwrap();
        let residuals = dunne_residuals(&wf, Budget::new(0.3).unwrap()).unwrap();
        assert_eq!(residuals.len(), 2);
    }

    #[test]
    fn dunne_residuals_large_budget_yields_full_power_set() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.5).unwrap();
        let residuals = dunne_residuals(&wf, Budget::new(10.0).unwrap()).unwrap();
        assert_eq!(residuals.len(), 4);
    }

    #[test]
    fn dunne_residuals_rejects_oversized_framework() {
        let mut wf: WeightedFramework<u32> = WeightedFramework::new();
        for i in 0..(ATTACK_ENUMERATION_LIMIT as u32 + 1) {
            wf.add_weighted_attack(2 * i, 2 * i + 1, 0.1).unwrap();
        }
        let err = dunne_residuals(&wf, Budget::new(1.0).unwrap()).unwrap_err();
        assert!(matches!(err, Error::TooManyAttacks { .. }));
    }

    #[test]
    fn dunne_residuals_preserves_all_arguments_in_every_residual() {
        let mut wf = WeightedFramework::new();
        wf.add_argument("isolated");
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let residuals = dunne_residuals(&wf, Budget::new(1.0).unwrap()).unwrap();
        for r in &residuals {
            assert_eq!(r.len(), 3);
        }
    }
}
