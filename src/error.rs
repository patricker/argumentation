//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An argument referenced in an operation was not present in the framework.
    /// The payload is a `Debug` rendering of the missing argument when available.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// A parser failed to decode input.
    #[error("parse error: {0}")]
    Parse(String),

    /// An ASPIC+ operation failed structurally (e.g., cyclic rule dependencies,
    /// rule references a literal not in the language).
    #[error("aspic error: {0}")]
    Aspic(String),

    /// An extension-enumeration call was rejected because the framework is too
    /// large for the subset-enumeration algorithm. Use a SAT-based semantics
    /// entry point (future) for frameworks above this threshold.
    #[error("framework too large for subset enumeration: {arguments} arguments (limit is {limit})")]
    TooLarge {
        /// Number of arguments in the framework.
        arguments: usize,
        /// Enumeration limit (currently 30).
        limit: usize,
    },
}
