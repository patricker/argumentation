# Phase C: Societas Relationship Adapter — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Phase A `RelationshipSnapshot` stub with a real adapter that reads attack weights from live `societas-relations` state, and fix the Phase A soundness bug where `WeightSource` treated `ArgumentId` (a conclusion literal) as if it were an actor name.

**Architecture:** A new `SocietasRelationshipSource<'a, R>` implements `WeightSource<ArgumentId>` by (1) looking up attacker/target *actors* via the bridge's `actors_by_argument` map (exposed via a new public accessor), (2) resolving actor-name strings to `EntityId` via a pluggable `NameResolver`, (3) querying the five societas relationship dimensions per (attacker_actor, target_actor) pair at a caller-supplied `Tick`, (4) aggregating by arithmetic mean across pairs, and (5) applying the same coefficient recipe as Phase A to produce a weight in `[0.0, 1.0]`. The Phase A stub (`RelationshipDims`, `RelationshipSnapshot`, old `RelationshipWeightSource`) is deleted — no backward-compat shims. Bumps to v0.4.0.

**Tech Stack:** Rust 2024 edition. Adds `societas-core`, `societas-relations` as path-dep dependencies of `encounter-argumentation`, plus `societas-memory` as a dev-dependency for test fixtures.

---

## Verified Societas APIs (quoted from source, NOT paraphrased)

Every signature below was read verbatim from source before writing this plan. If an implementer encounters a mismatch, stop and flag it — do NOT adapt the plan silently.

### `societas_core`

```rust
// societas/crates/core/src/types.rs:7-11
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub [u8; 16]);

impl EntityId {
    pub const fn from_u64(n: u64) -> Self;
    pub fn uniform(byte: u8) -> Self;
    pub fn hex(&self) -> String;
    pub fn as_u64(&self) -> u64;
    pub fn new(id: u64) -> Self;
}

// societas/crates/core/src/types.rs:57
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tick(pub i64);
impl Tick { pub fn new(t: i64) -> Self; }
// NOTE: no Tick::ZERO constant — use Tick(0).

// societas/crates/core/src/types.rs:95
pub type Predicate = u32;

// societas/crates/core/src/store.rs:10-58
pub trait SocialStore {
    fn assert(&mut self, subject: EntityId, predicate: Predicate, value: Value, start: Tick) -> AssertionId;
    fn close(&mut self, id: AssertionId, end: Tick);
    fn replace(&mut self, subject: EntityId, predicate: Predicate, value: Value, at: Tick) -> AssertionId;
    fn active_for_entity(&self, entity: EntityId, at: Tick) -> Vec<Fact>;
    fn active_for_entity_pred(&self, entity: EntityId, predicate: Predicate, at: Tick) -> Vec<Fact>;
    fn scan_predicate(&self, predicate: Predicate, at: Tick) -> Vec<Fact>;
    // ...plus history_*, get, next_id, allocate_entity
}
// NOTE: no Send + Sync supertrait. A &dyn SocialStore is NOT Sync.
```

### `societas_relations`

```rust
// societas/crates/relations/src/registry.rs:39-50
pub struct RelationshipRegistry {
    types: Vec<RelationshipType>,
    decay_fn: Box<dyn DecayFn>,
}

impl RelationshipRegistry {
    pub fn new() -> Self;
    pub fn score(
        &self,
        store: &dyn SocialStore,
        source: EntityId,
        target: EntityId,
        dimension: Dimension,
        tick: Tick,
    ) -> f64;  // clamped to [-1.0, 1.0]
    // ...plus add_modifier, register_type, with_decay, etc.
}
// NOTE: there is NO batch "score all five dimensions" method.
// Callers loop over Dimension::{Trust, Friendship, Attraction, Fear, Respect}.

// societas/crates/relations/src/types.rs:8-16
pub enum Dimension {
    Trust, Friendship, Attraction, Fear, Respect,
    Custom(u16),
}
```

### `societas_memory` (for test fixtures only)

```rust
// societas/crates/memory/src/lib.rs:5-15
#[derive(Clone, Debug, Default)]
pub struct MemStore { /* private */ }

impl MemStore {
    pub fn new() -> Self;
    pub fn fact_count(&self) -> usize;
}
impl SocialStore for MemStore { /* ... */ }
```

### Example from societas's own tests — copy this fixture shape

```rust
// societas/crates/relations/tests/integration.rs:213-216 (paraphrased minimal)
use societas_core::{EntityId, Tick, ModifierSource};
use societas_memory::MemStore;
use societas_relations::{RelationshipRegistry, types::Dimension};

let mut store = MemStore::new();
let reg = RelationshipRegistry::new();
let alice = EntityId::from_u64(1);
let bob = EntityId::from_u64(2);

reg.add_modifier(
    &mut store, alice, bob, Dimension::Trust,
    0.8,  // magnitude
    0.0,  // decay_rate (0.0 = permanent)
    ModifierSource::Personality,
    Tick(0),
);

let score: f64 = reg.score(&store, alice, bob, Dimension::Trust, Tick(0));
// score ≈ 0.8
```

---

## File Structure

**New files (2):**
- `crates/encounter-argumentation/src/name_resolver.rs` — `NameResolver` trait and `HashMap<String, EntityId>` blanket impl.
- `crates/encounter-argumentation/src/societas_relationship.rs` — the new `SocietasRelationshipSource<'a, R>` adapter + coefficient constants. (Replaces the stub WeightSource.)

**Modified files (5):**
- `crates/encounter-argumentation/Cargo.toml` — add societas path-deps + dev-dep on `societas-memory`, bump version to `0.4.0`.
- `crates/encounter-argumentation/src/state.rs` — add `pub fn actors_by_argument(&self) -> &HashMap<ArgumentId, Vec<String>>` accessor.
- `crates/encounter-argumentation/src/relationship.rs` — delete entire file contents (we no longer export a `relationship` module at all).
- `crates/encounter-argumentation/src/lib.rs` — drop the `relationship` mod + its re-exports; add `name_resolver` and `societas_relationship` mods + re-exports.
- `crates/encounter-argumentation/CHANGELOG.md` — add `[0.4.0]` section.

**Modified test (1):**
- `crates/encounter-argumentation/tests/uc_relationship_modulation.rs` — rewrite to use `SocietasRelationshipSource` + `MemStore` fixtures.

---

## Open Design Questions (resolved)

All four open questions from the scoping discussion have been resolved in favor of simplicity:

| # | Question | Resolution | Rationale |
|---|---|---|---|
| Q1 | Name → EntityId resolution strategy | Trait `NameResolver` with a blanket impl for `HashMap<String, EntityId>` | Lets consumers either plug in a real name-directory service or pass a literal HashMap. Blanket impl gives tests a zero-ceremony fixture. |
| Q2 | Tick ownership | Owned `Tick` held on the adapter, construction-time only, no setter | The adapter is consulted at *seeding* time (inside `add_weighted_attack`), not during resolution — the tick doesn't move between weight computations within a seeding pass. Consumers that want a new tick build a new adapter. |
| Q3 | Multi-actor pair aggregation | Arithmetic mean across (attacker_actor × target_actor) Cartesian product | Documented. No alternative offered in v0.4.0. |
| Q4 | Coefficient calibration | Ship the Phase A coefficients unchanged, exposed as `pub const` | Preserves observable behavior vs the stub. Calibration deferred to a follow-up (coefficient tuning without API churn). |

---

## Coefficient recipe (preserved from Phase A, now formalized)

```rust
// In societas_relationship.rs, as pub consts so tests and consumers can pin them.
pub const BASELINE_WEIGHT: f64 = 0.5;
pub const TRUST_COEF: f64 = -0.15;
pub const FEAR_COEF: f64 = 0.25;
pub const RESPECT_COEF: f64 = -0.05;
pub const ATTRACTION_COEF: f64 = -0.05;
pub const FRIENDSHIP_COEF: f64 = -0.10;

// For a single (attacker, target) pair:
// weight = clamp(
//   BASELINE_WEIGHT
//     + TRUST_COEF       * trust
//     + FEAR_COEF        * fear
//     + RESPECT_COEF     * respect
//     + ATTRACTION_COEF  * attraction
//     + FRIENDSHIP_COEF  * friendship,
//   0.0, 1.0,
// )
// For multi-pair arguments, mean the per-pair weights AFTER the clamp.
```

---

## Task overview

| # | Task | Files |
|---|---|---|
| 1 | Cargo deps + `SocialStore` unused-import guard | `Cargo.toml` |
| 2 | `NameResolver` trait + HashMap impl | `src/name_resolver.rs` |
| 3 | `actors_by_argument` accessor | `src/state.rs` |
| 4 | `SocietasRelationshipSource` struct + constants | `src/societas_relationship.rs` |
| 5 | `WeightSource::weight_for` — empty/missing paths | `src/societas_relationship.rs` |
| 6 | `WeightSource::weight_for` — single-pair scoring | `src/societas_relationship.rs` |
| 7 | `WeightSource::weight_for` — mean-aggregate multi-pair | `src/societas_relationship.rs` |
| 8 | Delete stubs + update `lib.rs` re-exports | `src/relationship.rs` (delete) + `src/lib.rs` |
| 9 | Rewrite `uc_relationship_modulation.rs` integration test | `tests/uc_relationship_modulation.rs` |
| 10 | CHANGELOG + version 0.4.0 + tag | `CHANGELOG.md` + `Cargo.toml` |
| 11 | Dispatch code review | — |

Each task is self-contained and commits on green. Every piece of code below is copy-pasteable — no paraphrased snippets.

---

### Task 1: Cargo deps

**Files:**
- Modify: `crates/encounter-argumentation/Cargo.toml`

Adds societas crates as path deps. `societas-memory` is dev-only because production consumers bring their own `SocialStore` backend.

- [ ] **Step 1: Verify the societas workspace is at the expected path**

Run: `ls /home/peter/code/societas/crates/core /home/peter/code/societas/crates/relations /home/peter/code/societas/crates/memory`
Expected: all three directories exist.

- [ ] **Step 2: Add deps to Cargo.toml**

Edit `crates/encounter-argumentation/Cargo.toml`. Replace the current `[dependencies]` / `[dev-dependencies]` sections with:

```toml
[dependencies]
encounter = { path = "../../../encounter" }
argumentation = { path = "../.." }
argumentation-schemes = { path = "../argumentation-schemes" }
argumentation-bipolar = { path = "../argumentation-bipolar" }
argumentation-weighted = { path = "../argumentation-weighted" }
argumentation-weighted-bipolar = { path = "../argumentation-weighted-bipolar" }
societas-core = { path = "../../../societas/crates/core" }
societas-relations = { path = "../../../societas/crates/relations" }
thiserror = "2.0"

[dev-dependencies]
societas-memory = { path = "../../../societas/crates/memory" }
```

Do NOT bump the version field yet — that happens in Task 10 with the CHANGELOG.

- [ ] **Step 3: Verify the crate still compiles**

Run: `cargo check -p encounter-argumentation`
Expected: `Checking encounter-argumentation v0.3.0 ... Finished`.

If societas's `rust-version.workspace = true` requires a newer toolchain than ours, check `rustc --version` against `/home/peter/code/societas/Cargo.toml`'s `rust-version` field and bump local rust-toolchain if necessary.

- [ ] **Step 4: Run the existing test suite**

Run: `cargo test -p encounter-argumentation`
Expected: all existing tests pass (no source changes yet, only Cargo.toml).

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/Cargo.toml
git commit -m "deps(encounter-argumentation): add societas-core, societas-relations (and memory as dev)"
```

---

### Task 2: `NameResolver` trait + HashMap blanket impl

**Files:**
- Create: `crates/encounter-argumentation/src/name_resolver.rs`

Trait that converts an actor's display-name string to its `EntityId`. Ships with a blanket impl for `HashMap<String, EntityId>` so tests — and simple consumers — get it for free.

- [ ] **Step 1: Write the failing tests**

Create `crates/encounter-argumentation/src/name_resolver.rs` with this content:

```rust
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
```

- [ ] **Step 2: Register the module so the tests can run**

Edit `crates/encounter-argumentation/src/lib.rs`. Add to the module list, keeping alphabetical ordering with the existing mods:

```rust
pub mod name_resolver;
```

Place it between `pub mod knowledge;` and `pub mod relationship;` (which still exists at this point — we delete it in Task 8).

Also add the re-export near the existing `pub use knowledge::{...};`:

```rust
pub use name_resolver::NameResolver;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p encounter-argumentation --lib name_resolver`
Expected: 3 tests pass.

- [ ] **Step 4: Run the full test suite to verify no regression**

Run: `cargo test -p encounter-argumentation`
Expected: all tests pass.

- [ ] **Step 5: Clippy check**

Run: `cargo clippy -p encounter-argumentation --all-targets --no-deps -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add crates/encounter-argumentation/src/name_resolver.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): NameResolver trait with HashMap<String,EntityId> blanket impl"
```

---

### Task 3: `actors_by_argument` accessor on state

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

The Phase A soundness bug stems from the fact that `RelationshipWeightSource` could not see the actor-per-argument map — it's private. Expose it as a read-only accessor. No behavioral change to the state itself.

- [ ] **Step 1: Add the failing test**

Edit `crates/encounter-argumentation/src/state.rs`. Inside the existing `#[cfg(test)] mod tests { ... }` block (near the end of the file, after the existing `has_accepted_counter_by_detects_responder_attacker_at_beta` test), add:

```rust
    #[test]
    fn actors_by_argument_exposes_actor_map() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let instance = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "alice".to_string()),
                    ("domain".to_string(), "military".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance("alice", instance);
        let map = state.actors_by_argument();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&id), Some(&vec!["alice".to_string()]));
    }

    #[test]
    fn actors_by_argument_is_empty_on_new_state() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert!(state.actors_by_argument().is_empty());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation --lib actors_by_argument`
Expected: FAIL — `no method named 'actors_by_argument' found for struct 'EncounterArgumentationState'`.

- [ ] **Step 3: Add the accessor**

Still in `crates/encounter-argumentation/src/state.rs`. Directly after the `pub fn actors_for(...)` method (around the current line 181-186), add:

```rust
    /// Read-only access to the actor-per-argument map. Used by
    /// bridge weight sources (notably
    /// [`crate::societas_relationship::SocietasRelationshipSource`])
    /// to resolve an [`ArgumentId`] back to the actors whose
    /// asserted schemes produce that conclusion.
    #[must_use]
    pub fn actors_by_argument(&self) -> &HashMap<ArgumentId, Vec<String>> {
        &self.actors_by_argument
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p encounter-argumentation --lib actors_by_argument`
Expected: 2 tests pass.

- [ ] **Step 5: Run the full test suite**

Run: `cargo test -p encounter-argumentation`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): expose actors_by_argument() accessor for bridge weight sources"
```

---

### Task 4: `SocietasRelationshipSource` struct + constants

**Files:**
- Create: `crates/encounter-argumentation/src/societas_relationship.rs`

Defines the adapter struct, its constructor, the public coefficient constants, and the lifetime story. WeightSource impl comes in the next three tasks.

- [ ] **Step 1: Write the failing test**

Create `crates/encounter-argumentation/src/societas_relationship.rs` with:

```rust
//! `SocietasRelationshipSource`: a [`WeightSource<ArgumentId>`] that
//! derives attack weights from live `societas-relations` state.
//!
//! Replaces the Phase A `RelationshipWeightSource` stub. Unlike the
//! stub, this adapter handles the [`ArgumentId`]-vs-actor-name mismatch
//! correctly: it looks up the actors that asserted each argument via
//! `EncounterArgumentationState::actors_by_argument`, resolves their
//! names to `EntityId`s via a pluggable [`NameResolver`], queries the
//! five societas relationship dimensions per actor pair, applies a
//! coefficient recipe (see module-level constants), and returns the
//! arithmetic mean across pairs.
//!
//! A worked example using `WeightSource::weight_for` appears in the
//! crate-level docs on `lib.rs` and in
//! `tests/uc_relationship_modulation.rs`.

use crate::arg_id::ArgumentId;
use crate::name_resolver::NameResolver;
use societas_core::{EntityId, SocialStore, Tick};
use societas_relations::RelationshipRegistry;
use std::collections::HashMap;

/// Neutral-relationship baseline weight used when no actor pair has
/// resolvable relationship data.
pub const BASELINE_WEIGHT: f64 = 0.5;
/// Coefficient on `Dimension::Trust`. Higher trust → lower attack weight.
pub const TRUST_COEF: f64 = -0.15;
/// Coefficient on `Dimension::Fear`. Higher fear → higher attack weight.
pub const FEAR_COEF: f64 = 0.25;
/// Coefficient on `Dimension::Respect`. Higher respect → lower attack weight.
pub const RESPECT_COEF: f64 = -0.05;
/// Coefficient on `Dimension::Attraction`. Higher attraction → lower attack weight.
pub const ATTRACTION_COEF: f64 = -0.05;
/// Coefficient on `Dimension::Friendship`. Higher friendship → lower attack weight.
pub const FRIENDSHIP_COEF: f64 = -0.10;

/// A [`WeightSource<ArgumentId>`] that reads relationship dimensions
/// from a live `societas-relations` registry + store.
///
/// See the module-level documentation for the coefficient recipe and
/// the aggregation strategy for multi-actor arguments.
pub struct SocietasRelationshipSource<'a, R> {
    registry: &'a RelationshipRegistry,
    store: &'a dyn SocialStore,
    resolver: &'a R,
    actors_by_argument: &'a HashMap<ArgumentId, Vec<String>>,
    tick: Tick,
}

impl<'a, R: NameResolver> SocietasRelationshipSource<'a, R> {
    /// Construct a new source bound to the given registry, store,
    /// resolver, actor map, and evaluation tick.
    ///
    /// All references are borrowed for the adapter's lifetime `'a`.
    /// `tick` is owned and fixed at construction time — consumers
    /// wanting a new tick should build a fresh adapter.
    ///
    /// `actors_by_argument` is typically obtained by calling
    /// [`EncounterArgumentationState::actors_by_argument`].
    #[must_use]
    pub fn new(
        registry: &'a RelationshipRegistry,
        store: &'a dyn SocialStore,
        resolver: &'a R,
        actors_by_argument: &'a HashMap<ArgumentId, Vec<String>>,
        tick: Tick,
    ) -> Self {
        Self {
            registry,
            store,
            resolver,
            actors_by_argument,
            tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use societas_memory::MemStore;

    #[test]
    fn constants_match_phase_a_stub() {
        // Locks in the exact coefficient values shipped in Phase A.
        // Calibration changes should be conscious version bumps with
        // CHANGELOG entries — not drift.
        assert_eq!(BASELINE_WEIGHT, 0.5);
        assert_eq!(TRUST_COEF, -0.15);
        assert_eq!(FEAR_COEF, 0.25);
        assert_eq!(RESPECT_COEF, -0.05);
        assert_eq!(ATTRACTION_COEF, -0.05);
        assert_eq!(FRIENDSHIP_COEF, -0.10);
    }

    #[test]
    fn new_source_constructs() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        // Compile-time check: the constructor accepts these types.
        let _source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
    }
}
```

- [ ] **Step 2: Register the module**

Edit `crates/encounter-argumentation/src/lib.rs`. Add:

```rust
pub mod societas_relationship;
```

Place it alphabetically between `pub mod scoring;` and `pub mod state;`. No re-export yet — we'll add that when the API stabilizes in Task 6. (Design the module path as the source of truth; re-exports are for ergonomics.)

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p encounter-argumentation --lib societas_relationship`
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/src/societas_relationship.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): SocietasRelationshipSource struct + coefficient constants"
```

---

### Task 5: `WeightSource::weight_for` — fallback paths

**Files:**
- Modify: `crates/encounter-argumentation/src/societas_relationship.rs`

Covers the three early-return paths: (a) attacker `ArgumentId` has no actors recorded, (b) target `ArgumentId` has no actors recorded, (c) an actor's name doesn't resolve. In all cases we fall through to the baseline weight. `weight_for` always returns `Some(w)` — this source is opinionated about every edge.

- [ ] **Step 1: Write the failing tests**

Edit `crates/encounter-argumentation/src/societas_relationship.rs`. In the `#[cfg(test)] mod tests` block (after `new_source_constructs`), add:

```rust
    #[test]
    fn baseline_weight_when_attacker_has_no_actors() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    #[test]
    fn baseline_weight_when_target_has_no_actors() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    #[test]
    fn baseline_weight_when_neither_name_resolves() {
        // Actors are recorded, but the resolver has no EntityId for either.
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver: HashMap<String, EntityId> = HashMap::new();
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        actors.insert(ArgumentId::new("y"), vec!["bob".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation --lib societas_relationship`
Expected: FAIL — the `WeightSource` impl does not yet exist; test lines calling `source.weight_for(...)` fail to compile with "no method named `weight_for` found for struct `SocietasRelationshipSource`".

- [ ] **Step 3: Implement `WeightSource` with the fallback logic**

Still in `crates/encounter-argumentation/src/societas_relationship.rs`, **between** the `impl<'a, R: NameResolver> SocietasRelationshipSource<'a, R>` block and the `#[cfg(test)] mod tests` block, add:

```rust
use argumentation_weighted::WeightSource;

impl<R: NameResolver> WeightSource<ArgumentId> for SocietasRelationshipSource<'_, R> {
    /// Compute the attack weight for `attacker → target` from live
    /// societas relationship state. See the module-level documentation
    /// for the coefficient recipe and aggregation strategy.
    ///
    /// Always returns `Some(w)` — this source has an opinion on every
    /// edge. Unseeded or unresolvable pairs fall back to
    /// [`BASELINE_WEIGHT`].
    fn weight_for(&self, attacker: &ArgumentId, target: &ArgumentId) -> Option<f64> {
        let Some(attacker_actors) = self.actors_by_argument.get(attacker) else {
            return Some(BASELINE_WEIGHT);
        };
        let Some(target_actors) = self.actors_by_argument.get(target) else {
            return Some(BASELINE_WEIGHT);
        };

        let attacker_ids: Vec<EntityId> = attacker_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        let target_ids: Vec<EntityId> = target_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        if attacker_ids.is_empty() || target_ids.is_empty() {
            return Some(BASELINE_WEIGHT);
        }

        // Placeholder for Task 6: when we have at least one resolvable
        // pair, we should query societas. For now, still baseline.
        Some(BASELINE_WEIGHT)
    }
}
```

This is deliberately minimal — it does not yet call societas. Task 6 replaces the "Placeholder" block with real scoring logic.

- [ ] **Step 4: Run tests to verify the fallback tests now pass**

