//! Hand-verified Dung-semantics fixtures.
//!
//! Each fixture is a small, canonical argumentation framework whose
//! extensions under each semantic have been computed by hand. They pin
//! behavior that the internal proptest invariants ([tests/invariants.rs],
//! [tests/stress.rs]) cannot catch, because those tests only verify
//! cross-semantic consistency (e.g. grounded ⊆ every preferred), not
//! absolute correctness.

use argumentation::{ArgumentationFramework, Label};
use std::collections::{BTreeSet, HashSet};

/// Compare two collections of extensions for set-of-set equality, ignoring
/// order.
fn assert_extensions_equal<A: Ord + Clone + std::fmt::Debug>(
    actual: Vec<HashSet<A>>,
    expected: Vec<HashSet<A>>,
    semantic_name: &str,
) {
    let a: BTreeSet<BTreeSet<A>> = actual
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect();
    let e: BTreeSet<BTreeSet<A>> = expected
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect();
    assert_eq!(a, e, "{semantic_name} extensions diverge");
}

/// Construct a HashSet literal from a slice.
fn ext(args: &[&'static str]) -> HashSet<&'static str> {
    args.iter().copied().collect()
}

// ---------------------------------------------------------------------------
// Fixture 1: Mutual attack `a ↔ b`.
// ---------------------------------------------------------------------------

fn mutual_attack() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"a").unwrap();
    af
}

#[test]
fn mutual_attack_grounded() {
    let af = mutual_attack();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn mutual_attack_complete() {
    let af = mutual_attack();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[]), ext(&["a"]), ext(&["b"])],
        "complete",
    );
}

#[test]
fn mutual_attack_preferred() {
    let af = mutual_attack();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a"]), ext(&["b"])],
        "preferred",
    );
}

#[test]
fn mutual_attack_stable() {
    let af = mutual_attack();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a"]), ext(&["b"])],
        "stable",
    );
}

#[test]
fn mutual_attack_ideal() {
    let af = mutual_attack();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn mutual_attack_semi_stable() {
    let af = mutual_attack();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a"]), ext(&["b"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 2: 3-cycle `a → b → c → a`.
// ---------------------------------------------------------------------------

fn three_cycle() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af.add_attack(&"c", &"a").unwrap();
    af
}

#[test]
fn three_cycle_grounded() {
    let af = three_cycle();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn three_cycle_complete() {
    let af = three_cycle();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[])],
        "complete",
    );
}

#[test]
fn three_cycle_preferred() {
    let af = three_cycle();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&[])],
        "preferred",
    );
}

#[test]
fn three_cycle_stable() {
    let af = three_cycle();
    assert_extensions_equal(af.stable_extensions().unwrap(), vec![], "stable");
}

#[test]
fn three_cycle_ideal() {
    let af = three_cycle();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn three_cycle_semi_stable() {
    let af = three_cycle();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&[])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 3: 4-chain `a → b → c → d`.
// ---------------------------------------------------------------------------

fn four_chain() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_argument("d");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af.add_attack(&"c", &"d").unwrap();
    af
}

#[test]
fn four_chain_grounded() {
    let af = four_chain();
    assert_eq!(af.grounded_extension(), ext(&["a", "c"]));
}

#[test]
fn four_chain_complete() {
    let af = four_chain();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&["a", "c"])],
        "complete",
    );
}

#[test]
fn four_chain_preferred() {
    let af = four_chain();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a", "c"])],
        "preferred",
    );
}

#[test]
fn four_chain_stable() {
    let af = four_chain();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a", "c"])],
        "stable",
    );
}

#[test]
fn four_chain_ideal() {
    let af = four_chain();
    assert_eq!(af.ideal_extension().unwrap(), ext(&["a", "c"]));
}

#[test]
fn four_chain_semi_stable() {
    let af = four_chain();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a", "c"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 4: Self-attack alone (`a → a`).
// ---------------------------------------------------------------------------

fn self_attack_alone() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_attack(&"a", &"a").unwrap();
    af
}

#[test]
fn self_attack_alone_grounded() {
    let af = self_attack_alone();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn self_attack_alone_complete() {
    let af = self_attack_alone();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[])],
        "complete",
    );
}

