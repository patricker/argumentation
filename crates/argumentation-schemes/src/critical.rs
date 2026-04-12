//! Critical questions for argumentation schemes.

use crate::types::Challenge;

/// A critical question that probes a scheme's weak points.
///
/// Each Walton scheme carries 2-6 critical questions. When a character
/// uses a scheme in an encounter, these become the available follow-up
/// moves for the opposing party.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalQuestion {
    /// Question number within the scheme (1-based).
    pub number: u32,
    /// Human-readable question text with `?slot` references.
    pub text: String,
    /// What aspect of the scheme this question challenges.
    pub challenge: Challenge,
}

impl CriticalQuestion {
    /// Convenience constructor.
    pub fn new(number: u32, text: impl Into<String>, challenge: Challenge) -> Self {
        Self {
            number,
            text: text.into(),
            challenge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_question_stores_challenge() {
        let cq = CriticalQuestion::new(
            1,
            "Is ?expert really an expert in ?domain?",
            Challenge::PremiseTruth("expert".into()),
        );
        assert_eq!(cq.number, 1);
        assert!(cq.text.contains("?expert"));
        assert!(matches!(cq.challenge, Challenge::PremiseTruth(_)));
    }
}
