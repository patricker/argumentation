//! `NameResolver`: trait mapping actor-name strings to `societas` `EntityId`s.
//!
//! The bridge works in actor names (`String`) but `societas-relations`
//! queries on `EntityId` (a 16-byte opaque). Consumers supply a resolver
//! that knows the mapping for their world (e.g. a persona-registry
//! lookup, or a literal `HashMap<String, EntityId>` seeded at scene
//! setup).
//!
//! A blanket impl for `HashMap<String, EntityId>` is provided so that
//! consumers with a fixed cast list can pass a `HashMap` directly
//! without writing a wrapper type.

use societas_core::EntityId;
use std::collections::HashMap;

/// Maps actor-name strings to the `EntityId` used by societas.
///
/// Implementations should return `None` for unknown names rather than
/// panic. [`crate::societas_relationship::SocietasRelationshipSource`]
/// treats `None` as "this actor has no relationship data" and falls back
/// to the baseline weight for any pair involving the unknown actor.
pub trait NameResolver {
    /// Look up the `EntityId` for the given actor name. Returns `None`
    /// if no mapping exists.
    fn resolve(&self, name: &str) -> Option<EntityId>;
}

impl NameResolver for HashMap<String, EntityId> {
    fn resolve(&self, name: &str) -> Option<EntityId> {
        self.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_resolves_known_name() {
        let mut m = HashMap::new();
        m.insert("alice".to_string(), EntityId::from_u64(1));
        assert_eq!(m.resolve("alice"), Some(EntityId::from_u64(1)));
    }

    #[test]
    fn hashmap_returns_none_for_unknown_name() {
        let m: HashMap<String, EntityId> = HashMap::new();
        assert!(m.resolve("nobody").is_none());
    }

    #[test]
    fn hashmap_distinguishes_distinct_entities() {
        let mut m = HashMap::new();
        m.insert("alice".to_string(), EntityId::from_u64(1));
        m.insert("bob".to_string(), EntityId::from_u64(2));
        assert_ne!(m.resolve("alice"), m.resolve("bob"));
    }
}
