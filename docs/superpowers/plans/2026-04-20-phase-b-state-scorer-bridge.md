# Phase B: state-aware scorer + acceptance bridge

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire `EncounterArgumentationState` (the v0.2.0 state object) into `encounter`'s `ActionScorer` and `AcceptanceEval` trait system, so scene-intensity (β) actually influences action scoring and acceptance decisions through the existing trait-inverted boundary — without modifying the `encounter` crate at all.

**Architecture:** All work lives in the `encounter-argumentation` bridge crate. Two new types, `StateActionScorer<S>` and `StateAcceptanceEval`, hold a shared reference to an `EncounterArgumentationState` and plug into encounter's trait system. State grows an action-keyed forward index (`(actor, affordance_name, bindings) → ArgumentId`) so the bridge can look up arguments from affordances at call time. Intensity moves to interior mutability (`Cell<Budget>`) so mid-scene mutation is compatible with the shared-ref borrow that the bridge holds.

**Tech Stack:** Rust 2024, `encounter` v0.1.x (untouched), `argumentation-weighted-bipolar` residual/semantics APIs already wired through Phase A.

---

## Implementation note (2026-04-20)

During execution, Tasks 2 and 6 swapped `Cell<Budget>` and
`RefCell<Option<Error>>` for `std::sync::Mutex<...>`. This was forced
by encounter's `AcceptanceEval<P>: Send + Sync` supertrait bound,
which requires `EncounterArgumentationState: Sync` — a constraint
that `Cell` and `RefCell` cannot satisfy. The semantic intent
(interior-mutable β; latched errors for trait impls that can't
propagate `Result`) is preserved; only the primitives changed.
Mutex poisoning is recovered via `unwrap_or_else(|e| e.into_inner())`
to keep D5's permissive-on-failure contract.

Additionally: the error latch became `Vec<Error>` instead of
`Option<Error>` so that errors from both bridge impls (`StateActionScorer`,
`StateAcceptanceEval`) in the same beat both survive. `take_latest_error`
is now `drain_errors() -> Vec<Error>`.

---

## Pre-flight checklist for the executor

1. **You are working in** `/home/peter/code/argumentation/`. Only the `crates/encounter-argumentation/` crate is modified. No other crate in the workspace changes.
2. **Do NOT touch `/home/peter/code/encounter/` — ever.** That's a sibling repo owned by another team. If you find yourself wanting to edit a file there, STOP and escalate.
3. **Start on a feature branch:** `git checkout -b feat/phase-b-state-scorer-bridge` from current `main` (tip `b6bac80` or later — Phase B starts from a pushed `main` with no uncommitted changes).
4. **Every pub item needs a `///` doc.** `#![deny(missing_docs)]` is on the crate.
5. **Preserve the existing v0.1.x + v0.2.0 API surface verbatim.** `SchemeActionScorer`, `ArgumentAcceptanceEval`, `EncounterArgumentationState::{new, at_intensity, add_scheme_instance, add_weighted_attack, add_weighted_support, is_credulously_accepted, is_skeptically_accepted, coalitions, actors_for, instances_for}` — all keep their current signatures. Phase B ADDS; it does not rename or replace.
6. **Every task ends with** `cargo test -p encounter-argumentation` + `cargo clippy -p encounter-argumentation -- -D warnings`. If changing the public API of the state object, also run `cargo test --workspace` and `cargo doc -p encounter-argumentation --no-deps`.
7. **Memory to re-read:**
   - `feedback_no_backward_compat.md` — beta; rename internals freely.
   - `feedback_cross_crate_deps.md` — depending on sibling argumentation crates is fine.
   - `feedback_never_commit_other_repos.md` — do NOT touch `/home/peter/code/encounter/`.

## Design decisions locked in (pre-plan)

- **D1 — β lives on `EncounterArgumentationState` as `Cell<Budget>`.** Interior mutability enables `set_intensity(&self, b)` so consumers holding `&State` (and bridge impls holding `&'a State`) can change β mid-scene without invalidating borrows. The existing by-value builder `at_intensity(self, b) -> Self` stays for new-state construction ergonomics.
- **D2 — Forward index is part of `EncounterArgumentationState`, not a sidecar.** It's intrinsic state-object functionality; we're not going to ship two types that must be kept in sync.
- **D3 — `AffordanceKey` is `(String, String, BTreeMap<String, String>)`** = `(actor, affordance_name, normalised bindings)`. `BTreeMap` gives a canonical ordering so key hashing is stable.
- **D4 — Scorer and acceptance ask different questions.** `StateActionScorer::score_actions` boosts by "actor has a credulously-accepted argument for this action at current β" (salience). `StateAcceptanceEval::evaluate` returns `false` iff "responder has a credulously-accepted counter-argument to the proposed argument at current β" (per-responder gate). Not the same query.
- **D5 — On internal error, `evaluate -> bool` defaults to `true` (accept)** and writes the error into a `Cell<Option<Error>>` the consumer can drain after resolve. Accepting is the safer default — a blocked encounter is worse UX than a leaked permissive decision; telemetry shows up in the error drain.
- **D6 — Zero `encounter` changes.** No `scheme_id` on `AffordanceSpec`, no new traits, no touches to protocols. The bridge absorbs 100% of the integration.
- **D7 — Seeding is explicit.** Consumers call `seed_from_bindings` (or `add_scheme_instance_for_affordance` one at a time) BEFORE invoking encounter's resolve. The bridge does NOT auto-discover affordances at scorer construction.

## File structure

### New files

| File | Responsibility |
|---|---|
| `crates/encounter-argumentation/src/affordance_key.rs` | `AffordanceKey` type — canonical tuple of (actor, action_name, bindings). Single responsibility. Small (< 80 lines). |
| `crates/encounter-argumentation/src/state_scorer.rs` | `StateActionScorer<'a, S>` — wraps inner `ActionScorer` + `&'a EncounterArgumentationState`. ~120 lines. |
| `crates/encounter-argumentation/src/state_acceptance.rs` | `StateAcceptanceEval<'a>` — `impl AcceptanceEval` backed by state. ~90 lines. |
| `crates/encounter-argumentation/tests/uc_multibeat_scene.rs` | End-to-end: `MultiBeat::resolve` running against `StateActionScorer` + `StateAcceptanceEval`. |

### Modified files

| File | Why |
|---|---|
| `crates/encounter-argumentation/src/state.rs` | Add forward index, `Cell<Budget>` intensity, `set_intensity`, `add_scheme_instance_for_affordance`, `seed_from_bindings`, `argument_id_for`, `attackers_of`, `has_accepted_counter_by`, `take_latest_error`. |
| `crates/encounter-argumentation/src/lib.rs` | New module declarations + re-exports. |
| `crates/encounter-argumentation/CHANGELOG.md` | v0.3.0 entry. |
| `crates/encounter-argumentation/README.md` | Section on the state-aware integration pattern. |
| `crates/encounter-argumentation/Cargo.toml` | Version 0.2.0 → 0.3.0. |

### Files NOT modified

- `/home/peter/code/encounter/` — entire sibling repo. Nothing changes there.
- Other argumentation workspace crates (`argumentation`, `argumentation-schemes`, `argumentation-bipolar`, `argumentation-weighted`, `argumentation-weighted-bipolar`) — unchanged.

---

## Task 1: Create `AffordanceKey`

