//! # argumentation-weighted-bipolar
//!
//! Weighted bipolar argumentation frameworks: a composition of
//! [`argumentation-weighted`](../argumentation_weighted/index.html) and
//! [`argumentation-bipolar`](../argumentation_bipolar/index.html)
//! following Amgoud, Cayrol, Lagasquie-Schiex & Livet 2008, with
//! Dunne 2011 inconsistency-budget semantics applied uniformly over
//! attacks and supports.
//!
//! Each edge (attack or support) carries a non-negative finite weight.
//! A budget `β` permits any subset `S` of edges whose cumulative weight
//! is at most `β` to be tolerated (dropped). Acceptance queries iterate
//! every β-inconsistent subset and aggregate:
//!
//! - **Credulous**: accepted in some preferred extension of some residual.
//! - **Skeptical**: accepted in every preferred extension of every residual.
//!
//! ## Why compose
//!
//! `argumentation-bipolar` already implements necessary-support
//! semantics (Nouioua & Risch 2011) by flattening + filtering against
//! the core Dung layer. `argumentation-weighted` already implements
//! Dunne 2011 inconsistency-budget enumeration over attack subsets.
//! This crate glues them: residuals are bipolar (not plain Dung), so
//! the preferred-extension aggregation runs through the bipolar
//! semantics layer.
//!
//! ## References
//!
//! - Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P.
//!   (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10).
//! - Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., &
//!   Wooldridge, M. (2011). *Weighted argument systems.* AIJ 175(2).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod reduce;
pub mod semantics;
pub mod types;

pub use error::Error;
pub use framework::WeightedBipolarFramework;
pub use reduce::{wbipolar_residuals, EDGE_ENUMERATION_LIMIT};
pub use semantics::{is_credulously_accepted_at, is_skeptically_accepted_at};
pub use types::WeightedSupport;
