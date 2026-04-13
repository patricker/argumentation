//! # argumentation-weighted
//!
//! Weighted argumentation frameworks (Dunne, Hunter, McBurney, Parsons
//! & Wooldridge 2011) built on top of the [`argumentation`] crate's
//! Dung semantics.
//!
//! A weighted framework attaches an `f64` weight to each attack edge.
//! Under the **inconsistency-budget** semantics of Dunne et al., a
//! budget `β` permits attacks whose cumulative weight is at most `β`
//! to be tolerated (i.e., treated as if they did not exist) for the
//! purposes of computing Dung extensions. The budget acts as a single
//! knob: `β = 0` runs the standard Dung semantics over every attack;
//! increasing `β` progressively tolerates more attacks and accepts
//! more arguments. The flip points — the discrete `β` values at which
//! an argument's acceptance changes — are computable from the sorted
//! attack weights alone.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod reduce;
pub mod semantics;
pub mod types;

pub use error::Error;
pub use framework::WeightedFramework;