#[test]
fn self_attack_alone_preferred() {
    let af = self_attack_alone();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&[])],
        "preferred",
    );
}

#[test]
fn self_attack_alone_stable() {
    let af = self_attack_alone();
    assert_extensions_equal(af.stable_extensions().unwrap(), vec![], "stable");
}

#[test]
fn self_attack_alone_ideal() {
    let af = self_attack_alone();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn self_attack_alone_semi_stable() {
    let af = self_attack_alone();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&[])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 5: Y-attack `a → c, b → c`.
// ---------------------------------------------------------------------------

fn y_attack() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_attack(&"a", &"c").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af
}

#[test]
fn y_attack_grounded() {
    let af = y_attack();
    assert_eq!(af.grounded_extension(), ext(&["a", "b"]));
}

#[test]
fn y_attack_complete() {
    let af = y_attack();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&["a", "b"])],
        "complete",
    );
}

#[test]
fn y_attack_preferred() {
    let af = y_attack();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a", "b"])],
        "preferred",
    );
}

#[test]
fn y_attack_stable() {
    let af = y_attack();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a", "b"])],
        "stable",
    );
}

#[test]
fn y_attack_ideal() {
    let af = y_attack();
    assert_eq!(af.ideal_extension().unwrap(), ext(&["a", "b"]));
}

#[test]
fn y_attack_semi_stable() {
    let af = y_attack();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a", "b"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 6: Diamond `a → b, a → c, b → d, c → d`.
// ---------------------------------------------------------------------------

fn diamond() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_argument("d");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"a", &"c").unwrap();
    af.add_attack(&"b", &"d").unwrap();
    af.add_attack(&"c", &"d").unwrap();
    af
}

#[test]
fn diamond_grounded() {
    let af = diamond();
    assert_eq!(af.grounded_extension(), ext(&["a", "d"]));
}

#[test]
fn diamond_complete() {
    let af = diamond();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&["a", "d"])],
        "complete",
    );
}

#[test]
fn diamond_preferred() {
    let af = diamond();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a", "d"])],
        "preferred",
    );
}

#[test]
fn diamond_stable() {
    let af = diamond();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a", "d"])],
        "stable",
    );
}

#[test]
fn diamond_ideal() {
    let af = diamond();
    assert_eq!(af.ideal_extension().unwrap(), ext(&["a", "d"]));
}

#[test]
fn diamond_semi_stable() {
    let af = diamond();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a", "d"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 7: Two disjoint mutual attacks `{a, b, c, d}, a ↔ b, c ↔ d`.
// ---------------------------------------------------------------------------

fn two_disjoint_mutual() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_argument("d");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"a").unwrap();
    af.add_attack(&"c", &"d").unwrap();
    af.add_attack(&"d", &"c").unwrap();
    af
}

#[test]
fn two_disjoint_mutual_grounded() {
    let af = two_disjoint_mutual();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn two_disjoint_mutual_complete() {
    let af = two_disjoint_mutual();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![
            ext(&[]),
            ext(&["a"]),
            ext(&["b"]),
            ext(&["c"]),
            ext(&["d"]),
            ext(&["a", "c"]),
            ext(&["a", "d"]),
            ext(&["b", "c"]),
            ext(&["b", "d"]),
        ],
        "complete",
    );
}

#[test]
fn two_disjoint_mutual_preferred() {
    let af = two_disjoint_mutual();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![
            ext(&["a", "c"]),
            ext(&["a", "d"]),
            ext(&["b", "c"]),
            ext(&["b", "d"]),
        ],
        "preferred",
    );
}

#[test]
fn two_disjoint_mutual_stable() {
    let af = two_disjoint_mutual();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![
            ext(&["a", "c"]),
            ext(&["a", "d"]),
            ext(&["b", "c"]),
            ext(&["b", "d"]),
        ],
        "stable",
    );
}

