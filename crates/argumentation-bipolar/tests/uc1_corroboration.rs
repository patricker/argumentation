//! UC1: corroboration without attack. Alice says "I saw the queen meet
//! the stranger," Bob independently says "I also saw it." Charlie
//! attacks Alice's sighting. Necessary support for Alice from Bob.
//!
//! Expected: any extension that accepts Alice must also accept Bob
//! (support closure). Charlie is unattacked and defeats Alice in any
//! extension that accepts Charlie. Bob stands independently.

use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

#[test]
fn uc1_bob_is_in_every_preferred_extension() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");
    // Bob's claim is unattacked; his argument should be in every preferred extension.

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    assert!(
        !prefs.is_empty(),
        "should have at least one preferred extension"
    );
    for ext in &prefs {
        assert!(
            ext.contains(&"bob"),
            "bob should be accepted in every preferred extension"
        );
    }
}

#[test]
fn uc1_charlie_and_alice_never_coexist() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    for ext in &prefs {
        let has_alice = ext.contains(&"alice");
        let has_charlie = ext.contains(&"charlie");
        assert!(
            !(has_alice && has_charlie),
            "alice and charlie are in conflict"
        );
    }
}

#[test]
fn uc1_alice_requires_bob_via_support_closure() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    for ext in &prefs {
        if ext.contains(&"alice") {
            assert!(
                ext.contains(&"bob"),
                "alice cannot be accepted without her necessary supporter bob"
            );
        }
    }
}
