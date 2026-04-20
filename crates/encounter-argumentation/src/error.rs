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

    /// An error propagated directly from the core Dung/ASPIC+ layer.
    #[error("core argumentation error: {0}")]
    Dung(#[from] argumentation::Error),

    /// An error propagated from the argumentation-bipolar layer.
    #[error("bipolar error: {0}")]
    Bipolar(#[from] argumentation_bipolar::Error),

    /// An error propagated from the argumentation-weighted layer.
    #[error("weighted error: {0}")]
    Weighted(#[from] argumentation_weighted::Error),

    /// An error propagated from the argumentation-weighted-bipolar layer.
    #[error("weighted-bipolar error: {0}")]
    WeightedBipolar(#[from] argumentation_weighted_bipolar::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_bipolar_propagates() {
        let bipolar_err = argumentation_bipolar::Error::IllegalSelfSupport("x".into());
        let err: Error = bipolar_err.into();
        assert!(matches!(err, Error::Bipolar(_)));
    }

    #[test]
    fn error_from_weighted_propagates() {
        let weighted_err = argumentation_weighted::Error::InvalidWeight { weight: -1.0 };
        let err: Error = weighted_err.into();
        assert!(matches!(err, Error::Weighted(_)));
    }

    #[test]
    fn error_from_wbipolar_propagates() {
        let wbp_err = argumentation_weighted_bipolar::Error::IllegalSelfSupport;
        let err: Error = wbp_err.into();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }

    #[test]
    fn error_from_dung_propagates() {
        // TooLarge is the simplest constructible argumentation::Error variant.
        let core_err = argumentation::Error::TooLarge { arguments: 100, limit: 22 };
        let err: Error = core_err.into();
        assert!(matches!(err, Error::Dung(_)));
    }
}
