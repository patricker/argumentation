//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-schemes` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// A scheme instantiation failed because a required binding was missing.
    #[error("missing binding '{slot}' for scheme '{scheme}'")]
    MissingBinding {
        /// The scheme being instantiated.
        scheme: String,
        /// The slot that was not bound.
        slot: String,
    },

    /// A scheme was not found in the registry.
    #[error("scheme not found: {0}")]
    SchemeNotFound(String),

    /// An error from the underlying ASPIC+ layer.
    #[error("aspic error: {0}")]
    Aspic(#[from] argumentation::Error),
}
