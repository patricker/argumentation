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
//! ## Semantics notes
//!
//! v0.1.0 implements the **cumulative-weight threshold** approximation
//! of Dunne 2011's inconsistency-budget semantics: the cheapest
//! attacks are tolerated first until the cumulative weight would
//! exceed `β`. This is equivalent to the formal definition when
//! smaller attacks are strictly more expendable than larger ones
//! (which is the common case for relationship-modulated attack
//! strength). The full exponential enumeration over subsets of attacks
//! is a deferred v0.2.0 target.
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
pub use reduce::reduce_at_budget;
pub use semantics::{
    complete_at_budget, grounded_at_budget, is_credulously_accepted_at, is_skeptically_accepted_at,
    preferred_at_budget, stable_at_budget,
};
pub use sweep::{
    AcceptanceMode, SweepPoint, acceptance_trajectory, flip_points, min_budget_for_credulous,
};
pub use types::{AttackWeight, Budget, WeightedAttack};
pub use weight_source::{ClosureWeightSource, WeightSource, populate_from_source};
