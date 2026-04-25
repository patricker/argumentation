# Move Societas Bridge to `societas-encounter` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Relocate `SocietasRelationshipSource` (Phase C, currently in `encounter-argumentation`) into the existing `societas-encounter` crate, delete the duplicate `NameResolver` in favor of `societas-encounter`'s richer `NameResolver`, and ship `encounter-argumentation v0.5.0` minus the moved types.

**Architecture:** `societas-encounter` already exists in `/home/peter/code/societas/crates/encounter/` and houses every other societas↔encounter bridge type (`SocialActionScorer`, `PersonalityAcceptanceEval`, `NameResolver`/`StaticNameResolver`, catalog compilation, effects, preconditions). Phase C accidentally created a parallel-and-inferior `NameResolver` and stuffed `SocietasRelationshipSource` into the wrong crate. This plan corrects both: the societas-knowing weight source moves to `societas-encounter` behind a new `argumentation` feature flag, and `encounter-argumentation` returns to its original scope (encounter↔argumentation bridge only).

**Tech Stack:** Rust 2024. Adds `argumentation-weighted` + `encounter-argumentation` as new path-deps on `societas-encounter` (gated by feature `argumentation`). Drops `societas-core` + `societas-relations` deps from `encounter-argumentation`.

---

## Why this naming, not `societas-argumentation`?

The existing `societas-encounter` crate is consumer-shaped: it gives consumers a way to make encounter scenes reflect societas state. Its current modules — `scoring` (ActionScorer impl), `acceptance` (AcceptanceEval impl), `catalog`, `effects`, `precondition`, `names` — all implement *encounter-side surfaces* fed by *societas-side reads*. `SocietasRelationshipSource` belongs in exactly the same bucket: it implements an argumentation surface (`WeightSource<ArgumentId>`), reads from societas, and is consumed by encounter-aware scenes. The argumentation specifics are an implementation detail the feature flag captures.

A separate `societas-argumentation` crate would have one type today and no clear future. The `societas-encounter::argumentation` module is right-sized.

---

## Pre-conditions verified

- `societas-encounter::names::NameResolver` exists and is strictly richer than the Phase C duplicate:
  - `Send + Sync` supertrait (Phase C: no bound)
  - Concrete `StaticNameResolver` struct (Phase C: blanket impl on `HashMap`)
  - Reserved-binding-name validation via `RESERVED_BINDING_NAMES` (`["self", "target", "subject", "initiator", "aggressor", "actor", "recipient", "witness"]`) — closes a real bug class where a character named `"self"` would collide with affordance role-binding slots
- `societas-encounter` has no current dep on `argumentation-weighted` or `encounter-argumentation` — adding them is new wiring (gated by feature)
- `encounter-argumentation v0.4.0` is shipped + tagged + pushed; we will ship `v0.5.0` removing the moved types
- The plan assumes the user owns both repos. If `societas` is a separate-team repo, stop after Phase 1 Task 0 and coordinate before proceeding.

---

## File Structure

### New files
- `/home/peter/code/societas/crates/encounter/src/argumentation.rs` — `SocietasRelationshipSource` lives here, gated by `#[cfg(feature = "argumentation")]`.
- `/home/peter/code/societas/crates/encounter/tests/relationship_source.rs` — integration test for the societas → argumentation weight bridge (moved from encounter-argumentation).

### Modified files (societas repo)
- `/home/peter/code/societas/Cargo.toml` — add path-dep entries for `argumentation-weighted` + `encounter-argumentation` to `[workspace.dependencies]`.
- `/home/peter/code/societas/crates/encounter/Cargo.toml` — declare the optional deps + new `argumentation` feature.
- `/home/peter/code/societas/crates/encounter/src/lib.rs` — register the new module + re-export.
- `/home/peter/code/societas/crates/encounter/CHANGELOG.md` — add entry. (Create if absent — verify in Task 1.)

### Modified files (argumentation repo)
- `/home/peter/code/argumentation/crates/encounter-argumentation/Cargo.toml` — drop `societas-core`, `societas-relations`, and dev-dep `societas-memory`. Bump to `0.5.0`.
- `/home/peter/code/argumentation/crates/encounter-argumentation/src/lib.rs` — drop `pub mod name_resolver;`, `pub mod societas_relationship;`, and their re-exports. Update doc-link references in module/crate docs.
- `/home/peter/code/argumentation/crates/encounter-argumentation/src/state.rs` — update the doc comment on `EncounterArgumentationState::new` that currently points at `crate::societas_relationship::SocietasRelationshipSource` — point at the new path or drop the link.
- `/home/peter/code/argumentation/crates/encounter-argumentation/CHANGELOG.md` — `[0.5.0]` entry: removed `SocietasRelationshipSource`, `NameResolver`, coefficient constants. Migration note pointing to `societas-encounter`.
- `/home/peter/code/argumentation/crates/encounter-argumentation/README.md` — refresh "Relationship modulation (Phase C)" section: short note that the societas-aware weight source now lives in `societas-encounter`.

### Deleted files (argumentation repo)
- `/home/peter/code/argumentation/crates/encounter-argumentation/src/societas_relationship.rs`
- `/home/peter/code/argumentation/crates/encounter-argumentation/src/name_resolver.rs`
- `/home/peter/code/argumentation/crates/encounter-argumentation/tests/uc_relationship_modulation.rs` (the societas-knowing variant — its replacement lives in societas-encounter)

---

## Branching

**Phase 1 (societas):** branch `/home/peter/code/societas/` from main to `feat/argumentation-weight-source`. Commit there.

**Phase 2 (argumentation):** branch `/home/peter/code/argumentation/` from main to `feat/encounter-arg-v0.5.0-shed-societas`. Commit there.

Phase 1 must merge first because Phase 2 cannot test in isolation — its old integration test depended on the moved types.

---

## Open Design Decisions (resolved)

