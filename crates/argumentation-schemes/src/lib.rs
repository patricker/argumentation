//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod types;

pub use error::Error;
