//! # argumentation
//!
//! Formal argumentation in Rust. Two independent layers:
//!
//! 1. **Abstract argumentation frameworks** ([`ArgumentationFramework`]) in the
//!    Dung 1995 tradition, with all canonical semantics: grounded, complete,
//!    preferred, stable, ideal, and semi-stable extensions, plus Caminada
//!    three-valued labellings.
//! 2. **Structured argumentation** ([`aspic::StructuredSystem`]) in the ASPIC+
//!    tradition (Modgil & Prakken 2014), with strict and defeasible rules,
//!    preference-based defeat resolution via the last-link principle, and
//!    automatic generation of a Dung AF for evaluation.
//!
//! ## Quick example: abstract AF
//!
//! ```
//! use argumentation::ArgumentationFramework;
//!
//! let mut af = ArgumentationFramework::new();
//! af.add_argument("a");
//! af.add_argument("b");
//! af.add_argument("c");
//! af.add_attack(&"a", &"b").unwrap();
//! af.add_attack(&"b", &"c").unwrap();
//!
//! let grounded = af.grounded_extension();
//! assert!(grounded.contains(&"a"));
//! assert!(grounded.contains(&"c"));
//! ```
//!
//! ## Quick example: ASPIC+ penguin
//!
//! ```
//! use argumentation::aspic::{StructuredSystem, Literal};
//!
//! let mut sys = StructuredSystem::new();
//! sys.add_ordinary(Literal::atom("penguin"));
//! sys.add_strict_rule(vec![Literal::atom("penguin")], Literal::atom("bird"));
//! let r1 = sys.add_defeasible_rule(
//!     vec![Literal::atom("bird")],
//!     Literal::atom("flies"),
//! );
//! let r2 = sys.add_defeasible_rule(
//!     vec![Literal::atom("penguin")],
//!     Literal::neg("flies"),
//! );
//! sys.prefer_rule(r2, r1).unwrap();
//!
//! let built = sys.build_framework().unwrap();
//! let preferred = built.framework.preferred_extensions().unwrap();
//! assert_eq!(preferred.len(), 1);
//! ```
//!
//! ## Performance
//!
//! Extension enumeration is currently implemented via subset search, which is
//! exponential in the number of arguments. The crate is correct and well-suited
//! for small-to-medium frameworks (up to ~20 arguments). Larger instances
//! require SAT-based enumeration; that's a planned future extension.
//!
//! ## Choosing a defeat ordering
//!
//! ASPIC+ defines two argument orderings for last-link defeat resolution:
//!
//! - [`aspic::DefeatOrdering::LastLink`] (default) compares the last
//!   defeasible rule of each argument, falling through to the last
//!   ordinary premise when both rule frontiers are empty. Appropriate
//!   for legal and normative reasoning.
//! - [`aspic::DefeatOrdering::WeakestLink`] compares the full set of
//!   defeasible rules and ordinary premises used in an argument.
//!   Appropriate for empirical reasoning where a chain is only as strong
//!   as its weakest link.
//!
//! Select an ordering at system construction time:
//!
//! ```
//! use argumentation::aspic::{StructuredSystem, DefeatOrdering};
//!
//! let _sys = StructuredSystem::with_ordering(DefeatOrdering::WeakestLink);
//! ```
//!
//! ## References
//!
//! - Dung, P.M. (1995). *On the acceptability of arguments...* AIJ 77(2).
//! - Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework for structured argumentation.*
//!   Argument & Computation 5(1).
//! - Caminada, M. (2006). *On the issue of reinstatement in argumentation.* JELIA.
//! - Baroni, P., Caminada, M., Giacomin, M. (2011). *An introduction to argumentation semantics.*
//!   KER 26(4).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod aspic;
pub mod error;
pub mod framework;
pub mod parsers;
pub mod semantics;

pub use aspic::StructuredSystem;
pub use error::Error;
pub use framework::ArgumentationFramework;
pub use semantics::{Label, Labelling};

/// Maximum number of arguments supported by the subset-enumeration
/// extension algorithms before [`Error::TooLarge`] is returned.
///
/// This is exposed so that consumers can statically assert against it
/// (e.g. in tests) or feature-check their framework sizes before calling
/// exponential enumerators. The limit is set to 22, matching the
/// crate-level claim of "practical up to ~20 arguments" with a small
/// safety margin.
pub const ENUMERATION_LIMIT: usize = semantics::ENUMERATION_LIMIT;

#[cfg(test)]
mod tests {
    const _: () = assert!(crate::ENUMERATION_LIMIT >= 20);
    const _: () = assert!(crate::ENUMERATION_LIMIT <= 25);

    #[test]
    fn enumeration_limit_matches_semantics() {
        assert_eq!(
            crate::ENUMERATION_LIMIT,
            crate::semantics::ENUMERATION_LIMIT
        );
    }
}