| # | Question | Resolution | Rationale |
|---|---|---|---|
| Q1 | Drop my Phase C `NameResolver` or keep it as a re-export? | Drop entirely. | societas-encounter's is strictly better (Send+Sync, reserved-name guard). No backward-compat shim needed (beta phase). |
| Q2 | Use existing `StaticNameResolver` or keep my `HashMap` blanket impl? | Use `StaticNameResolver` in tests + accept any `&dyn NameResolver` in the API. | Standard pattern in the crate. The `HashMap` impl introduced a foot-gun (no reserved-name check) that the existing `StaticNameResolver` avoids. |
| Q3 | Make argumentation deps optional (feature-gated) on societas-encounter? | Yes — feature `argumentation`, default off. | Other societas-encounter consumers shouldn't pay for argumentation's transitive deps if they don't need them. Default-off keeps the existing minimal footprint; consumers explicitly opt in. |
| Q4 | Keep `SocietasRelationshipSource` generic over `R: NameResolver`, or use `&dyn NameResolver`? | `&dyn NameResolver`. | societas-encounter's `NameResolver` already has `Send + Sync` so trait objects are well-behaved. Drops a generic parameter, simplifies signatures, costs one virtual call per name lookup (negligible). |
| Q5 | Where does the integration test go after the move? | New `tests/relationship_source.rs` in societas-encounter. The encounter-argumentation copy is deleted. | encounter-argumentation already has WeightSource integration coverage via UC1/UC4; its societas-specific test belonged on the societas side. |

---

## Verified APIs (quoted from source)

### societas-encounter (existing) — `NameResolver` + `StaticNameResolver`

From `/home/peter/code/societas/crates/encounter/src/names.rs:11-57`:

```rust
pub trait NameResolver: Send + Sync {
    fn resolve(&self, name: &str) -> Option<EntityId>;
}

#[derive(Debug, Clone, Default)]
pub struct StaticNameResolver { /* private */ }

impl StaticNameResolver {
    pub fn new() -> Self;
    pub fn add(&mut self, name: &str, entity: EntityId);  // panics on reserved names
    pub fn try_add(&mut self, name: &str, entity: EntityId) -> Result<(), NameError>;
}

impl NameResolver for StaticNameResolver {
    fn resolve(&self, name: &str) -> Option<EntityId> { ... }
}
```

The reserved names (would-collide with affordance role-binding slots): `["self", "target", "subject", "initiator", "aggressor", "actor", "recipient", "witness"]`.

### encounter-argumentation (existing) — what we depend on from societas-encounter

```rust
// crates/encounter-argumentation/src/state.rs (after Phase C):
impl EncounterArgumentationState {
    pub fn actors_by_argument(&self) -> &HashMap<ArgumentId, Vec<String>>;
}

// crates/encounter-argumentation/src/arg_id.rs:
pub struct ArgumentId(String);
impl ArgumentId { pub fn new(...) -> Self; pub fn as_str(&self) -> &str; }
```

### argumentation-weighted — `WeightSource`

From `/home/peter/code/argumentation/crates/argumentation-weighted/src/weight_source.rs:23-28`:

```rust
pub trait WeightSource<A> {
    fn weight_for(&self, attacker: &A, target: &A) -> Option<f64>;
}
```

(No Send+Sync supertrait, so `dyn` requires explicit bounds when needed.)

### societas-relations — `RelationshipRegistry::score` (unchanged from Phase C plan)

```rust
pub fn score(
    &self,
    store: &dyn SocialStore,
    source: EntityId, target: EntityId,
    dimension: Dimension, tick: Tick,
) -> f64;  // clamped to [-1, 1]
```

---

## Phase 1 — Add `argumentation` feature to `societas-encounter`

Working directory: `/home/peter/code/societas/`.

### Task 0: Branch + verify upstream

**Files:** none (verification only)

- [ ] **Step 1: Branch from main**

```bash
cd /home/peter/code/societas
git fetch
git checkout main && git pull --ff-only
git checkout -b feat/argumentation-weight-source
```

- [ ] **Step 2: Verify the encounter-argumentation v0.4.0 we will depend on**

```bash
ls /home/peter/code/argumentation/crates/encounter-argumentation/Cargo.toml
grep '^version' /home/peter/code/argumentation/crates/encounter-argumentation/Cargo.toml
# Expected: version = "0.4.0"
git -C /home/peter/code/argumentation tag -l "encounter-argumentation-v0.4.0"
# Expected: encounter-argumentation-v0.4.0
```

If the version is not 0.4.0 or the tag is missing, STOP and re-coordinate — Phase C may have not actually shipped.

- [ ] **Step 3: Verify the existing societas-encounter test suite passes baseline**

```bash
cargo test -p societas-encounter 2>&1 | grep "test result:"
```

Record the baseline number for sanity-checking after the new module lands. Expected: all pass.

### Task 1: Add path-deps to societas workspace + new feature

**Files:**
- Modify: `/home/peter/code/societas/Cargo.toml`
- Modify: `/home/peter/code/societas/crates/encounter/Cargo.toml`

Argumentation-weighted and encounter-argumentation become optional path-deps on the societas workspace. The encounter crate gates them behind a new `argumentation` feature.

- [ ] **Step 1: Add workspace dep entries**

Edit `/home/peter/code/societas/Cargo.toml`. Find the `[workspace.dependencies]` table (or create one if absent). Append the two new entries alphabetically:

```toml
argumentation-weighted = { path = "../argumentation/crates/argumentation-weighted" }
encounter-argumentation = { path = "../argumentation/crates/encounter-argumentation" }
```

If `[workspace.dependencies]` does not exist, create it. Do NOT touch any other workspace section.

- [ ] **Step 2: Add the optional deps + feature on societas-encounter**

Edit `/home/peter/code/societas/crates/encounter/Cargo.toml`. Under `[dependencies]`, add (alphabetically — `argumentation-weighted` precedes `encounter`):

```toml
argumentation-weighted = { workspace = true, optional = true }
encounter-argumentation = { workspace = true, optional = true }
```

Under (or above) `[dev-dependencies]` add a new `[features]` table:

```toml
[features]
default = []
argumentation = ["dep:argumentation-weighted", "dep:encounter-argumentation"]
```

- [ ] **Step 3: Verify build with feature OFF**

