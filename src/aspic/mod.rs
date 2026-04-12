//! ASPIC+ structured argumentation (Modgil & Prakken 2014).
//!
//! Build arguments from a knowledge base and strict/defeasible rules.
//! Compute attacks (undercut, undermine, rebut). Resolve defeats via
//! rule preferences. Emit an abstract framework for Dung-semantics evaluation.

pub mod language;

pub use language::Literal;
