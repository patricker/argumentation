//! Caminada–Amgoud (2007) rationality postulates for structured
//! argumentation.
//!
//! These postulates express properties that a "well-behaved" ASPIC+
//! system should satisfy. They are:
//!
//! 1. **Sub-argument closure**: if an extension contains an argument `A`,
//!    it contains every sub-argument of `A`.
//! 2. **Closure under strict rules**: if `S` is the set of conclusions of
//!    an extension, and `S ⊢ φ` via strict rules, then `φ` is also a
//!    conclusion of some argument in the extension.
//! 3. **Direct consistency**: no pair of contrary conclusions both appear
//!    in the extension.
//! 4. **Indirect consistency**: the closure of the extension's conclusions
//!    under strict rules is consistent.
//!
//! This module provides [`PostulateReport`] and a check function that
//! evaluates a given extension against all four postulates. Violations
//! typically signal a bug in the user's rule set (e.g. missing
//! transposition closure) rather than a bug in the crate.

use super::argument::{Argument, ArgumentId};
use super::language::Literal;
use super::rules::{Rule, RuleKind};
use std::collections::HashSet;

/// A report from checking the Caminada-Amgoud postulates against an
/// extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostulateReport {
    /// Violations found, if any. An empty vector means all postulates hold.
    pub violations: Vec<PostulateViolation>,
}

impl PostulateReport {
    /// Whether all four postulates hold.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// A specific postulate violation with a short human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostulateViolation {
    /// Argument `parent` is in the extension but its sub-argument `sub`
    /// is not. Violates sub-argument closure.
    SubArgumentNotInExtension {
        /// Argument that IS in the extension.
        parent: ArgumentId,
        /// Sub-argument of `parent` that is NOT in the extension.
        sub: ArgumentId,
    },
    /// The extension's conclusion set derives `literal` via strict rules
    /// but no argument in the extension concludes `literal`. Violates
    /// closure under strict rules.
    StrictClosureViolation {
        /// Literal derivable from the extension via strict rules but not
        /// itself a conclusion of any extension member.
        missing: Literal,
    },
    /// Both `literal` and its contrary appear in the extension's
    /// conclusion set. Violates direct consistency.
    DirectInconsistency {
        /// One of the conflicting literals (the positive side, by
        /// convention, to avoid duplicate reports).
        literal: Literal,
    },
    /// The strict-rule closure of the extension's conclusions is
    /// inconsistent (contains both a literal and its contrary).
    IndirectInconsistency {
        /// One of the conflicting literals.
        literal: Literal,
    },
}

/// Check all four rationality postulates against the given extension.
pub fn check_postulates(
    arguments: &[Argument],
    rules: &[Rule],
    extension: &HashSet<ArgumentId>,
) -> PostulateReport {
    let mut violations = Vec::new();

    // Postulate 1: sub-argument closure.
    for arg in arguments {
        if !extension.contains(&arg.id) {
            continue;
        }
        for sub_id in &arg.sub_arguments {
            if !extension.contains(sub_id) {
                violations.push(PostulateViolation::SubArgumentNotInExtension {
                    parent: arg.id,
                    sub: *sub_id,
                });
            }
        }
    }

    // Collect the extension's conclusions.
    let conclusions: HashSet<Literal> = arguments
        .iter()
        .filter(|a| extension.contains(&a.id))
        .map(|a| a.conclusion.clone())
        .collect();

    // Postulate 3: direct consistency.
    for lit in &conclusions {
        if conclusions.contains(&lit.contrary()) && !matches!(lit, Literal::Neg(_)) {
            violations.push(PostulateViolation::DirectInconsistency {
                literal: lit.clone(),
            });
        }
    }

    // Postulate 2: closure under strict rules.
    let closure = strict_closure(&conclusions, rules);
    for lit in &closure {
        if !conclusions.contains(lit) {
            violations.push(PostulateViolation::StrictClosureViolation {
                missing: lit.clone(),
            });
        }
    }

    // Postulate 4: indirect consistency.
    for lit in &closure {
        if closure.contains(&lit.contrary()) && !matches!(lit, Literal::Neg(_)) {
            violations.push(PostulateViolation::IndirectInconsistency {
                literal: lit.clone(),
            });
        }
    }

    PostulateReport { violations }
}

/// Compute the closure of `initial` under strict rules.
fn strict_closure(initial: &HashSet<Literal>, rules: &[Rule]) -> HashSet<Literal> {
    let mut closure = initial.clone();
    loop {
        let before = closure.len();
        for rule in rules.iter().filter(|r| r.kind == RuleKind::Strict) {
            if rule.premises.iter().all(|p| closure.contains(p)) {
                closure.insert(rule.conclusion.clone());
            }
        }
        if closure.len() == before {
            return closure;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspic::argument::construct_arguments;
    use crate::aspic::kb::KnowledgeBase;
    use crate::aspic::rules::{Rule, RuleId};

    #[test]
    fn clean_system_has_no_violations() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("p")],
            Literal::atom("q"),
        )];
        let args = construct_arguments(&kb, &rules).unwrap();
        let extension: HashSet<ArgumentId> = args.iter().map(|a| a.id).collect();
        let report = check_postulates(&args, &rules, &extension);
        assert!(
            report.is_clean(),
            "expected clean report, got {:?}",
            report.violations
        );
    }

    #[test]
    fn direct_inconsistency_is_detected() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("r"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("r")],
            Literal::neg("p"),
        )];
        let args = construct_arguments(&kb, &rules).unwrap();
        let extension: HashSet<ArgumentId> = args.iter().map(|a| a.id).collect();
        let report = check_postulates(&args, &rules, &extension);
        assert!(
            report
                .violations
                .iter()
                .any(|v| matches!(v, PostulateViolation::DirectInconsistency { .. })),
            "expected direct inconsistency, got {:?}",
            report.violations
        );
    }
}