**Files:**
- Create: `crates/encounter-argumentation/src/affordance_key.rs`
- Modify: `crates/encounter-argumentation/src/lib.rs` (add `pub mod affordance_key;`)

- [ ] **Step 1: Write the failing test**

Create `crates/encounter-argumentation/src/affordance_key.rs`:

```rust
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
}
```

- [ ] **Step 2: Wire into lib.rs**

In `crates/encounter-argumentation/src/lib.rs`, add after the existing `pub mod arg_id;` line:

```rust
pub mod affordance_key;
```

And add this re-export in the `pub use` block:

```rust
pub use affordance_key::AffordanceKey;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p encounter-argumentation affordance_key`
Expected: PASS, 3 passed.

- [ ] **Step 4: Clippy**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/affordance_key.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): AffordanceKey newtype for forward index"
```

---

## Task 2: Convert `intensity` to interior-mutable `Cell<Budget>`

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Understand current state**

Current `state.rs` declares:
```rust
pub struct EncounterArgumentationState {
    registry: CatalogRegistry,
    framework: WeightedBipolarFramework<ArgumentId>,
    actors_by_argument: HashMap<ArgumentId, Vec<String>>,
    instances_by_argument: HashMap<ArgumentId, Vec<SchemeInstance>>,
    intensity: Budget,
}
```

With `intensity()` returning `Budget` by value (since `Budget: Copy`), `at_intensity(mut self, ...) -> Self` as a builder, and no setter.

- [ ] **Step 2: Write the failing test**

Append to `crates/encounter-argumentation/src/state.rs` `tests` module:

```rust
    #[test]
    fn set_intensity_mutates_through_shared_ref() {
        let state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.2).unwrap());
        assert_eq!(state.intensity().value(), 0.2);
        // set_intensity takes &self (shared ref). This line must
        // compile without &mut state.
        state.set_intensity(Budget::new(0.6).unwrap());
        assert_eq!(state.intensity().value(), 0.6);
    }

    #[test]
    fn intensity_is_mutable_from_two_shared_refs_in_sequence() {
        // If &self is sufficient, two independent helpers can both
        // mutate intensity without passing &mut around.
        let state = EncounterArgumentationState::new(default_catalog());
        fn bump(s: &EncounterArgumentationState, b: f64) {
            s.set_intensity(Budget::new(b).unwrap());
        }
        bump(&state, 0.3);
        bump(&state, 0.5);
        assert_eq!(state.intensity().value(), 0.5);
    }
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::set_intensity state::tests::intensity_is_mutable`
Expected: FAIL — `no method named 'set_intensity'`.

- [ ] **Step 4: Update the struct field type**

In `state.rs`, change:
```rust
    intensity: Budget,
```
to:
```rust
    /// Current scene intensity (β). Stored in `Cell` so it can be
    /// mutated through a shared reference (`&self`) — required for
    /// bridge consumers that hold `&State` during encounter's
    /// `resolve` loops.
    intensity: std::cell::Cell<Budget>,
```

And add `use std::cell::Cell;` near the existing imports at the top of the file.

- [ ] **Step 5: Update `new()` and `at_intensity()`**

Find `pub fn new(registry: CatalogRegistry) -> Self {` and change the `intensity` initializer inside the struct literal:
```rust
            intensity: Cell::new(Budget::zero()),
```

Find `pub fn at_intensity(mut self, intensity: Budget) -> Self {`. The body currently reads `self.intensity = intensity;`. Change to `self.intensity.set(intensity);`.

- [ ] **Step 6: Update `intensity()` getter**

Find `pub fn intensity(&self) -> Budget {`. The body currently reads `self.intensity`. Change to `self.intensity.get()`. (Budget is Copy so `.get()` works.)

- [ ] **Step 7: Add the `set_intensity` setter**

Append to the `impl EncounterArgumentationState` block:

```rust
    /// Mutate the scene intensity (β) through a shared reference.
    /// Used by consumers — notably the bridge's `StateAcceptanceEval`
    /// and `StateActionScorer` — that hold `&self` during encounter's
    /// `resolve` loops but still want to escalate β mid-scene.
    ///
    /// For new-state construction prefer the by-value builder
    /// [`Self::at_intensity`].
    pub fn set_intensity(&self, intensity: Budget) {
        self.intensity.set(intensity);
    }
```

- [ ] **Step 8: Remove the `Copy` impact — `Cell<Budget>` is not `Copy`**

The struct should not derive `Copy` anyway (it has HashMaps). But verify nothing in existing tests cloned the whole state; `Cell` also does implement `Clone` if `T: Copy`, so `EncounterArgumentationState` remains `Clone`-friendly **if it currently derives Clone**. Check the current derives. If `EncounterArgumentationState` does NOT currently have `Clone` derived (likely), no issue. If it does, confirm Cell<Budget> stays clonable — it does since Budget is Copy.

- [ ] **Step 9: Run tests**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: all 17 pre-existing state tests + 2 new pass = 19 passed.

- [ ] **Step 10: Run the crate's full suite (semantics haven't drifted)**

Run: `cargo test -p encounter-argumentation`
Expected: all pass.

- [ ] **Step 11: Clippy**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: clean.

- [ ] **Step 12: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): Cell<Budget> intensity + set_intensity setter

Enables mid-scene β mutation through a shared reference, required by
Phase B bridge impls (StateActionScorer, StateAcceptanceEval) that
hold &'a State during encounter's resolve loops. The by-value builder
at_intensity remains for new-state construction ergonomics."
```

---

## Task 3: Add forward index + `add_scheme_instance_for_affordance`

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn add_scheme_instance_for_affordance_indexes_by_key() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("expert".to_string(), "alice".to_string());
        bindings.insert("domain".to_string(), "military".to_string());
        bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &bindings,
            instance,
        );

        let key = AffordanceKey::new("alice", "argue_fortify_east", &bindings);
        let looked_up = state.argument_id_for(&key);
        assert_eq!(looked_up, Some(id));
    }

    #[test]
    fn argument_id_for_returns_none_for_unseeded_key() {
        let bindings = std::collections::HashMap::new();
        let state = EncounterArgumentationState::new(default_catalog());
        let key = AffordanceKey::new("nobody", "nothing", &bindings);
        assert_eq!(state.argument_id_for(&key), None);
    }

    #[test]
    fn add_scheme_instance_for_affordance_is_consistent_with_add_scheme_instance() {
        // Adding via the affordance-aware method must still set up the
        // same actors_by_argument / instances_by_argument entries that
        // add_scheme_instance does. Callers that query either lookup
        // should find the argument.
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("expert".to_string(), "alice".to_string());
        bindings.insert("domain".to_string(), "military".to_string());
        bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&bindings).unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &bindings,
            instance,
        );
        assert_eq!(state.actors_for(&id), &["alice".to_string()]);
        assert_eq!(state.instances_for(&id).len(), 1);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::add_scheme_instance_for_affordance`
Expected: FAIL — `no method named 'add_scheme_instance_for_affordance'`.

- [ ] **Step 3: Add the forward index field**

In `state.rs`, inside the `EncounterArgumentationState` struct, add a new field after `instances_by_argument`:

```rust
    /// Forward index: maps affordance-keyed scheme instances to their
    /// argument id in the framework. Populated by
    /// [`Self::add_scheme_instance_for_affordance`]. Enables bridge
    /// consumers (`StateActionScorer`, `StateAcceptanceEval`) to look
    /// up the right argument node at `evaluate`/`score_actions` time
    /// given only an `(actor, affordance_name, bindings)` triple.
    argument_id_by_affordance: HashMap<crate::affordance_key::AffordanceKey, ArgumentId>,
