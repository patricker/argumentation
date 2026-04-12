//! Stress-test Dung semantics across many random frameworks.
//!
//! This test runs 2000 proptest cases against randomly-generated
//! argumentation frameworks in the 1-12 argument range, checking seven
//! universal invariants that must hold across all valid semantic
//! implementations.

use argumentation::ArgumentationFramework;
use proptest::prelude::*;
use std::collections::HashSet;

fn arb_framework() -> impl Strategy<Value = ArgumentationFramework<u8>> {
    (1u8..=12, prop::collection::vec((0u8..12, 0u8..12), 0..30)).prop_map(|(n, edges)| {
        let mut af = ArgumentationFramework::new();
        for i in 0..n {
            af.add_argument(i);
        }
        for (a, t) in edges {
            if a < n && t < n {
                let _ = af.add_attack(&a, &t);
            }
        }
        af
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// 1. Grounded is subset of every preferred.
    #[test]
    fn grounded_subset_of_preferred(af in arb_framework()) {
        let grounded = af.grounded_extension();
        let preferred = af.preferred_extensions().unwrap();
        for ext in &preferred {
            for g in &grounded {
                prop_assert!(ext.contains(g));
            }
        }
    }

    /// 2. Every stable is preferred.
    #[test]
    fn stable_subset_of_preferred(af in arb_framework()) {
        let stable = af.stable_extensions().unwrap();
        let preferred = af.preferred_extensions().unwrap();
        for s in &stable {
            prop_assert!(preferred.contains(s));
        }
    }

    /// 3. Every complete contains grounded.
    #[test]
    fn complete_contains_grounded(af in arb_framework()) {
        let grounded = af.grounded_extension();
        let complete = af.complete_extensions().unwrap();
        for ext in &complete {
            for g in &grounded {
                prop_assert!(ext.contains(g));
            }
        }
    }

    /// 4. Every preferred is complete.
    #[test]
    fn preferred_subset_of_complete(af in arb_framework()) {
        let preferred = af.preferred_extensions().unwrap();
        let complete = af.complete_extensions().unwrap();
        for p in &preferred {
            prop_assert!(complete.contains(p), "preferred extension {:?} not found in complete", p);
        }
    }

    /// 5. Every stable is a complete.
    #[test]
    fn stable_subset_of_complete(af in arb_framework()) {
        let stable = af.stable_extensions().unwrap();
        let complete = af.complete_extensions().unwrap();
        for s in &stable {
            prop_assert!(complete.contains(s), "stable extension {:?} not in complete", s);
        }
    }

    /// 6. Grounded is subset of ideal.
    #[test]
    fn grounded_subset_of_ideal(af in arb_framework()) {
        let grounded = af.grounded_extension();
        let ideal = af.ideal_extension().unwrap();
        for g in &grounded {
            prop_assert!(ideal.contains(g), "grounded arg {:?} missing from ideal", g);
        }
    }

    /// 7. Ideal is subset of preferred intersection.
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
            prop_assert!(intersection.contains(i));
        }
    }

    /// 8. Semi-stable extensions are complete extensions.
    #[test]
    fn semi_stable_subset_of_complete(af in arb_framework()) {
        let ss = af.semi_stable_extensions().unwrap();
        let complete = af.complete_extensions().unwrap();
        for s in &ss {
            prop_assert!(complete.contains(s), "semi-stable {:?} not complete", s);
        }
    }

    /// 9. If stable extensions exist, semi-stable extensions equal stable.
    #[test]
    fn semi_stable_equals_stable_when_stable_exists(af in arb_framework()) {
        let stable = af.stable_extensions().unwrap();
        let ss = af.semi_stable_extensions().unwrap();
        if !stable.is_empty() {
            // HashSet<u8> is not Hash, so canonicalize each extension to a
            // sorted Vec<u8> and compare as BTreeSet<Vec<u8>>.
            use std::collections::BTreeSet;
            let canon = |v: &Vec<HashSet<u8>>| -> BTreeSet<Vec<u8>> {
                v.iter()
                    .map(|e| {
                        let mut xs: Vec<u8> = e.iter().copied().collect();
                        xs.sort();
                        xs
                    })
                    .collect()
            };
            let stable_set = canon(&stable);
            let ss_set = canon(&ss);
            prop_assert_eq!(stable_set, ss_set, "when stable exists, semi-stable should equal stable");
        }
    }
}

#[test]
#[ignore] // run with `cargo test --test stress -- --ignored`
fn stress_fifteen_argument_chain() {
    use std::time::Instant;
    let mut af = ArgumentationFramework::new();
    for i in 0u8..15 {
        af.add_argument(i);
    }
    for i in 0u8..14 {
        af.add_attack(&i, &(i + 1)).unwrap();
    }
    let t0 = Instant::now();
    let complete = af.complete_extensions().unwrap();
    let dt_c = t0.elapsed();
    let t1 = Instant::now();
    let preferred = af.preferred_extensions().unwrap();
    let dt_p = t1.elapsed();
    eprintln!(
        "15-argument chain: complete={} extensions in {:?}, preferred={} in {:?}",
        complete.len(),
        dt_c,
        preferred.len(),
        dt_p
    );
    // Sanity: linear chain `0 -> 1 -> ... -> 14` has exactly one preferred
    // extension containing even-indexed arguments {0, 2, 4, 6, 8, 10, 12, 14}.
    assert_eq!(preferred.len(), 1);
    let expected: HashSet<u8> = (0u8..15).step_by(2).collect();
    assert_eq!(preferred[0], expected);
}

#[test]
fn aspic_small_structured_system_builds() {
    use argumentation::aspic::{Literal, StructuredSystem};
    let mut sys = StructuredSystem::new();
    // Layered structure: a => b => c, with neg-b rebut from d.
    sys.add_ordinary(Literal::atom("a"));
    sys.add_ordinary(Literal::atom("d"));
    let r1 = sys.add_defeasible_rule(vec![Literal::atom("a")], Literal::atom("b"));
    let _r2 = sys.add_defeasible_rule(vec![Literal::atom("b")], Literal::atom("c"));
    let r3 = sys.add_defeasible_rule(vec![Literal::atom("d")], Literal::neg("b"));
    sys.prefer_rule(r1, r3).unwrap(); // r1 > r3 -> b wins
    let built = sys.build_framework().unwrap();
    assert!(
        !built.arguments.is_empty(),
        "should have constructed arguments"
    );
    // Verify c-argument exists and is in some preferred extension (since r1 > r3).
    let c_arg = built
        .arguments
        .iter()
        .find(|a| a.conclusion == Literal::atom("c"));
    assert!(
        c_arg.is_some(),
        "c should be derivable via r1, r2 since b wins"
    );
    let preferred = built.framework.preferred_extensions().unwrap();
    assert!(!preferred.is_empty());
    if let Some(c) = c_arg {
        assert!(
            preferred.iter().any(|ext| ext.contains(&c.id)),
            "c-argument should be in some preferred extension"
        );
    }
}
