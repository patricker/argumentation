//! Worked examples from Modgil & Prakken 2014 "The ASPIC+ framework for
//! structured argumentation: a tutorial" (Argument & Computation 5(1):31-62).
//!
//! Each test builds an ASPIC+ system from the paper's definitions,
//! constructs arguments via forward chaining, and asserts that the
//! computed attacks and extensions match the paper's expected outputs.
//!
//! Crate-specific constraints encoded here:
//! - Our defeat resolution is last-link Elitist, rule-level preferences only.
//! - Premise-level preference orderings from the paper are NOT modeled.
//! - Undercut rules must go through add_undercut_rule (reserved namespace).

use argumentation::aspic::{ArgumentId, AttackKind, BuildOutput, Literal, StructuredSystem};
use std::collections::BTreeSet;

/// Convert a HashSet of ArgumentIds to a BTreeSet for unordered comparison.
fn as_btree(set: &std::collections::HashSet<ArgumentId>) -> BTreeSet<ArgumentId> {
    set.iter().copied().collect()
}

// -----------------------------------------------------------------------------
// Example A: Snores / Professor (M&P 2014 Example 3.25, §3.5)
//
// Kp = {Snores, Professor}
// Rd = { d1: Snores => Misbehaves,
//        d2: Misbehaves => AccessDenied,
//        d3: Professor => ¬AccessDenied }
// Priorities: Snores <' Professor (premise; NOT modeled here),
//             d1 < d2, d1 < d3, d3 < d2 (rule-level).
//
// Under last-link Elitist:
//   LastDefRules(A3) = {d2}, LastDefRules(B2) = {d3}, d3 < d2, so
//   A3 strictly defeats B2. Expected: single preferred extension
//   {A1, A2, A3, B1} — AccessDenied justified.
// -----------------------------------------------------------------------------

struct SnoresProfessorIds {
    a1: ArgumentId, // Snores (premise)
    a2: ArgumentId, // Misbehaves
    a3: ArgumentId, // AccessDenied
    b1: ArgumentId, // Professor (premise)
    b2: ArgumentId, // ¬AccessDenied
}

fn snores_professor() -> (BuildOutput, SnoresProfessorIds) {
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("Snores"));
    sys.add_ordinary(Literal::atom("Professor"));
    let d1 = sys.add_defeasible_rule(vec![Literal::atom("Snores")], Literal::atom("Misbehaves"));
    let d2 = sys.add_defeasible_rule(
        vec![Literal::atom("Misbehaves")],
        Literal::atom("AccessDenied"),
    );
    let d3 = sys.add_defeasible_rule(
        vec![Literal::atom("Professor")],
        Literal::neg("AccessDenied"),
    );
    // Paper priorities: d1 < d2, d1 < d3, d3 < d2.
    sys.prefer_rule(d2, d3).unwrap();
    sys.prefer_rule(d2, d1).unwrap();
    sys.prefer_rule(d3, d1).unwrap();
    let built = sys.build_framework().unwrap();
    let find = |concl: Literal| {
        built
            .arguments
            .iter()
            .find(|a| a.conclusion == concl)
            .unwrap()
            .id
    };
    let ids = SnoresProfessorIds {
        a1: find(Literal::atom("Snores")),
        a2: find(Literal::atom("Misbehaves")),
        a3: find(Literal::atom("AccessDenied")),
        b1: find(Literal::atom("Professor")),
        b2: find(Literal::neg("AccessDenied")),
    };
    (built, ids)
}

#[test]
fn snores_professor_has_5_arguments() {
    let (built, _) = snores_professor();
    assert_eq!(
        built.arguments.len(),
        5,
        "expected exactly 5 arguments (A1..A3, B1..B2)"
    );
}