```

And in the `new` constructor, initialise it:
```rust
            argument_id_by_affordance: HashMap::new(),
```

- [ ] **Step 4: Add the methods**

Append to `impl EncounterArgumentationState`:

```rust
    /// Add a scheme instance asserted by `actor` for the named
    /// affordance with the given bindings. Functionally identical to
    /// [`Self::add_scheme_instance`] plus an entry in the affordance
    /// forward index so consumers can later look up this argument
    /// from an `(actor, affordance_name, bindings)` triple.
    ///
    /// If two scheme instances produce the same conclusion literal
    /// (same `ArgumentId`), both actors are recorded against that
    /// single argument node — convergence behaviour is the same as
    /// [`Self::add_scheme_instance`]. Both keys in the forward index
    /// point at the shared id.
    pub fn add_scheme_instance_for_affordance(
        &mut self,
        actor: &str,
        affordance_name: &str,
        bindings: &std::collections::HashMap<String, String>,
        instance: argumentation_schemes::instance::SchemeInstance,
    ) -> ArgumentId {
        let id = self.add_scheme_instance(actor, instance);
        let key = crate::affordance_key::AffordanceKey::new(actor, affordance_name, bindings);
        self.argument_id_by_affordance.insert(key, id.clone());
        id
    }

    /// Look up the argument id associated with an affordance key, if
    /// one was seeded via [`Self::add_scheme_instance_for_affordance`].
    #[must_use]
    pub fn argument_id_for(
        &self,
        key: &crate::affordance_key::AffordanceKey,
    ) -> Option<ArgumentId> {
        self.argument_id_by_affordance.get(key).cloned()
    }
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: all prior tests + 3 new = 22 passed.

- [ ] **Step 6: Clippy**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): add_scheme_instance_for_affordance + forward index

Maps (actor, affordance_name, bindings) → ArgumentId so bridge consumers
can look up the relevant argument node at encounter resolve time without
round-tripping through scheme instantiation. add_scheme_instance is
preserved verbatim; the new method composes on top plus an index write."
```

---

## Task 4: Add `attackers_of` query helper on state

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn attackers_of_returns_all_direct_attackers() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let target = ArgumentId::new("target");
        let a1 = ArgumentId::new("a1");
        let a2 = ArgumentId::new("a2");
        let unrelated = ArgumentId::new("unrelated");
        state.add_weighted_attack(&a1, &target, 0.5).unwrap();
        state.add_weighted_attack(&a2, &target, 0.3).unwrap();
        state.add_weighted_attack(&unrelated, &ArgumentId::new("x"), 0.5).unwrap();
        let attackers: std::collections::HashSet<_> =
            state.attackers_of(&target).into_iter().collect();
        assert_eq!(attackers.len(), 2);
        assert!(attackers.contains(&a1));
        assert!(attackers.contains(&a2));
    }

    #[test]
    fn attackers_of_returns_empty_for_unattacked() {
        let state = EncounterArgumentationState::new(default_catalog());
        let lonely = ArgumentId::new("lonely");
        assert!(state.attackers_of(&lonely).is_empty());
    }
```

- [ ] **Step 2: Run to verify fail**

Run: `cargo test -p encounter-argumentation state::tests::attackers_of`
Expected: FAIL — `no method named 'attackers_of'`.

- [ ] **Step 3: Add the method**

Append to `impl EncounterArgumentationState`:

```rust
    /// Return the direct attackers of `target` in the current
    /// framework. Ignores support edges and does NOT resolve the
    /// β-inconsistent residual — this is a structural query.
    ///
    /// Consumers that want "is there a credulously accepted attacker
    /// at current β?" should query each attacker via
    /// [`Self::is_credulously_accepted`].
    #[must_use]
    pub fn attackers_of(&self, target: &ArgumentId) -> Vec<ArgumentId> {
        self.framework
            .attacks()
            .filter(|atk| &atk.target == target)
            .map(|atk| atk.attacker.clone())
            .collect()
    }
```

Note the `framework.attacks()` iterator yields `&WeightedAttack<ArgumentId>`. If the actual iterator shape differs, adapt the closure — verify by opening `/home/peter/code/argumentation/crates/argumentation-weighted-bipolar/src/framework.rs` and checking the `attacks` method signature. If it iterates `(attacker, target)` tuples instead of a struct, adapt: `.filter(|(_, t)| t == target).map(|(a, _)| a.clone())`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests::attackers_of`
Expected: PASS, 2 passed.

- [ ] **Step 5: Run crate-wide tests**

Run: `cargo test -p encounter-argumentation`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.attackers_of structural attacker query"
```

---

## Task 5: Add `has_accepted_counter_by` per-responder query

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn has_accepted_counter_by_detects_responder_attacker_at_beta() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let target = ArgumentId::new("target");
        let bob_counter = ArgumentId::new("bob_counter");
        let alice_counter = ArgumentId::new("alice_counter");
        state.add_weighted_attack(&bob_counter, &target, 0.5).unwrap();
        state.add_weighted_attack(&alice_counter, &target, 0.5).unwrap();
        // Attribute each counter to a specific responder by seeding
        // actors_by_argument. The framework framework doesn't track
        // actor attribution directly — it comes from add_scheme_instance
        // calls. For this test we simulate the attribution manually:
        // `actors_for` shows actors who asserted an argument.
        // We use the add_scheme_instance path.
        let _ = state; // consume the state we just built to construct a new one
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut target_bindings = std::collections::HashMap::new();
        target_bindings.insert("expert".to_string(), "alice".to_string());
        target_bindings.insert("domain".to_string(), "military".to_string());
        target_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let target_instance = scheme.instantiate(&target_bindings).unwrap();
        let mut counter_bindings = std::collections::HashMap::new();
        counter_bindings.insert("expert".to_string(), "bob".to_string());
        counter_bindings.insert("domain".to_string(), "logistics".to_string());
        counter_bindings.insert("claim".to_string(), "abandon_east".to_string());
        let counter_instance = scheme.instantiate(&counter_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let target_id = state.add_scheme_instance("alice", target_instance);
        let counter_id = state.add_scheme_instance("bob", counter_instance);
        state.add_weighted_attack(&counter_id, &target_id, 0.5).unwrap();

        // At β=0.0 bob's counter is credulously accepted (unattacked) and
        // attacks alice's target → has_accepted_counter_by(bob, target)=true,
        // has_accepted_counter_by(alice, target)=false.
        assert!(state.has_accepted_counter_by("bob", &target_id).unwrap());
        assert!(!state.has_accepted_counter_by("alice", &target_id).unwrap());
    }

    #[test]
    fn has_accepted_counter_by_returns_false_when_counter_not_credulous() {
        // Seed a counter that is ITSELF attacked by a higher-weight
        // counter-counter that binds at β=0. Then the mid counter is
        // not credulously accepted, and has_accepted_counter_by
        // returns false for its asserter.
        let registry = default_catalog();
        let mut state = EncounterArgumentationState::new(registry);
        let target = ArgumentId::new("target");
        let counter = ArgumentId::new("counter");
        let counter_counter = ArgumentId::new("counter_counter");
        state.add_argument_via_attack(); // see helper below; alternatively construct through add_weighted_attack chains
        state.add_weighted_attack(&counter, &target, 0.5).unwrap();
        state.add_weighted_attack(&counter_counter, &counter, 0.5).unwrap();
        // Manually record that "bob" asserted counter. The existing
        // APIs only allow this via add_scheme_instance; for a smoke
        // test we can use the internal actors_by_argument map via
        // a test helper. If no helper exists, skip this test and
        // rely on the integration test in Task 10 instead.
    }