```bash
cd /home/peter/code/societas
cargo build -p societas-encounter
```

Expected: clean build; no argumentation deps compiled (look at output — should NOT mention `argumentation-weighted` or `encounter-argumentation`).

- [ ] **Step 4: Verify build with feature ON**

```bash
cargo build -p societas-encounter --features argumentation
```

Expected: clean build; both new deps compile from the sibling argumentation workspace.

- [ ] **Step 5: Verify tests still pass with feature OFF (baseline preserved)**

```bash
cargo test -p societas-encounter 2>&1 | grep "test result:"
```

Expected: same number as Task 0 Step 3.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/encounter/Cargo.toml
git commit -m "deps(societas-encounter): optional argumentation-weighted + encounter-argumentation path-deps"
```

### Task 2: Add `argumentation` module scaffold

**Files:**
- Create: `/home/peter/code/societas/crates/encounter/src/argumentation.rs`
- Modify: `/home/peter/code/societas/crates/encounter/src/lib.rs`

Empty scaffold + module registration. Subsequent tasks fill in the type and impl.

- [ ] **Step 1: Create the empty module file**

Create `/home/peter/code/societas/crates/encounter/src/argumentation.rs`:

```rust
//! Argumentation-framework integration for societas-encounter.
//!
//! This module is gated by the `argumentation` feature. With the feature
//! enabled, it exposes [`SocietasRelationshipSource`], a
//! [`WeightSource<ArgumentId>`](argumentation_weighted::WeightSource)
//! implementation that derives attack weights from live
//! `societas-relations` state.
//!
//! The bridge resolves an [`ArgumentId`](encounter_argumentation::ArgumentId)
//! to its asserting actor(s) via
//! `EncounterArgumentationState::actors_by_argument`, looks each actor
//! name up via [`crate::names::NameResolver`], queries the five
//! relationship dimensions, applies a coefficient recipe, and returns
//! the arithmetic mean across actor pairs.
//!
//! See `tests/relationship_source.rs` for a worked end-to-end example.
```

- [ ] **Step 2: Register the module in lib.rs**

Edit `/home/peter/code/societas/crates/encounter/src/lib.rs`. Add (place alphabetically — `argumentation` precedes `catalog` in the existing `pub mod` list):

```rust
#[cfg(feature = "argumentation")]
pub mod argumentation;
```

Do NOT add a re-export yet — the module has no exports until Task 3.

- [ ] **Step 3: Verify both build modes**

```bash
cargo build -p societas-encounter
cargo build -p societas-encounter --features argumentation
```

Both expected: clean.

- [ ] **Step 4: Commit**

```bash
git add crates/encounter/src/argumentation.rs crates/encounter/src/lib.rs
git commit -m "feat(societas-encounter): scaffold argumentation feature module"
```

### Task 3: Move coefficient constants

**Files:**
- Modify: `/home/peter/code/societas/crates/encounter/src/argumentation.rs`

Six `pub const` coefficient values copied verbatim from the Phase C source (`encounter-argumentation/src/societas_relationship.rs:25-37`).

- [ ] **Step 1: Append constants to the module**

Edit `/home/peter/code/societas/crates/encounter/src/argumentation.rs`. After the module-level docstring, add:

```rust
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_phase_c_v0_4_0() {
        // Locks in coefficient values shipped in encounter-argumentation
        // v0.4.0. Calibration changes should be conscious version bumps
        // with CHANGELOG entries — not drift.
        assert_eq!(BASELINE_WEIGHT, 0.5);
        assert_eq!(TRUST_COEF, -0.15);
        assert_eq!(FEAR_COEF, 0.25);
        assert_eq!(RESPECT_COEF, -0.05);
        assert_eq!(ATTRACTION_COEF, -0.05);
        assert_eq!(FRIENDSHIP_COEF, -0.10);
    }
}
```

- [ ] **Step 2: Run the constants test**

```bash
cargo test -p societas-encounter --features argumentation argumentation::tests::constants
```

Expected: 1 test passes.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter/src/argumentation.rs
git commit -m "feat(societas-encounter): coefficient constants for argumentation weight recipe"
```

### Task 4: Add `SocietasRelationshipSource` struct + constructor

**Files:**
- Modify: `/home/peter/code/societas/crates/encounter/src/argumentation.rs`

Struct, constructor, and a smoke test that verifies the constructor accepts the expected types. No `WeightSource` impl yet.

- [ ] **Step 1: Append the struct + impl + smoke test**

Edit `/home/peter/code/societas/crates/encounter/src/argumentation.rs`. After the constants block, add:

