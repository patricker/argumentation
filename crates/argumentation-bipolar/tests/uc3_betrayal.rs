//! UC3: betrayal. Start with the UC2 framework, then remove Bob's
//! support for Alice (`bob → alice` support edge is retracted). Verify:
//!   - The alice-bob coalition no longer exists; both are singletons.
//!   - Both still appear in preferred extensions (they still attack Charlie).
//!   - Charlie is still defeated.

use argumentation_bipolar::coalition::detect_coalitions;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

fn build_post_betrayal_framework() -> BipolarFramework<&'static str> {
    let mut bf = BipolarFramework::new();
    bf.add_support("alice", "bob").unwrap();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("alice", "charlie");
    bf.add_attack("bob", "charlie");
    // Bob betrays Alice — remove his support for her.
    assert!(bf.remove_support(&"bob", &"alice"));
    bf
}

#[test]
fn uc3_coalition_dissolved_after_betrayal() {
    let bf = build_post_betrayal_framework();
    let coalitions = detect_coalitions(&bf);

    // No coalition of size 2+ should exist — the mutual support loop is broken.
    for c in &coalitions {
        assert_eq!(
            c.members.len(),
            1,
            "after betrayal, no coalition should contain more than one member"
        );
    }
}

#[test]
fn uc3_alice_still_attacks_charlie() {
    let bf = build_post_betrayal_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    for ext in &prefs {
        assert!(
            !ext.contains(&"charlie"),
            "charlie should still be defeated even after the coalition breaks"
        );
    }
    assert!(!prefs.is_empty());
}

#[test]
fn uc3_both_alice_and_bob_still_accepted_independently() {
    let bf = build_post_betrayal_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    // Neither is attacked by anything, so both should be in every
    // preferred extension.
    for ext in &prefs {
        assert!(ext.contains(&"alice"));
        assert!(ext.contains(&"bob"));
    }
}