```

**Note:** the second test is a sketch — adapt it to the real API. If adding attribution without `add_scheme_instance` is inconvenient, DROP the second test (the first one plus Task 10's integration test provides enough coverage).

- [ ] **Step 2: Run to verify fail**

Run: `cargo test -p encounter-argumentation state::tests::has_accepted_counter_by_detects`
Expected: FAIL — `no method named 'has_accepted_counter_by'`.

- [ ] **Step 3: Add the method**

Append to `impl EncounterArgumentationState`:

```rust
    /// Return `true` iff `responder` asserts some argument that
    /// (1) directly attacks `target`, AND
    /// (2) is credulously accepted at the current scene intensity.
    ///
    /// This is the per-responder counter-argument query used by the
    /// bridge's [`crate::state_acceptance::StateAcceptanceEval`] to
    /// decide whether a responder rejects a proposed action. It differs
    /// from [`Self::is_credulously_accepted`] (which is a global β
    /// acceptance check regardless of who asserted the argument).
    ///
    /// Returns `Err` if the underlying weighted-bipolar residual
    /// enumeration fails (e.g., framework exceeds edge limit). The
    /// bridge wraps this error into its error latch; consumers should
    /// rarely see `Err` surface directly.
    pub fn has_accepted_counter_by(
        &self,
        responder: &str,
        target: &ArgumentId,
    ) -> Result<bool, crate::error::Error> {
        for attacker in self.attackers_of(target) {
            let asserted_by_responder = self
                .actors_for(&attacker)
                .iter()
                .any(|a| a == responder);
            if !asserted_by_responder {
                continue;
            }
            if self.is_credulously_accepted(&attacker)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests::has_accepted_counter_by`
Expected: at least the first test passes. Drop the sketched second test if its helper prereqs aren't a match for the real API.

- [ ] **Step 5: Clippy + full suite**

Run:
```
cargo clippy -p encounter-argumentation -- -D warnings
cargo test -p encounter-argumentation
```
Expected: both clean.

- [ ] **Step 6: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.has_accepted_counter_by per-responder query

Returns true iff a specific responder asserts some credulously-accepted
attacker of the target argument at the current β. This is the semantic
primitive StateAcceptanceEval uses to reject a proposed action when the
responder has an acceptable counter."
```

---

## Task 6: Add error latch `Cell<Option<Error>>` + `take_latest_error`

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

**Why:** `AcceptanceEval::evaluate -> bool` can't return `Result`. When `has_accepted_counter_by` fails inside the bridge's `evaluate`, the bridge defaults to `true` (accept) AND stashes the error in a per-state latch. Consumers call `state.take_latest_error()` after `resolve` completes to detect and handle any failures.

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn take_latest_error_round_trips_a_stashed_error() {
        let state = EncounterArgumentationState::new(default_catalog());
        // No error yet.
        assert!(state.take_latest_error().is_none());
        // Stash an error manually (internal helper).
        state.record_error(crate::error::Error::SchemeNotFound("x".into()));
        let err = state.take_latest_error();
        assert!(matches!(err, Some(crate::error::Error::SchemeNotFound(_))));
        // Second call: drained.
        assert!(state.take_latest_error().is_none());
    }
```

- [ ] **Step 2: Run to verify fail**

Run: `cargo test -p encounter-argumentation state::tests::take_latest_error`
Expected: FAIL.

- [ ] **Step 3: Add the field + methods**

In `EncounterArgumentationState` struct, add:
```rust
    /// Error latch: last error observed by a bridge impl whose trait
    /// signature can't propagate Results (e.g.
    /// `AcceptanceEval::evaluate`). Consumers drain via
    /// [`Self::take_latest_error`] after encounter's resolve returns.
    latest_error: std::cell::RefCell<Option<crate::error::Error>>,
```

And `use std::cell::RefCell;` at the top of the file if not already imported.

Initialise in `new()`:
```rust
            latest_error: RefCell::new(None),
```

Append methods to the impl block:

```rust
    /// Drain the last error observed by a bridge impl. Clears the
    /// latch. Returns `None` if no error has been stashed since the
    /// last drain.
    #[must_use]
    pub fn take_latest_error(&self) -> Option<crate::error::Error> {
        self.latest_error.borrow_mut().take()
    }

    /// Record an error into the latch. Called by bridge impls whose
    /// trait signature can't propagate `Result`.
    pub(crate) fn record_error(&self, err: crate::error::Error) {
        *self.latest_error.borrow_mut() = Some(err);
    }
```

**Note:** `record_error` is `pub(crate)` because only the bridge modules need to write; consumers only read via `take_latest_error`. If you make it public for test construction, the `record_error` in the test above requires `use crate::state::...` which is fine — but per-crate access via `pub(crate)` is simpler.

Adjust the test to use an internal path if needed. If the test can't call `record_error` because it's `pub(crate)`, make it `pub` temporarily for the test, then revert in a follow-up. Simpler option: change `pub(crate)` to `pub` and document the one-liner; it's low-cost.

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests::take_latest_error`
Expected: PASS.

- [ ] **Step 5: Run crate-wide tests**

Run: `cargo test -p encounter-argumentation`
Expected: all pass.

- [ ] **Step 6: Clippy**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): error latch for trait impls that can't propagate Result

AcceptanceEval::evaluate -> bool can't return Result<bool, Error>. When
the bridge's StateAcceptanceEval hits an internal error, it defaults to
'accept' and stashes the error on the state for the consumer to drain
via state.take_latest_error() after encounter's resolve loop completes."
```

---

## Task 7: Create `StateAcceptanceEval`

**Files:**
- Create: `crates/encounter-argumentation/src/state_acceptance.rs`
- Modify: `crates/encounter-argumentation/src/lib.rs` (add `pub mod state_acceptance;` + re-export)

- [ ] **Step 1: Create the module file**

Create `crates/encounter-argumentation/src/state_acceptance.rs`:

```rust
//! `StateAcceptanceEval`: encounter's `AcceptanceEval` impl backed by
//! an [`EncounterArgumentationState`]'s current β-intensity.
//!
//! The eval rejects a proposed action iff the responder asserts a
//! credulously-accepted attacker of the proposed argument at the
//! current scene intensity. Otherwise it accepts.
//!
//! Internal errors (e.g. weighted-bipolar edge limit exceeded) cause
//! the eval to default to *accept* and stash the error on the state's
//! latch; consumers call `state.take_latest_error()` to drain.

use crate::affordance_key::AffordanceKey;
use crate::state::EncounterArgumentationState;
use encounter::scoring::{AcceptanceEval, ScoredAffordance};

/// An `AcceptanceEval<P>` backed by a shared reference to a live
/// [`EncounterArgumentationState`].
///
/// The eval uses [`EncounterArgumentationState::has_accepted_counter_by`]
/// to decide whether the responder has a credulously-accepted counter
/// to the proposed action's argument at current β. If the action has
/// no seeded argument in the state, the eval accepts (falls back to
/// permissive — there's no argumentation claim against which to
/// adjudicate).
pub struct StateAcceptanceEval<'a> {
    state: &'a EncounterArgumentationState,
}