#[test]
fn snores_professor_rebut_is_mutual_before_defeat_resolution() {
    // Before defeat resolution, both A3->B2 and B2->A3 are attacks (rebuts),
    // because both are defeasible-topped with contrary conclusions.
    let (built, ids) = snores_professor();
    let rebuts: Vec<_> = built
        .attacks
        .iter()
        .filter(|a| a.kind == AttackKind::Rebut)
        .collect();
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.a3 && a.target == ids.b2),
        "expected A3 rebuts B2 attack"
    );
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.b2 && a.target == ids.a3),
        "expected B2 rebuts A3 attack"
    );
}

#[test]
fn snores_professor_a3_defeats_b2_under_last_link_elitist() {
    // After defeat resolution, d3 < d2 means the B2->A3 attack is filtered out;
    // only A3->B2 survives as a defeat edge in the Dung AF.
    let (built, ids) = snores_professor();
    let attackers_of_b2: BTreeSet<_> = built
        .framework
        .attackers(&ids.b2)
        .into_iter()
        .copied()
        .collect();
    let attackers_of_a3: BTreeSet<_> = built
        .framework
        .attackers(&ids.a3)
        .into_iter()
        .copied()
        .collect();
    assert!(
        attackers_of_b2.contains(&ids.a3),
        "A3 should defeat B2 under last-link Elitist"
    );
    assert!(
        !attackers_of_a3.contains(&ids.b2),
        "B2 should NOT defeat A3 (d3 < d2)"
    );
}

#[test]
fn snores_professor_preferred_contains_access_denied() {
    let (built, ids) = snores_professor();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(
        preferred.len(),
        1,
        "expected a single preferred extension under the paper's priority"
    );
    let expected: BTreeSet<ArgumentId> = [ids.a1, ids.a2, ids.a3, ids.b1].into_iter().collect();
    assert_eq!(as_btree(&preferred[0]), expected);
    assert!(
        !preferred[0].contains(&ids.b2),
        "¬AccessDenied (B2) should not be in the preferred extension"
    );
}

// -----------------------------------------------------------------------------
// Example C: Snores / Professor WITHOUT preferences.
//
// Same KB and rules, no prefer_rule calls. A3 and B2 mutually defeat, so
// there are two preferred extensions — one committing to AccessDenied, one
// committing to ¬AccessDenied. This shows the preference machinery is
// load-bearing for the single-extension outcome above.
// -----------------------------------------------------------------------------

fn snores_professor_no_prefs() -> (BuildOutput, SnoresProfessorIds) {
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("Snores"));
    sys.add_ordinary(Literal::atom("Professor"));
    let _d1 = sys.add_defeasible_rule(vec![Literal::atom("Snores")], Literal::atom("Misbehaves"));
    let _d2 = sys.add_defeasible_rule(
        vec![Literal::atom("Misbehaves")],
        Literal::atom("AccessDenied"),
    );
    let _d3 = sys.add_defeasible_rule(
        vec![Literal::atom("Professor")],
        Literal::neg("AccessDenied"),
    );
    let built = sys.build_framework().unwrap();
    let find = |concl: Literal| {
        built
            .arguments
            .iter()
            .find(|a| a.conclusion == concl)
            .unwrap()
            .id
    };
    let ids = SnoresProfessorIds {
        a1: find(Literal::atom("Snores")),
        a2: find(Literal::atom("Misbehaves")),
        a3: find(Literal::atom("AccessDenied")),
        b1: find(Literal::atom("Professor")),
        b2: find(Literal::neg("AccessDenied")),
    };
    (built, ids)
}

#[test]
fn snores_professor_no_prefs_has_two_preferred_extensions() {
    let (built, ids) = snores_professor_no_prefs();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(
        preferred.len(),
        2,
        "expected two preferred extensions when no preference is declared"
    );
    // Both extensions include A2 (Misbehaves, unattacked) and B1 (Professor,
    // a premise). They differ on whether A3 (AccessDenied) or B2 (¬AccessDenied)
    // is committed to.
    let expected_access_denied: BTreeSet<ArgumentId> =
        [ids.a1, ids.a2, ids.a3, ids.b1].into_iter().collect();
    let expected_not_denied: BTreeSet<ArgumentId> =
        [ids.a1, ids.a2, ids.b1, ids.b2].into_iter().collect();
    let ext_sets: BTreeSet<BTreeSet<ArgumentId>> = preferred.iter().map(as_btree).collect();
    assert!(
        ext_sets.contains(&expected_access_denied),
        "expected extension containing AccessDenied (A3)"
    );
    assert!(
        ext_sets.contains(&expected_not_denied),
        "expected extension containing ¬AccessDenied (B2)"
    );
}

