//! Attack relations between ASPIC+ arguments.
//!
//! Per Modgil & Prakken 2014 §3.2:
//! - **Undermining:** `A` undermines `B` iff `A`'s conclusion is the contrary
//!   of some *ordinary premise* used by `B`.
//! - **Undercutting:** `A` undercuts `B` iff `A`'s conclusion expresses "the
//!   defeasible rule `r` does not apply," for some defeasible rule `r` used
//!   in `B`. Our encoding uses a reserved literal namespace
//!   `¬__applicable_<rule_id>`. Consumers should use
//!   [`crate::aspic::StructuredSystem::add_undercut_rule`] rather than
//!   constructing this literal by hand.
//! - **Rebutting:** `A` rebuts `B` iff `A`'s conclusion is the contrary of
//!   `B`'s conclusion, and `B`'s top rule is defeasible.
//!
//! **Note on strict-topped arguments and rebut.** An argument `B` that ends
//! in a strict rule cannot be directly rebut at its top level (because the
//! definition requires the top rule to be defeasible). However, because
//! `construct_arguments` materialises every intermediate sub-argument as its
//! own `Argument` with its own id, rebut attacks against *defeasible
//! sub-arguments* of `B` are picked up correctly by iterating `(attacker,
//! target)` over all argument pairs. Don't "optimise" this loop by skipping
//! sub-arguments; you'd lose attack coverage.

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

/// Compute all attacks between arguments.
pub fn compute_attacks(args: &[Argument], rules: &[Rule]) -> Vec<Attack> {
    let mut attacks = Vec::new();
    for attacker in args {
        for target in args {
            if attacker.id == target.id {
                continue;
            }
            // Rebut: attacker's conclusion is contrary of target's conclusion,
            // and target ends in a defeasible rule.
            if attacker.conclusion.is_contrary_of(&target.conclusion)
                && target.top_rule_is_defeasible(rules)
            {
                attacks.push(Attack {
                    attacker: attacker.id,
                    target: target.id,
                    kind: AttackKind::Rebut,
                });
            }
            // Undermine: attacker's conclusion is contrary of an ordinary premise
            // used by target.
            for prem in ordinary_premises_used(target, args) {
                if attacker.conclusion.is_contrary_of(prem.literal()) {
                    attacks.push(Attack {
                        attacker: attacker.id,
                        target: target.id,
                        kind: AttackKind::Undermine,
                    });
                    break;
                }
            }
            // Undercut: attacker's conclusion is the reserved literal
            // `¬__applicable_<rule_id>` for some defeasible rule used by target.
            // This namespace is reserved; see add_undercut_rule in defeat.rs.
            for used in defeasible_rules_used(target, args, rules) {
                let undercut_marker =
                    super::language::Literal::neg(format!("__applicable_{}", used.0));
                if attacker.conclusion == undercut_marker {
                    attacks.push(Attack {
                        attacker: attacker.id,
                        target: target.id,
                        kind: AttackKind::Undercut,
                    });
                    break;
                }
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
}
