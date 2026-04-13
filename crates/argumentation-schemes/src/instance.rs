//! `SchemeInstance`: a scheme instantiated with concrete bindings.

use crate::Error;
use crate::critical::CriticalQuestion;
use crate::scheme::SchemeSpec;
use argumentation::aspic::Literal;
use std::collections::HashMap;

/// A critical question instantiated with concrete bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalQuestionInstance {
    /// Question number (from the parent scheme).
    pub number: u32,
    /// Human-readable text with `?slot` references resolved.
    pub text: String,
    /// The challenge type (from the parent CriticalQuestion).
    pub challenge: crate::types::Challenge,
    /// The literal that, if asserted, would undermine the original argument.
    /// Always negated.
    pub counter_literal: Literal,
}

/// A scheme instantiated with concrete bindings, ready for ASPIC+ integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeInstance {
    /// The scheme this instance was created from.
    pub scheme_name: String,
    /// The resolved premise literals.
    pub premises: Vec<Literal>,
    /// The resolved conclusion literal.
    pub conclusion: Literal,
    /// Instantiated critical questions with resolved text and counter-literals.
    pub critical_questions: Vec<CriticalQuestionInstance>,
}

/// Resolve a template string by replacing `?slot` references with bound values.
///
/// Bindings are processed in descending key-length order so that longer
/// slot names are substituted before any shorter slot names that happen
/// to be a prefix. Without this, a template `?threatener` containing
/// the substring `?threat` would be corrupted by an earlier substitution
/// of slot `threat`.
///
/// **Note on binding values:** substitution is multi-pass over the
/// accumulated string, so a binding *value* that contains `?slot` syntax
/// matching a later (shorter-key) binding will itself be substituted.
/// Walton scheme bindings are concrete entity names (`"alice"`,
/// `"darth_vader"`) so this never fires in practice, but consumers
/// passing bindings from less-controlled sources (LLM output, free-text
/// dialog) should sanitise `?` from values first.
fn resolve_template(template: &str, bindings: &HashMap<String, String>) -> String {
    let mut sorted: Vec<(&String, &String)> = bindings.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut result = template.to_string();
    for (key, val) in sorted {
        result = result.replace(&format!("?{}", key), val);
    }
    result
}

/// Instantiate a scheme with concrete bindings.
///
/// Every premise slot in the scheme must have a corresponding entry in
/// `bindings`. Returns [`Error::MissingBinding`] if any required slot is
/// unbound.
///
/// Also available as [`SchemeSpec::instantiate`] (which delegates here).
pub fn instantiate(
    scheme: &SchemeSpec,
    bindings: &HashMap<String, String>,
) -> Result<SchemeInstance, Error> {
    // Validate all slots are bound. Iterate in declared order so the error
    // message names the FIRST missing slot deterministically.
    for slot in &scheme.premises {
        if !bindings.contains_key(&slot.name) {
            return Err(Error::MissingBinding {
                scheme: scheme.name.clone(),
                slot: slot.name.clone(),
            });
        }
    }

    // Build premise literals: for each slot, create an atom encoding
    // "this slot is filled by this value in this scheme instance."
    // E.g., slot "expert" + binding "alice" → Literal::atom("expert_alice").
    let premises: Vec<Literal> = scheme
        .premises
        .iter()
        .map(|slot| {
            let val = &bindings[&slot.name];
            Literal::atom(format!("{}_{}", slot.name, val))
        })
        .collect();

    // Resolve conclusion template, respecting the is_negated flag.
    let conclusion_name = resolve_template(&scheme.conclusion.literal_template, bindings);
    let conclusion = if scheme.conclusion.is_negated {
        Literal::neg(&conclusion_name)
    } else {
        Literal::atom(&conclusion_name)
    };

    // Instantiate critical questions.
    let critical_questions = scheme
        .critical_questions
        .iter()
        .map(|cq| {
            let text = resolve_template(&cq.text, bindings);
            let counter_literal = build_counter_literal(cq, bindings, scheme, &conclusion_name);
            CriticalQuestionInstance {
                number: cq.number,
                text,
                challenge: cq.challenge.clone(),
                counter_literal,
            }
        })
        .collect();

    Ok(SchemeInstance {
        scheme_name: scheme.name.clone(),
        premises,
        conclusion,
        critical_questions,
    })
}

