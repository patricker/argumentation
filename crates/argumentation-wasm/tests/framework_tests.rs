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
