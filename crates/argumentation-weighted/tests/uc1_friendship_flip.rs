//! UC1: Alice and Bob are friends. Bob attacks Alice's argument with
//! weight 0.9 (mild-to-significant in context). At β=0, the attack
//! fires and Alice is not accepted. At β=1.0 (budget exceeds 0.9),
//! the attack is tolerated and Alice is accepted.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::semantics::{is_credulously_accepted_at, is_skeptically_accepted_at};
use argumentation_weighted::sweep::{AcceptanceMode, flip_points, min_budget_for_credulous};
use argumentation_weighted::types::Budget;

fn alice_bob_framework() -> WeightedFramework<&'static str> {
    let mut wf = WeightedFramework::new();
    wf.add_argument("alice_claim");
    wf.add_weighted_attack("bob_attack", "alice_claim", 0.9)
        .unwrap();
    wf
}

#[test]
fn uc1_alice_defeated_at_zero_budget() {
    let wf = alice_bob_framework();
    assert!(!is_credulously_accepted_at(&wf, &"alice_claim", Budget::zero()).unwrap());
    assert!(!is_skeptically_accepted_at(&wf, &"alice_claim", Budget::zero()).unwrap());
}

#[test]
fn uc1_alice_accepted_when_budget_exceeds_attack_weight() {
    let wf = alice_bob_framework();
    let budget = Budget::new(1.0).unwrap();
    assert!(is_credulously_accepted_at(&wf, &"alice_claim", budget).unwrap());
}

#[test]
fn uc1_flip_point_is_exactly_zero_point_nine() {
    let wf = alice_bob_framework();
    let flips = flip_points(&wf, &"alice_claim", AcceptanceMode::Credulous).unwrap();
    assert_eq!(flips.len(), 1);
    assert!((flips[0] - 0.9).abs() < 1e-9);
}

#[test]
fn uc1_min_budget_for_credulous_alice_is_zero_point_nine() {
    let wf = alice_bob_framework();
    let min = min_budget_for_credulous(&wf, &"alice_claim").unwrap();
    assert_eq!(min, Some(0.9));
}
