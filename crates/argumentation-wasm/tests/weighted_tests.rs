use argumentation_wasm::WasmWeightedFramework;

#[test]
fn unattacked_argument_credulous_at_any_beta() {
    let mut fw = WasmWeightedFramework::new();
    fw.add_argument("A");
    fw.set_intensity(0.0);
    assert!(fw.is_credulous("A"));
    fw.set_intensity(1.0);
    assert!(fw.is_credulous("A"));
}

#[test]
fn weight_below_beta_makes_attack_droppable() {
    let mut fw = WasmWeightedFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_weighted_attack("B", "A", 0.4);

    fw.set_intensity(0.0);
    let live: Vec<String> = fw.live_attacks_at_current_beta();
    assert_eq!(live, vec!["B->A".to_string()], "at β=0 nothing droppable");

    fw.set_intensity(0.4);
    let live: Vec<String> = fw.live_attacks_at_current_beta();
    assert!(live.is_empty(), "at β=0.4 the 0.4 attack is droppable, got {live:?}");
}

#[test]
fn credulous_set_at_beta_reflects_droppable_attacker() {
    let mut fw = WasmWeightedFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_weighted_attack("B", "A", 0.4);

    fw.set_intensity(0.0);
    assert!(!fw.is_credulous("A"), "B's attack binds at β=0");

    fw.set_intensity(0.4);
    assert!(fw.is_credulous("A"), "B's attack droppable at β=0.4");
}
