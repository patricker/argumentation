//! Additional unit tests for the derived-attack closure that exercise
//! interaction between multiple support edges and attacks in one
//! framework. These live in a separate integration test file so the
//! unit tests in `derived.rs` stay focused on individual rule firings.

use argumentation_bipolar::derived::closed_attacks;
use argumentation_bipolar::flatten::flatten;
use argumentation_bipolar::framework::BipolarFramework;

#[test]
fn parallel_support_branches_both_propagate_attacks() {
    // a supports x1, a supports x2 (two parallel branches).
    // x1 attacks target, x2 attacks target.
    // Closure should include (a, target) via either branch.
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "x1").unwrap();
    bf.add_support("a", "x2").unwrap();
    bf.add_attack("x1", "target");
    bf.add_attack("x2", "target");

    let closed = closed_attacks(&bf);
    assert!(closed.contains(&("a", "target")));
    assert!(closed.contains(&("x1", "target")));
    assert!(closed.contains(&("x2", "target")));
}

#[test]
fn flattened_framework_has_same_argument_set_as_bipolar() {
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "b").unwrap();
    bf.add_attack("c", "d");
    bf.add_argument("isolated");
    let af = flatten(&bf).unwrap();
    assert_eq!(af.len(), 5);
}

#[test]
fn closure_is_deterministic_across_rebuilds() {
    let build = || {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        bf.add_attack("c", "d");
        closed_attacks(&bf)
    };
    let first = build();
    let second = build();
    assert_eq!(first, second);
}

#[test]
fn closure_handles_cycles_in_support_graph() {
    // Mutual support cycle a ↔ b, then a attacks c.
    // Both a and b become derived attackers of c.
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "b").unwrap();
    bf.add_support("b", "a").unwrap();
    bf.add_attack("a", "c");
    let closed = closed_attacks(&bf);
    assert!(closed.contains(&("a", "c")));
    assert!(
        closed.contains(&("b", "c")),
        "mutual support should propagate direct attacks through the cycle"
    );
}
