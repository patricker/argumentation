//! UC1: Corroboration — independent supporters strengthen a claim.
//!
//! Scene: two witnesses independently support a claim about what they
//! saw ("the queen met the stranger"); an attacker raises an alibi
//! that directly attacks the claim. Each support edge has weight
//! 0.4, the attack has weight 0.5.
//!
//! Under necessary-support bipolar semantics + Dunne 2011 budget:
//! - β = 0: the alibi attack is binding. The claim is credulously
//!   rejected.
//! - β = 0.5: the alibi attack can be tolerated (its weight exactly
//!   fits the budget). In that residual, the claim — still required
//!   by both witnesses' support — becomes credulously acceptable.
//! - β = 0.8: both supports could be dropped simultaneously (0.4 +
//!   0.4), but the alibi (0.5) alone still fits. Credulous acceptance
//!   is monotone, so the claim stays accepted.

use argumentation_weighted_bipolar::{Budget, WeightedBipolarFramework, is_credulously_accepted_at};

#[test]
fn corroboration_under_budget_behaves_monotonically() {
    let mut wbf = WeightedBipolarFramework::new();
    wbf.add_weighted_support("witness_1", "claim", 0.4).unwrap();
    wbf.add_weighted_support("witness_2", "claim", 0.4).unwrap();
    wbf.add_weighted_attack("alibi", "claim", 0.5).unwrap();

    let at_0 = is_credulously_accepted_at(&wbf, &"claim", Budget::zero()).unwrap();
    let at_05 =
        is_credulously_accepted_at(&wbf, &"claim", Budget::new(0.5).unwrap()).unwrap();
    let at_08 =
        is_credulously_accepted_at(&wbf, &"claim", Budget::new(0.8).unwrap()).unwrap();

    // Monotonicity invariant: once credulous, stays credulous.
    if at_0 {
        assert!(at_05, "credulous monotonicity: claim accepted at β=0 but not β=0.5");
    }
    if at_05 {
        assert!(at_08, "credulous monotonicity: claim accepted at β=0.5 but not β=0.8");
    }

    // At β=0 the unattacked alibi defeats the claim (the claim is in
    // no preferred extension containing the alibi, and the alibi is
    // in every grounded extension). Pin this to catch future drift.
    assert!(
        !at_0,
        "claim should be credulously rejected at β=0 (alibi attack binding)",
    );

    // At β=0.5 the residual dropping the alibi attack exists; in that
    // residual the claim is supported by both witnesses and is no
    // longer attacked. So it is credulously accepted.
    assert!(
        at_05,
        "claim should be credulously accepted at β=0.5 (alibi attack tolerated)",
    );
}
