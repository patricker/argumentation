//! The built-in scheme catalog. Each submodule exports `pub fn all() -> Vec<SchemeSpec>`
//! plus individual `pub fn <scheme_name>() -> SchemeSpec` constructors.
//! [`default_catalog`] collects all of them into a [`CatalogRegistry`].
//!
//! ## ID assignment
//!
//! Each category has a 100-element ID range starting at the corresponding
//! `*_ID_OFFSET` constant. Within a category, IDs are assigned sequentially
//! from the offset. This prevents cross-category collisions without a
//! central ID registry. The `catalog_coverage::scheme_ids_are_unique` test
//! enforces this at runtime.

pub mod causal;
pub mod epistemic;
pub mod popular;
pub mod practical;
pub mod source;

use crate::registry::CatalogRegistry;

/// First epistemic-scheme ID. Range: 1..100.
pub const EPISTEMIC_ID_OFFSET: u32 = 1;
/// First practical-scheme ID. Range: 100..200.
pub const PRACTICAL_ID_OFFSET: u32 = 100;
/// First source-based-scheme ID. Range: 200..300.
pub const SOURCE_ID_OFFSET: u32 = 200;
/// First popular-scheme ID. Range: 300..400.
pub const POPULAR_ID_OFFSET: u32 = 300;
/// First causal-scheme ID. Range: 400..500.
pub const CAUSAL_ID_OFFSET: u32 = 400;
/// First analogical-scheme ID. Range: 500..600.
#[allow(dead_code)]
pub const ANALOGICAL_ID_OFFSET: u32 = 500;

/// Build the default catalog containing all built-in schemes.
pub fn default_catalog() -> CatalogRegistry {
    let mut reg = CatalogRegistry::new();
    for scheme in epistemic::all() {
        reg.register(scheme);
    }
    for scheme in practical::all() {
        reg.register(scheme);
    }
    for scheme in source::all() {
        reg.register(scheme);
    }
    for scheme in popular::all() {
        reg.register(scheme);
    }
    for scheme in causal::all() {
        reg.register(scheme);
    }
    reg
}
