//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.
//!
//! Bench-Capon (2003) extended Dung frameworks with *values* — each argument
//! promotes a value, and an *audience* is an ordering over values. Different
//! audiences reach different rational conclusions from the same framework.
//!
//! Module barrel grows as Tasks 2/3/5/6/9/10 add their files. The full
//! crate-level docs land in Task 2 along with the public types.

pub mod error;

pub use error::Error;
