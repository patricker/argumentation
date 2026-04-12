//! Attack relations between ASPIC+ arguments.
//!
//! Per Modgil & Prakken 2014 §3.3.1 Definition 3.10:
//! - **Undermining:** `A` undermines `B` (on `ϕ`) iff `A`'s conclusion is the
//!   contrary of some *ordinary premise* `ϕ` used anywhere in `B`'s tree.
//! - **Undercutting:** `A` undercuts `B` (on `B'`) iff `A`'s conclusion
//!   expresses "the defeasible rule `r` does not apply," for some defeasible
//!   rule `r` used in a defeasible-topped sub-argument `B' ∈ Sub(B)`. Our
//!   encoding uses the reserved literal namespace `¬__applicable_<rule_id>`;
//!   consumers should use
//!   [`crate::aspic::StructuredSystem::add_undercut_rule`] rather than
//!   constructing this literal by hand.
//! - **Rebutting:** `A` rebuts `B` (on `B'`) iff `A`'s conclusion is the
//!   contrary of some sub-argument `B' ∈ Sub(B)`'s conclusion, and `B'`'s
//!   top rule is defeasible.
//!
//! **All three attack kinds are recorded against the outer target `B`**, not
//! just against the sub-argument where the conflict lands. This matches the
//! paper's "A rebuts B on B'" phrasing: the Dung AF edge is `(A, B)`. Because
//! `construct_arguments` also materialises every intermediate sub-argument as
//! its own `Argument` with its own id, the (attacker, target) iteration
//! additionally yields `(A, B')` as a separate edge — both edges exist in the
//! resulting framework, which is consistent with the paper's treatment of
//! every sub-argument as an independent argument.
//!
//! Historical note: an earlier version of this module only checked the
//! target's top-level conclusion for rebut, which produced logically
//! inconsistent extensions when strict rules wrapped defeasible
//! sub-arguments (e.g. Example 4.1 Married/Bachelor from M&P 2014). The
//! sub-argument traversal in [`compute_attacks`]'s rebut loop is the fix.

use super::argument::{Argument, ArgumentId, Origin};
use super::kb::Premise;
use super::rules::Rule;

/// The kind of attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackKind {
    /// Attacks an ordinary premise used by the target argument.
    Undermine,
    /// Attacks a defeasible rule used by the target argument.
    Undercut,
    /// Attacks the target's conclusion (and the target ends in a defeasible rule).
    Rebut,
}

impl std::fmt::Display for AttackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttackKind::Undermine => write!(f, "undermine"),
            AttackKind::Undercut => write!(f, "undercut"),
            AttackKind::Rebut => write!(f, "rebut"),
        }
    }
}

/// An attack between two ASPIC+ arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attack {
    /// The attacking argument.
    pub attacker: ArgumentId,
    /// The argument being attacked.
    pub target: ArgumentId,
    /// The kind of attack.
    pub kind: AttackKind,
}

/// Recursively collect all defeasible rules used in an argument's construction.
fn defeasible_rules_used(
    arg: &Argument,
    args: &[Argument],
    rules: &[Rule],
) -> Vec<super::rules::RuleId> {
    let mut out = Vec::new();
    match &arg.origin {
        Origin::Premise(_) => {}
        Origin::RuleApplication(rid) => {
            if let Some(r) = rules.iter().find(|r| r.id == *rid)
                && r.is_defeasible()
            {
                out.push(*rid);
            }
            for sub_id in &arg.sub_arguments {
                if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                    out.extend(defeasible_rules_used(sub, args, rules));
                }
            }
        }
    }
    out
}

/// Recursively collect all ordinary premises used in an argument.
fn ordinary_premises_used<'a>(arg: &'a Argument, args: &'a [Argument]) -> Vec<&'a Premise> {
    let mut out = Vec::new();
    match &arg.origin {
        Origin::Premise(p) => {
            if p.is_defeasible() {
                out.push(p);
            }
        }
        Origin::RuleApplication(_) => {
            for sub_id in &arg.sub_arguments {
                if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                    out.extend(ordinary_premises_used(sub, args));
                }
            }
        }
    }
    out
}

/// Recursively collect `arg` and all its sub-arguments (transitive closure).
///
/// Used by the rebut check to find defeasible-topped sub-arguments whose
/// conclusion the attacker might contradict; per M&P 2014 §3.3.1, rebutting
/// on a sub-argument is an attack on the outer parent.
fn all_sub_arguments<'a>(arg: &'a Argument, args: &'a [Argument]) -> Vec<&'a Argument> {
    let mut out = vec![arg];
    for sub_id in &arg.sub_arguments {
        if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
            out.extend(all_sub_arguments(sub, args));
        }
    }
    out
}

