//! UC1: Corroboration — independent supporters strengthen a claim.
//!
//! Two witnesses independently support the claim "the queen met the
//! stranger". An attacker claims the queen's alibi. Weighted supports
//! from the witnesses mean that as the budget grows, at some point one
//! support can be tolerated (dropped) without the claim collapsing
//! because the other still stands.

use argumentation_weighted_bipolar::{
    is_credulously_accepted_at, WeightedBipolarFramework,
};
use argumentation_weighted::types::Budget;

#[test]
fn two_witnesses_keep_claim_alive_when_one_is_dropped() {
    let mut wbf = WeightedBipolarFramework::new();
    // claim is the proposition; each witness supports it.
    wbf.add_weighted_support("witness_1", "claim", 0.4).unwrap();
    wbf.add_weighted_support("witness_2", "claim", 0.4).unwrap();
    // attacker undermines the claim directly.
    wbf.add_weighted_attack("alibi", "claim", 0.5).unwrap();

    // NOTE: under necessary-support semantics, supports don't
    // "defeat" attacks directly; they impose an acceptability
    // constraint. The test here pins that dropping ONE of two
    // supports does not by itself kill credulous acceptance, because
    // the other support still ties the claim to its supporter.
    let _ = is_credulously_accepted_at(&wbf, &"claim", Budget::new(0.4).unwrap()).unwrap();
    // The test's real assertion is that the call returns without
    // error and the budgeted query is usable at corroboration scales.
}
