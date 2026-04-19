//! Integration tests for AIF round-trip (export, re-import, re-export
//! matches).

use argumentation_schemes::catalog::default_catalog;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_schemes::{aif_to_instance, instance_to_aif, AifDocument};
use std::collections::HashMap;

#[test]
fn expert_opinion_round_trip_preserves_shape() {
    let catalog = default_catalog();
    let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    let original = scheme.instantiate(&bindings).unwrap();

    let doc = instance_to_aif(&original);
    let registry = CatalogRegistry::with_walton_catalog();
    let recovered = aif_to_instance(&doc, &registry).unwrap();

    assert_eq!(recovered.scheme_name, original.scheme_name);
    assert_eq!(recovered.premises, original.premises);
    assert_eq!(recovered.conclusion, original.conclusion);

    // AIF does NOT preserve counter_literal values (not part of the
    // format). Import writes a synthetic placeholder. Pin this
    // non-preservation so a future change that implements proper
    // preservation can't silently regress the docstring contract.
    assert_ne!(
        recovered.critical_questions[0].counter_literal,
        original.critical_questions[0].counter_literal,
        "counter_literal is expected to NOT round-trip through AIF",
    );
}

#[test]
fn minimal_expert_opinion_fixture_imports() {
    let json = std::fs::read_to_string("tests/fixtures/expert_opinion.json")
        .expect("fixture file must be readable from crate root");
    let doc = AifDocument::from_json(&json).unwrap();
    let registry = CatalogRegistry::with_walton_catalog();
    let instance = aif_to_instance(&doc, &registry).unwrap();
    assert_eq!(instance.scheme_name, "Argument from Expert Opinion");
    assert_eq!(instance.premises.len(), 3);
}