#[test]
fn two_disjoint_mutual_ideal() {
    let af = two_disjoint_mutual();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn two_disjoint_mutual_semi_stable() {
    let af = two_disjoint_mutual();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![
            ext(&["a", "c"]),
            ext(&["a", "d"]),
            ext(&["b", "c"]),
            ext(&["b", "d"]),
        ],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 8: Bidirectional `a ↔ c, b ↔ c` — triangular bidirectional attack
// where each pair is mutually adversarial and the three-way symmetry admits
// multiple preferred extensions. `{a, b}` (both attack c) and `{c}` (c defends
// itself against both a and b) are both subset-maximal admissible sets.
// ---------------------------------------------------------------------------

fn bidirectional_ac_bc() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_attack(&"a", &"c").unwrap();
    af.add_attack(&"c", &"a").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af.add_attack(&"c", &"b").unwrap();
    af
}

#[test]
fn bidirectional_ac_bc_grounded() {
    let af = bidirectional_ac_bc();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn bidirectional_ac_bc_complete() {
    let af = bidirectional_ac_bc();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[]), ext(&["a", "b"]), ext(&["c"])],
        "complete",
    );
}

#[test]
fn bidirectional_ac_bc_preferred() {
    let af = bidirectional_ac_bc();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a", "b"]), ext(&["c"])],
        "preferred",
    );
}

#[test]
fn bidirectional_ac_bc_stable() {
    let af = bidirectional_ac_bc();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a", "b"]), ext(&["c"])],
        "stable",
    );
}

#[test]
fn bidirectional_ac_bc_ideal() {
    let af = bidirectional_ac_bc();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn bidirectional_ac_bc_semi_stable() {
    let af = bidirectional_ac_bc();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a", "b"]), ext(&["c"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 9: Semi-stable vs preferred distinguisher —
// `{a, b, c}, a ↔ b, b → c, c → c`.
// ---------------------------------------------------------------------------

fn semi_stable_distinguisher() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("a");
    af.add_argument("b");
    af.add_argument("c");
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"a").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af.add_attack(&"c", &"c").unwrap();
    af
}

#[test]
fn semi_stable_distinguisher_grounded() {
    let af = semi_stable_distinguisher();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn semi_stable_distinguisher_complete() {
    let af = semi_stable_distinguisher();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[]), ext(&["a"]), ext(&["b"])],
        "complete",
    );
}

#[test]
fn semi_stable_distinguisher_preferred() {
    let af = semi_stable_distinguisher();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a"]), ext(&["b"])],
        "preferred",
    );
}

#[test]
fn semi_stable_distinguisher_stable() {
    let af = semi_stable_distinguisher();
    assert_extensions_equal(af.stable_extensions().unwrap(), vec![ext(&["b"])], "stable");
}

#[test]
fn semi_stable_distinguisher_ideal() {
    let af = semi_stable_distinguisher();
    assert_eq!(af.ideal_extension().unwrap(), HashSet::new());
}

#[test]
fn semi_stable_distinguisher_semi_stable() {
    let af = semi_stable_distinguisher();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["b"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture 10: Dung 1995 Example 2 — reinstatement via a third party.
//
// Structure from Dung 1995 Example 2 (continuation of Example 1):
// `AR = {i1, i2, a}, attacks = {(i1, a), (a, i1), (i2, a)}`.
//
// Intuition: i1 and a attack each other (mutual), but i2 unilaterally
// attacks a. Since i2 has no attackers, i2 is in the grounded extension;
// i2 defeats a, which reinstates i1. This is a canonical example of
// argument reinstatement through a defender.
//
// Extensions explicitly stated in the paper:
// - Example 3 (continuation): "AF has exactly one preferred extension
//   E = {i1, i2}".
// - Example 5 (continuation): F_AF(∅) = {i2}, F_AF^2(∅) = {i1, i2},
//   so grounded extension = {i1, i2}.
//
// The framework is well-founded, so by Theorem 3 grounded, preferred,
// stable and (unique) complete extensions all coincide; ideal and
// semi-stable follow.
// ---------------------------------------------------------------------------

fn dung1995_example2() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    af.add_argument("i1");
    af.add_argument("i2");
    af.add_argument("a");
    af.add_attack(&"i1", &"a").unwrap();
    af.add_attack(&"a", &"i1").unwrap();
    af.add_attack(&"i2", &"a").unwrap();
    af
}

#[test]
fn dung1995_example2_grounded() {
    let af = dung1995_example2();
    assert_eq!(af.grounded_extension(), ext(&["i1", "i2"]));
}

#[test]
fn dung1995_example2_complete() {
    let af = dung1995_example2();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&["i1", "i2"])],
        "complete",
    );
}

