//! Error types for encounter-argumentation bridge operations.

use thiserror::Error;

/// Errors that can occur during argumentation bridge operations.
#[derive(Debug, Error)]
pub enum Error {
    /// A requested scheme key was not found in the registry.
    #[error("scheme not found: {0}")]
    SchemeNotFound(String),

    /// A required slot binding was missing when instantiating a scheme.
    #[error("missing binding for scheme {scheme}: slot {slot}")]
    MissingBinding {
        /// The name of the scheme that required the binding.
        scheme: String,
        /// The slot that was not bound.
        slot: String,
    },

    /// An error propagated from the argumentation-schemes layer.
    #[error("scheme error: {0}")]
    Scheme(#[from] argumentation_schemes::Error),
}
