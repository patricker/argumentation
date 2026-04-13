//! Analogical schemes: analogy, classification, commitment.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::ANALOGICAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all analogical schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_analogy(),
        argument_from_verbal_classification(),
        argument_from_commitment(),
    ]
}

/// Argument from Analogy (Walton 2008 p.315).
pub fn argument_from_analogy() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET),
        name: "Argument from Analogy".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new("similar_case", "The analogous case", SlotRole::Property),
            PremiseSlot::new(
                "current_case",
                "The case being reasoned about",
                SlotRole::Property,
            ),
            PremiseSlot::new(
                "property",
                "The property that holds in the analogous case",
                SlotRole::Proposition,
            ),
        ],
        conclusion: ConclusionTemplate::positive(
            "?property also holds in ?current_case because it holds in ?similar_case",
            "?property",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Are ?similar_case and ?current_case truly similar in relevant respects?",
                Challenge::DisanalogyClaim,
            ),
            CriticalQuestion::new(
                2,
                "Is ?property the kind of thing that transfers between analogous cases?",
                Challenge::RuleValidity,
            ),
            CriticalQuestion::new(
                3,
                "Are there relevant differences between ?similar_case and ?current_case that block the analogy?",
                Challenge::DisanalogyClaim,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.315".into(),
            domain_tags: vec!["analogical".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Verbal Classification (Walton 2008 p.320).
pub fn argument_from_verbal_classification() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET + 1),
        name: "Argument from Verbal Classification".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new("subject", "The entity being classified", SlotRole::Agent),
            PremiseSlot::new(
                "classification",
                "The classification being applied",
                SlotRole::Property,
            ),
        ],
        conclusion: ConclusionTemplate::positive(
            "?subject has the properties associated with ?classification",
            "is_a_?classification_?subject",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Does ?subject actually fit the definition of ?classification?",
                Challenge::PremiseTruth("classification".into()),
            ),
            CriticalQuestion::new(
                2,
                "Is ?classification the right category for this context?",
                Challenge::RuleValidity,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.320".into(),
            domain_tags: vec!["analogical".into(), "definition".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Commitment (Walton 2008 p.322).
pub fn argument_from_commitment() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET + 2),
        name: "Argument from Commitment".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new(
                "agent",
                "The person who made the commitment",
                SlotRole::Agent,
            ),
            PremiseSlot::new(
                "commitment",
                "The commitment that was made",
                SlotRole::Action,
            ),
            PremiseSlot::new(
                "claim",
                "The claim that follows from the commitment",
                SlotRole::Proposition,
            ),
        ],
        conclusion: ConclusionTemplate::positive(
            "?agent should act consistently with ?commitment, therefore ?claim",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Did ?agent actually make ?commitment?",
                Challenge::PremiseTruth("commitment".into()),
            ),
            CriticalQuestion::new(
                2,
                "Does ?claim actually follow from ?commitment?",
                Challenge::RuleValidity,
            ),
            CriticalQuestion::new(
                3,
                "Have circumstances changed such that ?commitment no longer applies?",
                Challenge::AlternativeCause,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.322".into(),
            domain_tags: vec!["analogical".into(), "social_contract".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_three_analogical_schemes() {
        assert_eq!(all().len(), 3);
    }

    #[test]
    fn analogical_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= ANALOGICAL_ID_OFFSET);
            assert!(s.id.0 < ANALOGICAL_ID_OFFSET + 100);
        }
    }
}
