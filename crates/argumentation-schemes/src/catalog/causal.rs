//! Causal schemes: reasoning about causes, effects, signs, and chains.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::CAUSAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all causal schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_cause_to_effect(),
        argument_from_correlation_to_cause(),
        argument_from_sign(),
        argument_from_gradual_slippery_slope(),
    ]
}

/// Argument from Cause to Effect (Walton 2008 p.327).
pub fn argument_from_cause_to_effect() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET),
        name: "Argument from Cause to Effect".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("cause", "The causal event or condition", SlotRole::Action),
            PremiseSlot::new("effect", "The effect that follows", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?effect will occur because ?cause has occurred",
            "?effect",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a strong causal link between ?cause and ?effect?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Has ?cause actually occurred or will it occur?", Challenge::PremiseTruth("cause".into())),
            CriticalQuestion::new(3, "Could something else prevent ?effect despite ?cause?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.327".into(),
            domain_tags: vec!["causal".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Correlation to Cause (Walton 2008 p.328).
pub fn argument_from_correlation_to_cause() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 1),
        name: "Argument from Correlation to Cause".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("antecedent", "The first correlated event", SlotRole::Action),
            PremiseSlot::new("consequent", "The second correlated event", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?antecedent causes ?consequent",
            "causes_?antecedent_to_?consequent",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a genuine correlation between ?antecedent and ?consequent?", Challenge::PremiseTruth("antecedent".into())),
            CriticalQuestion::new(2, "Could both ?antecedent and ?consequent be caused by a third factor?", Challenge::AlternativeCause),
            CriticalQuestion::new(3, "Could the causal direction be reversed (?consequent causes ?antecedent)?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.328".into(),
            domain_tags: vec!["causal".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Sign (Walton 2008 p.329).
pub fn argument_from_sign() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 2),
        name: "Argument from Sign".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("sign", "The observed sign or indicator", SlotRole::Property),
            PremiseSlot::new("indicated", "What the sign indicates", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?indicated is plausible based on ?sign",
            "?indicated",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?sign a reliable indicator of ?indicated?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Could ?sign indicate something other than ?indicated?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.329".into(),
            domain_tags: vec!["causal".into(), "abductive".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Gradual Slippery Slope (Walton 2008 p.338).
///
/// Negated-conclusion scheme.
pub fn argument_from_gradual_slippery_slope() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 3),
        name: "Argument from Gradual Slippery Slope".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("first_step", "The initial innocuous action", SlotRole::Action),
            PremiseSlot::new("final_outcome", "The undesirable end state", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::negated(
            "?first_step should not be taken because it leads to ?final_outcome",
            "do_?first_step",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a plausible chain from ?first_step to ?final_outcome?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Can the chain be stopped at some intermediate point?", Challenge::AlternativeCause),
            CriticalQuestion::new(3, "Is ?final_outcome really as bad as claimed?", Challenge::PremiseTruth("final_outcome".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.338".into(),
            domain_tags: vec!["causal".into(), "slippery_slope".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_causal_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn slippery_slope_has_negated_conclusion() {
        assert!(argument_from_gradual_slippery_slope().conclusion.is_negated);
    }

    #[test]
    fn causal_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= CAUSAL_ID_OFFSET);
            assert!(s.id.0 < CAUSAL_ID_OFFSET + 100);
        }
    }
}
