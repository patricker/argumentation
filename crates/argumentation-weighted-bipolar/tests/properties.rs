//! Randomized property tests for `argumentation-weighted-bipolar` v0.1.0
//!
//! Validates the Amgoud 2008 + Dunne 2011 invariants across a generated
//! distribution of small weighted bipolar frameworks (≤5 distinct
//! arguments, ≤4 attacks + ≤4 supports = ≤8 edges total, well below
//! EDGE_ENUMERATION_LIMIT=24). Each proptest block runs 64 cases.

use argumentation_bipolar::{BipolarFramework, bipolar_preferred_extensions};
use argumentation_weighted_bipolar::{
    Budget, WeightedBipolarFramework,
    is_credulously_accepted_at, is_skeptically_accepted_at,
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

/// Generate a small `WeightedBipolarFramework<String>` with 0–4 attacks and
/// 0–4 supports over the argument alphabet {a, b, c, d, e}. Total edges ≤ 8,
/// well below EDGE_ENUMERATION_LIMIT (24).  Self-supports are skipped (they
/// are illegal under necessary-support semantics). Weights are in [0, 1).
fn weighted_bipolar_framework_strategy(
) -> impl Strategy<Value = WeightedBipolarFramework<String>> {
    (
        vec(
            (arg_strategy(), arg_strategy(), 0.0_f64..1.0_f64),
            0..=4usize,
        ),
        vec(
            (arg_strategy(), arg_strategy(), 0.0_f64..1.0_f64),
            0..=4usize,
        ),
    )
        .prop_map(|(attacks, supports)| {
            let mut wbf = WeightedBipolarFramework::new();
            for (a, b, w) in attacks {
                let _ = wbf.add_weighted_attack(a, b, w);
            }
            for (a, b, w) in supports {
                // Skip self-support — add_weighted_support returns
                // Err::IllegalSelfSupport for a == b.
                if a != b {
                    let _ = wbf.add_weighted_support(a, b, w);
                }
            }
            wbf
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
    // Mirrors the Dunne 2011 monotonicity theorem applied to the bipolar
    // layer: relaxing the budget can only add residuals, never remove them,
    // so an argument accepted in some residual at β₁ remains accepted at
    // every β₂ ≥ β₁.
    // -----------------------------------------------------------------
    #[test]
    fn credulous_monotone_in_budget(
        wbf in weighted_bipolar_framework_strategy(),
        b1 in budget_strategy(),
        b2 in budget_strategy(),
    ) {
        let target = wbf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let (lo, hi) = if b1.value() <= b2.value() {
            (b1, b2)
        } else {
            (b2, b1)
        };

        let at_lo = is_credulously_accepted_at(&wbf, &target, lo).unwrap();
        let at_hi = is_credulously_accepted_at(&wbf, &target, hi).unwrap();

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
    // As β grows, more residuals are considered and skeptical acceptance
    // (in every preferred ext of every residual) can only weaken. So:
    // skeptically accepted at β₂ implies skeptically accepted at β₁ ≤ β₂.
    // -----------------------------------------------------------------
    #[test]
    fn skeptical_monotone_nonincreasing_in_budget(
        wbf in weighted_bipolar_framework_strategy(),
        b1 in budget_strategy(),
        b2 in budget_strategy(),
    ) {
        let target = wbf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let (lo, hi) = if b1.value() <= b2.value() {
            (b1, b2)
        } else {
            (b2, b1)
        };

        let at_lo = is_skeptically_accepted_at(&wbf, &target, lo).unwrap();
        let at_hi = is_skeptically_accepted_at(&wbf, &target, hi).unwrap();

        prop_assert!(
            !at_hi || at_lo,
            "skeptical non-increasing violated: accepted at β={} but not at β={}",
            hi.value(),
            lo.value()
        );
    }

    // -----------------------------------------------------------------
    // Property 3: Skeptical acceptance implies credulous acceptance.
    // -----------------------------------------------------------------
    #[test]
    fn skeptical_implies_credulous(
        wbf in weighted_bipolar_framework_strategy(),
        b in budget_strategy(),
    ) {
        let target = wbf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        let skeptical = is_skeptically_accepted_at(&wbf, &target, b).unwrap();
        let credulous = is_credulously_accepted_at(&wbf, &target, b).unwrap();

        prop_assert!(
            !skeptical || credulous,
            "skeptical accepted but credulous rejected at β={} for target={:?}",
            b.value(),
            target
        );
    }

    // -----------------------------------------------------------------
    // Property 4: β = 0 matches direct bipolar preferred-extension
    // credulous acceptance.
    //
    // At β = 0 the only β-inconsistent subset is empty, so the unique
    // residual is the full bipolar framework. Credulous acceptance at
    // β = 0 must match `bipolar_preferred_extensions` on a plain
    // `BipolarFramework` built from the same edges.
    // -----------------------------------------------------------------
    #[test]
    fn zero_budget_matches_bipolar_preferred_credulous(
        wbf in weighted_bipolar_framework_strategy(),
    ) {
        let target = wbf.arguments().next().cloned();
        prop_assume!(target.is_some());
        let target = target.unwrap();

        // Reconstruct an unweighted BipolarFramework with the same edges.
        let mut bf: BipolarFramework<String> = BipolarFramework::new();
        for a in wbf.arguments() {
            bf.add_argument(a.clone());
        }
        for atk in wbf.attacks() {
            bf.add_attack(atk.attacker.clone(), atk.target.clone());
        }
        for sup in wbf.supports() {
            // Self-supports were excluded during generation, so this is safe.
            bf.add_support(sup.supporter.clone(), sup.supported.clone()).unwrap();
        }

        let bipolar_credulous = bipolar_preferred_extensions(&bf)
            .unwrap()
            .iter()
            .any(|e| e.contains(&target));

        let weighted_at_zero =
            is_credulously_accepted_at(&wbf, &target, Budget::zero()).unwrap();

        prop_assert_eq!(
            bipolar_credulous,
            weighted_at_zero,
            "β=0 mismatch for target={:?}: direct bipolar={}, weighted={}",
            target,
            bipolar_credulous,
            weighted_at_zero
        );
    }
}
