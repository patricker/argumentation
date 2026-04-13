//! Source-based schemes: attacking or bolstering the person, not the argument.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 5 + Appendix 1.

use crate::catalog::SOURCE_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all source-based schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        ad_hominem(),
        ad_hominem_circumstantial(),
        argument_from_bias(),
        ethotic_argument(),
    ]
}

/// Ad Hominem — generic (Walton 2008 p.141).
///
/// Negated-conclusion scheme: target has flaw F, therefore ¬claim.
pub fn ad_hominem() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET),
        name: "Ad Hominem".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("target", "The person being attacked", SlotRole::Agent),
            PremiseSlot::new("flaw", "The character flaw alleged", SlotRole::Property),
            PremiseSlot::new(
                "claim",
                "The claim being challenged via the attack",
                SlotRole::Proposition,
            ),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim should be rejected because ?target has ?flaw",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Does ?target really have ?flaw?",
                Challenge::PremiseTruth("flaw".into()),
            ),
            CriticalQuestion::new(
                2,
                "Does ?flaw actually bear on the credibility of ?claim?",
                Challenge::Proportionality,
            ),
            CriticalQuestion::new(
                3,
                "Is the attack on ?target proportionate to ?flaw?",
                Challenge::Proportionality,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.141".into(),
            domain_tags: vec!["source".into(), "attack".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Ad Hominem — circumstantial (Walton 2008 p.143).
///
/// Negated-conclusion scheme.
pub fn ad_hominem_circumstantial() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 1),
        name: "Ad Hominem Circumstantial".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new(
                "target",
                "The person whose circumstances are cited",
                SlotRole::Agent,
            ),
            PremiseSlot::new(
                "inconsistency",
                "How target's circumstances conflict with the claim",
                SlotRole::Property,
            ),
            PremiseSlot::new("claim", "The claim being undermined", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim is undermined because ?target's circumstances (?inconsistency) are inconsistent with it",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Does ?target actually have the alleged ?inconsistency?",
                Challenge::PremiseTruth("inconsistency".into()),
            ),
            CriticalQuestion::new(
                2,
                "Is the inconsistency relevant to ?claim?",
                Challenge::Proportionality,
            ),
            CriticalQuestion::new(
                3,
                "Could ?target's ?claim still be valid despite the inconsistency?",
                Challenge::RuleValidity,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.143".into(),
            domain_tags: vec!["source".into(), "attack".into(), "hypocrisy".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Bias (Walton 2008 p.340).
///
/// Negated-conclusion scheme.
pub fn argument_from_bias() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 2),
        name: "Argument from Bias".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("source", "The biased source", SlotRole::Agent),
            PremiseSlot::new("bias", "The alleged bias", SlotRole::Property),
            PremiseSlot::new(
                "claim",
                "The claim made by the biased source",
                SlotRole::Proposition,
            ),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim should be treated with suspicion because ?source has ?bias",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Does ?source actually have the alleged ?bias?",
                Challenge::PremiseTruth("bias".into()),
            ),
            CriticalQuestion::new(
                2,
                "Does ?bias actually affect ?source's assertion of ?claim?",
                Challenge::RuleValidity,
            ),
            CriticalQuestion::new(
                3,
                "Even if ?source is biased, might ?claim still be true?",
                Challenge::AlternativeCause,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.340".into(),
            domain_tags: vec!["source".into(), "attack".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Ethotic Argument — positive (Walton 2008 p.146).
pub fn ethotic_argument() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 3),
        name: "Ethotic Argument".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new(
                "person",
                "The person whose character is cited",
                SlotRole::Agent,
            ),
            PremiseSlot::new(
                "good_character",
                "The positive character trait",
                SlotRole::Property,
            ),
            PremiseSlot::new(
                "claim",
                "The claim bolstered by good character",
                SlotRole::Proposition,
            ),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is more plausible because ?person has ?good_character",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(
                1,
                "Does ?person actually have ?good_character?",
                Challenge::PremiseTruth("good_character".into()),
            ),
            CriticalQuestion::new(
                2,
                "Does ?good_character make ?claim more plausible?",
                Challenge::RuleValidity,
            ),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.146".into(),
            domain_tags: vec!["source".into(), "support".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_source_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn ad_hominem_has_negated_conclusion() {
        assert!(
            ad_hominem().conclusion.is_negated,
            "ad hominem must conclude ¬claim"
        );
    }

    #[test]
    fn ad_hominem_circumstantial_has_negated_conclusion() {
        assert!(ad_hominem_circumstantial().conclusion.is_negated);
    }

    #[test]
    fn bias_has_negated_conclusion() {
        assert!(argument_from_bias().conclusion.is_negated);
    }

    #[test]
    fn ethotic_has_positive_conclusion() {
        assert!(!ethotic_argument().conclusion.is_negated);
    }

    #[test]
    fn source_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= SOURCE_ID_OFFSET);
            assert!(s.id.0 < SOURCE_ID_OFFSET + 100);
        }
    }
}