/// Compute all attacks between arguments.
pub fn compute_attacks(args: &[Argument], rules: &[Rule]) -> Vec<Attack> {
    // Precompute per-target helper results once. Each is indexed by
    // ArgumentId so the inner loop is O(1) lookup.
    let mut premises_by_target: std::collections::HashMap<ArgumentId, Vec<&Premise>> =
        std::collections::HashMap::with_capacity(args.len());
    let mut defeasible_rules_by_target: std::collections::HashMap<
        ArgumentId,
        Vec<super::rules::RuleId>,
    > = std::collections::HashMap::with_capacity(args.len());
    let mut subs_by_target: std::collections::HashMap<ArgumentId, Vec<&Argument>> =
        std::collections::HashMap::with_capacity(args.len());
    for target in args {
        premises_by_target.insert(target.id, ordinary_premises_used(target, args));
        defeasible_rules_by_target.insert(target.id, defeasible_rules_used(target, args, rules));
        subs_by_target.insert(target.id, all_sub_arguments(target, args));
    }

    let mut attacks = Vec::new();
    for attacker in args {
        for target in args {
            if attacker.id == target.id {
                continue;
            }
            // Rebut: attacker's conclusion is contrary of some
            // defeasible-topped sub-argument of target. Per M&P 2014 §3.3.1
            // Def 3.10, the edge is (attacker, target) regardless of
            // whether the rebut lands at the top or deeper in the tree.
            let subs = subs_by_target.get(&target.id).expect("target precomputed");
            if subs.iter().any(|sub| {
                attacker.conclusion.is_contrary_of(&sub.conclusion)
                    && sub.top_rule_is_defeasible(rules)
            }) {
                attacks.push(Attack {
                    attacker: attacker.id,
                    target: target.id,
                    kind: AttackKind::Rebut,
                });
            }
            // Undermine: attacker's conclusion is contrary of an ordinary
            // premise used anywhere in target's sub-tree.
            let target_premises = premises_by_target
                .get(&target.id)
                .expect("target precomputed");
            if target_premises
                .iter()
                .any(|prem| attacker.conclusion.is_contrary_of(prem.literal()))
            {
                attacks.push(Attack {
                    attacker: attacker.id,
                    target: target.id,
                    kind: AttackKind::Undermine,
                });
            }
            // Undercut: attacker's conclusion is the reserved
            // `¬__applicable_<rule_id>` marker for some defeasible rule used
            // in target's sub-tree.
            let target_rules = defeasible_rules_by_target
                .get(&target.id)
                .expect("target precomputed");
            if target_rules
                .iter()
                .any(|rid| attacker.conclusion == super::language::Literal::undercut_marker(rid.0))
            {
                attacks.push(Attack {
                    attacker: attacker.id,
                    target: target.id,
                    kind: AttackKind::Undercut,
                });
            }
        }
    }
    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspic::argument::construct_arguments;
    use crate::aspic::kb::KnowledgeBase;
    use crate::aspic::language::Literal;
    use crate::aspic::rules::{Rule, RuleId};

    #[test]
    fn rebut_detected_between_contrary_conclusions() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("r"));
        let rules = vec![
            Rule::defeasible(RuleId(0), vec![Literal::atom("p")], Literal::atom("q")),
            Rule::defeasible(RuleId(1), vec![Literal::atom("r")], Literal::neg("q")),
        ];
        let args = construct_arguments(&kb, &rules).unwrap();
        let attacks = compute_attacks(&args, &rules);
        let rebuts: Vec<&Attack> = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Rebut)
            .collect();
        assert!(rebuts.len() >= 2);
    }

    #[test]
    fn undermine_detected_on_ordinary_premise() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("r"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("r")],
            Literal::neg("p"),
        )];
        let args = construct_arguments(&kb, &rules).unwrap();
        let attacks = compute_attacks(&args, &rules);
        let undermines: Vec<&Attack> = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Undermine)
            .collect();
        assert!(!undermines.is_empty());
    }

    #[test]
    fn undercut_detected_via_reserved_marker() {
        // Target: defeasible rule r0: p => q. Arg `q` uses r0.
        // Attacker: defeasible rule concluding ¬__applicable_0 (the reserved
        // undercut marker for rule id 0). This is what `add_undercut_rule`
        // produces internally, but we construct it here by hand so we're
        // testing the attack-detection path independently.
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("trigger"));
        let rules = vec![
            Rule::defeasible(RuleId(0), vec![Literal::atom("p")], Literal::atom("q")),
            Rule::defeasible(
                RuleId(1),
                vec![Literal::atom("trigger")],
                Literal::undercut_marker(0),
            ),
        ];
        let args = construct_arguments(&kb, &rules).unwrap();
        let attacks = compute_attacks(&args, &rules);
        let undercuts: Vec<&Attack> = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Undercut)
            .collect();
        assert!(
            !undercuts.is_empty(),
            "expected at least one Undercut attack, got {:?}",
            attacks
        );
        // The undercut must target the q-argument (built from r0).
        let q_arg = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("q"))
            .unwrap();
        assert!(
            undercuts.iter().any(|u| u.target == q_arg.id),
            "expected an undercut attack targeting the q-argument"
        );
    }

    #[test]
    fn rebut_propagates_through_strict_wrapper() {
        // Regression test for M&P 2014 Example 4.1 (Married/Bachelor).
        //
        // KB: WearsRing, PartyAnimal (both ordinary)
        // Rd: d1: WearsRing ⇒ Married
        //     d2: PartyAnimal ⇒ Bachelor
        // Rs: s1: Married → ¬Bachelor
        //     s2: Bachelor → ¬Married
        //
        // Arguments:
        //   A1 = WearsRing (premise)
        //   A2 = A1 ⇒ Married (via d1, defeasible top)
        //   A3 = A2 → ¬Bachelor (via s1, strict top)
        //   B1 = PartyAnimal (premise)
        //   B2 = B1 ⇒ Bachelor (via d2, defeasible top)
        //   B3 = B2 → ¬Married (via s2, strict top)
        //
        // The paper says "A3 rebuts B3 on its subargument B2, and B3 rebuts
        // A3 on its subargument A2." So in the Dung AF we must have BOTH:
        //   - (A3, B2) and (A3, B3) for the A3-rebuts-B3-on-B2 claim
        //   - (B3, A2) and (B3, A3) for the B3-rebuts-A3-on-A2 claim
        //
        // A previous implementation only detected the sub-argument landing
        // (A3, B2) and (B3, A2), which left A3 and B3 unattacked in the AF —
        // producing a grounded extension that contained both ¬Bachelor AND
        // ¬Married while excluding both Married and Bachelor. Incoherent.
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("WearsRing"));
        kb.add_ordinary(Literal::atom("PartyAnimal"));
        let rules = vec![
            Rule::defeasible(
                RuleId(0),
                vec![Literal::atom("WearsRing")],
                Literal::atom("Married"),
            ),
            Rule::defeasible(
                RuleId(1),
                vec![Literal::atom("PartyAnimal")],
                Literal::atom("Bachelor"),
            ),
            Rule::strict(
                RuleId(2),
                vec![Literal::atom("Married")],
                Literal::neg("Bachelor"),
            ),
            Rule::strict(
                RuleId(3),
                vec![Literal::atom("Bachelor")],
                Literal::neg("Married"),
            ),
        ];
        let args = construct_arguments(&kb, &rules).unwrap();
        let all_attacks = compute_attacks(&args, &rules);
        let rebuts: Vec<&Attack> = all_attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Rebut)
            .collect();
        // Locate arguments by conclusion.
        let married = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("Married"))
            .unwrap();
        let bachelor = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("Bachelor"))
            .unwrap();
        let not_bachelor = args
            .iter()
            .find(|a| a.conclusion == Literal::neg("Bachelor"))
            .unwrap();
        let not_married = args
            .iter()
            .find(|a| a.conclusion == Literal::neg("Married"))
            .unwrap();

        // Direct sub-arg rebuts (pre-existing behaviour):
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == not_bachelor.id && r.target == bachelor.id),
            "expected ¬Bachelor rebuts Bachelor (direct sub)"
        );
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == not_married.id && r.target == married.id),
            "expected ¬Married rebuts Married (direct sub)"
        );

        // Strict-wrapper propagation (the fix):
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == not_bachelor.id && r.target == not_married.id),
            "expected ¬Bachelor rebuts ¬Married via sub-argument B2 (strict wrapper)"
        );
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == not_married.id && r.target == not_bachelor.id),
            "expected ¬Married rebuts ¬Bachelor via sub-argument A2 (strict wrapper)"
        );
    }

    #[test]
    fn attack_kind_displays_as_lowercase_word() {
        assert_eq!(format!("{}", AttackKind::Undermine), "undermine");
        assert_eq!(format!("{}", AttackKind::Undercut), "undercut");
        assert_eq!(format!("{}", AttackKind::Rebut), "rebut");
    }

    #[test]
    fn compute_attacks_is_stable_across_refactors() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("r"));
        kb.add_ordinary(Literal::atom("trigger"));
        let rules = vec![
            Rule::defeasible(RuleId(0), vec![Literal::atom("p")], Literal::atom("q")),
            Rule::defeasible(RuleId(1), vec![Literal::atom("r")], Literal::neg("q")),
            Rule::strict(
                RuleId(2),
                vec![Literal::atom("q")],
                Literal::atom("derived"),
            ),
            Rule::defeasible(
                RuleId(3),
                vec![Literal::atom("trigger")],
                Literal::undercut_marker(0),
            ),
        ];
        let args = construct_arguments(&kb, &rules).unwrap();
        let attacks = compute_attacks(&args, &rules);

        let rebuts = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Rebut)
            .count();
        let undermines = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Undermine)
            .count();
        let undercuts = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Undercut)
            .count();

        assert!(rebuts >= 3, "expected >= 3 rebuts, got {}", rebuts);
        assert_eq!(undermines, 0);
        // Undercut hits both the defeasible-topped argument using rule 0
        // directly (p ⇒ q) and its strict-wrapped parent (q → derived),
        // because rule 0 lives in the latter's sub-argument tree too.
        assert_eq!(undercuts, 2);
    }

    #[test]
    fn rebut_propagates_through_three_level_strict_wrapper() {
        // KB: p (ordinary), q (ordinary)
        // Rules:
        //   d1: p ⇒ x            (defeasible)
        //   s1: x → y            (strict; wraps d1)
        //   s2: y → z            (strict; wraps s1(d1))
        //   d2: q ⇒ ¬x           (defeasible, rebuts d1's x-conclusion)
        //
        // Arguments:
        //   A1: p          premise
        //   A2: p ⇒ x      d1
        //   A3: x → y      s1(A2)         [top strict, defeasible sub A2]
        //   A4: y → z      s2(A3)         [top strict, defeasible sub two levels down]
        //   B1: q          premise
        //   B2: q ⇒ ¬x     d2             [top defeasible]
        //
        // Expected: B2 rebuts A2 directly (same level), and per the strict-wrap
        // fix B2 must ALSO rebut A3 (via sub A2) AND A4 (via sub A2 two levels
        // deep). If the recursion stops at depth 1, the A4 rebut is missed and
        // A4 remains unattacked in the Dung AF — producing the same incoherent
        // extensions as the original strict-wrap bug.
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("q"));
        let rules = vec![
            Rule::defeasible(RuleId(0), vec![Literal::atom("p")], Literal::atom("x")),
            Rule::strict(RuleId(1), vec![Literal::atom("x")], Literal::atom("y")),
            Rule::strict(RuleId(2), vec![Literal::atom("y")], Literal::atom("z")),
            Rule::defeasible(RuleId(3), vec![Literal::atom("q")], Literal::neg("x")),
        ];
        let args = construct_arguments(&kb, &rules).unwrap();
        let attacks = compute_attacks(&args, &rules);
        let rebuts: Vec<&Attack> = attacks
            .iter()
            .filter(|a| a.kind == AttackKind::Rebut)
            .collect();

        let a2 = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("x"))
            .expect("x-argument");
        let a3 = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("y"))
            .expect("y-argument");
        let a4 = args
            .iter()
            .find(|a| a.conclusion == Literal::atom("z"))
            .expect("z-argument");
        let b2 = args
            .iter()
            .find(|a| a.conclusion == Literal::neg("x"))
            .expect("¬x-argument");

        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == b2.id && r.target == a2.id),
            "expected B2 rebuts A2 (direct)"
        );
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == b2.id && r.target == a3.id),
            "expected B2 rebuts A3 (1-level strict wrap over A2)"
        );
        assert!(
            rebuts
                .iter()
                .any(|r| r.attacker == b2.id && r.target == a4.id),
            "expected B2 rebuts A4 (2-level strict wrap over A2)"
        );
    }
}
