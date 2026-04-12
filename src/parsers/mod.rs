//! Parsers for standard argumentation-framework file formats.

pub mod apx;
pub mod tgf;

pub use apx::parse_apx;
pub use tgf::parse_tgf;
