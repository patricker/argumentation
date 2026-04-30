//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.
//!
//! Bench-Capon (2003) extended Dung frameworks with *values* — each argument
//! promotes a value, and an *audience* is an ordering over values. Different
//! audiences reach different rational conclusions from the same framework.
//!
//! # Multi-value support
//!
//! This implementation follows Kaci & van der Torre (2008) and supports
//! arguments promoting multiple values. The defeat rule (Pareto-defeating)
//! degenerates to Bench-Capon (2003) single-value when each argument
//! promotes exactly one value. See [`framework::ValueBasedFramework::defeats`].
//!
//! Module barrel grows as Task 10 adds its remaining files.

pub mod acceptance;
pub mod apx;
pub mod error;
pub mod framework;
pub mod scheme_bridge;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use scheme_bridge::from_scheme_instances;
pub use types::{Audience, Value, ValueAssignment};