Run: `cargo test -p encounter-argumentation --lib societas_relationship`
Expected: all `societas_relationship` tests pass (including the 3 new fallback-path tests).

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/societas_relationship.rs
git commit -m "feat(encounter-argumentation): SocietasRelationshipSource fallback paths (unseeded/unresolvable → baseline)"
```

---

### Task 6: `WeightSource::weight_for` — single-pair scoring

**Files:**
- Modify: `crates/encounter-argumentation/src/societas_relationship.rs`

Replace the Task 5 placeholder with real societas queries: score all five dimensions for a single (attacker_actor, target_actor) pair and apply the coefficient recipe. Multi-actor aggregation comes in Task 7.

- [ ] **Step 1: Write the failing test**

Edit `crates/encounter-argumentation/src/societas_relationship.rs`. In the tests module, add the helper and tests below the existing tests:

```rust
    use societas_core::ModifierSource;
    use societas_relations::types::Dimension;

    /// Helper: build a minimal single-pair scene. Alice is the attacker
    /// actor (EntityId 1), Bob the target (EntityId 2).
    fn single_pair_fixture() -> (
        RelationshipRegistry,
        MemStore,
        HashMap<String, EntityId>,
        HashMap<ArgumentId, Vec<String>>,
    ) {
        let registry = RelationshipRegistry::new();
        let store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("bob".to_string(), EntityId::from_u64(2));
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("alice_arg"), vec!["alice".to_string()]);
        actors.insert(ArgumentId::new("bob_arg"), vec!["bob".to_string()]);
        (registry, store, resolver, actors)
    }

    #[test]
    fn neutral_societas_state_yields_baseline_weight() {
        let (registry, store, resolver, actors) = single_pair_fixture();
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        assert!(
            (w - BASELINE_WEIGHT).abs() < 1e-9,
            "neutral-state pair should produce baseline weight, got {w}"
        );
    }

    #[test]
    fn high_trust_lowers_attack_weight() {
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(2),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "high trust should produce baseline + TRUST_COEF = {expected}, got {w}"
        );
    }

    #[test]
    fn high_fear_raises_attack_weight() {
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(2),
            Dimension::Fear,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + FEAR_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "high fear should produce baseline + FEAR_COEF = {expected}, got {w}"
        );
    }

    #[test]
    fn weight_is_clamped_to_unit_interval_on_extreme_values() {
        // Simultaneously max out every dimension in both directions.
        // The raw linear combination can exceed [0, 1]; verify we clamp.
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        for dim in [
            Dimension::Trust,
            Dimension::Fear,
            Dimension::Friendship,
            Dimension::Respect,
            Dimension::Attraction,
        ] {
            registry.add_modifier(
                &mut store,
                EntityId::from_u64(1),
                EntityId::from_u64(2),
                dim,
                1.0,
                0.0,
                ModifierSource::Personality,
                Tick(0),
            );
        }
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        assert!(
            (0.0..=1.0).contains(&w),
            "weight should be clamped to [0, 1], got {w}"
        );
    }

    #[test]
    fn asymmetric_relationship_produces_asymmetric_weights() {
        // alice's view of bob has high trust; bob's view of alice is neutral.
        // alice → bob attack gets dampened (trust reduces weight);
        // bob → alice attack stays at baseline.
        let (registry, mut store, resolver, actors) = single_pair_fixture();
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1), // alice
            EntityId::from_u64(2), // bob
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w_alice_on_bob = source
            .weight_for(&ArgumentId::new("alice_arg"), &ArgumentId::new("bob_arg"))
            .unwrap();
        let w_bob_on_alice = source
            .weight_for(&ArgumentId::new("bob_arg"), &ArgumentId::new("alice_arg"))
            .unwrap();
        assert!(
            w_alice_on_bob < w_bob_on_alice,
            "alice→bob with high trust should weigh less than bob→alice baseline; got {w_alice_on_bob} vs {w_bob_on_alice}"
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation --lib societas_relationship`
Expected: the four new tests FAIL, because the impl still returns `BASELINE_WEIGHT` for every non-fallback case.

- [ ] **Step 3: Replace the placeholder with real scoring**

In `crates/encounter-argumentation/src/societas_relationship.rs`, replace the **body** of the `impl<R: NameResolver> WeightSource<ArgumentId> for SocietasRelationshipSource<'_, R>` with:

```rust
use argumentation_weighted::WeightSource;
use societas_relations::types::Dimension;

impl<R: NameResolver> WeightSource<ArgumentId> for SocietasRelationshipSource<'_, R> {
    fn weight_for(&self, attacker: &ArgumentId, target: &ArgumentId) -> Option<f64> {
        let Some(attacker_actors) = self.actors_by_argument.get(attacker) else {
            return Some(BASELINE_WEIGHT);
        };
        let Some(target_actors) = self.actors_by_argument.get(target) else {
            return Some(BASELINE_WEIGHT);
        };

        let attacker_ids: Vec<EntityId> = attacker_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        let target_ids: Vec<EntityId> = target_actors
            .iter()
            .filter_map(|n| self.resolver.resolve(n))
            .collect();
        if attacker_ids.is_empty() || target_ids.is_empty() {
            return Some(BASELINE_WEIGHT);
        }

        let mut sum = 0.0_f64;
        let mut count = 0_u32;
        for &src in &attacker_ids {
            for &tgt in &target_ids {
                sum += self.pairwise_weight(src, tgt);
                count += 1;
            }
        }
        let mean = sum / f64::from(count);
        Some(mean)
    }
}

impl<R: NameResolver> SocietasRelationshipSource<'_, R> {
    /// Compute the per-pair weight for a single (source, target)
    /// `EntityId` pair by summing the coefficient-weighted dimension
    /// scores and clamping to the unit interval.
    fn pairwise_weight(&self, source: EntityId, target: EntityId) -> f64 {
        let trust =
            self.registry
                .score(self.store, source, target, Dimension::Trust, self.tick);
        let fear =
            self.registry
                .score(self.store, source, target, Dimension::Fear, self.tick);
        let respect =
            self.registry
                .score(self.store, source, target, Dimension::Respect, self.tick);
        let attraction = self.registry.score(
            self.store,
            source,
            target,
            Dimension::Attraction,
            self.tick,
        );
        let friendship = self.registry.score(
            self.store,
            source,
            target,
            Dimension::Friendship,
            self.tick,
        );
        let raw = BASELINE_WEIGHT
            + TRUST_COEF * trust
            + FEAR_COEF * fear
            + RESPECT_COEF * respect
            + ATTRACTION_COEF * attraction
            + FRIENDSHIP_COEF * friendship;
        raw.clamp(0.0, 1.0)
    }
}
```

Notes:
- `attacker_ids` is guaranteed non-empty at the loop start (we returned baseline above otherwise), so `count >= 1` and the division is safe.
- The clamp happens *inside* `pairwise_weight`. Task 7 (mean of already-clamped per-pair weights) therefore produces a mean in `[0, 1]` without re-clamping.
- The `Dimension` import sits at module scope alongside the existing `use` statements, not inside the impl. If clippy complains about the second `use argumentation_weighted::WeightSource;`, remove the duplicate — the first `use` at the top of the impl block from Task 5 is the only one needed.

**CRITICAL:** When merging this step you will have TWO `use argumentation_weighted::WeightSource;` lines if you leave Task 5's in place. Delete the Task-5 `use` inside the impl; keep a single `use argumentation_weighted::WeightSource;` at the top of the file near the other `use` statements. Your final `use` list should be:

```rust
use crate::arg_id::ArgumentId;
use crate::name_resolver::NameResolver;
use argumentation_weighted::WeightSource;
use societas_core::{EntityId, SocialStore, Tick};
use societas_relations::types::Dimension;
use societas_relations::RelationshipRegistry;
use std::collections::HashMap;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p encounter-argumentation --lib societas_relationship`
Expected: all `societas_relationship` tests pass, including the five new scoring tests.

- [ ] **Step 5: Add the re-export in lib.rs**

Edit `crates/encounter-argumentation/src/lib.rs`. Add:

```rust
pub use societas_relationship::{
    ATTRACTION_COEF, BASELINE_WEIGHT, FEAR_COEF, FRIENDSHIP_COEF, RESPECT_COEF, TRUST_COEF,
    SocietasRelationshipSource,
};
```

Place it alphabetically — after `pub use scoring::SchemeActionScorer;` is natural.

- [ ] **Step 6: Full test + clippy**

Run: `cargo test -p encounter-argumentation && cargo clippy -p encounter-argumentation --all-targets --no-deps -- -D warnings`
Expected: all tests pass, clippy clean.

- [ ] **Step 7: Commit**

```bash
git add crates/encounter-argumentation/src/societas_relationship.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): SocietasRelationshipSource single-pair scoring via societas-relations"
```

---

### Task 7: `WeightSource::weight_for` — multi-actor mean aggregation

**Files:**
- Modify: `crates/encounter-argumentation/src/societas_relationship.rs`

Task 6 already wrote the aggregation loop, but did not test it. This task adds the behavioral tests that lock in the mean-aggregation contract for arguments whose `actors_by_argument` entry contains more than one name.

- [ ] **Step 1: Write the failing tests**

In the tests module of `crates/encounter-argumentation/src/societas_relationship.rs`, add:

```rust
    #[test]
    fn multi_attacker_averages_per_pair_weights() {
        // Two attackers, one target. Alice has high trust of target;
        // Bob has neutral. The result should be the arithmetic mean of
        // the two per-pair weights, not a min/max/first-wins.
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("bob".to_string(), EntityId::from_u64(2));
        resolver.insert("target".to_string(), EntityId::from_u64(3));
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "bob".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["target".to_string()],
        );

        // Alice trusts the target strongly; Bob is neutral.
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(3),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );

        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("duo_attack"), &ArgumentId::new("target_arg"))
            .unwrap();

        let alice_pair = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        let bob_pair = BASELINE_WEIGHT;
        let expected = (alice_pair + bob_pair) / 2.0;
        assert!(
            (w - expected).abs() < 1e-9,
            "two-attacker case should mean per-pair weights: expected {expected}, got {w}"
        );
    }

    #[test]
    fn unresolvable_actors_are_skipped_not_treated_as_neutral_pair() {
        // Two attackers: alice (resolvable), eve (NOT in resolver).
        // Expected behavior: eve is skipped entirely; result equals
        // the alice-only weight, NOT mean(alice_pair, neutral_baseline).
        // This makes the adapter robust to partial registries.
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver: HashMap<String, EntityId> = HashMap::new();
        resolver.insert("alice".to_string(), EntityId::from_u64(1));
        resolver.insert("target".to_string(), EntityId::from_u64(3));
        // NOTE: "eve" is deliberately NOT in the resolver.
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "eve".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["target".to_string()],
        );
        registry.add_modifier(
            &mut store,
            EntityId::from_u64(1),
            EntityId::from_u64(3),
            Dimension::Trust,
            1.0,
            0.0,
            ModifierSource::Personality,
            Tick(0),
        );
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source
            .weight_for(&ArgumentId::new("duo_attack"), &ArgumentId::new("target_arg"))
            .unwrap();
        let expected = (BASELINE_WEIGHT + TRUST_COEF).clamp(0.0, 1.0);
        assert!(
            (w - expected).abs() < 1e-9,
            "unresolvable eve should be filtered out; result should match alice-only pair = {expected}, got {w}"
        );
    }
```

- [ ] **Step 2: Run tests to verify the mean test passes**

Run: `cargo test -p encounter-argumentation --lib societas_relationship -- multi_attacker_averages_per_pair_weights unresolvable_actors_are_skipped_not_treated_as_neutral_pair`
Expected: both tests pass (the Task 6 impl already implements this correctly via the `filter_map(|n| self.resolver.resolve(n))` + Cartesian product loop).

If either test fails, the bug is in Task 6's impl — go back and fix it, don't patch it here.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter-argumentation/src/societas_relationship.rs
git commit -m "test(encounter-argumentation): multi-actor mean aggregation + skip-unresolvable coverage"
```

---

### Task 8: Delete Phase A stubs + update `lib.rs` re-exports

**Files:**
- Delete: `crates/encounter-argumentation/src/relationship.rs`
- Modify: `crates/encounter-argumentation/src/lib.rs`

Removes `RelationshipDims`, `RelationshipSnapshot`, and the old Phase A `RelationshipWeightSource`. No backward-compat shims: per user preference, rename freely in the beta phase.

- [ ] **Step 1: Delete the relationship module file**

Run: `git rm crates/encounter-argumentation/src/relationship.rs`
Expected: file removed; staged for deletion.

- [ ] **Step 2: Remove the mod declaration and re-exports**

Edit `crates/encounter-argumentation/src/lib.rs`. Delete these two lines:

```rust
pub mod relationship;
```

and

```rust
pub use relationship::{RelationshipDims, RelationshipSnapshot, RelationshipWeightSource};
```

- [ ] **Step 3: Verify nothing else references the deleted items**

Run: `grep -rn 'RelationshipDims\|RelationshipSnapshot\|RelationshipWeightSource' crates/encounter-argumentation/`
Expected: no results. If the rewritten integration test in `tests/uc_relationship_modulation.rs` still references them, that test is getting rewritten in Task 9 — but for now stabilize on the lib build first, since Task 9 depends on this mod being gone.

- [ ] **Step 4: Verify lib still builds**

Run: `cargo build -p encounter-argumentation --lib`
Expected: success.

- [ ] **Step 5: The full test suite will FAIL here on uc_relationship_modulation.rs**

Run: `cargo test -p encounter-argumentation`
Expected: `tests/uc_relationship_modulation.rs` fails to compile with "unresolved import `encounter_argumentation::RelationshipDims`" etc. This is expected — Task 9 fixes it. Do NOT commit in a state where tests fail; continue to Task 9 before commiting.

- [ ] **Step 6: (DO NOT COMMIT YET)**

This task's deletions are not independently committable because they break the integration test. Task 9's new test shape replaces it atomically. Stage the changes here (they're already staged from `git rm` + the `lib.rs` edits) and proceed to Task 9.

Expected staged state: `crates/encounter-argumentation/src/lib.rs` modified, `crates/encounter-argumentation/src/relationship.rs` deleted.

---

### Task 9: Rewrite `uc_relationship_modulation.rs`

**Files:**
- Modify: `crates/encounter-argumentation/tests/uc_relationship_modulation.rs`

Fully replace the Phase A test body with a MemStore-backed scenario that actually exercises `SocietasRelationshipSource` end to end — seeding a single scheme instance, passing the computed weight into `add_weighted_attack`, and observing credulous acceptance flip with intensity.

- [ ] **Step 1: Replace the test file contents**

Overwrite `crates/encounter-argumentation/tests/uc_relationship_modulation.rs` with:

```rust
//! UC3 relationship modulation — Phase C rewrite.
//!
//! Phase A used a snapshot stub that mapped actor-name strings → dims
//! directly, which was unsound: `WeightSource::weight_for` receives
//! `ArgumentId` (a conclusion literal), not an actor name. Phase C
//! replaces the stub with `SocietasRelationshipSource`, which uses the
//! bridge's `actors_by_argument` map to resolve an `ArgumentId` to the
//! actors who asserted that conclusion, then reads relationship
//! dimensions from a live `societas-relations` registry + store.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::{types::Budget, WeightSource};
use encounter_argumentation::{
    ArgumentId, EncounterArgumentationState, SocietasRelationshipSource,
};
use societas_core::{EntityId, ModifierSource, Tick};
use societas_memory::MemStore;
use societas_relations::{types::Dimension, RelationshipRegistry};
use std::collections::HashMap;

fn seed_state_with_pairwise_debate() -> (
    EncounterArgumentationState,
    ArgumentId,
    ArgumentId,
    HashMap<String, EntityId>,
) {
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let alice_instance = expert
        .instantiate(
            &[
                ("expert".to_string(), "alice".to_string()),
                ("domain".to_string(), "military".to_string()),
                ("claim".to_string(), "fortify_east".to_string()),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();
    let bob_instance = expert
        .instantiate(
            &[
                ("expert".to_string(), "bob".to_string()),
                ("domain".to_string(), "logistics".to_string()),
                ("claim".to_string(), "abandon_east".to_string()),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let alice_id = state.add_scheme_instance("alice", alice_instance);
    let bob_id = state.add_scheme_instance("bob", bob_instance);

    let mut resolver: HashMap<String, EntityId> = HashMap::new();
    resolver.insert("alice".to_string(), EntityId::from_u64(1));
    resolver.insert("bob".to_string(), EntityId::from_u64(2));

    (state, alice_id, bob_id, resolver)
}

#[test]
fn high_trust_reduces_effective_attack_weight() {
    // Alice asserts fortify_east; Bob asserts abandon_east.
    // Bob has HIGH trust of Alice (bob→alice Trust = 1.0). This means
    // Bob's attack on Alice's conclusion should be DAMPENED — from the
    // baseline 0.5 down to 0.5 + TRUST_COEF = 0.35.
    let (state, alice_id, bob_id, resolver) = seed_state_with_pairwise_debate();
    let mut store = MemStore::new();
    let registry = RelationshipRegistry::new();
    registry.add_modifier(
        &mut store,
        EntityId::from_u64(2), // bob
        EntityId::from_u64(1), // alice
        Dimension::Trust,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );

    let source = SocietasRelationshipSource::new(
        &registry,
        &store,
        &resolver,
        state.actors_by_argument(),
        Tick(0),
    );

    let w = source.weight_for(&bob_id, &alice_id).unwrap();
    assert!(
        w < 0.5,
        "bob→alice with high trust should produce sub-baseline weight, got {w}"
    );

    // Reverse direction: alice has no trust recorded of bob → baseline.
    let w_reverse = source.weight_for(&alice_id, &bob_id).unwrap();
    assert!(
        (w_reverse - 0.5).abs() < 1e-9,
        "alice→bob with no recorded trust should sit at baseline 0.5, got {w_reverse}"
    );
}

#[test]
fn same_scenario_flips_acceptance_at_different_budgets_for_different_weights() {
    // Construct two parallel states: one where bob has high trust of
    // alice (attack weight < 0.5), one where bob has high fear
    // (attack weight > 0.5). At an intensity β that sits between the
    // two, only the high-trust state accepts alice credulously.
    let (state_trust, alice_trust_id, bob_trust_id, resolver_trust) =
        seed_state_with_pairwise_debate();
    let (state_fear, alice_fear_id, bob_fear_id, resolver_fear) =
        seed_state_with_pairwise_debate();

    let mut store_trust = MemStore::new();
    let registry_trust = RelationshipRegistry::new();
    registry_trust.add_modifier(
        &mut store_trust,
        EntityId::from_u64(2),
        EntityId::from_u64(1),
        Dimension::Trust,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );
    let source_trust = SocietasRelationshipSource::new(
        &registry_trust,
        &store_trust,
        &resolver_trust,
        state_trust.actors_by_argument(),
        Tick(0),
    );

    let mut store_fear = MemStore::new();
    let registry_fear = RelationshipRegistry::new();
    registry_fear.add_modifier(
        &mut store_fear,
        EntityId::from_u64(2),
        EntityId::from_u64(1),
        Dimension::Fear,
        1.0,
        0.0,
        ModifierSource::Personality,
        Tick(0),
    );
    let source_fear = SocietasRelationshipSource::new(
        &registry_fear,
        &store_fear,
        &resolver_fear,
        state_fear.actors_by_argument(),
        Tick(0),
    );

    let w_trust = source_trust
        .weight_for(&bob_trust_id, &alice_trust_id)
        .unwrap();
    let w_fear = source_fear.weight_for(&bob_fear_id, &alice_fear_id).unwrap();
    assert!(
        w_trust < w_fear,
        "trust-based weight ({w_trust}) should be below fear-based weight ({w_fear})"
    );

    // Wire the weights into their respective states.
    let mut state_trust_mut = state_trust;
    state_trust_mut
        .add_weighted_attack(&bob_trust_id, &alice_trust_id, w_trust)
        .unwrap();
    let mut state_fear_mut = state_fear;
    state_fear_mut
        .add_weighted_attack(&bob_fear_id, &alice_fear_id, w_fear)
        .unwrap();

    // Pick a β strictly between the two weights. Alice survives in the
    // trust state (β > w_trust → residual drops the attack), but NOT in
    // the fear state (β < w_fear → residual retains the attack).
    let mid = (w_trust + w_fear) / 2.0;
    let beta = Budget::new(mid).unwrap();
    let trust_acceptance = state_trust_mut
        .at_intensity(beta)
        .is_credulously_accepted(&alice_trust_id)
        .unwrap();
    let fear_acceptance = state_fear_mut
        .at_intensity(beta)
        .is_credulously_accepted(&alice_fear_id)
        .unwrap();
    assert!(
        trust_acceptance,
        "at β between the two weights, trust-dampened attack should be dropped → alice credulous"
    );
    assert!(
        !fear_acceptance,
        "at β between the two weights, fear-amplified attack should bind → alice not credulous"
    );
}
```

- [ ] **Step 2: Run the integration test**

Run: `cargo test -p encounter-argumentation --test uc_relationship_modulation`
Expected: 2 tests pass.

- [ ] **Step 3: Run the full test suite**

Run: `cargo test -p encounter-argumentation`
Expected: all tests pass across lib unit tests, all other `tests/` files, and doctests.

- [ ] **Step 4: Clippy + full workspace check**

Run: `cargo clippy -p encounter-argumentation --all-targets --no-deps -- -D warnings && cargo check --workspace`
Expected: clippy clean, workspace builds.

- [ ] **Step 5: Commit Tasks 8 + 9 together atomically**

```bash
git add crates/encounter-argumentation/src/lib.rs crates/encounter-argumentation/src/relationship.rs crates/encounter-argumentation/tests/uc_relationship_modulation.rs
git commit -m "refactor(encounter-argumentation): delete Phase A relationship stubs; rewrite UC3 against SocietasRelationshipSource"
```

---

### Task 10: CHANGELOG + version 0.4.0 + tag

**Files:**
- Modify: `crates/encounter-argumentation/CHANGELOG.md`
- Modify: `crates/encounter-argumentation/Cargo.toml`

- [ ] **Step 1: Add the CHANGELOG entry**

Edit `crates/encounter-argumentation/CHANGELOG.md`. At the very top, below `# Changelog`, insert a new section (above the existing `## [0.3.0] - 2026-04-20` header):

```markdown
## [0.4.0] - 2026-04-23

### Added — societas-backed relationship weights
- `SocietasRelationshipSource<'a, R>` — a `WeightSource<ArgumentId>`
  implementation that reads attack weights from live
  `societas-relations` state. Resolves an `ArgumentId` to its asserting
  actors via `EncounterArgumentationState::actors_by_argument`, then
  queries the five societas relationship dimensions (Trust, Fear,
  Respect, Attraction, Friendship) at a caller-supplied `Tick` to
  compute a per-edge weight.
- `NameResolver` trait — maps actor-name strings to `EntityId`. Ships
  with a blanket impl for `HashMap<String, EntityId>` so tests and
  consumers with a fixed cast list can pass a HashMap directly.
- Public coefficient constants: `BASELINE_WEIGHT`, `TRUST_COEF`,
  `FEAR_COEF`, `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF`.
  These are the same values the Phase A stub used, now observable and
  pinnable from tests. Calibration is provisional — see README.
- `EncounterArgumentationState::actors_by_argument() -> &HashMap<ArgumentId, Vec<String>>`
  — read-only accessor exposing the previously-private actor map.

### Removed — Phase A relationship stubs
- `RelationshipDims`, `RelationshipSnapshot`, and the old
  `RelationshipWeightSource` are deleted. The stub was soundness-broken:
  it treated `ArgumentId` (a conclusion literal) as if it were an actor
  name, so scheme-derived `ArgumentId`s always missed the snapshot and
  fell back to the neutral baseline. No migration shim — consumers
  rewrite against `SocietasRelationshipSource`.

### Multi-actor aggregation
When `actors_by_argument` maps one `ArgumentId` to more than one actor
(convergent conclusions), the per-pair weights are combined by
arithmetic mean across the (attacker_actor × target_actor) Cartesian
product. Unresolvable actor names are silently skipped; if all pairs
are unresolvable, the source returns `BASELINE_WEIGHT`.

### Dependencies
- Added: `societas-core`, `societas-relations` (path deps on the
  sibling societas workspace).
- Added (dev): `societas-memory` — for `MemStore`-backed test
  fixtures.
```

- [ ] **Step 2: Bump the version**

Edit `crates/encounter-argumentation/Cargo.toml`. Change line 3:

```toml
version = "0.3.0"
```

to:

```toml
version = "0.4.0"
```

- [ ] **Step 3: Verify everything builds and tests pass**

Run: `cargo test -p encounter-argumentation && cargo clippy -p encounter-argumentation --all-targets --no-deps -- -D warnings`
Expected: all pass, clippy clean.

- [ ] **Step 4: Commit and tag**

```bash
git add crates/encounter-argumentation/CHANGELOG.md crates/encounter-argumentation/Cargo.toml
git commit -m "chore(encounter-argumentation): v0.4.0 release — societas-backed relationship adapter"
git tag encounter-argumentation-v0.4.0
```

Do NOT push the tag automatically — the user approves pushes separately.

---

### Task 11: Dispatch code review

**Files:** none (review task only)

- [ ] **Step 1: Get the git SHAs bracketing Phase C**

Run:
```bash
BASE_SHA=$(git log --format=%H --grep="Merge branch 'feat/phase-b-state-scorer-bridge'" -n 1)
HEAD_SHA=$(git rev-parse HEAD)
echo "BASE=$BASE_SHA"
echo "HEAD=$HEAD_SHA"
```

Expected: `BASE_SHA` resolves to the Phase B merge commit (`87b77ef` as of this plan); `HEAD_SHA` resolves to the `v0.4.0` release commit.

- [ ] **Step 2: Invoke the review skill**

Use `superpowers:requesting-code-review` with:

- `{WHAT_WAS_IMPLEMENTED}`: "Phase C — societas-backed `SocietasRelationshipSource` replacing the Phase A `RelationshipWeightSource` stub. Adds `NameResolver` trait, public coefficient constants, and `actors_by_argument` accessor. Deletes Phase A stubs. Bumps to v0.4.0."
- `{PLAN_OR_REQUIREMENTS}`: "docs/superpowers/plans/2026-04-23-phase-c-societas-relationship-adapter.md"
- `{BASE_SHA}`: the BASE_SHA from Step 1
- `{HEAD_SHA}`: the HEAD_SHA from Step 1
- `{DESCRIPTION}`: "Bridge-only change. Zero modifications to societas crates. Fixes the Phase A soundness bug where `WeightSource` treated `ArgumentId` as an actor name. Review particularly: (1) the multi-actor mean aggregation semantics, (2) the permissive fallbacks (missing actors → baseline; unresolvable names → skipped), (3) whether coefficient constants should stay `pub const` or move behind a tuning struct."

- [ ] **Step 3: Act on feedback**

Apply the receiving-code-review skill:
- Critical issues — fix immediately before anything else.
- Important issues — fix before declaring Phase C done.
- Minor issues — note for a v0.4.1 follow-up unless trivial.
- If the reviewer pushes back on a design decision (e.g. mean vs max aggregation) — that's covered by Q3 in the plan, push back with the rationale ("v0.4.0 ships one policy; alternatives deferred").

- [ ] **Step 4: Announce completion**

Once review is clean: report "Phase C merged on branch. v0.4.0 tagged locally. Ready to push when you are."

---

## Self-review

**Spec coverage** (from the scoping conversation):

- ✅ Replace stub `RelationshipSnapshot` with real societas adapter — Tasks 4-7.
- ✅ Fix Phase A soundness bug (ArgumentId-as-actor-name) — Tasks 3 + 6.
- ✅ Q1: NameResolver trait + HashMap impl — Task 2.
- ✅ Q2: Tick owned at construction — Task 4 constructor.
- ✅ Q3: Mean aggregation — Task 6 impl + Task 7 tests.
- ✅ Q4: Coefficients preserved as pub const — Task 4.
- ✅ Delete Phase A stubs — Task 8.
- ✅ Migrate UC3 integration test — Task 9.
- ✅ v0.4.0 CHANGELOG + tag — Task 10.
- ✅ Code review — Task 11.

**Placeholder scan:** No "TBD" / "implement later" / "similar to Task N" references. Every code block is copy-pasteable. The only forward reference is Task 5's placeholder comment which Task 6 explicitly replaces — the comment itself is in the code, not the plan.

**Type consistency check:**
- `SocietasRelationshipSource::new(registry, store, resolver, actors_by_argument, tick)` has the same parameter order in Task 4 (constructor), Task 5 (fallback tests), Task 6 (scoring tests), Task 7 (mean-aggregation tests), and Task 9 (integration test). ✓
- `EncounterArgumentationState::actors_by_argument() -> &HashMap<ArgumentId, Vec<String>>` — same return type in Task 3 (accessor), Task 4 (doc example), Task 9 (integration test). ✓
- Constants `BASELINE_WEIGHT`, `TRUST_COEF`, ... — same names throughout Tasks 4-10. ✓
- `NameResolver::resolve(&self, name: &str) -> Option<EntityId>` — same signature in Task 2 (definition), Task 6 (used in impl), Task 9 (implicit use via HashMap). ✓

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-23-phase-c-societas-relationship-adapter.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, two-stage review between tasks, fast iteration.

**2. Inline Execution** — execute tasks in this session using executing-plans, batch checkpoints.

Which approach?
