//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-weighted` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An attack weight was non-finite (NaN or infinity) or negative.
    /// Dunne 2011 requires non-negative finite weights.
    #[error("invalid attack weight {weight}: weights must be non-negative finite f64")]
    InvalidWeight {
        /// The weight that failed validation.
        weight: f64,
    },

    /// A budget value was non-finite or negative.
    #[error("invalid budget {budget}: budgets must be non-negative finite f64")]
    InvalidBudget {
        /// The budget that failed validation.
        budget: f64,
    },

    /// A weighted framework exceeded the Dunne 2011 subset-enumeration
    /// attack-count limit. The exact semantics enumerate the power set
    /// of attacks, so the limit caps memory+time at 2^limit subsets.
    #[error("too many attacks for exact Dunne 2011 enumeration: {attacks} attacks exceed the limit of {limit}")]
    TooManyAttacks {
        /// The number of attacks in the offending framework.
        attacks: usize,
        /// The current enumeration limit.
        limit: usize,
    },

    /// An operation referenced an argument that is not in the framework.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// An error from the underlying Dung layer (e.g., framework too
    /// large for subset enumeration).
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_many_attacks_error_carries_count_and_limit() {
        let err = Error::TooManyAttacks { attacks: 30, limit: 24 };
        let msg = format!("{}", err);
        assert!(msg.contains("30"));
        assert!(msg.contains("24"));
    }
}
