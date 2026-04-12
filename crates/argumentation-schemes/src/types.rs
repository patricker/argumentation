//! Foundational types for argumentation schemes.

/// Unique identifier for a scheme in a catalog.
///
/// IDs are assigned per category via the offset constants in
/// [`crate::catalog`]. Within a category, IDs are sequential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemeId(pub u32);

/// Category of argumentation scheme. Used for catalog filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeCategory {
    /// Knowledge-based: expert opinion, witness testimony, position to know.
    Epistemic,
    /// Cause and effect: cause to effect, correlation, sign.
    Causal,
    /// Action-oriented: consequences, values, goals, waste.
    Practical,
    /// Attacking the source: ad hominem, bias, credibility.
    SourceBased,
    /// Social proof: popularity, tradition, precedent.
    Popular,
    /// Structural reasoning: analogy, classification, commitment.
    Analogical,
}

/// What role a premise slot plays in the scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotRole {
    /// A person or character (the expert, the witness, the attacker).
    Agent,
    /// A proposition being argued for or against.
    Proposition,
    /// A property or trait being attributed to someone.
    Property,
    /// An action being proposed, evaluated, or warned about.
    Action,
    /// A domain, field, or context constraining the argument.
    Domain,
    /// A consequence or outcome.
    Consequence,
}

/// How strong a scheme's inference typically is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeStrength {
    /// Strong presumption (e.g., argument from established rule).
    Strong,
    /// Moderate presumption (e.g., argument from expert opinion).
    Moderate,
    /// Weak presumption (e.g., argument from popularity).
    Weak,
}

/// What aspect of a scheme a critical question challenges.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Challenge {
    /// Challenges the truth of a specific premise (by slot name).
    PremiseTruth(String),
    /// Challenges the credibility or reliability of the source agent.
    SourceCredibility,
    /// Challenges the validity of the inference rule itself.
    RuleValidity,
    /// Raises a conflicting authority or counter-expert.
    ConflictingAuthority,
    /// Raises an alternative cause or explanation.
    AlternativeCause,
    /// Raises unconsidered consequences.
    UnseenConsequences,
    /// Challenges the relevance or proportionality of the attack.
    Proportionality,
    /// Challenges the analogy's structural similarity.
    DisanalogyClaim,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_id_equality_is_value_based() {
        assert_eq!(SchemeId(1), SchemeId(1));
        assert_ne!(SchemeId(1), SchemeId(2));
    }

    #[test]
    fn challenge_distinguishes_premise_slots() {
        let c1 = Challenge::PremiseTruth("expert".into());
        let c2 = Challenge::PremiseTruth("domain".into());
        assert_ne!(c1, c2);
    }
}
