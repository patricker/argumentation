//! Argumentation semantics: extensions and labellings.

pub mod admissibility;
pub mod complete;
pub mod grounded;
pub mod ideal;
pub mod labelling;
pub mod preferred;
pub mod semi_stable;
pub mod stable;
pub(crate) mod subset_enum;

pub use labelling::{Label, Labelling};
pub(crate) use subset_enum::ENUMERATION_LIMIT;
