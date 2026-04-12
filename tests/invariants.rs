//! Property-based universal invariants for Dung semantics.
//!
//! Generates small random argumentation frameworks and asserts properties
//! that must hold across all valid semantic implementations. See Baroni,
//! Caminada & Giacomin 2011 (KER 26(4)) for the theoretical justifications.

use argumentation::ArgumentationFramework;
use proptest::prelude::*;
use std::collections::HashSet;

/// Generate a random AF with 1-6 arguments and 0-10 attack edges.
fn arb_framework() -> impl Strategy<Value = ArgumentationFramework<u8>> {
    (1u8..=6, prop::collection::vec((0u8..6, 0u8..6), 0..10)).prop_map(
        |(n, edges)| {
            let mut af = ArgumentationFramework::new();
            for i in 0..n {
                af.add_argument(i);
            }
            for (a, t) in edges {
                if a < n && t < n {
                    // Infallible: both args are in 0..n and were added above; add_attack is
                    // idempotent and accepts self-loops, so no error mode applies.
                    let _ = af.add_attack(&a, &t);
                }
            }
            af
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Invariant 1: the grounded extension is a subset of every preferred extension.
    #[test]
    fn grounded_subset_of_every_preferred(af in arb_framework()) {
        let grounded = af.grounded_extension();
        let preferred = af.preferred_extensions().unwrap();
        for ext in &preferred {
            for g in &grounded {
                prop_assert!(ext.contains(g), "grounded argument {:?} missing from preferred", g);
            }
        }
    }

    /// Invariant 2: every stable extension is a preferred extension.
    #[test]
    fn stable_subset_of_preferred(af in arb_framework()) {
        let stable = af.stable_extensions().unwrap();
        let preferred = af.preferred_extensions().unwrap();
        for s in &stable {
            prop_assert!(preferred.contains(s), "stable extension {:?} not in preferred", s);
        }
    }

    /// Invariant 3: every complete extension contains the grounded extension.
    #[test]
    fn complete_contains_grounded(af in arb_framework()) {
        let grounded = af.grounded_extension();
        let complete = af.complete_extensions().unwrap();
        for ext in &complete {
            for g in &grounded {
                prop_assert!(ext.contains(g), "complete extension {:?} missing grounded arg {:?}", ext, g);
            }
        }
    }

    /// Invariant 4: the ideal extension is a subset of the intersection of all preferred extensions.
    #[test]
    fn ideal_subset_of_preferred_intersection(af in arb_framework()) {
        let preferred = af.preferred_extensions().unwrap();
        let ideal = af.ideal_extension().unwrap();
        if preferred.is_empty() {
            prop_assert!(ideal.is_empty());
            return Ok(());
        }
        let mut intersection: HashSet<u8> = preferred[0].clone();
        for ext in &preferred[1..] {
            intersection = intersection.intersection(ext).copied().collect();
        }
        for i in &ideal {
            prop_assert!(intersection.contains(i), "ideal arg {:?} not in preferred intersection", i);
        }
    }
}
