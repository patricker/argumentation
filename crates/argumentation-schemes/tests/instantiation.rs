//! UC1: end-to-end instantiation pipeline. Uses the public API exclusively
//! (no internal modules) to validate that consumers can perform the full
//! scheme → instance → ASPIC+ flow without touching crate internals.

use argumentation::aspic::{Literal, StructuredSystem};
use argumentation_schemes::aspic::add_scheme_to_system;
use argumentation_schemes::catalog::default_catalog;
use std::collections::HashMap;

#[test]
fn uc1_expert_opinion_full_pipeline() {
    let catalog = default_catalog();
    let scheme = catalog
        .by_key("argument_from_expert_opinion")
        .expect("expert opinion scheme should be in default catalog");

    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();

    let instance = scheme
        .instantiate(&bindings)
        .expect("instantiation should succeed");
    assert_eq!(
        instance.premises.len(),
        3,
        "expert opinion has 3 premise slots"
    );
    assert_eq!(instance.conclusion, Literal::atom("fortify_east"));
    assert_eq!(
        instance.critical_questions.len(),
        6,
        "expert opinion has 6 CQs"
    );

    // Feed into ASPIC+ and verify the AF contains an argument concluding fortify_east.
    let mut system = StructuredSystem::new();
    add_scheme_to_system(&instance, &mut system);
    let built = system
        .build_framework()
        .expect("framework build should succeed");
    let conclusion_args = built.arguments_with_conclusion(&Literal::atom("fortify_east"));
    assert!(
        !conclusion_args.is_empty(),
        "AF should contain at least one argument concluding fortify_east"
    );

    let preferred = built
        .framework
        .preferred_extensions()
        .expect("preferred enumeration should succeed");
    assert!(
        !preferred.is_empty(),
        "an unattacked argument should appear in some preferred extension"
    );
}

#[test]
fn uc1_missing_binding_returns_error() {
    let catalog = default_catalog();
    let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
    let bindings: HashMap<String, String> = [("expert".to_string(), "alice".to_string())]
        .into_iter()
        .collect();
    let err = scheme.instantiate(&bindings).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("domain") || msg.contains("claim"));
}
