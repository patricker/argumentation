//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.

pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use types::{Audience, Value, ValueAssignment};