#[test]
fn snores_professor_no_prefs_grounded_is_just_premises() {
    // With mutual defeat between A3 and B2, A2 is unattacked, but AccessDenied
    // (A3) is not skeptically justified. The grounded extension contains
    // exactly what's unconditionally defensible: A1, A2, B1.
    let (built, ids) = snores_professor_no_prefs();
    let grounded = built.framework.grounded_extension();
    let expected: BTreeSet<ArgumentId> = [ids.a1, ids.a2, ids.b1].into_iter().collect();
    assert_eq!(as_btree(&grounded), expected);
}

// -----------------------------------------------------------------------------
// Example B: Married / Bachelor (M&P 2014 Example 4.1, §4.1.1)
//
// Kp = {WearsRing, PartyAnimal}
// Rd = { d1: WearsRing => Married, d2: PartyAnimal => Bachelor }
// Rs = { s1: Married -> ¬Bachelor, s2: Bachelor -> ¬Married }
// No preferences.
//
// This exercises the strict-wrap rebut fix (commit e387259): without it, the
// rebut attacks from ¬Bachelor to ¬Married (and vice versa) via strict-wrapped
// sub-arguments are missed, producing an incoherent grounded extension
// containing both ¬Bachelor AND ¬Married.
// -----------------------------------------------------------------------------

struct MarriedBachelorIds {
    a1: ArgumentId, // WearsRing (premise)
    a2: ArgumentId, // Married (via d1)
    a3: ArgumentId, // ¬Bachelor (via s1 on A2)
    b1: ArgumentId, // PartyAnimal (premise)
    b2: ArgumentId, // Bachelor (via d2)
    b3: ArgumentId, // ¬Married (via s2 on B2)
}

fn married_bachelor() -> (BuildOutput, MarriedBachelorIds) {
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("WearsRing"));
    sys.add_ordinary(Literal::atom("PartyAnimal"));
    sys.add_defeasible_rule(vec![Literal::atom("WearsRing")], Literal::atom("Married"));
    sys.add_defeasible_rule(
        vec![Literal::atom("PartyAnimal")],
        Literal::atom("Bachelor"),
    );
    sys.add_strict_rule(vec![Literal::atom("Married")], Literal::neg("Bachelor"));
    sys.add_strict_rule(vec![Literal::atom("Bachelor")], Literal::neg("Married"));
    let built = sys.build_framework().unwrap();
    let find = |concl: Literal| {
        built
            .arguments
            .iter()
            .find(|a| a.conclusion == concl)
            .unwrap()
            .id
    };
    let ids = MarriedBachelorIds {
        a1: find(Literal::atom("WearsRing")),
        a2: find(Literal::atom("Married")),
        a3: find(Literal::neg("Bachelor")),
        b1: find(Literal::atom("PartyAnimal")),
        b2: find(Literal::atom("Bachelor")),
        b3: find(Literal::neg("Married")),
    };
    (built, ids)
}

#[test]
fn married_bachelor_has_6_arguments() {
    let (built, _) = married_bachelor();
    assert_eq!(
        built.arguments.len(),
        6,
        "expected 6 arguments (A1..A3, B1..B3)"
    );
}

#[test]
fn married_bachelor_rebuts_include_strict_wrapper_propagation() {
    let (built, ids) = married_bachelor();
    let rebuts: Vec<_> = built
        .attacks
        .iter()
        .filter(|a| a.kind == AttackKind::Rebut)
        .collect();

    // Direct defeasible-topped rebuts:
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.a3 && a.target == ids.b2),
        "(A3, B2): ¬Bachelor rebuts Bachelor"
    );
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.b3 && a.target == ids.a2),
        "(B3, A2): ¬Married rebuts Married"
    );
    // Strict-wrapper propagation (the fix):
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.a3 && a.target == ids.b3),
        "(A3, B3): ¬Bachelor rebuts ¬Married via sub-argument B2"
    );
    assert!(
        rebuts
            .iter()
            .any(|a| a.attacker == ids.b3 && a.target == ids.a3),
        "(B3, A3): ¬Married rebuts ¬Bachelor via sub-argument A2"
    );
}

