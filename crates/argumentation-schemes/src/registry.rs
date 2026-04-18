//! `CatalogRegistry`: an in-memory collection of schemes with lookup by
//! name, id, and category.

use crate::scheme::SchemeSpec;
use crate::types::{SchemeCategory, SchemeId};
use std::collections::HashMap;

/// A collection of argumentation schemes, indexed for lookup.
#[derive(Debug, Clone)]
pub struct CatalogRegistry {
    schemes: Vec<SchemeSpec>,
    by_id: HashMap<SchemeId, usize>,
    by_key: HashMap<String, usize>,
}

impl CatalogRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            schemes: Vec::new(),
            by_id: HashMap::new(),
            by_key: HashMap::new(),
        }
    }

    /// Register a scheme. The last write wins for duplicate ids or keys —
    /// the [`crate::catalog`] tests guarantee no duplicates exist in the
    /// default catalog.
    pub fn register(&mut self, scheme: SchemeSpec) {
        let key = scheme.key();
        let idx = self.schemes.len();
        self.by_id.insert(scheme.id, idx);
        self.by_key.insert(key, idx);
        self.schemes.push(scheme);
    }

    /// Look up a scheme by its unique id.
    pub fn by_id(&self, id: SchemeId) -> Option<&SchemeSpec> {
        self.by_id.get(&id).map(|&idx| &self.schemes[idx])
    }

    /// Look up a scheme by its snake_case key (derived from the name).
    pub fn by_key(&self, key: &str) -> Option<&SchemeSpec> {
        self.by_key.get(key).map(|&idx| &self.schemes[idx])
    }

    /// Return all schemes in a given category.
    pub fn by_category(&self, category: SchemeCategory) -> Vec<&SchemeSpec> {
        self.schemes
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Return all registered schemes.
    pub fn all(&self) -> &[SchemeSpec] {
        &self.schemes
    }

    /// Number of registered schemes.
    pub fn len(&self) -> usize {
        self.schemes.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.schemes.is_empty()
    }

    /// Build a registry pre-loaded with the default Walton catalog.
    #[must_use]
    pub fn with_default() -> Self {
        crate::catalog::default_catalog()
    }

    /// Look up a scheme by its canonical (human-readable) name.
    ///
    /// Names are matched exactly (case-sensitive). Use [`Self::by_key`] to
    /// look up by the derived snake_case key instead.
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<&SchemeSpec> {
        self.schemes.iter().find(|s| s.name == name)
    }
}

impl Default for CatalogRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critical::CriticalQuestion;
    use crate::scheme::*;
    use crate::types::*;

    fn test_scheme(id: u32, name: &str, cat: SchemeCategory) -> SchemeSpec {
        SchemeSpec {
            id: SchemeId(id),
            name: name.into(),
            category: cat,
            premises: vec![PremiseSlot::new("p", "premise", SlotRole::Proposition)],
            conclusion: ConclusionTemplate::positive("c", "?p"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "?p?",
                Challenge::PremiseTruth("p".into()),
            )],
            metadata: SchemeMetadata {
                citation: "test".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        }
    }

    #[test]
    fn register_and_lookup_by_id() {
        let mut reg = CatalogRegistry::new();
        reg.register(test_scheme(1, "Test Scheme", SchemeCategory::Epistemic));
        assert!(reg.by_id(SchemeId(1)).is_some());
        assert!(reg.by_id(SchemeId(99)).is_none());
    }

    #[test]
    fn lookup_by_key_uses_snake_case_name() {
        let mut reg = CatalogRegistry::new();
        reg.register(test_scheme(
            1,
            "Argument from Expert Opinion",
            SchemeCategory::Epistemic,
        ));
        assert!(reg.by_key("argument_from_expert_opinion").is_some());
        assert!(reg.by_key("Argument from Expert Opinion").is_none());
    }

    #[test]
    fn filter_by_category_returns_only_matching() {
        let mut reg = CatalogRegistry::new();
        reg.register(test_scheme(1, "Scheme A", SchemeCategory::Epistemic));
        reg.register(test_scheme(2, "Scheme B", SchemeCategory::Practical));
        reg.register(test_scheme(3, "Scheme C", SchemeCategory::Epistemic));
        assert_eq!(reg.by_category(SchemeCategory::Epistemic).len(), 2);
        assert_eq!(reg.by_category(SchemeCategory::Practical).len(), 1);
        assert_eq!(reg.by_category(SchemeCategory::Causal).len(), 0);
    }

    #[test]
    fn len_and_is_empty_track_registrations() {
        let mut reg = CatalogRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        reg.register(test_scheme(1, "A", SchemeCategory::Causal));
        reg.register(test_scheme(2, "B", SchemeCategory::Causal));
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 2);
    }
}
