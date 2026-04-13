//! UC4: the drama manager asks "how much relationship stress before
//! Alice accepts Bob's argument?" — i.e., find the smallest budget at
//! which a target argument becomes credulously accepted.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::min_budget_for_credulous;

#[test]
fn uc4_unattacked_argument_accepted_at_zero() {
    let mut wf = WeightedFramework::new();
    wf.add_argument("free");
    wf.add_weighted_attack("noise1", "noise2", 0.5).unwrap();
    let min = min_budget_for_credulous(&wf, &"free").unwrap();
    assert_eq!(min, Some(0.0));
}

#[test]
fn uc4_singly_attacked_argument_needs_budget_equal_to_attack_weight() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("attacker", "target", 0.75).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    assert_eq!(min, Some(0.75));
}

#[test]
fn uc4_argument_with_multiple_attackers_needs_cumulative_budget() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "target", 0.3).unwrap();
    wf.add_weighted_attack("b", "target", 0.4).unwrap();
    wf.add_weighted_attack("c", "target", 0.5).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    // Needs all three tolerated to accept target: 0.3 + 0.4 + 0.5 = 1.2
    assert_eq!(min, Some(1.2));
}

#[test]
fn uc4_argument_only_needs_direct_attackers_tolerated() {
    // b attacks target. a attacks b. At β=0, a defeats b, b can't
    // attack target, so target is accepted at β=0.
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("b", "target", 0.5).unwrap();
    wf.add_weighted_attack("a", "b", 0.2).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    assert_eq!(
        min,
        Some(0.0),
        "target should be accepted at β=0 because a defeats b"
    );
}
