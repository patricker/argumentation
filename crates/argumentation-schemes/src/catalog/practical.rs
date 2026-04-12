//! Practical schemes: reasoning about actions, consequences, and values.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::PRACTICAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all practical schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_positive_consequences(),
        argument_from_negative_consequences(),
        argument_from_values(),
        argument_from_threat(),
        argument_from_fear_appeal(),
        argument_from_waste(),
        argument_from_sunk_cost(),
    ]
}

/// Argument from Positive Consequences (Walton 2008 p.332).
///
/// If action A is brought about, good consequence G will occur.
/// Therefore A should be brought about.
pub fn argument_from_positive_consequences() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET),
        name: "Argument from Positive Consequences".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being proposed", SlotRole::Action),
            PremiseSlot::new("good_consequence", "The beneficial outcome", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be brought about because ?good_consequence will result",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Will ?action actually lead to ?good_consequence?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?good_consequence actually good on balance?", Challenge::PremiseTruth("good_consequence".into())),
            CriticalQuestion::new(3, "Are there negative consequences that offset ?good_consequence?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.332".into(),
            domain_tags: vec!["practical".into(), "consequences".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Negative Consequences (Walton 2008 p.333).
///
/// Negated-conclusion scheme: concludes ¬do_?action.
pub fn argument_from_negative_consequences() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 1),
        name: "Argument from Negative Consequences".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being warned against", SlotRole::Action),
            PremiseSlot::new("bad_consequence", "The harmful outcome", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::negated(
            "?action should not be brought about because ?bad_consequence will result",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Will ?action actually lead to ?bad_consequence?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?bad_consequence actually bad on balance?", Challenge::PremiseTruth("bad_consequence".into())),
            CriticalQuestion::new(3, "Are there positive consequences that offset ?bad_consequence?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.333".into(),
            domain_tags: vec!["practical".into(), "consequences".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Values (Walton 2008 p.321).
pub fn argument_from_values() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 2),
        name: "Argument from Values".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being evaluated", SlotRole::Action),
            PremiseSlot::new("value", "The value being promoted", SlotRole::Property),
            PremiseSlot::new("agent", "The agent whose values are at stake", SlotRole::Agent),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be carried out because it promotes ?value for ?agent",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?action really promote ?value?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?value relevant to the current context?", Challenge::PremiseTruth("value".into())),
            CriticalQuestion::new(3, "Are there competing values that take precedence over ?value?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.321".into(),
            domain_tags: vec!["practical".into(), "values".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Threat (Walton 2008 p.335).
///
/// Note: slot names `threatener` and `threat` overlap as prefixes —
/// the `resolve_template` function in `instance.rs` sorts bindings by
/// length descending to handle this correctly.
pub fn argument_from_threat() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 3),
        name: "Argument from Threat".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("threatener", "The person making the threat", SlotRole::Agent),
            PremiseSlot::new("threat", "The bad thing that will happen", SlotRole::Consequence),
            PremiseSlot::new("demand", "The action being demanded", SlotRole::Action),
        ],
        conclusion: ConclusionTemplate::positive(
            "?demand should be complied with to avoid ?threat",
            "comply_?demand",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?threatener have the ability to carry out ?threat?", Challenge::PremiseTruth("threatener".into())),
            CriticalQuestion::new(2, "Is ?threat proportionate to ?demand?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Is there an alternative to complying with ?demand?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.335".into(),
            domain_tags: vec!["practical".into(), "coercion".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Fear Appeal (Walton 2008 p.336).
pub fn argument_from_fear_appeal() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 4),
        name: "Argument from Fear Appeal".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The recommended action", SlotRole::Action),
            PremiseSlot::new("fearful_outcome", "The feared consequence of inaction", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be taken to avoid ?fearful_outcome",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?fearful_outcome truly likely if ?action is not taken?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is the fear of ?fearful_outcome proportionate to the actual risk?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Is ?action the only way to avoid ?fearful_outcome?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.336".into(),
            domain_tags: vec!["practical".into(), "emotion".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Waste (Walton 2008 p.339).
pub fn argument_from_waste() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 5),
        name: "Argument from Waste".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action already started", SlotRole::Action),
            PremiseSlot::new("investment", "What has been invested so far", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be continued to avoid wasting ?investment",
            "continue_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "How much has actually been invested in ?action?", Challenge::PremiseTruth("investment".into())),
            CriticalQuestion::new(2, "Would continuing ?action actually recoup ?investment?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.339".into(),
            domain_tags: vec!["practical".into(), "sunk_cost".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Sunk Cost (Walton 2008 p.340) — distinct from Waste:
/// emphasises the prior commitment as a reason for continuation.
pub fn argument_from_sunk_cost() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 6),
        name: "Argument from Sunk Cost".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The committed action", SlotRole::Action),
            PremiseSlot::new("commitment", "The prior commitment that locks the agent in", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be continued to honour ?commitment",
            "continue_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?commitment still binding given current circumstances?", Challenge::PremiseTruth("commitment".into())),
            CriticalQuestion::new(2, "Does honouring ?commitment actually require continuing ?action?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.340".into(),
            domain_tags: vec!["practical".into(), "sunk_cost".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_seven_practical_schemes() {
        let schemes = all();
        assert_eq!(schemes.len(), 7);
        assert!(schemes.iter().all(|s| s.category == SchemeCategory::Practical));
    }

    #[test]
    fn negative_consequences_has_negated_conclusion() {
        assert!(argument_from_negative_consequences().conclusion.is_negated);
    }

    #[test]
    fn positive_consequences_has_non_negated_conclusion() {
        assert!(!argument_from_positive_consequences().conclusion.is_negated);
    }

    #[test]
    fn threat_scheme_has_three_premises() {
        assert_eq!(argument_from_threat().premises.len(), 3);
    }

    #[test]
    fn practical_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= PRACTICAL_ID_OFFSET);
            assert!(s.id.0 < PRACTICAL_ID_OFFSET + 100);
        }
    }
}
