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
//! Module barrel grows as Tasks 5/6/9/10 add their files.

pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use types::{Audience, Value, ValueAssignment};
