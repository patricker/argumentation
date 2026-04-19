#![deny(missing_docs)]
#![warn(clippy::all)]

//! Bridge between encounter social interactions and argumentation schemes.
//!
//! Provides formal argument resolution for encounter's social interaction
//! engine, using Walton argumentation schemes evaluated via ASPIC+ and
//! Dung extension semantics.
//!
//! # Quick example
//!
//! ```
//! use argumentation_schemes::catalog::default_catalog;
//! use argumentation_schemes::instantiate;
//! use encounter_argumentation::resolver::{resolve_argument, ArgumentOutcome};
//!
//! let registry = default_catalog();
//! let expert = registry.by_key("argument_from_expert_opinion").unwrap();
//! let instance = instantiate(expert, &[
//!     ("expert".into(), "alice".into()),
//!     ("domain".into(), "military".into()),
//!     ("claim".into(), "fortify_east".into()),
//! ].into_iter().collect()).unwrap();
//!
//! let outcome = resolve_argument(&[instance], &[], &registry);
//! assert!(matches!(outcome, ArgumentOutcome::ProposerWins { .. }));
//! ```

pub mod acceptance;
pub mod arg_id;
pub mod critical_moves;
/// Error types for encounter-argumentation operations.
pub mod error;
pub mod knowledge;
pub mod resolver;
pub mod scoring;
pub mod value_argument;

pub use acceptance::ArgumentAcceptanceEval;
pub use arg_id::ArgumentId;
pub use critical_moves::{cq_to_beat, critical_question_beats};
pub use error::Error;
pub use knowledge::{ArgumentKnowledge, ArgumentPosition, StaticKnowledge};
pub use resolver::{ArgumentOutcome, resolve_argument};
pub use scoring::SchemeActionScorer;
pub use value_argument::scheme_value_argument;

/// Numeric rank for a scheme strength (higher = stronger).
pub(crate) fn strength_rank(strength: argumentation_schemes::types::SchemeStrength) -> u8 {
    use argumentation_schemes::types::SchemeStrength;
    match strength {
        SchemeStrength::Strong => 2,
        SchemeStrength::Moderate => 1,
        SchemeStrength::Weak => 0,
    }
}
