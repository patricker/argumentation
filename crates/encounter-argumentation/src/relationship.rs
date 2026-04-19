//! Placeholder relationship-state types for Phase A. These will be
//! replaced in Phase C with real `societas` types.
//!
//! The shape is designed to match the five relationship dimensions
//! that `societas` tracks between character pairs: trust, fear,
//! respect, attraction, friendship. Each dimension is in `[-1.0, 1.0]`
//! where negative = adversarial and positive = positive affinity.

use std::collections::HashMap;

/// Five-dimensional relationship state between a pair of characters.
///
/// **Phase A stub.** Phase C will remove this in favour of
/// `societas::relationship::Relationship` (or equivalent) read from
/// the live societas state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelationshipDims {
    /// Trust. Higher = more belief in the other's word. Range: [-1, 1].
    pub trust: f64,
    /// Fear. Higher = more inhibited by the other's threat potential.
    pub fear: f64,
    /// Respect. Higher = more willing to defer to the other's judgment.
    pub respect: f64,
    /// Attraction. Higher = more positive affinity overall.
    pub attraction: f64,
    /// Friendship. Higher = deeper social bond.
    pub friendship: f64,
}

impl RelationshipDims {
    /// All-zero neutral relationship.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            trust: 0.0,
            fear: 0.0,
            respect: 0.0,
            attraction: 0.0,
            friendship: 0.0,
        }
    }
}

/// A snapshot of relationship state between pairs of characters.
///
/// **Phase A stub.** Phase C will replace this with a thin adapter
/// around `societas`'s live relationship tables.
#[derive(Debug, Clone, Default)]
pub struct RelationshipSnapshot {
    pairs: HashMap<(String, String), RelationshipDims>,
}

impl RelationshipSnapshot {
    /// Create an empty snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the relationship dimensions from `a`'s perspective of `b`.
    /// Relationships are directional in this model — `(a, b)` may
    /// differ from `(b, a)`.
    pub fn set(&mut self, a: impl Into<String>, b: impl Into<String>, dims: RelationshipDims) {
        self.pairs.insert((a.into(), b.into()), dims);
    }

    /// Look up the relationship dimensions for `(a, b)`. Returns
    /// `RelationshipDims::neutral()` if no explicit entry exists.
    #[must_use]
    pub fn get(&self, a: &str, b: &str) -> RelationshipDims {
        self.pairs
            .get(&(a.to_string(), b.to_string()))
            .copied()
            .unwrap_or_else(RelationshipDims::neutral)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_dims_are_zero() {
        let d = RelationshipDims::neutral();
        assert_eq!(d.trust, 0.0);
        assert_eq!(d.friendship, 0.0);
    }

    #[test]
    fn snapshot_get_returns_neutral_for_missing_pair() {
        let s = RelationshipSnapshot::new();
        let d = s.get("alice", "bob");
        assert_eq!(d, RelationshipDims::neutral());
    }

    #[test]
    fn snapshot_set_then_get_round_trips() {
        let mut s = RelationshipSnapshot::new();
        let d = RelationshipDims {
            trust: 0.8,
            fear: -0.2,
            respect: 0.5,
            attraction: 0.1,
            friendship: 0.6,
        };
        s.set("alice", "bob", d);
        assert_eq!(s.get("alice", "bob"), d);
        // Directional: (bob, alice) was never set.
        assert_eq!(s.get("bob", "alice"), RelationshipDims::neutral());
    }
}
