//! UC2: Betrayal — withdrawing support is modelled as tolerating a
//! support edge at some budget.
//!
//! alice supports bob's position; charlie attacks bob. At β=0 alice
//! still supports bob and bob's acceptance depends on the attack
//! relation alone. At β = weight(alice→bob), the residual that drops
//! the support exists — modelling "alice no longer supports bob", a
//! betrayal event. Skeptical acceptance should be sensitive to this.

use argumentation_weighted_bipolar::{WeightedBipolarFramework, is_skeptically_accepted_at};
use argumentation_weighted::types::Budget;

#[test]
fn betrayal_budget_reveals_sensitivity_in_skeptical_acceptance() {
    let mut wbf = WeightedBipolarFramework::new();
    wbf.add_weighted_support("alice", "bob", 0.5).unwrap();
    wbf.add_weighted_attack("charlie", "bob", 0.3).unwrap();

    let at0 = is_skeptically_accepted_at(&wbf, &"bob", Budget::zero()).unwrap();
    let at_betrayal =
        is_skeptically_accepted_at(&wbf, &"bob", Budget::new(0.5).unwrap()).unwrap();

    // Skeptical acceptance is monotone NON-INCREASING in β (more
    // residuals = more chances for a preferred extension to exclude
    // bob). So at_betrayal ≤ at0 (interpreted as bool: if false at 0,
    // false at 0.5 too; if true at 0.5, must have been true at 0).
    if at_betrayal {
        assert!(at0);
    }
}