#[test]
fn married_bachelor_has_two_preferred_extensions() {
    // Without preferences, all four rebuts succeed as defeats. The stable
    // choice is to commit to one side entirely.
    let (built, ids) = married_bachelor();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 2, "expected two preferred extensions");
    let married_side: BTreeSet<ArgumentId> = [ids.a1, ids.a2, ids.a3, ids.b1].into_iter().collect();
    let bachelor_side: BTreeSet<ArgumentId> =
        [ids.a1, ids.b1, ids.b2, ids.b3].into_iter().collect();
    let ext_sets: BTreeSet<BTreeSet<ArgumentId>> = preferred.iter().map(as_btree).collect();
    assert!(
        ext_sets.contains(&married_side),
        "expected extension committing to Married/¬Bachelor"
    );
    assert!(
        ext_sets.contains(&bachelor_side),
        "expected extension committing to Bachelor/¬Married"
    );
}

#[test]
fn married_bachelor_grounded_is_premises_only() {
    // Because of the strict-wrap rebut propagation, A2, A3, B2, B3 are all
    // attacked; only the premises A1 and B1 are unconditionally defensible.
    let (built, ids) = married_bachelor();
    let grounded = built.framework.grounded_extension();
    let expected: BTreeSet<ArgumentId> = [ids.a1, ids.b1].into_iter().collect();
    assert_eq!(as_btree(&grounded), expected);
}

#[test]
fn married_bachelor_stable_equals_preferred() {
    let (built, _) = married_bachelor();
    let stable = built.framework.stable_extensions().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    let stable_sets: BTreeSet<BTreeSet<ArgumentId>> = stable.iter().map(as_btree).collect();
    let preferred_sets: BTreeSet<BTreeSet<ArgumentId>> = preferred.iter().map(as_btree).collect();
    assert_eq!(
        stable_sets, preferred_sets,
        "stable and preferred should coincide for this example"
    );
}

#[test]
fn married_bachelor_ideal_is_premises_only() {
    let (built, ids) = married_bachelor();
    let ideal = built.framework.ideal_extension().unwrap();
    let expected: BTreeSet<ArgumentId> = [ids.a1, ids.b1].into_iter().collect();
    assert_eq!(as_btree(&ideal), expected);
}

// -----------------------------------------------------------------------------
// Example D: Undercut rule via add_undercut_rule
//
// Kp = {rain, umbrella}
// Rd = { d1: rain => wet (defeasible),
//        d2: umbrella => ¬applicable(d1)  (undercut on d1) }
//
// Under last-link, undercut always succeeds — the wet-argument is defeated.
// Grounded contains: rain, umbrella, the undercut argument. NOT wet.
// -----------------------------------------------------------------------------

