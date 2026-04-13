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

    /// An operation referenced an argument that is not in the framework.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// An error from the underlying Dung layer (e.g., framework too
    /// large for subset enumeration).
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}
