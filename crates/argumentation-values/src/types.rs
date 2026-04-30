//! Core types: `Value`, `ValueAssignment`, `Audience`.

use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::Hash;

/// A value that an argument can promote.
///
/// Currently a thin newtype around `String`. May become an extensible
/// trait in the future if consumers need richer value semantics
/// (numeric magnitudes, hierarchical taxonomies). v0 keeps it simple.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value(String);

impl Value {
    /// Construct a `Value` from any string-like input.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Maps each argument to the set of values it promotes.
///
/// An empty set (or absent entry) means "promotes no value" — under VAF
/// semantics such arguments defeat unconditionally (no value preference
/// can save a target whose attacker promotes no value, and vice versa).
///
/// Multi-value support per Kaci & van der Torre (2008): an argument may
/// promote several values simultaneously. Single-value (Bench-Capon 2003)
/// is the degenerate case where every set has exactly one element.
#[derive(Debug, Clone)]
pub struct ValueAssignment<A: Eq + Hash> {
    /// `SmallVec<[Value; 1]>` keeps the common single-value case allocation-free.
    promoted: HashMap<A, SmallVec<[Value; 1]>>,
}

impl<A: Eq + Hash> Default for ValueAssignment<A> {
    fn default() -> Self {
        Self {
            promoted: HashMap::new(),
        }
    }
}

impl<A: Eq + Hash + Clone> ValueAssignment<A> {
    /// Construct an empty assignment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a value to the set of values promoted by `arg`.
    /// Returns `&mut self` for builder chaining.
    pub fn promote(&mut self, arg: A, value: Value) -> &mut Self {
        let entry = self.promoted.entry(arg).or_default();
        if !entry.contains(&value) {
            entry.push(value);
        }
        self
    }

    /// The set of values promoted by `arg`. Returns an empty slice if
    /// `arg` is not present (which is semantically the "no values" case).
    pub fn values(&self, arg: &A) -> &[Value] {
        self.promoted
            .get(arg)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Iterator over (argument, values) entries.
    pub fn entries(&self) -> impl Iterator<Item = (&A, &[Value])> {
        self.promoted.iter().map(|(k, v)| (k, v.as_slice()))
    }

    /// All distinct values mentioned anywhere in the assignment.
    pub fn distinct_values(&self) -> std::collections::BTreeSet<&Value> {
        self.promoted.values().flatten().collect()
    }
}

/// An audience is a strict partial order over values, represented as
/// ranked tiers. Each inner `Vec<Value>` is one tier; values within a
/// tier are equally preferred. Earlier tiers are strictly more preferred
/// than later tiers.
///
/// # Examples
///
/// ```rust
/// use argumentation_values::{Audience, Value};
/// let life = Value::new("life");
/// let property = Value::new("property");
///
/// // Total order: life > property
/// let strict = Audience::total([life.clone(), property.clone()]);
/// assert!(strict.prefers(&life, &property));
/// assert!(!strict.prefers(&property, &life));
///
/// // Incomparable values
/// let flat = Audience::from_tiers(vec![vec![life.clone(), property.clone()]]);
/// assert!(!flat.prefers(&life, &property));
/// assert!(!flat.prefers(&property, &life));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Audience {
    /// Each inner Vec is a tier; index 0 is most preferred.
    tiers: Vec<Vec<Value>>,
}

impl Audience {
    /// Construct an empty audience (no preferences — all attacks survive).
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a total ordering from an iterator of values, most
    /// preferred first.
    pub fn total<I: IntoIterator<Item = Value>>(ranked: I) -> Self {
        Self {
            tiers: ranked.into_iter().map(|v| vec![v]).collect(),
        }
    }

    /// Construct from explicit ranked tiers. Each inner vec is one tier
    /// of equally preferred values.
    pub fn from_tiers(tiers: Vec<Vec<Value>>) -> Self {
        Self { tiers }
    }

    /// Returns true iff `a` is *strictly* preferred to `b` under this audience.
    /// Returns false if either value is unranked (incomparable).
    pub fn prefers(&self, a: &Value, b: &Value) -> bool {
        match (self.rank(a), self.rank(b)) {
            (Some(ra), Some(rb)) => ra < rb,
            _ => false,
        }
    }

    /// 0-indexed tier of `v` (0 = most preferred), or `None` if `v` is
    /// unranked (not mentioned in any tier).
    ///
    /// Public so consumers (e.g., `ValueAwareScorer`) can compute boost
    /// magnitudes without re-implementing the lookup.
    pub fn rank(&self, v: &Value) -> Option<usize> {
        self.tiers
            .iter()
            .position(|tier| tier.iter().any(|x| x == v))
    }

    /// Iterate the distinct values mentioned in this audience.
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.tiers.iter().flatten()
    }

    /// Number of distinct values in this audience.
    pub fn value_count(&self) -> usize {
        self.tiers.iter().map(|t| t.len()).sum()
    }

    /// Number of tiers (rank levels).
    pub fn tier_count(&self) -> usize {
        self.tiers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_assignment_dedupes_promotions() {
        let mut va: ValueAssignment<&str> = ValueAssignment::new();
        va.promote("a", Value::new("life"));
        va.promote("a", Value::new("life"));
        assert_eq!(va.values(&"a").len(), 1);
    }

    #[test]
    fn value_assignment_accepts_multi_value() {
        let mut va: ValueAssignment<&str> = ValueAssignment::new();
        va.promote("a", Value::new("life"));
        va.promote("a", Value::new("autonomy"));
        assert_eq!(va.values(&"a").len(), 2);
    }

    #[test]
    fn audience_total_orders_strictly() {
        let a = Audience::total([Value::new("life"), Value::new("property")]);
        assert!(a.prefers(&Value::new("life"), &Value::new("property")));
        assert!(!a.prefers(&Value::new("property"), &Value::new("life")));
    }

    #[test]
    fn audience_unranked_values_are_incomparable() {
        let a = Audience::total([Value::new("life")]);
        assert!(!a.prefers(&Value::new("property"), &Value::new("life")));
        assert!(!a.prefers(&Value::new("life"), &Value::new("property")));
    }

    #[test]
    fn audience_intra_tier_values_are_incomparable() {
        let a = Audience::from_tiers(vec![vec![Value::new("life"), Value::new("liberty")]]);
        assert!(!a.prefers(&Value::new("life"), &Value::new("liberty")));
        assert!(!a.prefers(&Value::new("liberty"), &Value::new("life")));
    }

    #[test]
    fn audience_distinct_values_count() {
        let a = Audience::from_tiers(vec![
            vec![Value::new("a"), Value::new("b")],
            vec![Value::new("c")],
        ]);
        assert_eq!(a.value_count(), 3);
        assert_eq!(a.tier_count(), 2);
    }

    #[test]
    fn audience_rank_returns_tier_index() {
        let a = Audience::from_tiers(vec![
            vec![Value::new("life"), Value::new("liberty")],
            vec![Value::new("property")],
        ]);
        assert_eq!(a.rank(&Value::new("life")), Some(0));
        assert_eq!(a.rank(&Value::new("liberty")), Some(0));
        assert_eq!(a.rank(&Value::new("property")), Some(1));
        assert_eq!(a.rank(&Value::new("comfort")), None);
    }
}