#[test]
fn undercut_rule_defeats_wet_argument() {
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("rain"));
    sys.add_ordinary(Literal::atom("umbrella"));
    let d1 = sys.add_defeasible_rule(vec![Literal::atom("rain")], Literal::atom("wet"));
    let _d2 = sys.add_undercut_rule(d1, vec![Literal::atom("umbrella")]);
    let built = sys.build_framework().unwrap();

    assert_eq!(
        built.arguments.len(),
        4,
        "expected 4 arguments: rain, umbrella, wet, ¬applicable(d1)"
    );

    let rain_arg = built
        .arguments
        .iter()
        .find(|a| a.conclusion == Literal::atom("rain"))
        .unwrap()
        .id;
    let umbrella_arg = built
        .arguments
        .iter()
        .find(|a| a.conclusion == Literal::atom("umbrella"))
        .unwrap()
        .id;
    let wet_arg = built
        .arguments
        .iter()
        .find(|a| a.conclusion == Literal::atom("wet"))
        .unwrap()
        .id;
    // The undercut argument's conclusion is the reserved marker — find it by
    // checking the surviving argument that isn't one of the others.
    let undercut_arg = built
        .arguments
        .iter()
        .find(|a| a.id != rain_arg && a.id != umbrella_arg && a.id != wet_arg)
        .unwrap()
        .id;

    // Exactly one undercut attack: the undercut-argument undercuts wet_arg.
    let undercuts: Vec<_> = built
        .attacks
        .iter()
        .filter(|a| a.kind == AttackKind::Undercut)
        .collect();
    assert_eq!(undercuts.len(), 1, "expected exactly one undercut attack");
    assert_eq!(undercuts[0].attacker, undercut_arg);
    assert_eq!(undercuts[0].target, wet_arg);

    // Grounded: rain, umbrella, undercut argument — NOT wet.
    let grounded = built.framework.grounded_extension();
    let grounded_set = as_btree(&grounded);
    assert!(grounded_set.contains(&rain_arg));
    assert!(grounded_set.contains(&umbrella_arg));
    assert!(grounded_set.contains(&undercut_arg));
    assert!(
        !grounded_set.contains(&wet_arg),
        "wet should be undercut-defeated"
    );
}

/// M&P 2014 Example 3.7 + Example 3.22: the "running example" used
/// throughout §3. Encodes the argumentation theory from §3.2 plus the
/// last-link preferences from Example 3.22:
///
///   Kn = {p}, Kp = {s, u, x}
///   Rd:  d1: p ⇒ q         d2: s ⇒ t         d3: t ⇒ ¬d1 (undercut)
///        d4: u ⇒ v         d5: v, x ⇒ ¬t     d6: s ⇒ ¬p
///   Rs:  s1: p, q → r      s2: v → ¬s
///
///   Priorities: d4 < d2, d2 < d5 (rule level)
///               u <' s, x <' s    (premise level)
///
/// Expected outcome (paper Example 3.18, scenario "D4 strictly defeats B2
/// AND C3 does not defeat B1"): the unique extension across all semantics
/// contains `{A1, A2, A3, B1, C1, C2, C3, D3, D4}`. r is skeptically
/// justified.
///
/// **Note 1 (premise preferences are decorative for our implementation).**
/// The paper attributes "C3 does not defeat B1" to the premise ordering
/// `u <' s`. In our crate, the same outcome is derived from M&P Def 3.19
/// rule 2: C3's last-defeasible-rule frontier is `{d4}` (non-empty)
/// while B1's is `∅` (B1 is a premise leaf), and a non-empty frontier is
/// strictly less preferred than an empty one — so C3 ≺ B1 by rule-level
/// reasoning alone, no premise ordering required. We retain the
/// `prefer_premise(s, u)` and `prefer_premise(s, x)` calls below for
/// paper fidelity, but they could be removed without changing any defeat
/// outcome in this example. Premise-level preference behaviour is
/// validated by the dedicated unit test
/// `premise_preference_blocks_undermine_when_target_premise_stronger`
/// in `src/aspic/defeat.rs`.
///
/// **Note 2 (the resulting extension violates direct consistency).**
/// Per M&P 2014 Example 4.4, the running example as encoded above
/// produces an extension containing both `s` (B1) and `¬s` (C3) — a
/// direct consistency violation. The paper notes this is the expected
/// outcome when `Rs` is not closed under transposition: adding `s → ¬v`
/// (the transposition of `s2: v → ¬s`) would let B1 be continued to a
/// `¬v` argument that successfully rebuts C3 on C2, excluding the
/// consistency-violating extensions. The
/// `running_example_with_transposition_satisfies_postulates` test below
/// demonstrates the fix.
fn running_example() -> argumentation::aspic::BuildOutput {
    let mut sys = StructuredSystem::new();
    sys.add_necessary(Literal::atom("p"));
    sys.add_ordinary(Literal::atom("s"));
    sys.add_ordinary(Literal::atom("u"));
    sys.add_ordinary(Literal::atom("x"));
    let d1 = sys.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
    let d2 = sys.add_defeasible_rule(vec![Literal::atom("s")], Literal::atom("t"));
    let _d3 = sys.add_undercut_rule(d1, vec![Literal::atom("t")]);
    let d4 = sys.add_defeasible_rule(vec![Literal::atom("u")], Literal::atom("v"));
    let d5 = sys.add_defeasible_rule(
        vec![Literal::atom("v"), Literal::atom("x")],
        Literal::neg("t"),
    );
    let _d6 = sys.add_defeasible_rule(vec![Literal::atom("s")], Literal::neg("p"));
    let _s1 = sys.add_strict_rule(
        vec![Literal::atom("p"), Literal::atom("q")],
        Literal::atom("r"),
    );
    let _s2 = sys.add_strict_rule(vec![Literal::atom("v")], Literal::neg("s"));
    sys.prefer_rule(d2, d4).unwrap();
    sys.prefer_rule(d5, d2).unwrap();
    sys.prefer_premise(Literal::atom("s"), Literal::atom("u"))
        .unwrap();
    sys.prefer_premise(Literal::atom("s"), Literal::atom("x"))
        .unwrap();
    sys.build_framework().unwrap()
}

