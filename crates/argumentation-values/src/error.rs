//! Error types for argumentation-values.

/// Errors produced by VAF operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wrapped error from the underlying Dung framework operations.
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),

    /// `objectively_accepted` / `subjectively_accepted` bail out when the
    /// audience contains too many distinct values for tractable enumeration
    /// of all linear extensions of the partial order. The hard limit is 6
    /// values (= 720 linear extensions in the worst case).
    #[error("audience too large for exhaustive enumeration: {values} values (limit is {limit})")]
    AudienceTooLarge {
        /// Number of distinct values in the audience.
        values: usize,
        /// Hard cap on values past which we refuse to enumerate.
        limit: usize,
    },

    /// An argument referenced by `ValueAssignment::promote` or by an attack
    /// edge is not registered in the underlying framework.
    #[error("argument not in framework: {0}")]
    ArgumentNotFound(String),

    /// APX text input failed to parse (Phase 3).
    #[error("apx parse error at line {line}: {reason}")]
    ApxParse {
        /// 1-indexed line number where parsing failed.
        line: usize,
        /// What went wrong.
        reason: String,
    },
}
