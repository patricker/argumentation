//! UC3: the drama manager sweeps scene intensity (budget) over a
//! framework with three arguments in conflict. These tests pin useful
//! properties of the trajectory/flip-point API under v0.2.0 Dunne 2011
//! existential-subset semantics, which guarantees monotonicity (because
//! increasing β strictly enlarges the set of valid tolerated subsets).

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::{AcceptanceMode, acceptance_trajectory};

#[test]
fn uc3_max_budget_accepts_chain_middle() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    // At the highest breakpoint (sum of weights = 1.0), every attack is
    // tolerated and b is accepted in some preferred extension.
    let traj = acceptance_trajectory(&wf, &"b", AcceptanceMode::Credulous).unwrap();
    assert!(
        traj.last().unwrap().accepted,
        "b should be accepted at max budget"
    );
}

#[test]
fn uc3_zero_budget_runs_pure_dung() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    let traj = acceptance_trajectory(&wf, &"b", AcceptanceMode::Credulous).unwrap();
    // At β=0, the chain a→b→c makes b defeated (attacked by unattacked a).
    assert!(!traj[0].accepted);
}

#[test]
fn uc3_chained_defense_is_monotone_under_dunne_semantics() {
    // Scene: a → b (weight 0.4), b → c (weight 0.6). Under v0.1.0
    // cumulative-threshold approximation, c's trajectory was non-monotone
    // (true at β=0, false at β=0.4, true at β=1.0). Under v0.2.0 Dunne
    // enumeration it is monotone non-decreasing.
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    let traj = acceptance_trajectory(&wf, &"c", AcceptanceMode::Credulous).unwrap();
    assert!(traj[0].accepted, "c credulously accepted at β=0");
    assert!(traj[1].accepted, "c credulously accepted at β=0.4");
    assert!(traj[2].accepted, "c credulously accepted at β=1.0");

    // Spot monotonicity: once accepted, stays accepted.
    let first = traj.iter().position(|p| p.accepted);
    if let Some(i) = first {
        for p in &traj[i..] {
            assert!(p.accepted, "monotonicity violated at β={}", p.budget);
        }
    }
}
