//! # argumentation-bipolar
//!
//! Bipolar argumentation frameworks (Cayrol & Lagasquie-Schiex 2005,
//! Amgoud et al. 2008, Nouioua & Risch 2011) built on top of the
//! [`argumentation`] crate's Dung semantics.
//!
//! A bipolar framework extends Dung's abstract argumentation with a
//! second directed edge relation: **support**. Arguments can attack and
//! support one another. This crate implements *necessary support*
//! semantics: `A` supports `B` means `A` must be accepted for `B` to be
//! acceptable.
//!
//! The semantics pipeline flattens a bipolar framework into an
//! equivalent Dung framework (direct attacks + derived attacks from
//! supported/mediated/secondary attack rules), runs the existing Dung
//! semantics via [`argumentation::ArgumentationFramework`], then filters
//! the resulting extensions to those that are support-closed (every
//! accepted argument has all its necessary supporters also accepted).
//!
//! Coalitions are strongly-connected components of the support graph:
//! characters who mutually back each other's positions form a coalition
//! that the drama manager can reason about as a unit.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod coalition;
pub mod derived;
pub mod error;
pub mod flatten;
pub mod framework;
pub mod semantics;
pub mod types;

pub use error::Error;
pub use framework::BipolarFramework;
