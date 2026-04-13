//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable reasoning pattern
//! (expert opinion, ad hominem, argument from consequences) that carries
//! its own critical questions — the follow-up challenges a competent
//! reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_schemes::catalog::default_catalog;
//! use std::collections::HashMap;
//!
//! let catalog = default_catalog();
//! let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
//!
//! let bindings: HashMap<String, String> = [
//!     ("expert".to_string(), "alice".to_string()),
//!     ("domain".to_string(), "military".to_string()),
//!     ("claim".to_string(), "fortify_east".to_string()),
//! ]
//! .into_iter()
//! .collect();
//!
//! let instance = scheme.instantiate(&bindings).unwrap();
//! assert_eq!(instance.premises.len(), 3);
//! assert_eq!(instance.critical_questions.len(), 6);
//! ```
//!
//! ## References
//!
//! - Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.*
//!   Cambridge University Press.
//! - Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework.*
//!   Argument & Computation 5(1).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod aspic;
pub mod catalog;
pub mod critical;
pub mod error;
pub mod instance;
pub mod registry;
pub mod scheme;
pub mod types;

pub use error::Error;
pub use instance::{CriticalQuestionInstance, SchemeInstance, instantiate};
pub use registry::CatalogRegistry;
pub use scheme::{ConclusionTemplate, PremiseSlot, SchemeMetadata, SchemeSpec};
pub use types::{Challenge, SchemeCategory, SchemeId, SchemeStrength, SlotRole};