/// M&P 2014 Example 3.26: the Scotland/Whisky case.
///
///   Kp = {BornInScotland, FitnessLover}
///   Rd:  d1: BornInScotland ⇒ Scottish
///        d2: Scottish ⇒ LikesWhisky
///        d3: FitnessLover ⇒ ¬LikesWhisky
///
///   Priorities: BornInScotland <' FitnessLover, d1 < d2, d1 < d3, d3 < d2
///
/// Under LAST-LINK (default), comparison is {d2} vs {d3}: since d3 < d2,
/// A3 strictly defeats B2 → conclude LikesWhisky.
///
/// Under WEAKEST-LINK, comparison is {d1, d2} vs {d3} with the premise
/// ordering also factored in. Per the paper, B2 ≻ A3 → conclude ¬LikesWhisky.
fn build_scotland_whisky(
    ordering: argumentation::aspic::DefeatOrdering,
) -> argumentation::aspic::BuildOutput {
    use argumentation::aspic::StructuredSystem;
    let mut sys = StructuredSystem::with_ordering(ordering);
    sys.add_ordinary(Literal::atom("BornInScotland"));
    sys.add_ordinary(Literal::atom("FitnessLover"));
    let d1 = sys.add_defeasible_rule(
        vec![Literal::atom("BornInScotland")],
        Literal::atom("Scottish"),
    );
    let d2 = sys.add_defeasible_rule(
        vec![Literal::atom("Scottish")],
        Literal::atom("LikesWhisky"),
    );
    let d3 = sys.add_defeasible_rule(
        vec![Literal::atom("FitnessLover")],
        Literal::neg("LikesWhisky"),
    );
    sys.prefer_rule(d2, d1).unwrap();
    sys.prefer_rule(d3, d1).unwrap();
    sys.prefer_rule(d2, d3).unwrap();
    sys.prefer_premise(
        Literal::atom("FitnessLover"),
        Literal::atom("BornInScotland"),
    )
    .unwrap();
    sys.build_framework().unwrap()
}

#[test]
fn whisky_last_link_concludes_likes_whisky() {
    use argumentation::aspic::DefeatOrdering;
    let built = build_scotland_whisky(DefeatOrdering::LastLink);
    let likes = built
        .argument_by_conclusion(&Literal::atom("LikesWhisky"))
        .expect("LikesWhisky argument");
    let grounded = built.framework.grounded_extension();
    assert!(
        grounded.contains(&likes.id),
        "last-link should accept LikesWhisky, got {:?}",
        grounded
    );
}

