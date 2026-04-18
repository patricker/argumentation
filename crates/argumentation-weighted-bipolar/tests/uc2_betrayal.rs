//! UC2: Betrayal — withdrawing support is modelled as tolerating a
//! support edge at some budget.
//!
//! Scene: alice supports bob's position (support weight 0.5); charlie
//! attacks bob (attack weight 0.3). At β=0 alice still supports bob,
//! and bob's acceptance hinges on whether charlie's attack is
//! defended against. At β ≥ 0.5, residuals where alice's support is
//! dropped appear — modelling "alice no longer supports bob", a
//! betrayal event.
//!
//! Skeptical acceptance is monotone NON-INCREASING in β (more
//! β-inconsistent subsets → more residuals → more chances for some
//! preferred extension to exclude bob).

use argumentation_weighted_bipolar::{Budget, WeightedBipolarFramework, is_skeptically_accepted_at};

#[test]
fn betrayal_preserves_skeptical_monotonicity_non_increasing() {
    let mut wbf = WeightedBipolarFramework::new();
    wbf.add_weighted_support("alice", "bob", 0.5).unwrap();
    wbf.add_weighted_attack("charlie", "bob", 0.3).unwrap();

    let at_0 = is_skeptically_accepted_at(&wbf, &"bob", Budget::zero()).unwrap();
    let at_betrayal =
        is_skeptically_accepted_at(&wbf, &"bob", Budget::new(0.5).unwrap()).unwrap();

    // Monotone non-increasing: if accepted at β=0.5 (larger budget),
    // must have been accepted at β=0 too.
    if at_betrayal {
        assert!(at_0, "skeptical monotonicity: bob accepted at β=0.5 but not β=0");
    }

    // At β=0: charlie's attack on bob is binding; bob is skeptically
    // rejected.
    assert!(
        !at_0,
        "bob should be skeptically rejected at β=0 (charlie attack binding)",
    );

    // At β=0.5: residuals include one where charlie's attack is
    // dropped (charlie costs 0.3, fits in 0.5); in that residual bob
    // is no longer attacked. But OTHER residuals still attack bob, so
    // skeptical acceptance across all residuals is still false.
    // (Strictly non-increasing from false at β=0.)
    assert!(
        !at_betrayal,
        "bob should be skeptically rejected at β=0.5 (attacking residuals still exist)",
    );
}
