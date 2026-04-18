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

    /// The AIF JSON document failed to parse into our data model:
    /// missing required field, dangling node reference, unknown node
    /// type, etc. Contains a free-text explanation.
    #[error("AIF parse error: {0}")]
    AifParse(String),

    /// The AIF document referenced a scheme by name that is not
    /// present in the registry supplied to the importer.
    #[error("AIF unknown scheme: {0}")]
    AifUnknownScheme(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aif_parse_error_carries_message() {
        let err = Error::AifParse("bad edge reference".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("bad edge reference"));
    }

    #[test]
    fn aif_unknown_scheme_error_names_scheme() {
        let err = Error::AifUnknownScheme("Argument from Flapdoodle".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Flapdoodle"));
    }
}
