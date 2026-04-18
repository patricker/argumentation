//! Reduction correctness tests: Dunne 2011 paper examples plus
//! boundary-value fixtures.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::reduce::dunne_residuals;
use argumentation_weighted::semantics::grounded_at_budget;
use argumentation_weighted::types::Budget;

#[test]
fn reduction_at_zero_preserves_every_attack() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.1).unwrap();
    wf.add_weighted_attack("c", "d", 0.2).unwrap();
    wf.add_weighted_attack("e", "f", 0.3).unwrap();
    let residuals = dunne_residuals(&wf, Budget::zero()).unwrap();
    assert_eq!(residuals.len(), 1);
    let af = &residuals[0];
    // Every argument attacked exactly once.
    for target in ["b", "d", "f"] {
        assert_eq!(af.attackers(&target).len(), 1);
    }
}

#[test]
fn reduction_at_large_budget_tolerates_everything() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    wf.add_weighted_attack("c", "d", 0.7).unwrap();
    let residuals = dunne_residuals(&wf, Budget::new(100.0).unwrap()).unwrap();
    // At large budget, we get residuals where all attacks can be tolerated
    let all_tolerated = residuals
        .iter()
        .find(|af| af.attackers(&"b").is_empty() && af.attackers(&"d").is_empty());
    assert!(all_tolerated.is_some());
}

#[test]
fn grounded_agrees_with_dung_at_zero_budget() {
    // Framework: a → b, b → c, c → d (chain). At β=0 this is pure Dung.
    // Grounded = {a, c} (odd positions in the chain).
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    wf.add_weighted_attack("b", "c", 0.5).unwrap();
    wf.add_weighted_attack("c", "d", 0.5).unwrap();
    let grounded = grounded_at_budget(&wf, Budget::zero()).unwrap();
    assert!(grounded.contains(&"a"));
    assert!(grounded.contains(&"c"));
    assert!(!grounded.contains(&"b"));
    assert!(!grounded.contains(&"d"));
}

#[test]
fn reduction_is_deterministic_across_rebuilds() {
    let build = || {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("c", "d", 0.3).unwrap();
        dunne_residuals(&wf, Budget::new(0.25).unwrap())
            .unwrap()
            .len()
    };
    assert_eq!(build(), build());
}

#[test]
fn reduction_preserves_argument_set() {
    let mut wf = WeightedFramework::new();
    wf.add_argument("isolated");
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    let residuals = dunne_residuals(&wf, Budget::new(1.0).unwrap()).unwrap();
    // All residuals should have all arguments
    for af in residuals {
        assert_eq!(af.len(), 3);
    }
}