/// Build the counter-literal for a critical question.
///
/// The counter-literal is what would be asserted to undermine the scheme.
/// Different challenge types target different aspects:
/// - `PremiseTruth(slot)` negates the premise literal for that slot.
/// - `SourceCredibility` negates `credible_<agent>` for the relevant agent.
/// - Others negate a synthetic marker derived from the conclusion or scheme key.
fn build_counter_literal(
    cq: &CriticalQuestion,
    bindings: &HashMap<String, String>,
    scheme: &SchemeSpec,
    conclusion_name: &str,
) -> Literal {
    use crate::types::Challenge;
    match &cq.challenge {
        Challenge::PremiseTruth(slot_name) => {
            let val = bindings
                .get(slot_name)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            Literal::neg(format!("{}_{}", slot_name, val))
        }
        Challenge::SourceCredibility => {
            // Slot priority: epistemic-source roles first, then generic
            // agent roles, then adversarial-repurpose roles.
            // expert > witness > source > agent > person > target > threatener.
            let agent = bindings
                .get("expert")
                .or_else(|| bindings.get("witness"))
                .or_else(|| bindings.get("source"))
                .or_else(|| bindings.get("agent"))
                .or_else(|| bindings.get("person"))
                .or_else(|| bindings.get("target"))
                .or_else(|| bindings.get("threatener"))
                .map(|s| s.as_str())
                .unwrap_or("source");
            Literal::neg(format!("credible_{}", agent))
        }
        Challenge::RuleValidity => Literal::neg(format!("valid_rule_{}", scheme.key())),
        Challenge::ConflictingAuthority => {
            Literal::neg(format!("consensus_on_{}", conclusion_name))
        }
        Challenge::AlternativeCause => Literal::neg(format!("sole_cause_{}", conclusion_name)),
        Challenge::UnseenConsequences => {
            Literal::neg(format!("all_consequences_considered_{}", conclusion_name))
        }
        Challenge::Proportionality => {
            let target = bindings
                .get("target")
                .or_else(|| bindings.get("threatener"))
                .or_else(|| bindings.get("agent"))
                .or_else(|| bindings.get("person"))
                .map(|s| s.as_str())
                .unwrap_or("target");
            Literal::neg(format!("proportionate_attack_{}", target))
        }
        Challenge::DisanalogyClaim => Literal::neg(format!("analogy_holds_{}", conclusion_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critical::CriticalQuestion;
    use crate::scheme::{ConclusionTemplate, PremiseSlot, SchemeMetadata, SchemeSpec};
    use crate::types::*;

    fn expert_opinion_scheme() -> SchemeSpec {
        SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![
                PremiseSlot::new("expert", "The claimed expert", SlotRole::Agent),
                PremiseSlot::new("domain", "Field of expertise", SlotRole::Domain),
                PremiseSlot::new("claim", "The asserted proposition", SlotRole::Proposition),
            ],
            conclusion: ConclusionTemplate::positive("?claim is plausibly true", "?claim"),
            critical_questions: vec![
                CriticalQuestion::new(
                    1,
                    "Is ?expert an expert in ?domain?",
                    Challenge::PremiseTruth("expert".into()),
                ),
                CriticalQuestion::new(2, "Is ?expert credible?", Challenge::SourceCredibility),
            ],
            metadata: SchemeMetadata {
                citation: "Walton 2008 p.14".into(),
                domain_tags: vec!["epistemic".into()],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        }
    }

    fn full_bindings() -> HashMap<String, String> {
        [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn instantiate_produces_three_premises_and_positive_conclusion() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(instance.premises.len(), 3);
        assert_eq!(instance.conclusion, Literal::atom("fortify_east"));
    }

    #[test]
    fn instantiate_resolves_critical_question_text() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert!(instance.critical_questions[0].text.contains("alice"));
        assert!(instance.critical_questions[0].text.contains("military"));
    }

    #[test]
    fn instantiate_fails_on_missing_binding() {
        let scheme = expert_opinion_scheme();
        let mut bindings = full_bindings();
        bindings.remove("domain");
        let err = instantiate(&scheme, &bindings).unwrap_err();
        match err {
            Error::MissingBinding { slot, .. } => assert_eq!(slot, "domain"),
            other => panic!("expected MissingBinding, got {:?}", other),
        }
    }

    #[test]
    fn counter_literals_for_premise_truth_match_premise_encoding() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        // CQ1 challenges PremiseTruth("expert") → counter is ¬expert_alice,
        // which is the contrary of the premise literal expert_alice.
        let cq1 = &instance.critical_questions[0];
        assert_eq!(cq1.counter_literal, Literal::neg("expert_alice"));
        assert!(cq1.counter_literal.is_contrary_of(&instance.premises[0]));
    }

    #[test]
    fn counter_literal_for_source_credibility_uses_agent_binding() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        let cq2 = &instance.critical_questions[1];
        assert_eq!(cq2.counter_literal, Literal::neg("credible_alice"));
    }

    #[test]
    fn source_credibility_falls_back_to_agent_slot_when_no_epistemic_role_bound() {
        // Forward-compat guard for the v0.1.0 fallback-chain fix: if a
        // future scheme uses SourceCredibility alongside an `agent`-named
        // slot (as argument_from_values and argument_from_commitment do),
        // the counter-literal must resolve to credible_<agent_binding>,
        // not the literal string "credible_source". Regressing this
        // behaviour was the bug the fallback fix was written to prevent.
        let scheme = SchemeSpec {
            id: SchemeId(9999),
            name: "synthetic agent credibility".into(),
            category: SchemeCategory::Practical,
            premises: vec![
                PremiseSlot::new("agent", "the actor", SlotRole::Agent),
                PremiseSlot::new("action", "what they did", SlotRole::Action),
            ],
            conclusion: ConclusionTemplate::positive("?agent did ?action", "did_?action"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "Is ?agent credible?",
                Challenge::SourceCredibility,
            )],
            metadata: SchemeMetadata {
                citation: "synthetic".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        };
        let bindings: HashMap<String, String> = [
            ("agent".to_string(), "alice".to_string()),
            ("action".to_string(), "sign_treaty".to_string()),
        ]
        .into_iter()
        .collect();
        let instance = instantiate(&scheme, &bindings).unwrap();
        assert_eq!(
            instance.critical_questions[0].counter_literal,
            Literal::neg("credible_alice"),
        );
    }

    #[test]
    fn source_credibility_prefers_agent_over_target_when_both_are_bound() {
        // The fallback chain must put `agent` ahead of `target` so that a
        // scheme with BOTH slots resolves credibility to the agent (the
        // person whose epistemic standing matters) rather than the target
        // (the person being attacked). No current scheme has both slots;
        // this test pins the ordering for the future case.
        let scheme = SchemeSpec {
            id: SchemeId(9998),
            name: "synthetic agent over target".into(),
            category: SchemeCategory::Practical,
            premises: vec![
                PremiseSlot::new("agent", "the actor", SlotRole::Agent),
                PremiseSlot::new("target", "the recipient", SlotRole::Agent),
            ],
            conclusion: ConclusionTemplate::positive("outcome", "outcome"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "Is ?agent credible?",
                Challenge::SourceCredibility,
            )],
            metadata: SchemeMetadata {
                citation: "synthetic".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        };
        let bindings: HashMap<String, String> = [
            ("agent".to_string(), "alice".to_string()),
            ("target".to_string(), "bob".to_string()),
        ]
        .into_iter()
        .collect();
        let instance = instantiate(&scheme, &bindings).unwrap();
        assert_eq!(
            instance.critical_questions[0].counter_literal,
            Literal::neg("credible_alice"),
            "agent should win over target in the SourceCredibility chain"
        );
    }

    #[test]
    fn proportionality_falls_back_to_agent_slot() {
        // Forward-compat guard for the Proportionality fallback chain:
        // chain is target > threatener > agent > person, so a scheme
        // that uses Proportionality without a `target` or `threatener`
        // slot must still pick up an `agent` binding.
        let scheme = SchemeSpec {
            id: SchemeId(9997),
            name: "synthetic proportionality".into(),
            category: SchemeCategory::Practical,
            premises: vec![PremiseSlot::new("agent", "the actor", SlotRole::Agent)],
            conclusion: ConclusionTemplate::positive("did_it", "did_it"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "Is the reaction to ?agent proportionate?",
                Challenge::Proportionality,
            )],
            metadata: SchemeMetadata {
                citation: "synthetic".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        };
        let bindings: HashMap<String, String> = [("agent".to_string(), "alice".to_string())]
            .into_iter()
            .collect();
        let instance = instantiate(&scheme, &bindings).unwrap();
        assert_eq!(
            instance.critical_questions[0].counter_literal,
            Literal::neg("proportionate_attack_alice"),
        );
    }

    #[test]
    fn negated_conclusion_template_produces_negated_literal() {
        let mut scheme = expert_opinion_scheme();
        scheme.conclusion = ConclusionTemplate::negated("¬?claim", "?claim");
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(instance.conclusion, Literal::neg("fortify_east"));
    }

    #[test]
    fn resolve_template_handles_prefix_overlapping_slot_names() {
        // Regression for the threat scheme: slots "threatener" and
        // "threat" share a prefix. Without length-descending sort,
        // ?threat would match inside ?threatener and corrupt it.
        let bindings: HashMap<String, String> = [
            ("threatener".to_string(), "darth_vader".to_string()),
            ("threat".to_string(), "destroy_planet".to_string()),
            ("demand".to_string(), "join_dark_side".to_string()),
        ]
        .into_iter()
        .collect();

        let template = "Does ?threatener carry out ?threat to force ?demand?";
        let resolved = resolve_template(template, &bindings);
        assert_eq!(
            resolved,
            "Does darth_vader carry out destroy_planet to force join_dark_side?"
        );
    }

    #[test]
    fn resolve_template_handles_three_way_prefix_overlap() {
        // Belt-and-braces regression: three slots whose names form a
        // strict prefix chain (event ⊏ event_a ⊏ event_a_extended).
        // Length-descending sort must substitute the longest first so
        // none of the shorter names match inside the longer ones.
        let bindings: HashMap<String, String> = [
            ("event".to_string(), "trigger".to_string()),
            ("event_a".to_string(), "alpha".to_string()),
            ("event_a_extended".to_string(), "alpha_plus".to_string()),
        ]
        .into_iter()
        .collect();

        let template = "saw ?event then ?event_a then ?event_a_extended";
        let resolved = resolve_template(template, &bindings);
        assert_eq!(resolved, "saw trigger then alpha then alpha_plus");
    }

    #[test]
    fn schemespec_instantiate_method_delegates_to_free_function() {
        let scheme = expert_opinion_scheme();
        let via_method = scheme.instantiate(&full_bindings()).unwrap();
        let via_free_fn = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(via_method.premises, via_free_fn.premises);
        assert_eq!(via_method.conclusion, via_free_fn.conclusion);
    }
}