#[test]
fn whisky_weakest_link_concludes_not_likes_whisky() {
    use argumentation::aspic::DefeatOrdering;
    let built = build_scotland_whisky(DefeatOrdering::WeakestLink);
    let not_likes = built
        .argument_by_conclusion(&Literal::neg("LikesWhisky"))
        .expect("¬LikesWhisky argument");
    let grounded = built.framework.grounded_extension();
    assert!(
        grounded.contains(&not_likes.id),
        "weakest-link should accept ¬LikesWhisky, got {:?}",
        grounded
    );
}

#[test]
fn penguin_preferred_extension_satisfies_rationality_postulates() {
    use argumentation::aspic::StructuredSystem;
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("penguin"));
    sys.add_strict_rule(vec![Literal::atom("penguin")], Literal::atom("bird"));
    let r1 = sys.add_defeasible_rule(vec![Literal::atom("bird")], Literal::atom("flies"));
    let r2 = sys.add_defeasible_rule(vec![Literal::atom("penguin")], Literal::neg("flies"));
    sys.prefer_rule(r2, r1).unwrap();

    let built = sys.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 1);
    let report = built.check_postulates(&preferred[0]);
    assert!(
        report.is_clean(),
        "penguin preferred extension should satisfy postulates, got {:?}",
        report.violations
    );
}

#[test]
fn married_bachelor_preferred_extensions_satisfy_postulates() {
    use argumentation::aspic::StructuredSystem;
    let mut sys = StructuredSystem::new();
    sys.add_ordinary(Literal::atom("WearsRing"));
    sys.add_ordinary(Literal::atom("PartyAnimal"));
    sys.add_defeasible_rule(vec![Literal::atom("WearsRing")], Literal::atom("Married"));
    sys.add_defeasible_rule(
        vec![Literal::atom("PartyAnimal")],
        Literal::atom("Bachelor"),
    );
    sys.add_strict_rule(vec![Literal::atom("Married")], Literal::neg("Bachelor"));
    sys.add_strict_rule(vec![Literal::atom("Bachelor")], Literal::neg("Married"));
    let built = sys.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    for ext in &preferred {
        let report = built.check_postulates(ext);
        assert!(
            report.is_clean(),
            "married/bachelor extension {:?} failed postulates: {:?}",
            ext,
            report.violations
        );
    }
}

#[test]
fn running_example_r_is_in_grounded_extension() {
    // M&P 2014 Example 3.18 + 4.4: paper-faithful pin of the running
    // example outcome. The grounded extension contains the full set
    // `{A1, A2, A3, B1, C1, C2, C3, D3, D4}` from Example 3.18, which
    // means BOTH `s` (B1) and `¬s` (C3) are in the extension — a direct
    // consistency violation that Example 4.4 explicitly identifies and
    // attributes to the strict-rule set not being closed under
    // transposition.
    let built = running_example();
    let grounded = built.framework.grounded_extension();

    // Locate the key arguments by conclusion. (B1, B2, B3 are the
    // s-derived chain; C1-C3 the u-derived chain; D3-D4 the x-derived
    // chain. We pin the ones the paper explicitly mentions.)
    let r_arg = built.argument_by_conclusion(&Literal::atom("r")).unwrap();
    let s_arg = built.argument_by_conclusion(&Literal::atom("s")).unwrap();
    let not_s_arg = built.argument_by_conclusion(&Literal::neg("s")).unwrap();
    let p_arg = built.argument_by_conclusion(&Literal::atom("p")).unwrap();
    let q_arg = built.argument_by_conclusion(&Literal::atom("q")).unwrap();
    let v_arg = built.argument_by_conclusion(&Literal::atom("v")).unwrap();
    let not_t_arg = built.argument_by_conclusion(&Literal::neg("t")).unwrap();

    // Paper Example 3.18: r is skeptically justified.
    assert!(
        grounded.contains(&r_arg.id),
        "expected r-argument (A3) in grounded, got {:?}",
        grounded
    );
    // Paper Example 3.18: B1 (s premise) is in the extension because
    // C3 does not defeat it. This is the load-bearing assertion for
    // Def 3.19 rule 2 (empty rule frontier dominates non-empty).
    assert!(
        grounded.contains(&s_arg.id),
        "expected s-argument (B1) in grounded — C3 should not defeat B1 \
         because B1's empty rule frontier dominates C3's {{d4}} \
         (M&P Def 3.19 rule 2)"
    );
    // C3 (concluding ¬s via the u-chain) is also in the extension.
    assert!(
        grounded.contains(&not_s_arg.id),
        "expected ¬s-argument (C3) in grounded"
    );
    // Other supporting arguments from the paper's `S` set.
    assert!(grounded.contains(&p_arg.id), "p (A1) should be in grounded");
    assert!(grounded.contains(&q_arg.id), "q (A2) should be in grounded");
    assert!(grounded.contains(&v_arg.id), "v (C2) should be in grounded");
    assert!(
        grounded.contains(&not_t_arg.id),
        "¬t (D4) should be in grounded — D4 strictly defeats B2 via d5 > d2"
    );

    // Paper Example 4.4: this extension violates direct consistency
    // because the strict rules are not closed under transposition.
    // Pin that the postulate checker correctly detects this.
    let report = built.check_postulates(&grounded);
    assert!(
        report.violations.iter().any(|v| matches!(
            v,
            argumentation::aspic::PostulateViolation::DirectInconsistency {
                literal,
            } if literal == &Literal::atom("s")
        )),
        "expected DirectInconsistency on s, got {:?}",
        report.violations
    );
}

