//! Randomized property tests for `argumentation-weighted` v0.2.0
//!
//! Validates the Dunne 2011 inconsistency-budget invariants across a
//! generated distribution of small weighted frameworks (≤5 distinct
//! arguments, ≤8 attacks, budgets in [0, 5]). Each proptest block
//! runs 64 cases, keeping total workspace test runtime well under 30 s.

use argumentation::ArgumentationFramework;
use argumentation_weighted::{
    Budget, WeightedFramework,
    is_credulously_accepted_at, is_skeptically_accepted_at, min_budget_for_credulous,
};
use proptest::collection::vec;
use proptest::prelude::*;

/// Generate a single argument name from the small alphabet {a, b, c, d, e}.
fn arg_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("a".to_string()),
        Just("b".to_string()),
        Just("c".to_string()),
        Just("d".to_string()),
        Just("e".to_string()),
    ]
}

/// Generate a small `WeightedFramework<String>` with 0–8 attacks over the
/// argument alphabet {a, b, c, d, e}.  Self-attacks (a→a) are valid under
/// Dunne 2011 and are kept; invalid weights are rejected by the framework
/// but the strategy only generates weights in [0, 1) which are always valid.
fn weighted_framework_strategy() -> impl Strategy<Value = WeightedFramework<String>> {
    vec(
        (arg_strategy(), arg_strategy(), 0.0_f64..1.0_f64),
        0..=8usize,
    )
    .prop_map(|attacks| {
        let mut wf = WeightedFramework::new();
        for (a, b, w) in attacks {
            // weight is in [0, 1) — always valid; ignore the result for
            // robustness (e.g. a TooManyAttacks guard won't fire here since
            // we cap at 8).
            let _ = wf.add_weighted_attack(a, b, w);
        }
        wf
    })
}

/// Generate a `Budget` value in [0, 5).
fn budget_strategy() -> impl Strategy<Value = Budget> {
    (0.0_f64..5.0_f64).prop_map(|b| Budget::new(b).expect("budget in [0,5) is always valid"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // -----------------------------------------------------------------
    // Property 1: Credulous acceptance is monotone non-decreasing in β.
    //
    // Paper reference: Dunne et al. 2011, Theorem 3 — under
    // inconsistency-budget semantics the set of credulously accepted
    // arguments grows monotonically as β increases. If x is accepted
    // at budget β₁, it is accepted at every β₂ ≥ β₁.
    // -----------------------------------------------------------------
    #[test]
    fn credulous_monotone_in_budget(
        wf in weighted_framework_strategy(),
        b1 in budget_strategy(),
        b2 in budget_strategy(),
    ) {
        let target = wf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let (lo, hi) = if b1.value() <= b2.value() {
            (b1, b2)
        } else {
            (b2, b1)
        };

        let at_lo = is_credulously_accepted_at(&wf, &target, lo).unwrap();
        let at_hi = is_credulously_accepted_at(&wf, &target, hi).unwrap();

        prop_assert!(
            !at_lo || at_hi,
            "credulous monotonicity violated: accepted at β={} but not at β={}",
            lo.value(),
            hi.value()
        );
    }

    // -----------------------------------------------------------------
    // Property 2: Skeptical acceptance is monotone non-increasing in β.
    //
    // As β grows, more residuals are considered, making it harder for an
    // argument to be in *every* preferred extension of *every* residual.
    // So: if x is skeptically accepted at β₂, it is also skeptically
    // accepted at every β₁ ≤ β₂.
    // -----------------------------------------------------------------
    #[test]
    fn skeptical_monotone_nonincreasing_in_budget(
        wf in weighted_framework_strategy(),
        b1 in budget_strategy(),
        b2 in budget_strategy(),
    ) {
        let target = wf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let (lo, hi) = if b1.value() <= b2.value() {
            (b1, b2)
        } else {
            (b2, b1)
        };

        let at_lo = is_skeptically_accepted_at(&wf, &target, lo).unwrap();
        let at_hi = is_skeptically_accepted_at(&wf, &target, hi).unwrap();

        // If skeptically accepted at hi, must be skeptically accepted at lo.
        prop_assert!(
            !at_hi || at_lo,
            "skeptical non-increasing violated: accepted at β={} but not at β={}",
            hi.value(),
            lo.value()
        );
    }

    // -----------------------------------------------------------------
    // Property 3: Skeptical acceptance implies credulous acceptance.
    //
    // Skeptical: in every preferred ext of every residual.
    // Credulous: in some preferred ext of some residual.
    // Every is strictly stronger than some.
    // -----------------------------------------------------------------
    #[test]
    fn skeptical_implies_credulous(
        wf in weighted_framework_strategy(),
        b in budget_strategy(),
    ) {
        let target = wf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let skeptical = is_skeptically_accepted_at(&wf, &target, b).unwrap();
        let credulous = is_credulously_accepted_at(&wf, &target, b).unwrap();

        prop_assert!(
            !skeptical || credulous,
            "skeptical accepted but credulous rejected at β={} for target={:?}",
            b.value(),
            target
        );
    }

    // -----------------------------------------------------------------
    // Property 4: β = 0 matches Dung credulous acceptance.
    //
    // At β = 0 the only β-inconsistent subset is the empty set, so the
    // unique residual is the full framework with all attacks retained.
    // Credulous acceptance under β = 0 must therefore match standard
    // Dung preferred-extension credulous acceptance.
    // -----------------------------------------------------------------
    #[test]
    fn zero_budget_matches_dung_credulous(
        wf in weighted_framework_strategy(),
    ) {
        let target = wf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        // Build an equivalent plain Dung framework.
        let mut af: ArgumentationFramework<String> = ArgumentationFramework::new();
        for a in wf.arguments() {
            af.add_argument(a.clone());
        }
        for atk in wf.attacks() {
            // add_attack returns Err only if an argument is missing; since
            // we added all arguments above this should not fail.
            af.add_attack(&atk.attacker, &atk.target).unwrap();
        }

        let dung_credulous = af
            .preferred_extensions()
            .unwrap()
            .iter()
            .any(|e| e.contains(&target));

        let weighted_at_zero =
            is_credulously_accepted_at(&wf, &target, Budget::zero()).unwrap();

        prop_assert_eq!(
            dung_credulous,
            weighted_at_zero,
            "β=0 credulous mismatch for target={:?}",
            target
        );
    }

    // -----------------------------------------------------------------
    // Property 5: min_budget_for_credulous consistency.
    //
    // If min_budget_for_credulous returns Some(β), then
    // is_credulously_accepted_at with budget β must return true.
    // -----------------------------------------------------------------
    #[test]
    fn min_budget_for_credulous_is_consistent(
        wf in weighted_framework_strategy(),
    ) {
        let target = wf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        if let Some(min_b) = min_budget_for_credulous(&wf, &target).unwrap() {
            let budget = Budget::new(min_b).unwrap();
            let accepted = is_credulously_accepted_at(&wf, &target, budget).unwrap();
            prop_assert!(
                accepted,
                "min_budget_for_credulous returned {} but argument not accepted at that budget",
                min_b
            );
        }
        // None means never accepted across all breakpoints — no assertion needed.
    }
}
