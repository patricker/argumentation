//! # argumentation-weighted
//!
//! Weighted argumentation frameworks (Dunne, Hunter, McBurney, Parsons
//! & Wooldridge 2011) built on top of the [`argumentation`] crate's
//! Dung semantics.
//!
//! A weighted framework attaches an `f64` weight to each attack edge.
//! Under the **inconsistency-budget** semantics of Dunne et al., a
//! budget `β` permits attacks whose cumulative weight is at most `β`
//! to be tolerated for the purposes of computing Dung extensions. The
//! budget acts as a single knob: `β = 0` runs standard Dung semantics
//! over every attack; increasing `β` progressively tolerates more
//! attacks and accepts more arguments.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_weighted::framework::WeightedFramework;
//! use argumentation_weighted::sweep::min_budget_for_credulous;
//!
//! let mut wf = WeightedFramework::new();
//! wf.add_weighted_attack("attacker", "target", 0.6).unwrap();
//!
//! // At what budget does `target` become accepted?
//! let min = min_budget_for_credulous(&wf, &"target").unwrap();
//! assert_eq!(min, Some(0.6));
//! ```
//!
//! ## Semantics
//!
//! Implements the **inconsistency-budget** semantics of Dunne et al.
//! 2011 via exact subset enumeration: a budget `β` permits any subset
//! `S` of attacks whose cumulative weight is at most `β` to be
//! tolerated, and an argument is accepted at β iff it is accepted in
//! the Dung sense on *some* (credulous) or *all* (skeptical) of the
//! resulting residual frameworks. Enumeration is O(2^m) in the number
//! of attacks `m`; see [`reduce::ATTACK_ENUMERATION_LIMIT`] for the
//! guard.
//!
//! ## References
//!
//! - Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., &
//!   Wooldridge, M. (2011). *Weighted argument systems: Basic
//!   definitions, algorithms, and complexity results.* Artificial
//!   Intelligence 175(2).
//! - Bistarelli, S., Rossi, F., & Santini, F. (2018). *A collective
//!   defence against grouped attacks for weighted abstract argumentation
//!   frameworks.* IJAR 92.
//! - Coste-Marquis, S. et al. (2012). *Weighted attacks in
//!   argumentation frameworks.* KR 2012.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod reduce;
pub mod semantics;
pub mod sweep;
pub mod types;
pub mod weight_source;

pub use error::Error;
pub use framework::WeightedFramework;
pub use reduce::{ATTACK_ENUMERATION_LIMIT, dunne_residuals};
pub use semantics::{
    complete_at_budget, grounded_at_budget, is_credulously_accepted_at, is_skeptically_accepted_at,
    preferred_at_budget, stable_at_budget,
};
pub use sweep::{
    AcceptanceMode, SweepPoint, acceptance_trajectory, flip_points, min_budget_for_credulous,
};
pub use types::{AttackWeight, Budget, WeightedAttack};
pub use weight_source::{ClosureWeightSource, WeightSource, populate_from_source};
