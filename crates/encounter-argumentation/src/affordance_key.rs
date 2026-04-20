//! Canonical key for an (actor, affordance_name, bindings) triple
//! used to index scheme instances against encounter affordances.
//!
//! Bindings are stored as a `BTreeMap` internally so the key's
//! hash and equality are deterministic regardless of insertion
//! order into the source `HashMap`.

use std::collections::{BTreeMap, HashMap};

/// Canonical identifier for a scheme instance seeded against a
/// specific (actor, affordance, bindings) triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AffordanceKey {
    actor: String,
    affordance_name: String,
    bindings: BTreeMap<String, String>,
}

impl AffordanceKey {
    /// Construct a key from raw parts. Bindings are normalised into
    /// a `BTreeMap` to give the key a deterministic hash.
    #[must_use]
    pub fn new(
        actor: impl Into<String>,
        affordance_name: impl Into<String>,
        bindings: &HashMap<String, String>,
    ) -> Self {
        Self {
            actor: actor.into(),
            affordance_name: affordance_name.into(),
            bindings: bindings.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }

    /// The actor component of the key.
    #[must_use]
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// The affordance-name component of the key.
    #[must_use]
    pub fn affordance_name(&self) -> &str {
        &self.affordance_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_key_round_trips_simple_bindings() {
        let mut b = HashMap::new();
        b.insert("expert".to_string(), "alice".to_string());
        let k = AffordanceKey::new("alice", "assert_claim", &b);
        assert_eq!(k.actor(), "alice");
        assert_eq!(k.affordance_name(), "assert_claim");
    }

    #[test]
    fn equal_bindings_in_different_insertion_orders_produce_equal_keys() {
        let mut b1 = HashMap::new();
        b1.insert("expert".to_string(), "alice".to_string());
        b1.insert("claim".to_string(), "fortify_east".to_string());
        let mut b2 = HashMap::new();
        b2.insert("claim".to_string(), "fortify_east".to_string());
        b2.insert("expert".to_string(), "alice".to_string());
        let k1 = AffordanceKey::new("alice", "x", &b1);
        let k2 = AffordanceKey::new("alice", "x", &b2);
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_actors_produce_different_keys() {
        let b = HashMap::new();
        let k1 = AffordanceKey::new("alice", "x", &b);
        let k2 = AffordanceKey::new("bob", "x", &b);
        assert_ne!(k1, k2);
    }

    #[test]
    fn equal_keys_produce_equal_hashes() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        fn h<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }
        let mut b1 = HashMap::new();
        b1.insert("a".to_string(), "1".to_string());
        b1.insert("b".to_string(), "2".to_string());
        let mut b2 = HashMap::new();
        b2.insert("b".to_string(), "2".to_string());
        b2.insert("a".to_string(), "1".to_string());
        let k1 = AffordanceKey::new("x", "y", &b1);
        let k2 = AffordanceKey::new("x", "y", &b2);
        assert_eq!(h(&k1), h(&k2));
    }
}
