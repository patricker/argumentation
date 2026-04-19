//! Randomized property tests for `argumentation-schemes` v0.2.0
//!
//! Validates that the AIF round-trip and JSON serialization invariants hold
//! for any scheme in the default Walton (2008) catalog instantiated with
//! valid (identifier-safe) bindings. Each proptest block runs 64 cases.

use argumentation_schemes::{
    AifDocument, CatalogRegistry,
    aif_to_instance, instance_to_aif,
    catalog::default_catalog,
};
use proptest::prelude::*;
use std::collections::HashMap;

/// The default catalog has exactly 25 schemes (6 categories × ~4 schemes each).
/// We pick by index so proptest can shrink the index cleanly.
const CATALOG_SIZE: usize = 25;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // -----------------------------------------------------------------
    // Property 1: AIF round-trip preserves instance shape.
    //
    // For any scheme in the default catalog, instantiated with
    // deterministically-generated identifier-safe bindings, converting
    // to AIF and back must produce an instance with:
    //   - the same scheme_name
    //   - the same premises (same count and same Literal values)
    //   - the same conclusion Literal
    //   - the same number of critical questions with identical .text fields
    //
    // AIF does NOT preserve CriticalQuestionInstance.counter_literal
    // (not part of the format — the importer writes a synthetic
    // placeholder). That non-preservation is documented and pinned by
    // the unit test in aif.rs; we do not re-assert it here.
    // -----------------------------------------------------------------
    #[test]
    fn aif_round_trip_preserves_instance_shape(
        scheme_idx in 0usize..CATALOG_SIZE,
        // Use a u64 seed to derive binding values deterministically.
        binding_seed in 0u64..10_000u64,
    ) {
        let catalog = default_catalog();
        let schemes: Vec<_> = catalog.all().iter().collect();
        prop_assume!(scheme_idx < schemes.len());
        let scheme = schemes[scheme_idx];

        // Build bindings: each premise slot gets a value "v<seed>_<slot_idx>".
        // These are plain identifier strings — no '¬', no whitespace, no '?',
        // so the literal round-trip is unambiguous.
        let bindings: HashMap<String, String> = scheme
            .premises
            .iter()
            .enumerate()
            .map(|(i, slot)| (slot.name.clone(), format!("v{}_{}", binding_seed, i)))
            .collect();

        let original = scheme.instantiate(&bindings).unwrap();
        let aif = instance_to_aif(&original);
        let registry = CatalogRegistry::with_walton_catalog();
        let recovered = aif_to_instance(&aif, &registry).unwrap();

        prop_assert_eq!(
            &recovered.scheme_name,
            &original.scheme_name,
            "scheme_name mismatch after AIF round-trip"
        );
        prop_assert_eq!(
            recovered.premises.len(),
            original.premises.len(),
            "premise count mismatch"
        );
        prop_assert_eq!(
            &recovered.premises,
            &original.premises,
            "premises mismatch after AIF round-trip"
        );
        prop_assert_eq!(
            &recovered.conclusion,
            &original.conclusion,
            "conclusion mismatch after AIF round-trip"
        );
        prop_assert_eq!(
            recovered.critical_questions.len(),
            original.critical_questions.len(),
            "critical question count mismatch"
        );
        for (r, o) in recovered.critical_questions.iter().zip(original.critical_questions.iter()) {
            prop_assert_eq!(
                &r.text,
                &o.text,
                "critical question text mismatch after AIF round-trip"
            );
        }
    }

    // -----------------------------------------------------------------
    // Property 2: AifDocument JSON serialization is a bijection.
    //
    // to_json() followed by from_json() must produce a document that
    // compares equal to the original. This exercises serde round-trip
    // across all structural variants of the AIF representation produced
    // by different scheme shapes (varying premise counts, CQ counts).
    // -----------------------------------------------------------------
    #[test]
    fn aif_document_json_round_trip(
        scheme_idx in 0usize..CATALOG_SIZE,
        binding_seed in 0u64..10_000u64,
    ) {
        let catalog = default_catalog();
        let schemes: Vec<_> = catalog.all().iter().collect();
        prop_assume!(scheme_idx < schemes.len());
        let scheme = schemes[scheme_idx];

        let bindings: HashMap<String, String> = scheme
            .premises
            .iter()
            .enumerate()
            .map(|(i, slot)| (slot.name.clone(), format!("v{}_{}", binding_seed, i)))
            .collect();

        let instance = scheme.instantiate(&bindings).unwrap();
        let aif = instance_to_aif(&instance);

        let json = aif.to_json().unwrap();
        let parsed = AifDocument::from_json(&json).unwrap();

        prop_assert_eq!(
            parsed,
            aif,
            "AifDocument JSON round-trip produced a different document"
        );
    }
}