#[test]
fn dung1995_example2_preferred() {
    let af = dung1995_example2();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["i1", "i2"])],
        "preferred",
    );
}

#[test]
fn dung1995_example2_stable() {
    let af = dung1995_example2();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["i1", "i2"])],
        "stable",
    );
}

#[test]
fn dung1995_example2_ideal() {
    let af = dung1995_example2();
    assert_eq!(af.ideal_extension().unwrap(), ext(&["i1", "i2"]));
}

#[test]
fn dung1995_example2_semi_stable() {
    let af = dung1995_example2();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["i1", "i2"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Fixture: grounded ⊊ ideal distinguisher.
//
// Structure:
//   Args: {a, b, c, d, e}
//   Attacks: a→b, b→a, a→c, b→c, c→d, d→c, d→e
//
// Reading: {a, b} form a mutual-attack pair, both attacking c. c is also
// in a mutual attack with d, and d attacks e. In the grounded labelling
// every argument is undec: a and b defeat each other; c is attacked by
// undec a/b AND in a mutual attack with d; d is attacked by undec c;
// e is attacked by undec d. So grounded = ∅.
//
// However, d is skeptically accepted under preferred: every preferred
// extension must contain d. There are two preferreds — {a, d} and {b, d} —
// so their intersection is {d}, and {d} is admissible (d's only attacker
// is c, which is defeated by any of a/b/d itself — in {d} alone, c is
// attacked by d). Therefore ideal = {d}, giving grounded = ∅ ⊊ {d} = ideal.
//
// This is the canonical structure witnessing strict inclusion of grounded
// in ideal; Dung, Mancarella & Toni (2007) "Computing ideal sceptical
// argumentation" motivates ideal semantics with examples of this flavor.
// The crate's existing invariant test `grounded_subset_of_ideal` checks
// grounded ⊆ ideal but cannot detect a regression that silently widens
// grounded to match ideal, nor one that narrows ideal to match grounded.
// ---------------------------------------------------------------------------

fn grounded_subset_of_ideal_distinguisher() -> ArgumentationFramework<&'static str> {
    let mut af = ArgumentationFramework::new();
    for arg in ["a", "b", "c", "d", "e"] {
        af.add_argument(arg);
    }
    af.add_attack(&"a", &"b").unwrap();
    af.add_attack(&"b", &"a").unwrap();
    af.add_attack(&"a", &"c").unwrap();
    af.add_attack(&"b", &"c").unwrap();
    af.add_attack(&"c", &"d").unwrap();
    af.add_attack(&"d", &"c").unwrap();
    af.add_attack(&"d", &"e").unwrap();
    af
}

#[test]
fn grounded_subset_of_ideal_grounded_is_empty() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_eq!(af.grounded_extension(), HashSet::new());
}

#[test]
fn grounded_subset_of_ideal_ideal_is_d() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_eq!(af.ideal_extension().unwrap(), ext(&["d"]));
}

#[test]
fn grounded_subset_of_ideal_distinguishes() {
    let af = grounded_subset_of_ideal_distinguisher();
    let grounded = af.grounded_extension();
    let ideal = af.ideal_extension().unwrap();
    // The whole point of this fixture: ideal must strictly contain grounded.
    assert!(
        grounded.is_subset(&ideal),
        "expected grounded ⊆ ideal, got grounded={grounded:?} ideal={ideal:?}"
    );
    assert_ne!(
        grounded, ideal,
        "expected grounded ⊊ ideal (strict), got grounded == ideal == {grounded:?}"
    );
}

#[test]
fn grounded_subset_of_ideal_complete() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_extensions_equal(
        af.complete_extensions().unwrap(),
        vec![ext(&[]), ext(&["d"]), ext(&["a", "d"]), ext(&["b", "d"])],
        "complete",
    );
}