```rust
use crate::names::NameResolver;
use encounter_argumentation::ArgumentId;
use societas_core::{EntityId, SocialStore, Tick};
use societas_relations::{Dimension, RelationshipRegistry};
use std::collections::HashMap;

/// A [`WeightSource<ArgumentId>`](argumentation_weighted::WeightSource)
/// that reads relationship dimensions from a live `societas-relations`
/// registry + store.
///
/// See the module-level documentation for the coefficient recipe and
/// the aggregation strategy for multi-actor arguments.
pub struct SocietasRelationshipSource<'a> {
    registry: &'a RelationshipRegistry,
    store: &'a dyn SocialStore,
    resolver: &'a dyn NameResolver,
    actors_by_argument: &'a HashMap<ArgumentId, Vec<String>>,
    tick: Tick,
}

impl<'a> SocietasRelationshipSource<'a> {
    /// Construct a new source bound to the given registry, store,
    /// resolver, actor map, and evaluation tick.
    ///
    /// All references are borrowed for the adapter's lifetime `'a`.
    /// `tick` is owned and fixed at construction time — consumers
    /// wanting a new tick should build a fresh adapter.
    ///
    /// `actors_by_argument` is typically obtained by calling
    /// [`encounter_argumentation::EncounterArgumentationState::actors_by_argument`].
    #[must_use]
    pub fn new(
        registry: &'a RelationshipRegistry,
        store: &'a dyn SocialStore,
        resolver: &'a dyn NameResolver,
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
```

Then in the `#[cfg(test)] mod tests { ... }` block, append (alongside `constants_match_phase_c_v0_4_0`):

```rust
    use crate::names::StaticNameResolver;
    use societas_memory::MemStore;

    #[test]
    fn new_source_constructs() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver = StaticNameResolver::new();
        let actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        // Compile-time check: the constructor accepts these types.
        let _source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
    }
```

- [ ] **Step 2: Suppress dead-field warnings until Task 5 wires WeightSource**

In the same file, on the struct definition, add `#[allow(dead_code)]` to the struct (over `pub struct SocietasRelationshipSource<'a>`). Drop it again in Task 5 once the fields are read.

- [ ] **Step 3: Build with feature on**

```bash
cargo build -p societas-encounter --features argumentation
```

Expected: clean.

- [ ] **Step 4: Run the new tests**

```bash
cargo test -p societas-encounter --features argumentation argumentation::tests
```

Expected: 2 tests pass (constants + constructor smoke).

- [ ] **Step 5: Commit**

```bash
git add crates/encounter/src/argumentation.rs
git commit -m "feat(societas-encounter): SocietasRelationshipSource struct + constructor (no WeightSource impl yet)"
```

### Task 5: Implement `WeightSource<ArgumentId>`

**Files:**
- Modify: `/home/peter/code/societas/crates/encounter/src/argumentation.rs`

Full impl — fallback paths (unseeded/unresolvable → baseline), per-pair scoring via `pairwise_weight`, mean aggregation across the (attacker × target) Cartesian product. Eight behavioral tests: 3 fallback, 3 single-pair, 2 multi-actor.

- [ ] **Step 1: Write the failing tests**

In the tests module of `/home/peter/code/societas/crates/encounter/src/argumentation.rs`, append (alongside `constants_match_phase_c_v0_4_0` and `new_source_constructs`):

```rust
    use societas_core::ModifierSource;

    #[test]
    fn baseline_weight_when_attacker_has_no_actors() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver = StaticNameResolver::new();
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
        let resolver = StaticNameResolver::new();
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    #[test]
    fn baseline_weight_when_neither_name_resolves() {
        let store = MemStore::new();
        let registry = RelationshipRegistry::new();
        let resolver = StaticNameResolver::new();  // empty
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(ArgumentId::new("x"), vec!["alice".to_string()]);
        actors.insert(ArgumentId::new("y"), vec!["bob".to_string()]);
        let source =
            SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, Tick(0));
        let w = source.weight_for(&ArgumentId::new("x"), &ArgumentId::new("y"));
        assert_eq!(w, Some(BASELINE_WEIGHT));
    }

    fn single_pair_fixture() -> (
        RelationshipRegistry,
        MemStore,
        StaticNameResolver,
        HashMap<ArgumentId, Vec<String>>,
    ) {
        let registry = RelationshipRegistry::new();
        let store = MemStore::new();
        let mut resolver = StaticNameResolver::new();
        resolver.add("alice", EntityId::from_u64(1));
        resolver.add("bob", EntityId::from_u64(2));
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
    fn multi_attacker_averages_per_pair_weights() {
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver = StaticNameResolver::new();
        resolver.add("alice", EntityId::from_u64(1));
        resolver.add("bob", EntityId::from_u64(2));
        resolver.add("hostile", EntityId::from_u64(3));
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "bob".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["hostile".to_string()],
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
        let registry = RelationshipRegistry::new();
        let mut store = MemStore::new();
        let mut resolver = StaticNameResolver::new();
        resolver.add("alice", EntityId::from_u64(1));
        resolver.add("hostile", EntityId::from_u64(3));
        // "eve" deliberately NOT added.
        let mut actors: HashMap<ArgumentId, Vec<String>> = HashMap::new();
        actors.insert(
            ArgumentId::new("duo_attack"),
            vec!["alice".to_string(), "eve".to_string()],
        );
        actors.insert(
            ArgumentId::new("target_arg"),
            vec!["hostile".to_string()],
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

- [ ] **Step 2: Run the tests to verify they fail to compile**

```bash
cargo test -p societas-encounter --features argumentation argumentation
```

Expected: compile error: `no method named 'weight_for' found for struct 'SocietasRelationshipSource'`.

- [ ] **Step 3: Add the WeightSource impl + private helper**

In `/home/peter/code/societas/crates/encounter/src/argumentation.rs`, after the constructor's `impl<'a>` block, add:

```rust
use argumentation_weighted::WeightSource;

impl WeightSource<ArgumentId> for SocietasRelationshipSource<'_> {
    /// Compute the attack weight for `attacker → target` from live
    /// societas relationship state. See the module-level documentation
    /// for the coefficient recipe and aggregation strategy.
    ///
    /// Always returns `Some(w)` — this source has an opinion on every
    /// edge. Unseeded or unresolvable pairs fall back to
    /// [`BASELINE_WEIGHT`].
    ///
    /// Multi-actor arguments are aggregated by arithmetic mean across
    /// the (attacker_actor × target_actor) Cartesian product.
    /// Unresolvable actor names are silently skipped.
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
        Some(sum / f64::from(count))
    }
}

impl SocietasRelationshipSource<'_> {
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

- [ ] **Step 4: Remove the `#[allow(dead_code)]` from the struct**

All five fields are now read by `weight_for` / `pairwise_weight`. Drop the `#[allow(dead_code)]` you added in Task 4 Step 2.

- [ ] **Step 5: Run all argumentation tests**

```bash
cargo test -p societas-encounter --features argumentation argumentation
```

Expected: 10 tests pass (1 constants + 1 constructor + 3 fallback + 3 single-pair + 2 multi-actor).

- [ ] **Step 6: Verify default-build still works (feature OFF)**

```bash
cargo test -p societas-encounter
```

Expected: same baseline test count as Task 0 Step 3 (no argumentation tests; module is gated out).

- [ ] **Step 7: Clippy check both modes**

```bash
cargo clippy -p societas-encounter --all-targets --no-deps -- -D warnings
cargo clippy -p societas-encounter --all-targets --features argumentation --no-deps -- -D warnings
```

Both expected: clean.

- [ ] **Step 8: Re-export from lib.rs**

Edit `/home/peter/code/societas/crates/encounter/src/lib.rs`. After the existing `pub use` block, add:

```rust
#[cfg(feature = "argumentation")]
pub use argumentation::{
    ATTRACTION_COEF, BASELINE_WEIGHT, FEAR_COEF, FRIENDSHIP_COEF, RESPECT_COEF, TRUST_COEF,
    SocietasRelationshipSource,
};
```

- [ ] **Step 9: Commit**

```bash
git add crates/encounter/src/argumentation.rs crates/encounter/src/lib.rs
git commit -m "feat(societas-encounter): SocietasRelationshipSource WeightSource impl + 8 behavioral tests"
```

### Task 6: Integration test against `EncounterArgumentationState`

**Files:**
- Create: `/home/peter/code/societas/crates/encounter/tests/relationship_source.rs`

End-to-end: build an `EncounterArgumentationState`, seed two scheme instances, build a `SocietasRelationshipSource`, compute weights, wire them into the state via `add_weighted_attack`, observe credulous acceptance flip. Mirrors the test that previously lived in `argumentation/crates/encounter-argumentation/tests/uc_relationship_modulation.rs`.

- [ ] **Step 1: Create the integration test**

Create `/home/peter/code/societas/crates/encounter/tests/relationship_source.rs`:

```rust
//! End-to-end: SocietasRelationshipSource feeding EncounterArgumentationState.

#![cfg(feature = "argumentation")]

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::{types::Budget, WeightSource};
use encounter_argumentation::{ArgumentId, EncounterArgumentationState};
use societas_core::{EntityId, ModifierSource, Tick};
use societas_encounter::names::StaticNameResolver;
use societas_encounter::{SocietasRelationshipSource, TRUST_COEF};
use societas_memory::MemStore;
use societas_relations::{Dimension, RelationshipRegistry};

fn seed_state_with_pairwise_debate() -> (
    EncounterArgumentationState,
    ArgumentId,
    ArgumentId,
    StaticNameResolver,
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

    let mut resolver = StaticNameResolver::new();
    resolver.add("alice", EntityId::from_u64(1));
    resolver.add("bob", EntityId::from_u64(2));

    (state, alice_id, bob_id, resolver)
}

#[test]
fn high_trust_reduces_effective_attack_weight() {
    let (state, alice_id, bob_id, resolver) = seed_state_with_pairwise_debate();
    let mut store = MemStore::new();
    let registry = RelationshipRegistry::new();
    registry.add_modifier(
        &mut store,
        EntityId::from_u64(2),
        EntityId::from_u64(1),
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
    let expected = (0.5_f64 + TRUST_COEF).clamp(0.0, 1.0);
    assert!(
        (w - expected).abs() < 1e-9,
        "bob→alice with Trust=1.0 should produce 0.5 + TRUST_COEF = {expected}, got {w}"
    );

    let w_reverse = source.weight_for(&alice_id, &bob_id).unwrap();
    assert!(
        (w_reverse - 0.5).abs() < 1e-9,
        "alice→bob with no recorded trust should sit at baseline 0.5, got {w_reverse}"
    );
}

#[test]
fn same_scenario_flips_acceptance_at_different_budgets_for_different_weights() {
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

    let mut state_trust_mut = state_trust;
    state_trust_mut
        .add_weighted_attack(&bob_trust_id, &alice_trust_id, w_trust)
        .unwrap();
    let mut state_fear_mut = state_fear;
    state_fear_mut
        .add_weighted_attack(&bob_fear_id, &alice_fear_id, w_fear)
        .unwrap();

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

- [ ] **Step 2: Add `argumentation-schemes` to societas-encounter dev-deps**

The integration test uses `argumentation_schemes::catalog::default_catalog`. Edit `/home/peter/code/societas/crates/encounter/Cargo.toml`. Under `[dev-dependencies]`, add (alphabetically):

```toml
argumentation-schemes = { path = "../../../argumentation/crates/argumentation-schemes" }
```

(Direct path-dep on the dev-only side rather than a workspace entry — keeps the workspace-deps minimal until another non-test consumer needs it.)

- [ ] **Step 3: Run the integration test**

```bash
cargo test -p societas-encounter --features argumentation --test relationship_source
```

Expected: 2 tests pass.

- [ ] **Step 4: Run the entire suite (feature on)**

```bash
cargo test -p societas-encounter --features argumentation
```

Expected: baseline + 10 argumentation unit tests + 2 integration tests, all green.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter/Cargo.toml crates/encounter/tests/relationship_source.rs
git commit -m "test(societas-encounter): end-to-end SocietasRelationshipSource integration with EncounterArgumentationState"
```

### Task 7: CHANGELOG + verification

**Files:**
- Modify: `/home/peter/code/societas/crates/encounter/CHANGELOG.md` (create if absent)

- [ ] **Step 1: Check for an existing CHANGELOG**

```bash
ls /home/peter/code/societas/crates/encounter/CHANGELOG.md
```

If it exists, prepend the new entry below `# Changelog`.
If it does not, create the file with:

```markdown
# Changelog

```

- [ ] **Step 2: Add the new entry**

Prepend (below `# Changelog`):

```markdown
## [Unreleased] - argumentation feature

### Added
- New optional feature `argumentation` (default off) — adds
  `argumentation-weighted` and `encounter-argumentation` as deps.
- `argumentation::SocietasRelationshipSource` — implements
  `argumentation_weighted::WeightSource<ArgumentId>` by reading
  five-axis relationship state from `societas-relations` and applying
  a coefficient recipe. Resolves `ArgumentId → actor names` via
  `EncounterArgumentationState::actors_by_argument`, then `actor name
  → EntityId` via the existing `NameResolver` trait.
- Public coefficient constants: `BASELINE_WEIGHT`, `TRUST_COEF`,
  `FEAR_COEF`, `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF`.

### Notes
- Multi-actor arguments aggregate per-pair weights by arithmetic mean
  across the (attacker × target) Cartesian product. Unresolvable actor
  names are silently skipped (not promoted to baseline pairs).
- Coefficient values mirror those shipped in
  `encounter-argumentation v0.4.0` and are *provisional* — calibration
  against gameplay telemetry is future work.
- `encounter-argumentation v0.5.0` removes its temporary
  `SocietasRelationshipSource` and `NameResolver` (which lived in the
  wrong crate). Consumers migrate to `societas-encounter` with feature
  `argumentation` enabled.
```

- [ ] **Step 3: Verify the workspace still builds with default features**

```bash
cargo check --workspace
```

Expected: clean. (No societas-internal crate accidentally relies on the new optional deps.)

- [ ] **Step 4: Final test run**

```bash
cargo test -p societas-encounter --features argumentation
cargo test -p societas-encounter
```

Both expected: green.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter/CHANGELOG.md
git commit -m "docs(societas-encounter): CHANGELOG entry for argumentation feature"
```

### Task 8: Merge Phase 1 to main

**Files:** none (git operations).

- [ ] **Step 1: Final review of branch contents**

```bash
git log --oneline main..HEAD
```

Expected: 7 commits (one per Task 1-7).

- [ ] **Step 2: Switch to main, merge `--no-ff`**

```bash
git checkout main
git merge --no-ff feat/argumentation-weight-source -m "Merge branch 'feat/argumentation-weight-source'"
```

- [ ] **Step 3: Verify post-merge tests**

```bash
cargo test -p societas-encounter --features argumentation
cargo test -p societas-encounter
```

Both expected: green.

- [ ] **Step 4: Delete the feature branch**

```bash
git branch -d feat/argumentation-weight-source
```

- [ ] **Step 5: Stop here. DO NOT proceed to Phase 2 until Phase 1 is on main.**

Phase 2 deletes the `SocietasRelationshipSource` from `encounter-argumentation`, which would break `societas-encounter` if it still depended on the now-absent symbols. Phase 1 must be the source of truth for the type before Phase 2 removes the duplicate.

---

## Phase 2 — Strip moved types from `encounter-argumentation`, ship v0.5.0

Working directory: `/home/peter/code/argumentation/`.

### Task 9: Branch + verify Phase 1 is in place

**Files:** none (verification only)

- [ ] **Step 1: Branch from main**

```bash
cd /home/peter/code/argumentation
git checkout main && git pull --ff-only
git checkout -b feat/encounter-arg-v0.5.0-shed-societas
```

- [ ] **Step 2: Verify Phase 1 landed in societas**

```bash
grep "argumentation = " /home/peter/code/societas/crates/encounter/Cargo.toml
ls /home/peter/code/societas/crates/encounter/src/argumentation.rs
ls /home/peter/code/societas/crates/encounter/tests/relationship_source.rs
```

Expected: feature flag declared, source + integration test files present. STOP if any missing.

### Task 10: Delete moved source files

**Files:**
- Delete: `crates/encounter-argumentation/src/societas_relationship.rs`
- Delete: `crates/encounter-argumentation/src/name_resolver.rs`
- Delete: `crates/encounter-argumentation/tests/uc_relationship_modulation.rs`

- [ ] **Step 1: Delete the three files**

```bash
git rm crates/encounter-argumentation/src/societas_relationship.rs
git rm crates/encounter-argumentation/src/name_resolver.rs
git rm crates/encounter-argumentation/tests/uc_relationship_modulation.rs
```

- [ ] **Step 2: Do NOT build yet — lib.rs still references the deleted modules**

Build will fail until Task 11 lands. This is expected; we commit Tasks 10–13 atomically at the end of Task 13.

### Task 11: Drop module declarations + re-exports from `lib.rs`

**Files:**
- Modify: `crates/encounter-argumentation/src/lib.rs`

- [ ] **Step 1: Remove the four lines**

Edit `crates/encounter-argumentation/src/lib.rs`. Delete:

```rust
pub mod name_resolver;
```

```rust
pub mod societas_relationship;
```

```rust
pub use name_resolver::NameResolver;
```

```rust
pub use societas_relationship::{
    ATTRACTION_COEF, BASELINE_WEIGHT, FEAR_COEF, FRIENDSHIP_COEF, RESPECT_COEF, TRUST_COEF,
    SocietasRelationshipSource,
};
```

- [ ] **Step 2: Verify nothing else in the lib references deleted symbols**

```bash
grep -rn 'name_resolver\|societas_relationship\|SocietasRelationshipSource\|NameResolver\|TRUST_COEF\|FEAR_COEF\|RESPECT_COEF\|ATTRACTION_COEF\|FRIENDSHIP_COEF\|BASELINE_WEIGHT' crates/encounter-argumentation/src/ crates/encounter-argumentation/tests/
```

Expected: no results except possibly historical doc comments and the doc comment on `state.rs:54-59` (which Task 12 fixes).

If any other reference appears (e.g. in a doctest in `lib.rs`), update or remove it.

- [ ] **Step 3: Build the lib (still expected to fail on the state.rs doc-link until Task 12)**

```bash
cargo build -p encounter-argumentation --lib
```

Expected: builds successfully (rustdoc warnings about the dangling state.rs intra-doc link are warnings, not errors). If it errors out due to a missed import, fix here.

### Task 12: Refresh stale doc references

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`
- Modify: `crates/encounter-argumentation/README.md`

- [ ] **Step 1: Update the `EncounterArgumentationState::new` doc comment**

Edit `crates/encounter-argumentation/src/state.rs`. The current Phase C-era comment around lines 53-60 reads:

```rust
    /// Create a new state with the given scheme registry and zero
    /// scene intensity. Consumers that want relationship-modulated
    /// attack weights should construct a [`crate::societas_relationship::SocietasRelationshipSource`]
    /// over a societas-relations registry and store, then pass its
    /// computed weights into
    /// [`add_weighted_attack`](Self::add_weighted_attack); the state
    /// does not auto-wire the source.
    #[must_use]
    pub fn new(registry: CatalogRegistry) -> Self {
```

Replace with:

```rust
    /// Create a new state with the given scheme registry and zero
    /// scene intensity. Consumers that want relationship-modulated
    /// attack weights should construct a societas-aware `WeightSource`
    /// (e.g. `societas_encounter::SocietasRelationshipSource` from the
    /// `societas-encounter` crate with the `argumentation` feature
    /// enabled), then pass its computed weights into
    /// [`add_weighted_attack`](Self::add_weighted_attack); the state
    /// does not auto-wire the source.
    #[must_use]
    pub fn new(registry: CatalogRegistry) -> Self {
```

(Plain backticks on the cross-crate type — no intra-doc link, since rustdoc cannot resolve names in a sibling-workspace crate that we don't depend on.)

- [ ] **Step 2: Refresh the README "Relationship modulation" section**

Edit `crates/encounter-argumentation/README.md`. Find the existing "Relationship modulation (Phase C)" section (around lines 85-100). Replace with:

```markdown
### Relationship modulation

Societas-aware attack weights live in the **`societas-encounter`** crate
(in the `societas` workspace) under the `argumentation` feature.
`societas_encounter::SocietasRelationshipSource` implements
`argumentation_weighted::WeightSource<ArgumentId>` by reading the five
relationship dimensions from `societas-relations` and applying a
coefficient recipe.

Wiring sketch:

```rust,ignore
use societas_encounter::{SocietasRelationshipSource, names::StaticNameResolver};
let resolver = StaticNameResolver::new();
let source = SocietasRelationshipSource::new(
    &registry, &store, &resolver,
    state.actors_by_argument(),
    tick,
);
let w = source.weight_for(&attacker_arg, &target_arg).unwrap();
state.add_weighted_attack(&attacker_arg, &target_arg, w)?;
```
```

- [ ] **Step 3: Re-grep for any remaining stale references**

```bash
grep -rn 'RelationshipDims\|RelationshipSnapshot\|RelationshipWeightSource' crates/encounter-argumentation/
grep -rn 'crate::societas_relationship\|crate::name_resolver' crates/encounter-argumentation/
```

Expected: only historical CHANGELOG mentions (`[0.2.0]`, `[0.4.0]`).

### Task 13: Drop societas deps + bump to 0.5.0 + CHANGELOG

**Files:**
- Modify: `crates/encounter-argumentation/Cargo.toml`
- Modify: `crates/encounter-argumentation/CHANGELOG.md`

- [ ] **Step 1: Drop deps**

Edit `crates/encounter-argumentation/Cargo.toml`. Delete the three lines:

```toml
societas-core = { path = "../../../societas/crates/core" }
societas-relations = { path = "../../../societas/crates/relations" }
```

```toml
societas-memory = { path = "../../../societas/crates/memory" }
```

The dependencies block should return to:

```toml
[dependencies]
encounter = { path = "../../../encounter" }
argumentation = { path = "../.." }
argumentation-schemes = { path = "../argumentation-schemes" }
argumentation-bipolar = { path = "../argumentation-bipolar" }
argumentation-weighted = { path = "../argumentation-weighted" }
argumentation-weighted-bipolar = { path = "../argumentation-weighted-bipolar" }
thiserror = "2.0"

[dev-dependencies]
```

(Empty `[dev-dependencies]` is fine — the section header alone causes no harm. If you prefer, delete the section header entirely.)

- [ ] **Step 2: Bump version**

In the same Cargo.toml, change line 3:

```toml
version = "0.4.0"
```

to:

```toml
version = "0.5.0"
```

- [ ] **Step 3: Add CHANGELOG entry**

Edit `crates/encounter-argumentation/CHANGELOG.md`. Prepend (below `# Changelog`, above the existing `## [0.4.0]` entry):

```markdown
## [0.5.0] - 2026-04-24

### Removed (breaking)
- `SocietasRelationshipSource`, `NameResolver`, and the six coefficient
  constants (`BASELINE_WEIGHT`, `TRUST_COEF`, `FEAR_COEF`,
  `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF`) are deleted.
  These were misplaced in v0.4.0 — the societas-aware weight source
  belongs in the `societas-encounter` crate alongside the other
  societas↔encounter bridge types (action scorer, acceptance eval,
  catalog, effects). No backward-compat shim.

### Migration
Consumers using the deleted types should add the `societas-encounter`
crate (in the `societas` workspace) with the `argumentation` feature:

```toml
[dependencies]
societas-encounter = { path = "../societas/crates/encounter", features = ["argumentation"] }
```

Imports change from:

```rust
use encounter_argumentation::{SocietasRelationshipSource, NameResolver, TRUST_COEF};
```

to:

```rust
use societas_encounter::{SocietasRelationshipSource, TRUST_COEF};
use societas_encounter::names::NameResolver;  // or StaticNameResolver
```

The `StaticNameResolver` in `societas-encounter` is strictly richer
than v0.4.0's `HashMap<String, EntityId>` blanket impl: it requires
`Send + Sync`, validates against affordance role-binding name
collisions (`"self"`, `"target"`, etc.), and offers a `try_add`
fallible variant.

### Removed dependencies
- `societas-core`, `societas-relations` (production deps)
- `societas-memory` (dev-dep)

### Preserved (no changes)
All v0.3.0 + v0.4.0 surface remains: `EncounterArgumentationState`
including the `actors_by_argument()` accessor (used by
`SocietasRelationshipSource` from its new home), `StateAcceptanceEval`,
`StateActionScorer`, `AffordanceKey`, error variants, scheme/CQ APIs.
```

- [ ] **Step 4: Verify the crate builds + tests pass**

```bash
cargo test -p encounter-argumentation
```

Expected: all preserved tests still pass (66 unit + 3 acceptance + 3 critical-moves + 2 integration + 3 resolver + 2 scoring + 1 intensity sweep + 3 multibeat + 3 parity + 4 value-argument + 2 doctests). **No more uc_relationship_modulation tests** — those moved to societas-encounter.

- [ ] **Step 5: Clippy + doc clean**

```bash
cargo clippy -p encounter-argumentation --all-targets --no-deps -- -D warnings
cargo doc -p encounter-argumentation --no-deps 2>&1 | grep -iE "warning|error"
```

Both expected: clean. (The state.rs doc-link change in Task 12 dropped the dangling intra-doc link; the README cross-crate ref is plain backticks.)

- [ ] **Step 6: Verify the workspace builds**

```bash
cargo check --workspace
```

Expected: clean.

- [ ] **Step 7: Commit Tasks 10–13 atomically**

```bash
git add crates/encounter-argumentation/Cargo.toml \
        crates/encounter-argumentation/CHANGELOG.md \
        crates/encounter-argumentation/src/lib.rs \
        crates/encounter-argumentation/src/state.rs \
        crates/encounter-argumentation/README.md
# (deletions from Task 10 are already staged via `git rm`)
git commit -m "refactor(encounter-argumentation): v0.5.0 — relocate societas weight source to societas-encounter"
```

- [ ] **Step 8: Tag locally**

```bash
git tag encounter-argumentation-v0.5.0
```

Do NOT push. User approves pushes separately.

### Task 14: Final code review + merge to main

**Files:** none (review + git operations)

- [ ] **Step 1: Get review SHAs**

```bash
BASE_SHA=$(git merge-base HEAD main)
HEAD_SHA=$(git rev-parse HEAD)
echo "BASE=$BASE_SHA"
echo "HEAD=$HEAD_SHA"
```

- [ ] **Step 2: Dispatch superpowers:requesting-code-review**

Use the skill with:

- WHAT_WAS_IMPLEMENTED: "encounter-argumentation v0.5.0: relocated SocietasRelationshipSource + NameResolver + coefficient constants to societas-encounter (Phase 1). Removed deps on societas-* crates. Refreshed stale doc references. Migration note in CHANGELOG."
- PLAN_OR_REQUIREMENTS: docs/superpowers/plans/2026-04-24-move-societas-bridge-to-societas-encounter.md
- BASE_SHA: from Step 1
- HEAD_SHA: from Step 1
- DESCRIPTION: "Architecture cleanup. v0.4.0 misplaced the societas-aware weight source in encounter-argumentation. v0.5.0 deletes those types — they ship from societas-encounter::argumentation under the new `argumentation` feature flag. Validate: (1) no symbols reference deleted modules, (2) CHANGELOG migration note is accurate (the new SocietasRelationshipSource takes `&dyn NameResolver` not a generic, and uses the richer Send+Sync trait from societas-encounter), (3) README example compiles against the actual societas-encounter API, (4) every preserved test still passes."

- [ ] **Step 3: Act on review feedback**

Apply receiving-code-review skill. Fix Critical + Important before merging. Note Minor for v0.5.1.

- [ ] **Step 4: Merge to main**

```bash
git checkout main
git merge --no-ff feat/encounter-arg-v0.5.0-shed-societas \
  -m "Merge branch 'feat/encounter-arg-v0.5.0-shed-societas'"
git branch -d feat/encounter-arg-v0.5.0-shed-societas
```

- [ ] **Step 5: Verify post-merge tests pass**

```bash
cargo test -p encounter-argumentation
```

Expected: green.

- [ ] **Step 6: Report**

"v0.5.0 merged on main. Tag `encounter-argumentation-v0.5.0` ready to push. Phase 1 (`societas-encounter` argumentation feature) already on main in `/home/peter/code/societas/`."

---

## Self-review

**Spec coverage:**
- ✅ Move SocietasRelationshipSource → societas-encounter (Tasks 4-5)
- ✅ Use societas-encounter's existing NameResolver (Q1, Q2, Q4) — Phase 1 uses `&dyn NameResolver`, no Phase C duplicate ships
- ✅ Feature-gate the argumentation deps so default societas-encounter consumers don't pay the cost (Q3, Tasks 1-2)
- ✅ Move integration test → societas-encounter (Q5, Task 6)
- ✅ Drop moved types from encounter-argumentation (Task 10)
- ✅ Drop societas deps from encounter-argumentation (Task 13)
- ✅ Refresh stale doc references in state.rs + README (Task 12)
- ✅ Bump encounter-argumentation to 0.5.0 with CHANGELOG migration note (Task 13)
- ✅ Cross-repo ordering enforced (Task 8 stops Phase 1; Task 9 verifies before Phase 2 starts)
- ✅ Final code review (Task 14)

**Placeholder scan:** No "TBD" / "implement later" / "fill in details" / "similar to Task N" in the plan. Every code block is copy-pasteable. The Task 4 placeholder (`#[allow(dead_code)]`) is explicitly marked with the deletion instruction in Task 5.

**Type consistency:**
- `SocietasRelationshipSource::new(&registry, &store, &dyn NameResolver, &actors_by_argument, Tick)` — same signature in Tasks 4 (constructor), 5 (test fixtures), 6 (integration test). ✓
- `weight_for(&self, &ArgumentId, &ArgumentId) -> Option<f64>` — same throughout. ✓
- `BASELINE_WEIGHT` etc. — identical names in Tasks 3, 5, 6, 7 CHANGELOG. ✓
- `StaticNameResolver::new() / .add(name, EntityId)` — used identically in Tasks 5, 6 (matches verified API in `societas/crates/encounter/src/names.rs`). ✓
- Feature name `"argumentation"` — same string in Tasks 1 (Cargo declaration), 2 (`#[cfg(feature = "argumentation")]`), 5 (test commands), 6 (`#![cfg(feature = "argumentation")]`), 7 (CHANGELOG). ✓

**Cross-repo ordering:** Phase 1 must merge to main before Phase 2 starts. Task 8 ends Phase 1 with merge + delete branch + STOP signal. Task 9 verifies Phase 1 landed before Phase 2 mutates anything. ✓

**Branching pattern matches Phase A/B/C convention:** feat-branch off main, develop, merge `--no-ff`, delete branch. ✓

---

## Execution handoff

Plan complete and saved to `/home/peter/code/argumentation/docs/superpowers/plans/2026-04-24-move-societas-bridge-to-societas-encounter.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, two-stage review between tasks, fast iteration. Cross-repo work coordinated as two phase blocks (Phase 1 fully on societas main before Phase 2 starts).

**2. Inline Execution** — execute tasks in this session using executing-plans, batch checkpoints.

Which approach?
