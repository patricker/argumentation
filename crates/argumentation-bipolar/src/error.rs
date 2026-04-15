//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-bipolar` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// A framework operation referenced an argument that is not in the
    /// framework.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// An edge was added that introduces a self-loop where the semantics
    /// reject them. Currently applies only to self-support (an argument
    /// cannot be its own necessary supporter).
    #[error("illegal self-loop: argument '{0}' cannot support itself")]
    IllegalSelfSupport(String),

    /// An error from the underlying Dung layer (e.g., framework too
    /// large for subset enumeration).
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}
