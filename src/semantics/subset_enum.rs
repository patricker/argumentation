//! Shared subset-enumeration helper for exponential extension enumerators.
//!
//! All subset-based semantics (complete, preferred, stable, ideal) iterate the
//! power set of the argument list. This module centralizes the size check
//! against [`ENUMERATION_LIMIT`] and the bit-to-subset decoding so the
//! enumerators only contain their own filter logic.

use crate::framework::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// Upper bound on the number of arguments we enumerate via subset search.
///
/// At `n = 22`, the power-set iteration runs `~4.2M` subsets, which
/// completes in well under a second even in debug builds with non-trivial
/// admissibility checks. The 22 limit matches the crate-level claim of
/// "practical up to ~20 arguments" from `src/lib.rs` and keeps an
/// additional safety margin for future per-iteration work growth.
///
/// Frameworks with more than [`ENUMERATION_LIMIT`] arguments are rejected
/// with [`crate::Error::TooLarge`]; a future SAT-based semantics entry
/// point would bypass this limit.
pub(crate) const ENUMERATION_LIMIT: usize = 22;

/// Collect the arguments of `af` into a deterministic sorted `Vec`, failing
/// fast with [`crate::Error::TooLarge`] when the count exceeds
/// [`ENUMERATION_LIMIT`]. Called by every subset-enumerating semantic.
pub(crate) fn sorted_args_or_too_large<A: Clone + Eq + Hash + Ord>(
    af: &ArgumentationFramework<A>,
) -> Result<Vec<A>, crate::Error> {
    let n = af.arguments().count();
    if n > ENUMERATION_LIMIT {
        return Err(crate::Error::TooLarge {
            arguments: n,
            limit: ENUMERATION_LIMIT,
        });
    }
    let mut v: Vec<A> = af.arguments().cloned().collect();
    v.sort();
    Ok(v)
}

/// Build the subset of `args` selected by the bitmask `bits`.
///
/// Bit `i` of `bits` includes `args[i]` in the output. Callers iterate
/// `0u64..(1u64 << args.len())` to visit every subset.
pub(crate) fn subset_from_bits<A: Clone + Eq + Hash>(args: &[A], bits: u64) -> HashSet<A> {
    (0..args.len())
        .filter(|i| bits & (1u64 << i) != 0)
        .map(|i| args[i].clone())
        .collect()
}
