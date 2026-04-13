//! UC3: the drama manager sweeps scene intensity (budget) over a
//! framework with three arguments in conflict. These tests pin useful
//! properties of the trajectory/flip-point API without asserting
//! global monotonicity, because the cumulative-weight threshold
//! approximation is NOT globally monotone — see
//! `uc3_chained_defense_produces_non_monotone_trajectory` for the
//! witness fixture.
//!
//! The full Dunne 2011 existential-subset semantics would be monotone
//! (because increasing β strictly enlarges the set of valid tolerated
//! subsets), but that's a deferred v0.2.0 target. v0.1.0 ships the
//! greedy cheapest-first approximation, which can flip an argument's
//! acceptance from true to false as β grows when removing a cheap
//! attack disrupts a chained defense.

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
fn uc3_chained_defense_produces_non_monotone_trajectory() {
    // KNOWN LIMITATION of the cumulative-weight threshold approximation:
    // the trajectory for `c` in the chain a→b (0.4), b→c (0.6) is
    // non-monotone:
    //
    //   β = 0.0: c is accepted (a defeats b, so c is defended)
    //   β = 0.4: a→b is tolerated ⇒ b is now unattacked ⇒ b defeats c
    //            ⇒ c is NOT accepted
    //   β = 1.0: b→c is also tolerated ⇒ c is unattacked ⇒ c is accepted
    //
    // This is a property of the greedy cheapest-first reduction, not a
    // bug. The full Dunne 2011 existential-subset semantics would pick
    // the best subset at each β and produce a monotone trajectory, but
    // that's a deferred v0.2.0 target. We pin the observed non-monotone
    // behavior here so a future refactor to the reduction algorithm
    // that silently switches to a different variant fails this test.
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    let traj = acceptance_trajectory(&wf, &"c", AcceptanceMode::Credulous).unwrap();
    // Breakpoints: [0.0, 0.4, 1.0]. Expected acceptance: [true, false, true].
    assert_eq!(traj.len(), 3);
    assert!(traj[0].accepted, "c defended by a at β=0");
    assert!(!traj[1].accepted, "c defeated by unblocked b at β=0.4");
    assert!(traj[2].accepted, "c unattacked at β=1.0");
}