#[test]
fn grounded_subset_of_ideal_preferred() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_extensions_equal(
        af.preferred_extensions().unwrap(),
        vec![ext(&["a", "d"]), ext(&["b", "d"])],
        "preferred",
    );
}

#[test]
fn grounded_subset_of_ideal_stable() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_extensions_equal(
        af.stable_extensions().unwrap(),
        vec![ext(&["a", "d"]), ext(&["b", "d"])],
        "stable",
    );
}

#[test]
fn grounded_subset_of_ideal_semi_stable() {
    let af = grounded_subset_of_ideal_distinguisher();
    assert_extensions_equal(
        af.semi_stable_extensions().unwrap(),
        vec![ext(&["a", "d"]), ext(&["b", "d"])],
        "semi-stable",
    );
}

// ---------------------------------------------------------------------------
// Caminada-style complete labelling ground truth.
//
// Caminada (2006) "On the Issue of Reinstatement in Argumentation" shows
// that complete extensions are in bijection with complete labellings: a
// labelling assigning each argument one of {in, out, undec} such that
// in-args have all attackers out and out-args have at least one attacker
// in. These tests pin the labellings for three fixtures above, exercising
// the three characteristic shapes: (1) multiple in/out labellings plus an
// all-undec one on a mutual attack, (2) the unique all-undec labelling on
// an odd cycle, and (3) the single alternating-in/out labelling on a chain.
// ---------------------------------------------------------------------------

#[test]
fn mutual_attack_complete_labellings_include_undec_case() {
    let af = mutual_attack();
    let labellings = af.complete_labellings().unwrap();
    // Three complete extensions → three complete labellings.
    assert_eq!(labellings.len(), 3);

    // One labelling has both args undec (corresponds to ∅ extension).
    let undec_labelling = labellings
        .iter()
        .find(|l| l.label_of(&"a") == Some(Label::Undec) && l.label_of(&"b") == Some(Label::Undec))
        .expect("expected one labelling with both args undec");
    assert_eq!(undec_labelling.in_set(), HashSet::new());

    // One labelling has a=in, b=out.
    assert!(
        labellings
            .iter()
            .any(|l| l.label_of(&"a") == Some(Label::In) && l.label_of(&"b") == Some(Label::Out))
    );
    // One labelling has b=in, a=out.
    assert!(
        labellings
            .iter()
            .any(|l| l.label_of(&"b") == Some(Label::In) && l.label_of(&"a") == Some(Label::Out))
    );
}

#[test]
fn three_cycle_complete_labelling_is_all_undec() {
    // Three-cycle a→b→c→a: no stable, no non-trivial complete → the only
    // complete labelling labels every argument undec.
    let af = three_cycle();
    let labellings = af.complete_labellings().unwrap();
    assert_eq!(labellings.len(), 1);
    let labelling = &labellings[0];
    assert_eq!(labelling.label_of(&"a"), Some(Label::Undec));
    assert_eq!(labelling.label_of(&"b"), Some(Label::Undec));
    assert_eq!(labelling.label_of(&"c"), Some(Label::Undec));
    assert_eq!(labelling.in_set(), HashSet::new());
}

#[test]
fn four_chain_single_labelling_assigns_alternating_in_out() {
    // Chain a→b→c→d. Unique complete extension {a, c}. Labelling:
    // a=in (unattacked), b=out (attacker a=in), c=in (attacker b=out),
    // d=out (attacker c=in).
    let af = four_chain();
    let labellings = af.complete_labellings().unwrap();
    assert_eq!(labellings.len(), 1);
    let labelling = &labellings[0];
    assert_eq!(labelling.label_of(&"a"), Some(Label::In));
    assert_eq!(labelling.label_of(&"b"), Some(Label::Out));
    assert_eq!(labelling.label_of(&"c"), Some(Label::In));
    assert_eq!(labelling.label_of(&"d"), Some(Label::Out));
    assert_eq!(labelling.in_set(), ext(&["a", "c"]));
}
