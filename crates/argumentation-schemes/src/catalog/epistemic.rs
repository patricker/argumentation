//! Epistemic schemes: reasoning about knowledge and expertise.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 4 + Appendix 1.

use crate::catalog::EPISTEMIC_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all epistemic schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_expert_opinion(),
        argument_from_witness_testimony(),
        argument_from_position_to_know(),
    ]
}

/// Argument from Expert Opinion (Walton 2008 p.14, Scheme 1).
///
/// E is an expert in domain D. E asserts that proposition A is true.
/// Therefore, A may plausibly be taken to be true.
pub fn argument_from_expert_opinion() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET),
        name: "Argument from Expert Opinion".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("expert", "Source E is an expert in subject domain D", SlotRole::Agent),
            PremiseSlot::new("domain", "The field of expertise containing the claim", SlotRole::Domain),
            PremiseSlot::new("claim", "The proposition E asserts", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim may plausibly be taken to be true",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "How credible is ?expert as an expert source?", Challenge::SourceCredibility),
            CriticalQuestion::new(2, "Is ?expert an expert in the field that ?claim is in?", Challenge::PremiseTruth("domain".into())),
            CriticalQuestion::new(3, "What did ?expert assert that implies ?claim?", Challenge::PremiseTruth("claim".into())),
            CriticalQuestion::new(4, "Is ?expert personally reliable as a source?", Challenge::SourceCredibility),
            CriticalQuestion::new(5, "Is ?claim consistent with what other experts assert?", Challenge::ConflictingAuthority),
            CriticalQuestion::new(6, "Is ?expert's assertion based on evidence?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.14 (Scheme 1)".into(),
            domain_tags: vec!["epistemic".into(), "authority".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Witness Testimony (Walton 2008 p.309, Scheme 2).
pub fn argument_from_witness_testimony() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET + 1),
        name: "Argument from Witness Testimony".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("witness", "The person who claims to have observed the event", SlotRole::Agent),
            PremiseSlot::new("event", "The event that was allegedly observed", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?event occurred as ?witness described",
            "?event",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?witness telling the truth (not lying)?", Challenge::SourceCredibility),
            CriticalQuestion::new(2, "Was ?witness in a position to observe ?event?", Challenge::PremiseTruth("witness".into())),
            CriticalQuestion::new(3, "Is ?witness's account of ?event consistent with other evidence?", Challenge::ConflictingAuthority),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.309 (Scheme 2)".into(),
            domain_tags: vec!["epistemic".into(), "testimony".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Position to Know (Walton 2008 p.310, Scheme 3).
pub fn argument_from_position_to_know() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET + 2),
        name: "Argument from Position to Know".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("source", "The person in a position to know", SlotRole::Agent),
            PremiseSlot::new("domain", "The domain of knowledge", SlotRole::Domain),
            PremiseSlot::new("claim", "The asserted proposition within that domain", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is plausibly true",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?source in a position to know whether ?claim is true?", Challenge::PremiseTruth("source".into())),
            CriticalQuestion::new(2, "Is ?source a truthful and reliable reporter?", Challenge::SourceCredibility),
            CriticalQuestion::new(3, "Did ?source actually assert ?claim?", Challenge::PremiseTruth("claim".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.310 (Scheme 3)".into(),
            domain_tags: vec!["epistemic".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expert_opinion_has_six_critical_questions() {
        let scheme = argument_from_expert_opinion();
        assert_eq!(scheme.critical_questions.len(), 6);
        assert_eq!(scheme.premises.len(), 3);
    }

    #[test]
    fn all_returns_three_epistemic_schemes() {
        let schemes = all();
        assert_eq!(schemes.len(), 3);
        assert!(schemes.iter().all(|s| s.category == SchemeCategory::Epistemic));
    }

    #[test]
    fn epistemic_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= EPISTEMIC_ID_OFFSET);
            assert!(s.id.0 < EPISTEMIC_ID_OFFSET + 100);
        }
    }
}
