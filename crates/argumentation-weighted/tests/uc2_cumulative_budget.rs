//! UC2: Three attackers on Dawn's position with weights 0.2, 0.3, 0.5
//! (total 1.0). Dawn is only accepted once the budget tolerates ALL
//! three attacks (β ≥ 1.0). At β = 0.99 she is still defeated.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::semantics::is_credulously_accepted_at;
use argumentation_weighted::sweep::{
    AcceptanceMode, acceptance_trajectory, min_budget_for_credulous,
};
use argumentation_weighted::types::Budget;

fn dawn_framework() -> WeightedFramework<&'static str> {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("alice", "dawn", 0.2).unwrap();
    wf.add_weighted_attack("bob", "dawn", 0.3).unwrap();
    wf.add_weighted_attack("charlie", "dawn", 0.5).unwrap();
    wf
}

#[test]
fn uc2_dawn_defeated_at_zero_budget() {
    let wf = dawn_framework();
    assert!(!is_credulously_accepted_at(&wf, &"dawn", Budget::zero()).unwrap());
}

#[test]
fn uc2_dawn_defeated_at_budget_zero_point_five() {
    // 0.5 tolerates the 0.2 + 0.3 but not the 0.5 (cumulative 0.5
    // exactly, but the last 0.5 would push us to 1.0).
    // Wait — 0.2 + 0.3 = 0.5 exactly. The 0.5 attack can't fit in the
    // remaining 0.0 budget. So one attack survives.
    let wf = dawn_framework();
    assert!(!is_credulously_accepted_at(&wf, &"dawn", Budget::new(0.5).unwrap()).unwrap());
}

#[test]
fn uc2_dawn_accepted_at_budget_one() {
    let wf = dawn_framework();
    assert!(is_credulously_accepted_at(&wf, &"dawn", Budget::new(1.0).unwrap()).unwrap());
}

#[test]
fn uc2_dawn_min_budget_is_exactly_one() {
    let wf = dawn_framework();
    let min = min_budget_for_credulous(&wf, &"dawn").unwrap();
    assert_eq!(min, Some(1.0));
}

#[test]
fn uc2_dawn_trajectory_has_single_flip() {
    let wf = dawn_framework();
    let trajectory = acceptance_trajectory(&wf, &"dawn", AcceptanceMode::Credulous).unwrap();
    // Breakpoints: [0.0, 0.2, 0.5, 1.0]. Acceptance should be
    // [false, false, false, true].
    let accepted: Vec<bool> = trajectory.iter().map(|p| p.accepted).collect();
    assert_eq!(accepted, vec![false, false, false, true]);
}
