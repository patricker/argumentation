//! UC4: sweeping scene intensity (β) over a 3-character scene flips
//! credulous acceptance at predictable budgets and never regresses
//! (monotonicity).

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::{ArgumentId, EncounterArgumentationState};

#[test]
fn credulous_acceptance_is_monotone_over_intensity_sweep() {
    // Scene: c attacks b (weight 0.4); b attacks a (weight 0.3).
    let a = ArgumentId::new("a");
    let b = ArgumentId::new("b");
    let c = ArgumentId::new("c");

    let budgets: Vec<Budget> = [0.0, 0.3, 0.4, 0.7, 1.0]
        .into_iter()
        .map(|v| Budget::new(v).unwrap())
        .collect();

    let mut last_a: Option<bool> = None;
    let mut last_b: Option<bool> = None;
    for &beta in &budgets {
        // Fresh state per budget (at_intensity consumes `self`).
        let mut s = EncounterArgumentationState::new(default_catalog()).at_intensity(beta);
        s.add_weighted_attack(&c, &b, 0.4).unwrap();
        s.add_weighted_attack(&b, &a, 0.3).unwrap();
        let a_ok = s.is_credulously_accepted(&a).unwrap();
        let b_ok = s.is_credulously_accepted(&b).unwrap();
        if let Some(prev) = last_a {
            assert!(
                !prev || a_ok,
                "credulous monotonicity: a was accepted at previous budget but not at {}",
                beta.value()
            );
        }
        if let Some(prev) = last_b {
            assert!(
                !prev || b_ok,
                "credulous monotonicity: b was accepted at previous budget but not at {}",
                beta.value()
            );
        }
        last_a = Some(a_ok);
        last_b = Some(b_ok);
    }

    // Final sanity: at β=1.0 every attack is droppable → all three are
    // credulously accepted.
    let mut s_big = EncounterArgumentationState::new(default_catalog())
        .at_intensity(Budget::new(1.0).unwrap());
    s_big.add_weighted_attack(&c, &b, 0.4).unwrap();
    s_big.add_weighted_attack(&b, &a, 0.3).unwrap();
    assert!(s_big.is_credulously_accepted(&a).unwrap());
    assert!(s_big.is_credulously_accepted(&b).unwrap());
    assert!(s_big.is_credulously_accepted(&c).unwrap());
}