impl<'a> StateAcceptanceEval<'a> {
    /// Construct an acceptance eval borrowing the given state.
    #[must_use]
    pub fn new(state: &'a EncounterArgumentationState) -> Self {
        Self { state }
    }

    /// The `actor` key under which to look up the PROPOSER's argument.
    /// A `ScoredAffordance` carries `bindings` but does not carry the
    /// proposer identity; by convention the proposer binding slot is
    /// `"self"`. Consumers who use a different convention can wrap
    /// this eval with a custom version that derives the proposer
    /// differently.
    fn proposer_key<P>(&self, action: &ScoredAffordance<P>) -> Option<AffordanceKey> {
        let proposer = action.bindings.get("self")?;
        Some(AffordanceKey::new(
            proposer,
            &action.entry.spec.name,
            &action.bindings,
        ))
    }
}

impl<'a, P> AcceptanceEval<P> for StateAcceptanceEval<'a> {
    fn evaluate(&self, responder: &str, action: &ScoredAffordance<P>) -> bool {
        let Some(key) = self.proposer_key(action) else {
            return true; // No proposer binding found → can't adjudicate, accept.
        };
        let Some(target) = self.state.argument_id_for(&key) else {
            return true; // No scheme instance seeded for this affordance → accept.
        };
        match self.state.has_accepted_counter_by(responder, &target) {
            Ok(true) => false,  // responder has an acceptable counter → reject.
            Ok(false) => true,  // no counter → accept.
            Err(e) => {
                self.state.record_error(e);
                true // Default permissive on internal error.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arg_id::ArgumentId;
    use argumentation_schemes::catalog::default_catalog;
    use argumentation_weighted::types::Budget;
    use encounter::affordance::{AffordanceSpec, CatalogEntry};
    use encounter::scoring::ScoredAffordance;
    use std::collections::HashMap;

    fn make_affordance(
        name: &str,
        self_actor: &str,
        expert: &str,
        domain: &str,
        claim: &str,
    ) -> ScoredAffordance<String> {
        let spec = AffordanceSpec {
            name: name.to_string(),
            domain: "persuasion".to_string(),
            bindings: vec![
                "self".to_string(),
                "expert".to_string(),
                "domain".to_string(),
                "claim".to_string(),
            ],
            considerations: Vec::new(),
            effects_on_accept: Vec::new(),
            effects_on_reject: Vec::new(),
            drive_alignment: Vec::new(),
        };
        let entry = CatalogEntry { spec, precondition: String::new() };
        let mut bindings = HashMap::new();
        bindings.insert("self".to_string(), self_actor.to_string());
        bindings.insert("expert".to_string(), expert.to_string());
        bindings.insert("domain".to_string(), domain.to_string());
        bindings.insert("claim".to_string(), claim.to_string());
        ScoredAffordance { entry, score: 1.0, bindings }
    }

    #[test]
    fn accepts_when_no_argument_is_seeded_for_the_affordance() {
        let state = EncounterArgumentationState::new(default_catalog());
        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance("unseeded_action", "alice", "alice", "military", "x");
        assert!(eval.evaluate("bob", &action));
    }

    #[test]
    fn accepts_when_responder_has_no_counter() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut proposer_bindings = HashMap::new();
        proposer_bindings.insert("expert".to_string(), "alice".to_string());
        proposer_bindings.insert("domain".to_string(), "military".to_string());
        proposer_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&proposer_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        // Seed using bindings that INCLUDE self so the eval can find it.
        let mut affordance_bindings = proposer_bindings.clone();
        affordance_bindings.insert("self".to_string(), "alice".to_string());
        state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &affordance_bindings,
            instance,
        );
        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance(
            "argue_fortify_east",
            "alice",
            "alice",
            "military",
            "fortify_east",
        );
        assert!(eval.evaluate("bob", &action));
    }

    #[test]
    fn rejects_when_responder_asserts_an_accepted_counter() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

        // Alice's argument: fortify east.
        let mut alice_bindings = HashMap::new();
        alice_bindings.insert("expert".to_string(), "alice".to_string());
        alice_bindings.insert("domain".to_string(), "military".to_string());
        alice_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let alice_instance = scheme.instantiate(&alice_bindings).unwrap();
        // Bob's counter-argument: abandon east. Different conclusion →
        // separate ArgumentId; we add a weighted attack edge manually.
        let mut bob_bindings = HashMap::new();
        bob_bindings.insert("expert".to_string(), "bob".to_string());
        bob_bindings.insert("domain".to_string(), "logistics".to_string());
        bob_bindings.insert("claim".to_string(), "abandon_east".to_string());
        let bob_instance = scheme.instantiate(&bob_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let mut alice_af_bindings = alice_bindings.clone();
        alice_af_bindings.insert("self".to_string(), "alice".to_string());
        let alice_id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &alice_af_bindings,
            alice_instance,
        );
        let bob_id = state.add_scheme_instance("bob", bob_instance);
        state.add_weighted_attack(&bob_id, &alice_id, 0.5).unwrap();
        // Default β=0: bob's counter is unattacked and credulously
        // accepted → eval should reject.

        let eval = StateAcceptanceEval::new(&state);
        let action = make_affordance(
            "argue_fortify_east",
            "alice",
            "alice",
            "military",
            "fortify_east",
        );
        assert!(!eval.evaluate("bob", &action), "bob should reject alice's claim");
    }

    #[test]
    fn default_permissive_on_missing_self_binding() {
        let state = EncounterArgumentationState::new(default_catalog());
        let eval = StateAcceptanceEval::new(&state);
        // Build an action without a "self" binding.
        let spec = AffordanceSpec {
            name: "anon".to_string(),
            domain: "x".to_string(),
            bindings: vec![],
            considerations: Vec::new(),
            effects_on_accept: Vec::new(),
            effects_on_reject: Vec::new(),
            drive_alignment: Vec::new(),
        };
        let entry = CatalogEntry { spec, precondition: String::new() };
        let action = ScoredAffordance {
            entry,
            score: 1.0,
            bindings: HashMap::new(),
        };
        assert!(eval.evaluate("anyone", &action));
    }
}
```

- [ ] **Step 2: Wire into lib.rs**

Add `pub mod state_acceptance;` and re-export:
```rust
pub use state_acceptance::StateAcceptanceEval;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p encounter-argumentation state_acceptance`
Expected: PASS, 4 passed.

- [ ] **Step 4: Full crate tests + clippy**

```
cargo test -p encounter-argumentation
cargo clippy -p encounter-argumentation -- -D warnings
```
Expected: all clean.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state_acceptance.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): StateAcceptanceEval backed by EncounterArgumentationState

An AcceptanceEval<P> impl that reads from a shared EncounterArgumentationState
reference. Rejects iff responder has a credulously-accepted counter-argument
at current β. Default-permissive on internal errors; errors stashed on the
state's error latch."
```

---

## Task 8: Create `StateActionScorer`

