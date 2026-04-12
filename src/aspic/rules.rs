//! Strict and defeasible inference rules.

use super::language::Literal;

/// A rule: premises → conclusion, either strict or defeasible.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    /// Unique rule id within a rule set.
    pub id: RuleId,
    /// Premises (antecedents).
    pub premises: Vec<Literal>,
    /// Conclusion (consequent).
    pub conclusion: Literal,
    /// Whether the rule is strict (indefeasible) or defeasible.
    pub kind: RuleKind,
}

/// A rule id, unique within a `RuleSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleId(pub usize);

/// Whether a rule is strict or defeasible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleKind {
    /// Strict rules are indefeasible.
    Strict,
    /// Defeasible rules can be undercut.
    Defeasible,
}

impl Rule {
    /// Construct a strict rule.
    pub fn strict(id: RuleId, premises: Vec<Literal>, conclusion: Literal) -> Self {
        Self {
            id,
            premises,
            conclusion,
            kind: RuleKind::Strict,
        }
    }

    /// Construct a defeasible rule.
    pub fn defeasible(id: RuleId, premises: Vec<Literal>, conclusion: Literal) -> Self {
        Self {
            id,
            premises,
            conclusion,
            kind: RuleKind::Defeasible,
        }
    }

    /// Whether this rule is defeasible.
    pub fn is_defeasible(&self) -> bool {
        matches!(self.kind, RuleKind::Defeasible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_rule_is_not_defeasible() {
        let r = Rule::strict(RuleId(0), vec![Literal::atom("p")], Literal::atom("q"));
        assert!(!r.is_defeasible());
    }

    #[test]
    fn defeasible_rule_is_defeasible() {
        let r = Rule::defeasible(RuleId(1), vec![Literal::atom("p")], Literal::atom("q"));
        assert!(r.is_defeasible());
    }
}