#[test]
fn running_example_with_transposition_satisfies_postulates() {
    // M&P 2014 Example 4.4 explicitly says: adding the transposition
    // `s → ¬v` of `v → ¬s` to Rs lets B1 be continued to a ¬v argument
    // that successfully rebuts C3 on C2, excluding the
    // consistency-violating extensions. This test demonstrates that fix.
    let mut sys = StructuredSystem::new();
    sys.add_necessary(Literal::atom("p"));
    sys.add_ordinary(Literal::atom("s"));
    sys.add_ordinary(Literal::atom("u"));
    sys.add_ordinary(Literal::atom("x"));
    let d1 = sys.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
    let d2 = sys.add_defeasible_rule(vec![Literal::atom("s")], Literal::atom("t"));
    let _d3 = sys.add_undercut_rule(d1, vec![Literal::atom("t")]);
    let d4 = sys.add_defeasible_rule(vec![Literal::atom("u")], Literal::atom("v"));
    let d5 = sys.add_defeasible_rule(
        vec![Literal::atom("v"), Literal::atom("x")],
        Literal::neg("t"),
    );
    let _d6 = sys.add_defeasible_rule(vec![Literal::atom("s")], Literal::neg("p"));
    let _s1 = sys.add_strict_rule(
        vec![Literal::atom("p"), Literal::atom("q")],
        Literal::atom("r"),
    );
    let _s2 = sys.add_strict_rule(vec![Literal::atom("v")], Literal::neg("s"));
    // The transposition: s → ¬v.
    let _s3 = sys.add_strict_rule(vec![Literal::atom("s")], Literal::neg("v"));
    sys.prefer_rule(d2, d4).unwrap();
    sys.prefer_rule(d5, d2).unwrap();
    sys.prefer_premise(Literal::atom("s"), Literal::atom("u"))
        .unwrap();
    sys.prefer_premise(Literal::atom("s"), Literal::atom("x"))
        .unwrap();

    let built = sys.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert!(
        !preferred.is_empty(),
        "expected at least one preferred extension"
    );
    // Every preferred extension must satisfy direct consistency now —
    // the transposition rule should exclude any extension containing
    // both s and ¬s.
    for ext in &preferred {
        let report = built.check_postulates(ext);
        let has_direct_inconsistency = report.violations.iter().any(|v| {
            matches!(
                v,
                argumentation::aspic::PostulateViolation::DirectInconsistency { .. }
            )
        });
        assert!(
            !has_direct_inconsistency,
            "preferred extension {:?} still has direct inconsistency: {:?}",
            ext, report.violations
        );
    }
}