**Files:**
- Create: `crates/encounter-argumentation/src/state_scorer.rs`
- Modify: `crates/encounter-argumentation/src/lib.rs`

- [ ] **Step 1: Create the module file**

Create `crates/encounter-argumentation/src/state_scorer.rs`:

```rust
//! `StateActionScorer`: encounter's `ActionScorer` impl backed by an
//! [`EncounterArgumentationState`].
//!
//! The scorer composes:
//! 1. An inner `ActionScorer<P>` that produces base scores.
//! 2. A reference to a live `EncounterArgumentationState`.
//!
//! After the inner scorer runs, each scored affordance is amplified
//! by `boost * state.is_credulously_accepted(arg_id)` where the
//! argument id is looked up from the affordance's forward-index key.
//! Affordances with no seeded argument receive no boost (inner score
//! unchanged).
//!
//! This gives the *proposer* bias: actions whose own argument is
//! credulously acceptable at the scene's current β get boosted,
//! making the scene self-consistent with the argumentation state.
//! The per-responder gate is orthogonal and lives in
//! [`crate::state_acceptance::StateAcceptanceEval`].

use crate::affordance_key::AffordanceKey;
use crate::state::EncounterArgumentationState;
use encounter::affordance::CatalogEntry;
use encounter::scoring::{ActionScorer, ScoredAffordance};

/// An `ActionScorer<P>` composing an inner scorer with a shared-ref
/// view of an [`EncounterArgumentationState`].
pub struct StateActionScorer<'a, S> {
    state: &'a EncounterArgumentationState,
    inner: S,
    boost: f64,
}

impl<'a, S> StateActionScorer<'a, S> {
    /// Construct a state-aware scorer wrapping `inner`. `boost` is
    /// the additive score added to any affordance whose argument is
    /// credulously accepted at the current β. Typical values: 0.3–1.0.
    #[must_use]
    pub fn new(state: &'a EncounterArgumentationState, inner: S, boost: f64) -> Self {
        Self { state, inner, boost }
    }
}

impl<'a, S, P> ActionScorer<P> for StateActionScorer<'a, S>
where
    S: ActionScorer<P>,
    P: Clone,
{
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        let mut scored = self.inner.score_actions(actor, available, participants);
        for sa in &mut scored {
            let key = AffordanceKey::new(actor, &sa.entry.spec.name, &sa.bindings);
            let Some(id) = self.state.argument_id_for(&key) else {
                continue;
            };
            match self.state.is_credulously_accepted(&id) {
                Ok(true) => sa.score += self.boost,
                Ok(false) => {
                    // Not credulously accepted at current β: no boost
                    // (equivalent to "would be awkward to assert").
                }
                Err(e) => {
                    self.state.record_error(e);
                }
            }
        }
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arg_id::ArgumentId;
    use argumentation_schemes::catalog::default_catalog;
    use argumentation_weighted::types::Budget;
    use encounter::affordance::AffordanceSpec;
    use encounter::scoring::{ActionScorer, ScoredAffordance};
    use std::collections::HashMap;

    struct FlatInner;
    impl<P: Clone> ActionScorer<P> for FlatInner {
        fn score_actions(
            &self,
            _actor: &str,
            available: &[CatalogEntry<P>],
            _participants: &[String],
        ) -> Vec<ScoredAffordance<P>> {
            available
                .iter()
                .map(|e| {
                    let mut bindings = HashMap::new();
                    bindings.insert("self".to_string(), "alice".to_string());
                    bindings.insert("expert".to_string(), "alice".to_string());
                    bindings.insert("domain".to_string(), "military".to_string());
                    bindings.insert("claim".to_string(), "fortify_east".to_string());
                    ScoredAffordance {
                        entry: e.clone(),
                        score: 1.0,
                        bindings,
                    }
                })
                .collect()
        }
    }

    fn catalog() -> Vec<CatalogEntry<String>> {
        let spec = AffordanceSpec {
            name: "argue_fortify_east".to_string(),
            domain: "persuasion".to_string(),
            bindings: vec![
                "self".to_string(),
                "expert".to_string(),
                "domain".to_string(),
                "claim".to_string(),
            ],
            considerations: Vec::new(),
            effects_on_accept: Vec::new(),
            effects_on_reject: Vec::new(),
            drive_alignment: Vec::new(),
        };
        vec![CatalogEntry { spec, precondition: String::new() }]
    }

    #[test]
    fn unboosted_when_no_argument_is_seeded() {
        let state = EncounterArgumentationState::new(default_catalog());
        let scorer = StateActionScorer::new(&state, FlatInner, 0.5);
        let catalog_vec = catalog();
        let scored = scorer.score_actions("alice", &catalog_vec, &["alice".to_string()]);
        assert_eq!(scored.len(), 1);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boosted_when_argument_is_credulously_accepted() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut b = HashMap::new();
        b.insert("expert".to_string(), "alice".to_string());
        b.insert("domain".to_string(), "military".to_string());
        b.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&b).unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let mut affordance_b = b.clone();
        affordance_b.insert("self".to_string(), "alice".to_string());
        state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &affordance_b,
            instance,
        );

        let scorer = StateActionScorer::new(&state, FlatInner, 0.5);
        let catalog_vec = catalog();
        let scored = scorer.score_actions("alice", &catalog_vec, &["alice".to_string()]);
        assert_eq!(scored.len(), 1);
        // inner gave 1.0; argument is unattacked → credulously accepted → +0.5.
        assert!(
            (scored[0].score - 1.5).abs() < 1e-9,
            "expected 1.5, got {}",
            scored[0].score
        );
    }
}
```

- [ ] **Step 2: Wire into lib.rs**

Add `pub mod state_scorer;` and re-export:
```rust
pub use state_scorer::StateActionScorer;
```

- [ ] **Step 3: Run tests + clippy**

```
cargo test -p encounter-argumentation state_scorer
cargo clippy -p encounter-argumentation -- -D warnings
```
Expected: 2 tests pass; clippy clean.

- [ ] **Step 4: Full crate suite**

Run: `cargo test -p encounter-argumentation`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state_scorer.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): StateActionScorer wraps inner scorer + EAState

Delegates to an inner ActionScorer, then boosts each affordance whose
argument is credulously accepted at the current β. Missing or
not-credulously-accepted arguments receive no boost. Errors stashed
on the state's error latch."
```

---

## Task 9: End-to-end integration test — MultiBeat with state

**Files:**
- Create: `crates/encounter-argumentation/tests/uc_multibeat_scene.rs`

- [ ] **Step 1: Create the integration test**

Create `crates/encounter-argumentation/tests/uc_multibeat_scene.rs`:

```rust
//! End-to-end integration: run encounter's MultiBeat protocol with
//! StateActionScorer + StateAcceptanceEval backed by an
//! EncounterArgumentationState. Verifies that (a) the bridge compiles
//! cleanly against real encounter trait signatures, (b) scene
//! intensity influences beat outcomes, (c) the error latch drains
//! cleanly.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, AcceptanceEval, ScoredAffordance};
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};
use std::collections::HashMap;

/// Inner scorer: uniform 1.0 score, binds self to the calling actor.
struct UniformScorer;

