//! `SchemeSpec`: the compile-time definition of one argumentation scheme.

use crate::critical::CriticalQuestion;
use crate::types::{SchemeCategory, SchemeId, SchemeStrength, SlotRole};

/// A named premise slot in a scheme.
///
/// When instantiated with bindings, each slot maps to a concrete value
/// (e.g., slot "expert" → "alice").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremiseSlot {
    /// Slot name (used as the binding key, e.g., "expert", "claim", "domain").
    pub name: String,
    /// Human-readable description of what this slot represents.
    pub description: String,
    /// What role this slot plays in the scheme.
    pub role: SlotRole,
}

impl PremiseSlot {
    /// Convenience constructor.
    pub fn new(name: impl Into<String>, description: impl Into<String>, role: SlotRole) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            role,
        }
    }
}

/// Template for the scheme's conclusion.
///
/// `literal_template` is a string with `?slot` references that get resolved
/// against the bindings at instantiation time. `is_negated` controls whether
/// the resulting [`argumentation::aspic::Literal`] is constructed via
/// `Literal::neg` (for rebut-concluding schemes like ad hominem) or
/// `Literal::atom`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConclusionTemplate {
    /// Human-readable description (e.g., "?claim is plausibly true").
    pub description: String,
    /// The literal name template. Slot references prefixed with `?` are
    /// replaced with bound values during instantiation.
    pub literal_template: String,
    /// If true, the conclusion is constructed as a negated literal.
    /// Required for rebuttal-concluding schemes (ad hominem, argument
    /// from negative consequences, slippery slope, etc.).
    pub is_negated: bool,
}

impl ConclusionTemplate {
    /// Convenience constructor for a positive (non-negated) conclusion.
    pub fn positive(description: impl Into<String>, literal_template: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            literal_template: literal_template.into(),
            is_negated: false,
        }
    }

    /// Convenience constructor for a negated conclusion (e.g., ad hominem
    /// concluding ¬claim).
    pub fn negated(description: impl Into<String>, literal_template: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            literal_template: literal_template.into(),
            is_negated: true,
        }
    }
}

/// Metadata about a scheme: citation, tags, strength.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeMetadata {
    /// Citation (e.g., "Walton 2008 p.14").
    pub citation: String,
    /// Domain tags for filtering (e.g., ["epistemic", "authority"]).
    pub domain_tags: Vec<String>,
    /// Whether the scheme is presumptive (virtually all Walton schemes are).
    pub presumptive: bool,
    /// How strong the scheme's inference typically is.
    pub strength: SchemeStrength,
}

/// The complete definition of one argumentation scheme.
///
/// A scheme is a recognisable pattern of reasoning with named premise slots,
/// a conclusion template, and critical questions that probe its weak points.
/// Schemes are compile-time data: each is constructed by a function in the
/// [`crate::catalog`] module. Consumers instantiate schemes with concrete
/// bindings via [`SchemeSpec::instantiate`] or [`crate::instance::instantiate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeSpec {
    /// Unique scheme id.
    pub id: SchemeId,
    /// Canonical name (e.g., "Argument from Expert Opinion").
    pub name: String,
    /// Scheme category for catalog filtering.
    pub category: SchemeCategory,
    /// Named premise slots. Order matters — the first N are the scheme's
    /// "core premises" as defined by Walton.
    pub premises: Vec<PremiseSlot>,
    /// Conclusion template. References premise slot names via `?name` syntax.
    pub conclusion: ConclusionTemplate,
    /// Critical questions that probe the scheme's weak points.
    pub critical_questions: Vec<CriticalQuestion>,
    /// Bibliographic and classification metadata.
    pub metadata: SchemeMetadata,
}

impl SchemeSpec {
    /// Instantiate this scheme with concrete bindings. Convenience method
    /// that delegates to [`crate::instance::instantiate`].
    pub fn instantiate(
        &self,
        bindings: &std::collections::HashMap<String, String>,
    ) -> Result<crate::instance::SchemeInstance, crate::Error> {
        crate::instance::instantiate(self, bindings)
    }

    /// The scheme's canonical name as a snake_case identifier suitable
    /// for lookup keys and affordance mapping.
    pub fn key(&self) -> String {
        self.name
            .to_lowercase()
            .replace([' ', '-'], "_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_key_is_snake_case() {
        let scheme = SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![PremiseSlot::new("expert", "The expert", SlotRole::Agent)],
            conclusion: ConclusionTemplate::positive("?claim is true", "?claim"),
            critical_questions: vec![],
            metadata: SchemeMetadata {
                citation: "Walton 2008".into(),
                domain_tags: vec!["epistemic".into()],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        };
        assert_eq!(scheme.key(), "argument_from_expert_opinion");
    }

    #[test]
    fn conclusion_template_positive_has_is_negated_false() {
        let t = ConclusionTemplate::positive("desc", "?claim");
        assert!(!t.is_negated);
    }

    #[test]
    fn conclusion_template_negated_has_is_negated_true() {
        let t = ConclusionTemplate::negated("desc", "?claim");
        assert!(t.is_negated);
    }
}
