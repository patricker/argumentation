//! # argumentation-bipolar
//!
//! Bipolar argumentation frameworks (Cayrol & Lagasquie-Schiex 2005,
//! Amgoud et al. 2008, Nouioua & Risch 2011) built on top of the
//! [`argumentation`] crate's Dung semantics.
//!
//! A bipolar framework extends Dung's abstract argumentation with a
//! second directed edge relation: **support**. Arguments can attack and
//! support one another. This crate implements *necessary support*
//! semantics: `A` supports `B` means `A` must be accepted for `B` to be
//! acceptable.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_bipolar::framework::BipolarFramework;
//! use argumentation_bipolar::coalition::detect_coalitions;
//! use argumentation_bipolar::semantics::bipolar_preferred_extensions;
//!
//! let mut bf = BipolarFramework::new();
//! bf.add_support("alice", "bob").unwrap();
//! bf.add_support("bob", "alice").unwrap();
//! bf.add_attack("alice", "charlie");
//! bf.add_attack("bob", "charlie");
//!
//! let coalitions = detect_coalitions(&bf);
//! assert!(coalitions.iter().any(|c| c.members.len() == 2));
//!
//! let prefs = bipolar_preferred_extensions(&bf).unwrap();
//! for ext in &prefs {
//!     assert!(!ext.contains(&"charlie"));
//! }
//! ```
//!
//! ## Semantics pipeline
//!
//! 1. [`derived::closed_attacks`] computes the set of all attacks under
//!    the closed attack relation (direct + supported + secondary +
//!    mediated) per C&LS 2005 §3.
//! 2. [`flatten::flatten`] produces an equivalent
//!    [`argumentation::ArgumentationFramework`] whose attack edges are
//!    the closure.
//! 3. [`semantics::bipolar_preferred_extensions`] runs the core crate's
//!    Dung preferred semantics on the flattened framework, then filters
//!    extensions that are not *support-closed* (every accepted argument
//!    must have all its necessary supporters in the extension too).
//! 4. [`coalition::detect_coalitions`] runs Tarjan SCC on the support
//!    graph to find mutually-supporting groups.
//!
//! ## References
//!
//! - Cayrol, C. & Lagasquie-Schiex, M.-C. (2005). *On the acceptability
//!   of arguments in bipolar argumentation frameworks.* ECSQARU / IJAR
//!   23(4).
//! - Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P.
//!   (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10).
//! - Nouioua, F. & Risch, V. (2011). *Bipolar argumentation frameworks
//!   with specialized supports.* ICTAI 2011.
//! - Cohen, A. et al. (2014). *A survey of different approaches to
//!   support in argumentation systems.* KER 29(5).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod coalition;
pub mod derived;
pub mod error;
pub mod flatten;
pub mod framework;
pub mod queries;
pub mod semantics;
pub mod types;

pub use coalition::{Coalition, detect_coalitions};
pub use error::Error;
pub use framework::BipolarFramework;
pub use semantics::{
    bipolar_complete_extensions, bipolar_grounded_extension, bipolar_preferred_extensions,
    bipolar_stable_extensions, is_support_closed,
};
pub use types::{CoalitionId, EdgeKind, SupportSemantics};
