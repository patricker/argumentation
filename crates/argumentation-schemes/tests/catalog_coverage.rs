//! UC3: catalog coverage. Pins the count, the per-category presence, the
//! uniqueness of ids and keys, and the presence of the schemes the
//! encounter bridge will look up by name.

use argumentation_schemes::catalog::default_catalog;
use argumentation_schemes::types::SchemeCategory;

#[test]
fn default_catalog_has_exactly_25_schemes() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.len(),
        25,
        "v0.1.0 ships with exactly 25 schemes; got {}",
        catalog.len()
    );
}

#[test]
fn every_category_has_at_least_one_scheme() {
    let catalog = default_catalog();
    let categories = [
        SchemeCategory::Epistemic,
        SchemeCategory::Causal,
        SchemeCategory::Practical,
        SchemeCategory::SourceBased,
        SchemeCategory::Popular,
        SchemeCategory::Analogical,
    ];
    for cat in &categories {
        assert!(
            !catalog.by_category(*cat).is_empty(),
            "category {:?} has no schemes",
            cat
        );
    }
}

#[test]
fn every_scheme_has_at_least_one_critical_question() {
    let catalog = default_catalog();
    for scheme in catalog.all() {
        assert!(
            !scheme.critical_questions.is_empty(),
            "scheme '{}' has no critical questions",
            scheme.name
        );
    }
}

#[test]
fn every_scheme_has_at_least_one_premise() {
    let catalog = default_catalog();
    for scheme in catalog.all() {
        assert!(
            !scheme.premises.is_empty(),
            "scheme '{}' has no premises",
            scheme.name
        );
    }
}

#[test]
fn scheme_keys_are_unique() {
    let catalog = default_catalog();
    let mut keys: Vec<String> = catalog.all().iter().map(|s| s.key()).collect();
    let total = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), total, "duplicate scheme keys found");
}

#[test]
fn scheme_ids_are_unique() {
    let catalog = default_catalog();
    let mut ids: Vec<u32> = catalog.all().iter().map(|s| s.id.0).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate scheme ids found");
}

#[test]
fn narrative_relevant_schemes_are_present() {
    let catalog = default_catalog();
    let expected = [
        "argument_from_expert_opinion",
        "argument_from_witness_testimony",
        "argument_from_positive_consequences",
        "argument_from_negative_consequences",
        "argument_from_threat",
        "ad_hominem",
        "argument_from_bias",
        "argument_from_tradition",
        "argument_from_precedent",
        "argument_from_cause_to_effect",
        "argument_from_analogy",
        "argument_from_commitment",
    ];
    for key in &expected {
        assert!(
            catalog.by_key(key).is_some(),
            "missing narrative-relevant scheme: {}",
            key
        );
    }
}

#[test]
fn category_counts_match_expected() {
    let catalog = default_catalog();
    assert_eq!(catalog.by_category(SchemeCategory::Epistemic).len(), 3);
    assert_eq!(catalog.by_category(SchemeCategory::Practical).len(), 7);
    assert_eq!(catalog.by_category(SchemeCategory::SourceBased).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Popular).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Causal).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Analogical).len(), 3);
}
