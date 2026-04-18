//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-weighted-bipolar` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An edge weight was non-finite or negative. Mirrors the rule in
    /// `argumentation-weighted`: weights must be non-negative finite.
    #[error("invalid edge weight {weight}: weights must be non-negative finite f64")]
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

    /// A framework had more edges (attacks + supports) than the exact
    /// Dunne 2011 subset enumeration can handle in finite time.
    #[error("too many edges for exact enumeration: {edges} exceeds limit of {limit}")]
    TooManyEdges {
        /// The total edge count.
        edges: usize,
        /// The current enumeration limit.
        limit: usize,
    },

    /// A support edge was added that made an argument support itself.
    #[error("illegal self-support: argument cannot be its own necessary supporter")]
    IllegalSelfSupport,

    /// An error propagated from `argumentation-bipolar`.
    #[error("bipolar error: {0}")]
    Bipolar(#[from] argumentation_bipolar::Error),

    /// An error propagated from the core Dung layer.
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}
