//! UC2: coalition against a common enemy. Alice and Bob mutually
//! support each other and both attack Charlie. Verify:
//!   - Coalition detection returns a single coalition {alice, bob}.
//!   - Charlie is not in any preferred extension.
//!   - Alice and Bob are both in every preferred extension.

use argumentation_bipolar::coalition::detect_coalitions;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

fn build_coalition_framework() -> BipolarFramework<&'static str> {
    let mut bf = BipolarFramework::new();
    bf.add_support("alice", "bob").unwrap();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("alice", "charlie");
    bf.add_attack("bob", "charlie");
    bf
}

#[test]
fn uc2_coalition_detection_returns_alice_bob_together() {
    let bf = build_coalition_framework();
    let coalitions = detect_coalitions(&bf);

    let alice_bob_coalition = coalitions
        .iter()
        .find(|c| c.members.contains(&"alice") && c.members.contains(&"bob"));
    assert!(
        alice_bob_coalition.is_some(),
        "alice and bob should be in the same coalition"
    );
    assert_eq!(alice_bob_coalition.unwrap().members.len(), 2);

    let charlie_coalition = coalitions
        .iter()
        .find(|c| c.members.contains(&"charlie"))
        .unwrap();
    assert_eq!(
        charlie_coalition.members.len(),
        1,
        "charlie is a singleton coalition"
    );
}

#[test]
fn uc2_charlie_never_in_preferred_extension() {
    let bf = build_coalition_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    for ext in &prefs {
        assert!(
            !ext.contains(&"charlie"),
            "charlie must be defeated by the alice-bob coalition"
        );
    }
}

#[test]
fn uc2_alice_and_bob_in_every_preferred_extension() {
    let bf = build_coalition_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    assert!(
        !prefs.is_empty(),
        "expected at least one preferred extension"
    );
    for ext in &prefs {
        assert!(ext.contains(&"alice"), "alice should be accepted");
        assert!(ext.contains(&"bob"), "bob should be accepted");
    }
}
