//! # argumentation
//!
//! Formal argumentation in Rust. Two layers:
//!
//! 1. [`ArgumentationFramework`] — Dung 1995 abstract argumentation frameworks
//!    with all canonical semantics (grounded, complete, preferred, stable,
//!    ideal, semi-stable) and Caminada labellings.
//! 2. [`aspic::StructuredSystem`] — ASPIC+ structured argumentation with
//!    strict/defeasible rules, preferences, and automatic AF generation.
//!
//! See the crate README for design rationale.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub use error::Error;

pub mod framework;
pub use framework::ArgumentationFramework;

pub mod semantics;

pub mod parsers;