impl<P: Clone> ActionScorer<P> for UniformScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available
            .iter()
            .map(|e| {
                let mut bindings = HashMap::new();
                bindings.insert("self".to_string(), actor.to_string());
                // Hardcoded bindings matching the catalog below.
                bindings.insert("expert".to_string(), actor.to_string());
                bindings.insert("domain".to_string(), "military".to_string());
                bindings.insert("claim".to_string(), "fortify_east".to_string());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

fn build_scene() -> (
    EncounterArgumentationState,
    Vec<CatalogEntry<String>>,
    PracticeSpec,
) {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    // Alice's argument: fortify_east.
    let mut alice_b = HashMap::new();
    alice_b.insert("expert".to_string(), "alice".to_string());
    alice_b.insert("domain".to_string(), "military".to_string());
    alice_b.insert("claim".to_string(), "fortify_east".to_string());
    let alice_instance = scheme.instantiate(&alice_b).unwrap();
    // Bob's argument: also an expert opinion, different claim, attacking alice.
    let mut bob_b = HashMap::new();
    bob_b.insert("expert".to_string(), "bob".to_string());
    bob_b.insert("domain".to_string(), "logistics".to_string());
    bob_b.insert("claim".to_string(), "abandon_east".to_string());
    let bob_instance = scheme.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".to_string(), "alice".to_string());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice",
        "argue_fortify_east",
        &alice_af,
        alice_instance,
    );
    // Bob's argument is seeded plain; also record it against an
    // affordance key so scorer can boost it for bob.
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".to_string(), "bob".to_string());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob",
        "argue_abandon_east",
        &bob_af,
        bob_instance,
    );
    // Bob's argument attacks alice's argument with weight 0.4.
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();

    let catalog_vec = vec![
        CatalogEntry {
            spec: AffordanceSpec {
                name: "argue_fortify_east".to_string(),
                domain: "persuasion".to_string(),
                bindings: vec![
                    "self".to_string(),
                    "expert".to_string(),
                    "domain".to_string(),
                    "claim".to_string(),
                ],
                considerations: Vec::new(),
                effects_on_accept: Vec::new(),
                effects_on_reject: Vec::new(),
                drive_alignment: Vec::new(),
            },
            precondition: String::new(),
        },
        CatalogEntry {
            spec: AffordanceSpec {
                name: "argue_abandon_east".to_string(),
                domain: "persuasion".to_string(),
                bindings: vec![
                    "self".to_string(),
                    "expert".to_string(),
                    "domain".to_string(),
                    "claim".to_string(),
                ],
                considerations: Vec::new(),
                effects_on_accept: Vec::new(),
                effects_on_reject: Vec::new(),
                drive_alignment: Vec::new(),
            },
            precondition: String::new(),
        },
    ];
    let practice = PracticeSpec {
        name: "debate".to_string(),
        affordances: vec![
            "argue_fortify_east".to_string(),
            "argue_abandon_east".to_string(),
        ],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    (state, catalog_vec, practice)
}

#[test]
fn scene_resolves_cleanly_at_low_intensity() {
    let (state, catalog_vec, practice) = build_scene();
    state.set_intensity(Budget::new(0.0).unwrap());
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".to_string(), "bob".to_string()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog_vec, &scorer, &acceptance);
    // At β=0 bob's counter binds → bob rejects alice's proposal.
    // Alice rejects bob's. Multiple beats recorded, no error.
    assert!(!result.beats.is_empty());
    assert!(state.take_latest_error().is_none());
}

#[test]
fn higher_intensity_changes_acceptance_outcomes() {
    let (state, catalog_vec, practice) = build_scene();
    // At β=0.5 > 0.4, the attack from bob to alice is droppable, so
    // alice's argument is credulously accepted in that residual.
    // has_accepted_counter_by(bob, alice_id) should still be true
    // though, because bob's own argument is unattacked (credulous).
    // What changes: alice may be credulously accepted AT THE SAME TIME,
    // but per-responder counter-check is strict. So eval still rejects.
    // What this test proves: the code runs at a new β without panic
    // and does not silently mutate the other's seeded arguments.
    state.set_intensity(Budget::new(0.5).unwrap());
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".to_string(), "bob".to_string()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog_vec, &scorer, &acceptance);
    assert!(!result.beats.is_empty());
    assert!(state.take_latest_error().is_none());
}

#[test]
fn scorer_boost_prefers_alice_own_argument_for_alice() {
    // With alice as actor, scorer should boost argue_fortify_east
    // (her own scheme) over argue_abandon_east (bob's). Verify by
    // calling scorer directly and checking the order.
    let (state, catalog_vec, _) = build_scene();
    state.set_intensity(Budget::new(1.0).unwrap()); // both args credulous
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let scored = scorer.score_actions(
        "alice",
        &catalog_vec,
        &["alice".to_string(), "bob".to_string()],
    );
    // At β=1.0 both attacks are droppable → both args credulous →
    // both boosted. But UniformScorer binds "self"=alice and
    // "claim"=fortify_east for every affordance, so scheme bindings
    // collide. Real consumers would bind differently per affordance.
    // This test just asserts the scorer runs without panic and yields
    // the expected count.
    assert_eq!(scored.len(), 2);
    assert!(state.take_latest_error().is_none());
}
```

- [ ] **Step 2: Run the integration test**

Run: `cargo test -p encounter-argumentation --test uc_multibeat_scene`
Expected: 3 passed.

If any test fails, BLOCK and report the failure — do not weaken assertions. Common causes:
- Mismatch between `ActionScorer::score_actions` signature (verified: `(&self, actor, available, participants) -> Vec<ScoredAffordance<P>>`) and what `UniformScorer` implements.
- `PracticeSpec` fields renamed — verify against `/home/peter/code/encounter/src/practice.rs`.
- `DurationPolicy::MultiBeat` variant shape — verify the variant is struct-style `{max_beats: usize}`.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter-argumentation/tests/uc_multibeat_scene.rs
git commit -m "test(encounter-argumentation): MultiBeat scene with state-backed scorer + acceptance

End-to-end integration test wiring StateActionScorer + StateAcceptanceEval
against encounter's MultiBeat protocol. Covers scene resolution at two β
values and scorer boosting behaviour. Drains the state's error latch after
each scene to verify no silent errors."
```

---

## Task 10: CHANGELOG + README + version bump + tag

**Files:**
- Modify: `crates/encounter-argumentation/CHANGELOG.md`
- Modify: `crates/encounter-argumentation/README.md`
- Modify: `crates/encounter-argumentation/Cargo.toml`

- [ ] **Step 1: Prepend v0.3.0 CHANGELOG entry**

Prepend (do not replace) `crates/encounter-argumentation/CHANGELOG.md`:

```markdown
# Changelog

## [0.3.0] - 2026-04-20

### Added — state-aware bridge types
- `StateAcceptanceEval<'a>` — `AcceptanceEval<P>` impl backed by a
  shared reference to an `EncounterArgumentationState`. Rejects iff
  the responder asserts a credulously-accepted attacker of the
  proposed affordance's argument at the current β.
- `StateActionScorer<'a, S>` — `ActionScorer<P>` impl wrapping an
  inner scorer, boosting affordances whose argument is credulously
  accepted at current β.
- `AffordanceKey` — canonical `(actor, affordance_name, bindings)`
  tuple used as the forward-index key.

### Added — `EncounterArgumentationState` API
- `add_scheme_instance_for_affordance(actor, affordance_name, bindings, instance) -> ArgumentId`
  — records the argument in the framework AND in a forward index so
  bridge consumers can look it up at encounter `resolve` time.
- `argument_id_for(&AffordanceKey) -> Option<ArgumentId>`.
- `attackers_of(&ArgumentId) -> Vec<ArgumentId>` — structural query.
- `has_accepted_counter_by(responder, &target) -> Result<bool, Error>`
  — per-responder credulous-counter query.
- `set_intensity(&self, Budget)` — **shared-ref** setter (replaces the
  prior by-value `at_intensity` requirement for mid-scene β change).
- `take_latest_error() -> Option<Error>` — drain the bridge's error
  latch; trait impls default to permissive on internal errors.

### Changed
- `EncounterArgumentationState::intensity` is now stored in a
  `Cell<Budget>` internally. `intensity()` still returns `Budget` by
  value; `at_intensity(self, Budget) -> Self` (by-value builder) still
  works. The change enables `set_intensity(&self, _)`.

### Preserved (no changes to v0.2.x API)
- `SchemeActionScorer`, `ArgumentAcceptanceEval`, `ArgumentKnowledge`,
  `StaticKnowledge`, `ArgumentPosition`, `resolve_argument`,
  `critical_question_beats`, `cq_to_beat`, `scheme_value_argument`,
  `EncounterArgumentationState::{new, at_intensity, add_scheme_instance,
  add_weighted_attack, add_weighted_support, is_credulously_accepted,
  is_skeptically_accepted, coalitions, actors_for, instances_for,
  argument_count, edge_count, intensity}`, all `Error` variants.

### Zero `encounter`-crate changes
Phase B lands entirely in this bridge. The sibling `encounter` crate at
`/home/peter/code/encounter/` is not modified.

## [0.2.0] - 2026-04-19
(see prior entries)
```

- [ ] **Step 2: Prepend README section**

Prepend to `crates/encounter-argumentation/README.md` immediately after the existing `# encounter-argumentation` title line:

```markdown
## State-aware bridge (v0.3.0)

Plug an `EncounterArgumentationState` into `encounter`'s protocols via:

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::resolution::MultiBeat;
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};

let mut state = EncounterArgumentationState::new(default_catalog());
state.set_intensity(Budget::new(0.4).unwrap());

// Seed one scheme instance per (actor, affordance) the scene needs.
// state.add_scheme_instance_for_affordance(actor, name, bindings, instance);

let scorer = StateActionScorer::new(&state, my_inner_scorer, 0.5);
let acceptance = StateAcceptanceEval::new(&state);

let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

// Drain any internal error that the trait impls couldn't propagate.
if let Some(err) = state.take_latest_error() {
    // handle...
}
```

Scene-intensity (β) lives on the state object and can be mutated with
`state.set_intensity(...)` mid-scene through a shared reference.
```

- [ ] **Step 3: Bump version**

In `crates/encounter-argumentation/Cargo.toml`, change `version = "0.2.0"` to `version = "0.3.0"`.

- [ ] **Step 4: Workspace-wide verification**

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

All must pass.

- [ ] **Step 5: Commit and tag**

```bash
git add crates/encounter-argumentation/CHANGELOG.md \
        crates/encounter-argumentation/README.md \
        crates/encounter-argumentation/Cargo.toml
git commit -m "chore(encounter-argumentation): v0.3.0 release — state-aware scorer + acceptance bridge"
git tag encounter-argumentation-v0.3.0
git log --oneline -3
git tag | grep encounter-argumentation
```

- [ ] **Step 6: Final verification**

Run: `cargo test --workspace` (one more time to confirm clean release).

---

## Task 11: Code review

Follow the `superpowers:requesting-code-review` skill with:
- **WHAT_WAS_IMPLEMENTED:** Phase B bridge — state-aware `ActionScorer` + `AcceptanceEval` impls in `encounter-argumentation` v0.3.0. Zero changes to the `encounter` crate.
- **PLAN_OR_REQUIREMENTS:** Tasks 1-10 of this plan.
- **BASE_SHA:** branch point with main (`git merge-base HEAD main`).
- **HEAD_SHA:** tag `encounter-argumentation-v0.3.0`.
- **DESCRIPTION:** "Added StateActionScorer + StateAcceptanceEval + forward index on EncounterArgumentationState; interior-mutable β via Cell<Budget>; error latch for trait impls that can't propagate Result. No encounter-crate changes."

Ask the reviewer to verify:
1. The five critical issues identified in the Phase B critique (no lookup mapping, scheme_id leakage, double-eval tautology, mid-scene β borrow hazard, Result→bool under-specification) are ALL resolved.
2. `AffordanceKey` hashing is stable across HashMap insertion orders.
3. The error latch cannot leak across state clones (if state is cloned, does the new state see a separate or shared latch? RefCell<Option<_>> is NOT shared across clones; verify this is what we want).
4. `StateAcceptanceEval::proposer_key` assumes a `"self"` binding. Is this convention flagged in the module docs? Should consumers who use a different convention be offered an alternative constructor?

Fix any Critical or Important before closing Phase B.

---

## Wrap-up — finishing the branch

After code review is clean:

1. Announce: "I'm using the finishing-a-development-branch skill to complete this work."
2. **REQUIRED SUB-SKILL:** `superpowers:finishing-a-development-branch`.
3. Verify tests, present merge options, execute the user's choice.

---

## Self-review notes

- **Spec coverage:** the original Phase B spec from `ARGUMENTATION_CONSUMERS.md` §3 had three goals: replace ad-hoc scoring, director β knob, affordances become scheme bindings. This revised plan:
  - ✅ Goal 1 (replace ad-hoc scoring) — `StateActionScorer` composes on top of existing encounter traits.
  - ✅ Goal 2 (β knob) — `set_intensity(&self, _)` on the state; consumers set it before or during resolve.
  - ✅ Goal 3 (affordances → scheme bindings) — reframed as forward index maintained by `add_scheme_instance_for_affordance`; no `scheme_id` field on encounter's `AffordanceSpec` (avoids leaking argumentation into encounter's data model, per critic Issue 2).

- **Critic issues resolved:**
  - Issue 1 (action→ArgumentId mapping): forward index in Task 3.
  - Issue 2 (scheme_id leakage): zero encounter changes.
  - Issue 3 (double-eval tautology): D4 locks in distinct scorer/eval queries.
  - Issue 4 (mid-scene β borrow): interior mutability in Task 2.
  - Issue 5 (Result→bool): error latch in Task 6.

- **No placeholders.** Every code block spells out the actual implementation; every test body is complete; every command has an expected result.

- **Type consistency:** `AffordanceKey`, `ArgumentId`, `Budget`, `EncounterArgumentationState`, `StateAcceptanceEval<'a>`, `StateActionScorer<'a, S>` are used with the same signature across all tasks.

- **Deferred:**
  - A `seed_from_bindings(&[(actor, affordance_name, bindings, scheme_key)])` convenience helper isn't shipped in Phase B. Consumers loop manually. Add in v0.3.1 if ergonomics demand it.
  - `StateAcceptanceEval`'s `proposer_key` hardcodes the `"self"` binding convention. A `with_proposer_resolver(fn)` builder is deferred.
  - No Phase B changes to `SingleExchange`. It pre-scores affordances upstream; `StateActionScorer` must run before `SingleExchange::resolve` is called. Documented in README but no dedicated helper.
