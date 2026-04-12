//! Popular schemes: social proof, tradition, precedent.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::POPULAR_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all popular schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_popular_opinion(),
        argument_from_tradition(),
        argument_from_precedent(),
        argument_from_established_rule(),
    ]
}

/// Argument from Popular Opinion (Walton 2008 p.311).
pub fn argument_from_popular_opinion() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET),
        name: "Argument from Popular Opinion".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("claim", "The claim widely accepted", SlotRole::Proposition),
            PremiseSlot::new("population", "The group that accepts the claim", SlotRole::Agent),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is plausible based on popular acceptance by ?population",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "What evidence supports that ?population actually accepts ?claim?", Challenge::PremiseTruth("population".into())),
            CriticalQuestion::new(2, "Is ?population's acceptance based on good reasoning?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Is ?claim the type of claim that popular acceptance makes more plausible?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.311".into(),
            domain_tags: vec!["popular".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Tradition (Walton 2008 p.316).
pub fn argument_from_tradition() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 1),
        name: "Argument from Tradition".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("practice", "The traditional practice", SlotRole::Action),
            PremiseSlot::new("tradition", "Evidence of longstanding tradition", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?practice should be continued based on ?tradition",
            "continue_?practice",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Has ?practice actually been a longstanding tradition?", Challenge::PremiseTruth("tradition".into())),
            CriticalQuestion::new(2, "Were the circumstances that justified ?practice still applicable?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Have conditions changed such that ?practice is no longer appropriate?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.316".into(),
            domain_tags: vec!["popular".into(), "tradition".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Precedent (Walton 2008 p.319).
pub fn argument_from_precedent() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 2),
        name: "Argument from Precedent".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("precedent_case", "The precedent case", SlotRole::Property),
            PremiseSlot::new("current_case", "The current situation", SlotRole::Property),
            PremiseSlot::new("action", "The action taken in the precedent", SlotRole::Action),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be taken in ?current_case as it was in ?precedent_case",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?current_case sufficiently similar to ?precedent_case?", Challenge::DisanalogyClaim),
            CriticalQuestion::new(2, "Was ?action the right decision in ?precedent_case?", Challenge::PremiseTruth("precedent_case".into())),
            CriticalQuestion::new(3, "Are there relevant differences between ?precedent_case and ?current_case?", Challenge::DisanalogyClaim),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.319".into(),
            domain_tags: vec!["popular".into(), "legal".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Established Rule (Walton 2008 p.318).
pub fn argument_from_established_rule() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 3),
        name: "Argument from Established Rule".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("rule", "The established rule or law", SlotRole::Property),
            PremiseSlot::new("case", "The case the rule applies to", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "The outcome prescribed by ?rule applies to ?case",
            "rule_applies_?case",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?rule actually apply to ?case?", Challenge::PremiseTruth("case".into())),
            CriticalQuestion::new(2, "Is ?rule still valid and in force?", Challenge::PremiseTruth("rule".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.318".into(),
            domain_tags: vec!["popular".into(), "legal".into(), "normative".into()],
            presumptive: true,
            strength: SchemeStrength::Strong,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_popular_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn established_rule_has_strong_strength() {
        assert_eq!(argument_from_established_rule().metadata.strength, SchemeStrength::Strong);
    }

    #[test]
    fn popular_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= POPULAR_ID_OFFSET);
            assert!(s.id.0 < POPULAR_ID_OFFSET + 100);
        }
    }
}
