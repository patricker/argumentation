//! ASPIC+ integration: feed a [`SchemeInstance`] into an
//! [`argumentation::aspic::StructuredSystem`] as ordinary premises plus
//! a defeasible rule.

use crate::instance::SchemeInstance;
use argumentation::aspic::{Literal, RuleId, StructuredSystem};

/// Feed a scheme instance into a `StructuredSystem` as ordinary premises
/// and a defeasible rule (premises → conclusion).
///
/// Returns the [`RuleId`] of the defeasible rule that was added, which
/// can be used for preference ordering via
/// [`StructuredSystem::prefer_rule`].
///
/// The instance's `premises` are added as ordinary (defeasible) premises
/// via [`StructuredSystem::add_ordinary`]. The instance's `conclusion`
/// (already polarised by the scheme's [`crate::scheme::ConclusionTemplate`])
/// becomes the rule's conclusion.
pub fn add_scheme_to_system(
    instance: &SchemeInstance,
    system: &mut StructuredSystem,
) -> RuleId {
    for premise in &instance.premises {
        system.add_ordinary(premise.clone());
    }
    system.add_defeasible_rule(instance.premises.clone(), instance.conclusion.clone())
}

/// Feed a critical question's counter-argument into a `StructuredSystem`
/// as an ordinary premise asserting the counter-literal, plus a defeasible
/// rule concluding the contrary of the original scheme's conclusion (rebut).
///
/// Returns the [`RuleId`] of the counter-rule.
pub fn add_counter_argument(
    counter_literal: &Literal,
    target_conclusion: &Literal,
    system: &mut StructuredSystem,
) -> RuleId {
    system.add_ordinary(counter_literal.clone());
    let neg_conclusion = target_conclusion.contrary();
    system.add_defeasible_rule(vec![counter_literal.clone()], neg_conclusion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critical::CriticalQuestion;
    use crate::instance::instantiate;
    use crate::scheme::*;
    use crate::types::*;
    use std::collections::HashMap;

    fn expert_scheme() -> SchemeSpec {
        SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![
                PremiseSlot::new("expert", "The expert", SlotRole::Agent),
                PremiseSlot::new("domain", "Field", SlotRole::Domain),
                PremiseSlot::new("claim", "The claim", SlotRole::Proposition),
            ],
            conclusion: ConclusionTemplate::positive("?claim is true", "?claim"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "Is ?expert an expert?",
                Challenge::SourceCredibility,
            )],
            metadata: SchemeMetadata {
                citation: "Walton 2008".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        }
    }

    fn alice_bindings() -> HashMap<String, String> {
        [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn add_scheme_creates_argument_with_expected_conclusion() {
        let scheme = expert_scheme();
        let instance = instantiate(&scheme, &alice_bindings()).unwrap();

        let mut system = StructuredSystem::new();
        let _rule_id = add_scheme_to_system(&instance, &mut system);

        assert!(!system.rules().is_empty());
        let built = system.build_framework().unwrap();
        let conclusion_args =
            built.arguments_with_conclusion(&Literal::atom("fortify_east"));
        assert!(
            !conclusion_args.is_empty(),
            "expected at least one argument concluding fortify_east"
        );
    }

    #[test]
    fn counter_argument_filters_original_when_preferred() {
        let scheme = expert_scheme();
        let instance = instantiate(&scheme, &alice_bindings()).unwrap();

        let mut system = StructuredSystem::new();
        let main_rule = add_scheme_to_system(&instance, &mut system);

        // Bob presses CQ1 by asserting the counter-literal.
        let cq = &instance.critical_questions[0];
        let counter_rule =
            add_counter_argument(&cq.counter_literal, &instance.conclusion, &mut system);
        // Prefer Bob's counter so it strictly defeats Alice's argument.
        system.prefer_rule(counter_rule, main_rule).unwrap();

        let built = system.build_framework().unwrap();
        let preferred = built.framework.preferred_extensions().unwrap();
        assert_eq!(preferred.len(), 1);

        let has_fortify = built
            .arguments_with_conclusion(&Literal::atom("fortify_east"))
            .iter()
            .any(|a| preferred[0].contains(&a.id));
        assert!(
            !has_fortify,
            "original claim should be defeated when counter-rule is preferred"
        );
    }
}
