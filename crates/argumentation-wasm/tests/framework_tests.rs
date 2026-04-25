use argumentation_wasm::WasmFramework;

#[test]
fn nixon_diamond_grounded_is_empty() {
    let mut fw = WasmFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_attack("A", "B");
    fw.add_attack("B", "A");
    let mut grounded = fw.grounded_extension();
    grounded.sort();
    assert!(grounded.is_empty(), "got {:?}", grounded);
}

#[test]
fn three_chain_accepts_endpoints_under_grounded() {
    // A → B → C  ⇒  grounded = {A, C}
    let mut fw = WasmFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_argument("C");
    fw.add_attack("A", "B");
    fw.add_attack("B", "C");
    let mut grounded = fw.grounded_extension();
    grounded.sort();
    assert_eq!(grounded, vec!["A".to_string(), "C".to_string()]);
}

#[test]
fn nixon_diamond_has_two_preferred_extensions() {
    let mut fw = WasmFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_attack("A", "B");
    fw.add_attack("B", "A");
    let prefs = fw.preferred_extensions_for_test();
    assert_eq!(prefs.len(), 2, "expected 2 preferred extensions, got {prefs:?}");
    // Each extension is a singleton; together they cover both args.
    let flat: std::collections::HashSet<String> =
        prefs.into_iter().flatten().collect();
    assert!(flat.contains("A") && flat.contains("B"));
}

#[test]
fn nixon_diamond_credulous_accepts_both_skeptical_neither() {
    let mut fw = WasmFramework::new();
    fw.add_argument("A");
    fw.add_argument("B");
    fw.add_attack("A", "B");
    fw.add_attack("B", "A");
    assert!(fw.is_credulously_accepted("A"));
    assert!(fw.is_credulously_accepted("B"));
    assert!(!fw.is_skeptically_accepted("A"));
    assert!(!fw.is_skeptically_accepted("B"));
}
