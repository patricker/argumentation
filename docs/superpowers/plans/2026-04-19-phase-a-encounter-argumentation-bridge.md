# Phase A: `encounter-argumentation` full-stack bridge upgrade

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade `encounter-argumentation` from a schemes-only bridge into a full-stack bridge that unifies `argumentation-schemes` + `argumentation-bipolar` + `argumentation-weighted` + `argumentation-weighted-bipolar` under one encounter-level state type.

**Architecture:** Add a new `EncounterArgumentationState` type that owns a `WeightedBipolarFramework<ArgumentId>` plus a scheme-instance registry plus an optional `WeightSource` plus a configurable scene-intensity `Budget`. Expose acceptance queries (credulous/skeptical at current intensity), coalition detection, and raw graph mutation on top. The existing v0.1.x API (`resolve_argument`, `ArgumentAcceptanceEval`, etc.) is preserved unchanged — Phase A is purely additive. Stub a `RelationshipWeightSource` that takes a placeholder `RelationshipSnapshot`; Phase C will later replace the snapshot type with real societas types.

**Tech Stack:** Rust 2024 edition, petgraph 0.6 (transitively via bipolar), thiserror 2.0. New in-workspace deps: `argumentation-bipolar`, `argumentation-weighted`, `argumentation-weighted-bipolar`.

---

## Pre-flight checklist for the executor

1. **You are working in** `/home/peter/code/argumentation/`. The workspace has 6 member crates; you only touch `crates/encounter-argumentation/`.
2. **Create a feature branch** before the first task: `git checkout -b feat/phase-a-encounter-arg-bridge`. Do NOT commit to `main` directly.
3. **The crate uses `#![deny(missing_docs)]`** — every `pub` item needs a doc comment.
4. **Preserve the existing API.** `resolve_argument`, `ArgumentAcceptanceEval`, `SchemeActionScorer`, `ArgumentKnowledge`, `StaticKnowledge`, `cq_to_beat`, `critical_question_beats`, `scheme_value_argument`, `Error`, `ArgumentOutcome`, `ArgumentPosition` — all keep working. Phase A adds on top; it does not replace.
5. **Every task ends with** `cargo test -p encounter-argumentation` (minimum) + `cargo clippy -p encounter-argumentation -- -D warnings`. If you change the crate's public API surface, also run `cargo test --workspace` and `cargo doc -p encounter-argumentation --no-deps` before committing.
6. **Read this memory first:**
   - `feedback_no_backward_compat.md` — in beta, rename freely where internal.
   - `feedback_cross_crate_deps.md` — depending on sibling argumentation crates is encouraged.
   - `feedback_never_commit_other_repos.md` — do NOT touch `/home/peter/code/encounter/`, `/home/peter/code/societas/`, or any other sibling repo.

## Design decisions locked in

**D1 — Scheme ↔ affordance coupling is loose.** `EncounterArgumentationState` stores (actor_name, ArgumentId) → SchemeInstance internally. `encounter::affordance::CatalogEntry` does NOT gain a `scheme_id` field. This defers vNEXT §7.2; Phase B can tighten if encounter wants it.

**D2 — Societas is stubbed.** `RelationshipSnapshot` is a placeholder struct owned by encounter-argumentation. Phase C replaces it with real societas types. No new cross-repo Cargo deps.

**D3 — `ArgumentId` is a newtype over `String`.** The string value is the scheme instance's conclusion literal rendered via `Literal::to_string()`. Two scheme instances with the same conclusion share one node — this is the correct convergence behaviour (both supporters reinforce the same claim).

**D4 — Existing API is preserved verbatim.** `resolve_argument` keeps its signature and semantics (pairwise, ASPIC+, Dung). Phase A's new state-based API is parallel, not a replacement.

**D5 — Intensity (β) is per-state, not per-scene.** vNEXT open question §5 partially deferred; one intensity per `EncounterArgumentationState` instance. Consumers with per-character intensity build multiple states. Revisit only if pinched.

---

## File structure

### New files

| File | Responsibility |
|---|---|
| `crates/encounter-argumentation/src/state.rs` | `EncounterArgumentationState` type + builders + graph mutators + acceptance queries + coalition query. Heart of the upgrade. |
| `crates/encounter-argumentation/src/arg_id.rs` | `ArgumentId(String)` newtype + `From<&Literal>` / `From<Literal>`. |
| `crates/encounter-argumentation/src/relationship.rs` | `RelationshipSnapshot` + `RelationshipDims` + `RelationshipWeightSource`. Stubbed for Phase A; replaceable in Phase C. |
| `crates/encounter-argumentation/tests/uc_parity.rs` | Integration test: new state API agrees with `resolve_argument` on UC1-style pairwise scenario. |
| `crates/encounter-argumentation/tests/uc_relationship_modulation.rs` | Integration test: same scheme, different relationships, different acceptance trajectories. |
| `crates/encounter-argumentation/tests/uc_intensity_sweep.rs` | Integration test: β sweep over a 3-character scene flips acceptance at expected points. |

### Modified files

| File | Why |
|---|---|
| `crates/encounter-argumentation/Cargo.toml` | Add three sibling deps. |
| `crates/encounter-argumentation/src/error.rs` | Add `#[from]` conversions for the three new crate errors. |
| `crates/encounter-argumentation/src/lib.rs` | Add `pub mod state`, `arg_id`, `relationship`; re-export new types. |
| `crates/encounter-argumentation/CHANGELOG.md` | v0.2.0 release entry. |
| `crates/encounter-argumentation/README.md` | Section describing the new state-object pattern. |

---

## Task 1: Set up feature branch + add deps

**Files:**
- Modify: `crates/encounter-argumentation/Cargo.toml`

- [ ] **Step 1: Create the feature branch**

Run:
```bash
cd /home/peter/code/argumentation
git checkout -b feat/phase-a-encounter-arg-bridge
git log --oneline -1
```
Expected: on branch `feat/phase-a-encounter-arg-bridge`, HEAD at the latest main commit.

- [ ] **Step 2: Add the three deps**

In `crates/encounter-argumentation/Cargo.toml`, find the `[dependencies]` block:

Before:
```toml
[dependencies]
encounter = { path = "../../../encounter" }
argumentation = { path = "../.." }
argumentation-schemes = { path = "../argumentation-schemes" }
thiserror = "2.0"
```

After:
```toml
[dependencies]
encounter = { path = "../../../encounter" }
argumentation = { path = "../.." }
argumentation-schemes = { path = "../argumentation-schemes" }
argumentation-bipolar = { path = "../argumentation-bipolar" }
argumentation-weighted = { path = "../argumentation-weighted" }
argumentation-weighted-bipolar = { path = "../argumentation-weighted-bipolar" }
thiserror = "2.0"
```

- [ ] **Step 3: Verify the workspace still builds**

Run: `cargo check -p encounter-argumentation 2>&1 | tail -5`
Expected: clean build (no compile errors; new deps are just available, not yet referenced in code).

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/Cargo.toml
git commit -m "deps(encounter-argumentation): add bipolar + weighted + wbipolar for Phase A bridge"
```

---

## Task 2: Extend `Error` with conversions for the new crates

**Files:**
- Modify: `crates/encounter-argumentation/src/error.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/encounter-argumentation/src/error.rs` (create `#[cfg(test)] mod tests` if absent):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_bipolar_propagates() {
        let bipolar_err = argumentation_bipolar::Error::IllegalSelfSupport("x".into());
        let err: Error = bipolar_err.into();
        assert!(matches!(err, Error::Bipolar(_)));
    }

    #[test]
    fn error_from_weighted_propagates() {
        let weighted_err = argumentation_weighted::Error::InvalidWeight { weight: -1.0 };
        let err: Error = weighted_err.into();
        assert!(matches!(err, Error::Weighted(_)));
    }

    #[test]
    fn error_from_wbipolar_propagates() {
        let wbp_err = argumentation_weighted_bipolar::Error::IllegalSelfSupport;
        let err: Error = wbp_err.into();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p encounter-argumentation error::tests`
Expected: FAIL — `no variant or associated item named 'Bipolar' found for enum 'Error'` (similar for Weighted, WeightedBipolar).

- [ ] **Step 3: Add the variants**

In `crates/encounter-argumentation/src/error.rs`, insert the three new variants inside the `Error` enum, after the existing `Scheme` variant:

```rust
    /// An error propagated from the argumentation-bipolar layer.
    #[error("bipolar error: {0}")]
    Bipolar(#[from] argumentation_bipolar::Error),

    /// An error propagated from the argumentation-weighted layer.
    #[error("weighted error: {0}")]
    Weighted(#[from] argumentation_weighted::Error),

    /// An error propagated from the argumentation-weighted-bipolar layer.
    #[error("weighted-bipolar error: {0}")]
    WeightedBipolar(#[from] argumentation_weighted_bipolar::Error),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p encounter-argumentation error::tests`
Expected: PASS, 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/error.rs
git commit -m "feat(encounter-argumentation): Error variants for bipolar/weighted/wbipolar propagation"
```

---

## Task 3: Create `ArgumentId` newtype

**Files:**
- Create: `crates/encounter-argumentation/src/arg_id.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/encounter-argumentation/src/arg_id.rs` with:

```rust
//! `ArgumentId`: the identifier type for argument nodes in the
//! encounter-level weighted bipolar framework.
//!
//! An `ArgumentId` is the stringified rendering of a literal
//! (e.g. `"fortify_east"` for a positive atom, `"¬deny_claim"` for a
//! negated literal). Two scheme instances that share a conclusion
//! literal share the same `ArgumentId`, so both count as supporting
//! the same argument node. This is the correct convergence behaviour.

use argumentation::aspic::Literal;

/// Opaque identifier for an argument node in the weighted bipolar
/// framework. Constructed from a `Literal` via `From`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArgumentId(String);

impl ArgumentId {
    /// Construct an `ArgumentId` from a raw string. Prefer `From<&Literal>`
    /// when converting from scheme conclusions.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The underlying string, e.g. for AIF export or logging.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Literal> for ArgumentId {
    fn from(lit: &Literal) -> Self {
        Self(lit.to_string())
    }
}

impl From<Literal> for ArgumentId {
    fn from(lit: Literal) -> Self {
        Self(lit.to_string())
    }
}

impl std::fmt::Display for ArgumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wraps_string() {
        let id = ArgumentId::new("fortify_east");
        assert_eq!(id.as_str(), "fortify_east");
    }

    #[test]
    fn from_positive_literal_renders_plain() {
        let lit = Literal::atom("fortify_east");
        let id: ArgumentId = (&lit).into();
        assert_eq!(id.as_str(), "fortify_east");
    }

    #[test]
    fn from_negated_literal_renders_with_prefix() {
        let lit = Literal::neg("deny_claim");
        let id: ArgumentId = (&lit).into();
        assert_eq!(id.as_str(), "¬deny_claim");
    }

    #[test]
    fn two_same_literals_produce_equal_ids() {
        let a = ArgumentId::from(&Literal::atom("x"));
        let b = ArgumentId::from(&Literal::atom("x"));
        assert_eq!(a, b);
    }

    #[test]
    fn display_matches_as_str() {
        let id = ArgumentId::new("foo");
        assert_eq!(format!("{}", id), "foo");
    }
}
```

- [ ] **Step 2: Wire the module into lib.rs**

In `crates/encounter-argumentation/src/lib.rs`, find the existing `pub mod ...` block (it currently has `acceptance`, `critical_moves`, `error`, `knowledge`, `resolver`, `scoring`, `value_argument`). Add:

```rust
pub mod arg_id;
```

immediately before `pub mod acceptance;`. Also add this re-export near the existing `pub use` lines at the bottom of `lib.rs`:

```rust
pub use arg_id::ArgumentId;
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p encounter-argumentation arg_id`
Expected: PASS, 5 passed.

- [ ] **Step 4: Clippy self-review**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/arg_id.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): add ArgumentId newtype"
```

---

## Task 4: Create `state.rs` skeleton with `new()`

**Files:**
- Create: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/encounter-argumentation/src/state.rs` with:

```rust
//! `EncounterArgumentationState`: the encounter-level state object
//! composing schemes + bipolar + weighted + weighted-bipolar.
//!
//! Consumers build a state via `new(registry)`, optionally configure
//! a weight source and scene intensity via builders, add scheme
//! instances and raw edges, then query acceptance and coalitions.

use crate::arg_id::ArgumentId;
use crate::error::Error;
use argumentation_schemes::instance::SchemeInstance;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_weighted::types::Budget;
use argumentation_weighted_bipolar::WeightedBipolarFramework;
use std::collections::HashMap;

/// Encounter-level argumentation state composing schemes (premises +
/// conclusion), bipolar graph structure (attacks + supports), weighted
/// edge strengths, and a configurable scene-intensity budget.
pub struct EncounterArgumentationState {
    /// Scheme catalog used for instantiation + CQ lookup.
    registry: CatalogRegistry,
    /// The underlying weighted bipolar framework.
    framework: WeightedBipolarFramework<ArgumentId>,
    /// Which actor asserted each argument. Multiple actors may share
    /// an `ArgumentId` (the same conclusion), so stored as a vec.
    actors_by_argument: HashMap<ArgumentId, Vec<String>>,
    /// Scheme instances backing each argument.
    instances_by_argument: HashMap<ArgumentId, Vec<SchemeInstance>>,
    /// Current scene intensity. Defaults to zero.
    intensity: Budget,
}

impl EncounterArgumentationState {
    /// Create a new state with the given scheme registry, no weight
    /// source, and zero scene intensity.
    #[must_use]
    pub fn new(registry: CatalogRegistry) -> Self {
        Self {
            registry,
            framework: WeightedBipolarFramework::new(),
            actors_by_argument: HashMap::new(),
            instances_by_argument: HashMap::new(),
            intensity: Budget::zero(),
        }
    }

    /// Read-only access to the current scene intensity.
    #[must_use]
    pub fn intensity(&self) -> Budget {
        self.intensity
    }

    /// Number of argument nodes in the framework.
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.framework.argument_count()
    }

    /// Number of edges (attacks + supports) in the framework.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.framework.edge_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;

    #[test]
    fn new_state_is_empty() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.argument_count(), 0);
        assert_eq!(state.edge_count(), 0);
    }

    #[test]
    fn new_state_has_zero_intensity() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.intensity().value(), 0.0);
    }
}
```

- [ ] **Step 2: Wire the module into lib.rs**

In `crates/encounter-argumentation/src/lib.rs`, add `pub mod state;` after `pub mod scoring;` (alphabetical-ish placement, matching the existing pattern). Add `pub use state::EncounterArgumentationState;` to the re-exports at the bottom.

- [ ] **Step 3: Run the tests**

Run: `cargo test -p encounter-argumentation state`
Expected: PASS, 2 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): state skeleton with new() + intensity + counters"
```

---

## Task 5: Add `add_scheme_instance` method

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to the `tests` module of `state.rs`:

```rust
    #[test]
    fn add_scheme_instance_creates_argument_node() {
        use argumentation_schemes::instantiate;

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

        assert_eq!(id.as_str(), "fortify_east");
        assert_eq!(state.argument_count(), 1);
    }

    #[test]
    fn add_scheme_instance_associates_actor_and_instance() {
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
        assert_eq!(state.actors_for(&id), &["alice".to_string()]);
        assert_eq!(state.instances_for(&id).len(), 1);
    }

    #[test]
    fn add_two_instances_with_same_conclusion_share_node() {
        use argumentation_schemes::instantiate;
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

        let inst1 = scheme
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
        // Different expert, different domain, same claim — same conclusion
        // literal, so they should share the argument node.
        let inst2 = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "bob".to_string()),
                    ("domain".to_string(), "logistics".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let id1 = state.add_scheme_instance("alice", inst1);
        let id2 = state.add_scheme_instance("bob", inst2);
        assert_eq!(id1, id2);
        assert_eq!(state.argument_count(), 1);
        assert_eq!(state.actors_for(&id1), &["alice".to_string(), "bob".to_string()]);
        assert_eq!(state.instances_for(&id1).len(), 2);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: FAIL — `no method named 'add_scheme_instance' found`.

- [ ] **Step 3: Implement the method**

Append to the `impl EncounterArgumentationState` block in `state.rs`:

```rust
    /// Add a scheme instance asserted by `actor`. The instance's
    /// conclusion literal becomes an argument node in the framework
    /// (if not already present). The actor and instance are recorded
    /// against that node for later lookup via `actors_for` /
    /// `instances_for`. Returns the argument's identifier.
    pub fn add_scheme_instance(
        &mut self,
        actor: &str,
        instance: SchemeInstance,
    ) -> ArgumentId {
        let id: ArgumentId = (&instance.conclusion).into();
        self.framework.add_argument(id.clone());
        self.actors_by_argument
            .entry(id.clone())
            .or_default()
            .push(actor.to_string());
        self.instances_by_argument
            .entry(id.clone())
            .or_default()
            .push(instance);
        id
    }

    /// Return the list of actors who have asserted the given argument.
    /// Empty slice if the argument is not associated with any actor.
    #[must_use]
    pub fn actors_for(&self, id: &ArgumentId) -> &[String] {
        self.actors_by_argument
            .get(id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Return the list of scheme instances backing the given argument.
    /// Empty slice if the argument is not scheme-backed.
    #[must_use]
    pub fn instances_for(&self, id: &ArgumentId) -> &[SchemeInstance] {
        self.instances_by_argument
            .get(id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: PASS, 5 passed (2 pre-existing + 3 new).

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.add_scheme_instance + actor/instance lookup"
```

---

## Task 6: Add raw `add_weighted_attack` and `add_weighted_support`

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn add_weighted_attack_propagates_to_framework() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.5).unwrap();
        assert_eq!(state.edge_count(), 1);
    }

    #[test]
    fn add_weighted_support_propagates_to_framework() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_support(&a, &b, 0.5).unwrap();
        assert_eq!(state.edge_count(), 1);
    }

    #[test]
    fn add_weighted_support_rejects_self_support() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let err = state.add_weighted_support(&a, &a, 0.5).unwrap_err();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }

    #[test]
    fn add_weighted_attack_rejects_invalid_weight() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        let err = state.add_weighted_attack(&a, &b, -0.1).unwrap_err();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::add_weighted`
Expected: FAIL — `no method named 'add_weighted_attack' found`.

- [ ] **Step 3: Implement the methods**

Append to the `impl EncounterArgumentationState` block in `state.rs`:

```rust
    /// Add a weighted attack edge. Both endpoints are implicitly added
    /// to the framework if not already present. Returns
    /// `Error::WeightedBipolar` for invalid weights.
    pub fn add_weighted_attack(
        &mut self,
        attacker: &ArgumentId,
        target: &ArgumentId,
        weight: f64,
    ) -> Result<(), Error> {
        self.framework
            .add_weighted_attack(attacker.clone(), target.clone(), weight)?;
        Ok(())
    }

    /// Add a weighted support edge. Both endpoints are implicitly
    /// added. Returns `Error::WeightedBipolar` for invalid weights or
    /// self-support.
    pub fn add_weighted_support(
        &mut self,
        supporter: &ArgumentId,
        supported: &ArgumentId,
        weight: f64,
    ) -> Result<(), Error> {
        self.framework
            .add_weighted_support(supporter.clone(), supported.clone(), weight)?;
        Ok(())
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: PASS, 9 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.add_weighted_attack/support raw mutators"
```

---

## Task 7: Add `at_intensity` builder

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn at_intensity_sets_budget() {
        let state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        assert_eq!(state.intensity().value(), 0.5);
    }

    #[test]
    fn at_intensity_is_chainable_with_add() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.25).unwrap());
        state
            .add_weighted_attack(&ArgumentId::new("a"), &ArgumentId::new("b"), 0.3)
            .unwrap();
        assert_eq!(state.intensity().value(), 0.25);
        assert_eq!(state.edge_count(), 1);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::at_intensity`
Expected: FAIL — `no method named 'at_intensity' found`.

- [ ] **Step 3: Implement the builder**

Append to the `impl EncounterArgumentationState` block:

```rust
    /// Builder method setting the scene-intensity budget. Returns
    /// `self` by value to allow chaining.
    #[must_use]
    pub fn at_intensity(mut self, intensity: Budget) -> Self {
        self.intensity = intensity;
        self
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests::at_intensity`
Expected: PASS, 2 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.at_intensity builder"
```

---

## Task 8: Add acceptance queries (credulous + skeptical)

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn unattacked_argument_is_credulously_accepted() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        state.add_weighted_attack(&a, &ArgumentId::new("unused"), 0.0).unwrap();
        // `a` is unattacked: it appears only as attacker.
        assert!(state.is_credulously_accepted(&a).unwrap());
    }

    #[test]
    fn attacked_argument_is_not_credulously_accepted_at_zero_intensity() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.5).unwrap();
        // `b` is attacked by `a` (unattacked); at β=0 the attack binds.
        assert!(!state.is_credulously_accepted(&b).unwrap());
    }

    #[test]
    fn raising_intensity_flips_acceptance_when_budget_covers_attack() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.4).unwrap();
        // At β=0.5 >= 0.4, the residual dropping a→b exists, so b is
        // credulously accepted in that residual.
        assert!(state.is_credulously_accepted(&b).unwrap());
    }

    #[test]
    fn skeptical_is_stricter_than_credulous() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.4).unwrap();
        // At β=0.5, b is credulous (residual drops the attack) but NOT
        // skeptical (the full-framework residual still attacks b).
        assert!(state.is_credulously_accepted(&b).unwrap());
        assert!(!state.is_skeptically_accepted(&b).unwrap());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::credulous`
Expected: FAIL — `no method named 'is_credulously_accepted' found`.

- [ ] **Step 3: Implement the queries**

Append to the `impl EncounterArgumentationState` block:

```rust
    /// Whether the argument is credulously accepted under the current
    /// scene intensity (at least one preferred extension of at least
    /// one β-inconsistent residual contains it).
    pub fn is_credulously_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_credulously_accepted_at(
            &self.framework,
            arg,
            self.intensity,
        )?)
    }

    /// Whether the argument is skeptically accepted under the current
    /// scene intensity (every preferred extension of every
    /// β-inconsistent residual contains it).
    pub fn is_skeptically_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_skeptically_accepted_at(
            &self.framework,
            arg,
            self.intensity,
        )?)
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter-argumentation state::tests`
Expected: PASS, 15 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.is_credulously/skeptically_accepted queries"
```

---

## Task 9: Add `coalitions()` query

**Files:**
- Modify: `crates/encounter-argumentation/src/state.rs`

- [ ] **Step 1: Write the failing test**

Append to `state::tests`:

```rust
    #[test]
    fn no_supports_means_no_coalitions() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        state.add_weighted_attack(&ArgumentId::new("a"), &ArgumentId::new("b"), 0.5).unwrap();
        let coalitions = state.coalitions();
        // Each argument is its own singleton coalition under SCC; we
        // filter trivial singletons out of the exposed API.
        assert!(coalitions.iter().all(|c| c.members.len() == 1));
    }

    #[test]
    fn mutual_support_forms_coalition() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_support(&a, &b, 0.5).unwrap();
        state.add_weighted_support(&b, &a, 0.5).unwrap();
        let coalitions = state.coalitions();
        // At least one coalition has both a and b.
        assert!(coalitions.iter().any(|c| c.members.len() == 2
            && c.members.contains(&a)
            && c.members.contains(&b)));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation state::tests::coalition`
Expected: FAIL — `no method named 'coalitions' found`.

- [ ] **Step 3: Confirm bipolar's `Coalition` signature**

Before implementing, read the bipolar crate's public API:
```bash
grep -n "pub struct Coalition\|pub fn detect_coalitions" crates/argumentation-bipolar/src/coalition.rs
```
Note the signature — it returns `Vec<Coalition<A>>` from a `BipolarFramework<A>`. You need to flatten the weighted bipolar framework to a plain bipolar first.

- [ ] **Step 4: Check how `argumentation-weighted-bipolar` exposes the bipolar view**

Read `crates/argumentation-weighted-bipolar/src/lib.rs` to see which helpers are public. If there is no `flatten_to_bipolar` helper, you'll need to materialise one residual (β=0 residual) and run `detect_coalitions` on it.

The simplest path: use `argumentation-weighted-bipolar::wbipolar_residuals` at `Budget::zero()` to get the single "all edges present" BipolarFramework, then call `argumentation_bipolar::detect_coalitions` on that.

- [ ] **Step 5: Implement the query**

Append to the `impl EncounterArgumentationState` block:

```rust
    /// Detect coalitions (strongly-connected components of the support
    /// graph) at the current framework state. Independent of scene
    /// intensity — coalitions are a structural property of supports,
    /// not a semantic query.
    #[must_use]
    pub fn coalitions(&self) -> Vec<argumentation_bipolar::Coalition<ArgumentId>> {
        // Materialise the full-edge bipolar residual (β=0 → one residual
        // containing every edge) and run SCC on the support graph.
        let residuals = argumentation_weighted_bipolar::wbipolar_residuals(
            &self.framework,
            argumentation_weighted::types::Budget::zero(),
        )
        .expect("Budget::zero() residual enumeration is infallible within edge limit");
        let bipolar = residuals
            .into_iter()
            .next()
            .expect("zero-budget residual always includes the empty subset");
        argumentation_bipolar::detect_coalitions(&bipolar)
    }
```

Note the `.expect()` calls: `wbipolar_residuals` at `Budget::zero()` with fewer than `EDGE_ENUMERATION_LIMIT` edges always returns exactly one residual. The expects document that invariant. If the framework exceeds the limit, the expect panics — which is OK for Phase A; a later version can surface `Result<Vec<_>, Error>` if scale demands.

- [ ] **Step 6: Add the `argumentation-bipolar` direct dep path**

`coalitions()` references `argumentation_bipolar::{Coalition, detect_coalitions}` — both are re-exported at the `argumentation_bipolar` crate root, so no additional imports are needed beyond what Task 1 already added.

- [ ] **Step 7: Run tests**

Run: `cargo test -p encounter-argumentation state::tests::coalition`
Expected: PASS, 2 passed.

- [ ] **Step 8: Commit**

```bash
git add crates/encounter-argumentation/src/state.rs
git commit -m "feat(encounter-argumentation): state.coalitions query via wbipolar flatten + Tarjan SCC"
```

---

## Task 10: Create `RelationshipSnapshot` + `RelationshipDims` stubs

**Files:**
- Create: `crates/encounter-argumentation/src/relationship.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/encounter-argumentation/src/relationship.rs`:

```rust
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
```

- [ ] **Step 2: Wire the module into lib.rs**

In `crates/encounter-argumentation/src/lib.rs`, add `pub mod relationship;` alongside the other `pub mod` declarations, and add this re-export to the bottom:

```rust
pub use relationship::{RelationshipDims, RelationshipSnapshot};
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p encounter-argumentation relationship`
Expected: PASS, 3 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/src/relationship.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): RelationshipSnapshot/Dims stubs (Phase C replaces)"
```

---

## Task 11: Add `RelationshipWeightSource` implementing `WeightSource<ArgumentId>`

**Files:**
- Modify: `crates/encounter-argumentation/src/relationship.rs`

- [ ] **Step 1: Write the failing test**

Append to the `tests` module in `relationship.rs`:

```rust
    use crate::arg_id::ArgumentId;
    use argumentation_weighted::WeightSource;

    #[test]
    fn neutral_relationship_yields_midrange_weight() {
        let snapshot = RelationshipSnapshot::new();
        let source = RelationshipWeightSource::new(snapshot);
        // Any attacker/target pair with a neutral snapshot produces
        // the neutral baseline weight (0.5 by convention).
        let w = source.weight_for(&ArgumentId::new("a"), &ArgumentId::new("b"));
        assert_eq!(w, Some(0.5));
    }

    #[test]
    fn high_trust_lowers_attack_weight() {
        // High trust means disagreement is absorbed without rupture →
        // the attack has low binding weight.
        let mut snapshot = RelationshipSnapshot::new();
        snapshot.set(
            "alice",
            "bob",
            RelationshipDims {
                trust: 1.0,
                fear: 0.0,
                respect: 0.0,
                attraction: 0.0,
                friendship: 0.0,
            },
        );
        let source = RelationshipWeightSource::new(snapshot);
        // WeightSource takes attacker/target as ArgumentIds, but the
        // source needs character names to look up the relationship.
        // For Phase A, the convention is that argument ids prefixed
        // with the actor name encode the attacker→target pair; the
        // default source uses a caller-provided character-identity
        // function. Default behavior without a function: attacker and
        // target ArgumentId strings are taken as-is as character names.
        let w = source
            .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
            .unwrap();
        assert!(w < 0.5, "high trust should reduce attack weight below neutral, got {}", w);
    }

    #[test]
    fn high_fear_raises_attack_weight() {
        let mut snapshot = RelationshipSnapshot::new();
        snapshot.set(
            "alice",
            "bob",
            RelationshipDims {
                trust: 0.0,
                fear: 1.0,
                respect: 0.0,
                attraction: 0.0,
                friendship: 0.0,
            },
        );
        let source = RelationshipWeightSource::new(snapshot);
        let w = source
            .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
            .unwrap();
        assert!(w > 0.5, "high fear should raise attack weight above neutral, got {}", w);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p encounter-argumentation relationship::tests::high`
Expected: FAIL — `cannot find type 'RelationshipWeightSource' in this scope`.

- [ ] **Step 3: Implement the type**

Append to `relationship.rs` (above the `#[cfg(test)] mod tests` block):

```rust
use crate::arg_id::ArgumentId;
use argumentation_weighted::WeightSource;

/// A `WeightSource` that derives attack weights from
/// `RelationshipSnapshot` dimensions.
///
/// The mapping (Phase A stub, tune in Phase C):
/// - baseline weight = 0.5
/// - trust reduces weight (a trusted speaker's attacks land softly)
/// - fear raises weight (a feared speaker's attacks bind harder)
/// - respect reduces weight slightly
/// - attraction reduces weight slightly
/// - friendship reduces weight
///
/// Weights are clamped to `[0.0, 1.0]`. Arguments whose attacker/target
/// `ArgumentId` strings are not registered in the snapshot receive the
/// neutral baseline.
///
/// **Phase C replaces this** with a societas-backed scorer whose
/// coefficients are calibrated against real gameplay telemetry.
pub struct RelationshipWeightSource {
    snapshot: RelationshipSnapshot,
}

impl RelationshipWeightSource {
    /// Create a new source wrapping the given snapshot.
    #[must_use]
    pub fn new(snapshot: RelationshipSnapshot) -> Self {
        Self { snapshot }
    }
}

impl WeightSource<ArgumentId> for RelationshipWeightSource {
    fn weight_for(&self, attacker: &ArgumentId, target: &ArgumentId) -> Option<f64> {
        // Phase-A convention: ArgumentId strings are taken as-is as
        // character names. Phase-C replaces this with a proper
        // character-ids extracted from scheme instances.
        let dims = self.snapshot.get(attacker.as_str(), target.as_str());
        let baseline = 0.5;
        let adjustment = -0.15 * dims.trust
            + 0.25 * dims.fear
            - 0.05 * dims.respect
            - 0.05 * dims.attraction
            - 0.10 * dims.friendship;
        let w = (baseline + adjustment).clamp(0.0, 1.0);
        Some(w)
    }
}
```

- [ ] **Step 4: Wire into lib.rs re-exports**

In `crates/encounter-argumentation/src/lib.rs`, extend the relationship re-export line:

Before:
```rust
pub use relationship::{RelationshipDims, RelationshipSnapshot};
```

After:
```rust
pub use relationship::{RelationshipDims, RelationshipSnapshot, RelationshipWeightSource};
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p encounter-argumentation relationship`
Expected: PASS, 6 passed.

- [ ] **Step 6: Clippy self-review**

Run: `cargo clippy -p encounter-argumentation -- -D warnings`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/encounter-argumentation/src/relationship.rs crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): RelationshipWeightSource stub scoring recipe"
```

---

## Task 12: Integration test — UC1 parity between new state API and `resolve_argument`

**Files:**
- Create: `crates/encounter-argumentation/tests/uc_parity.rs`

- [ ] **Step 1: Write the integration test**

Create `crates/encounter-argumentation/tests/uc_parity.rs`:

```rust
//! UC1 parity: the new state API agrees with `resolve_argument` on a
//! pairwise scenario. Both machinery paths should reach the same
//! verdict on a clean attack between two scheme instances.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::{
    ArgumentId, ArgumentOutcome, EncounterArgumentationState, resolve_argument,
};
use std::collections::HashMap;

#[test]
fn new_state_api_and_resolve_argument_agree_on_pairwise_expert_vs_rebuttal() {
    // Scenario: Alice uses expert opinion to support `fortify_east`.
    // Bob uses argument-from-consequences to support `¬fortify_east`
    // (negated conclusion). They attack each other.
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let alice_bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    let alice_instance = expert.instantiate(&alice_bindings).unwrap();

    // Path 1: the existing resolver.
    let legacy_outcome = resolve_argument(&[alice_instance.clone()], &[], &registry);
    // With no responder arguments, Alice's side survives.
    assert!(matches!(legacy_outcome, ArgumentOutcome::ProposerWins { .. }));

    // Path 2: the new state API.
    let mut state = EncounterArgumentationState::new(registry);
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    // `fortify_east` is unattacked → credulously accepted at β=0.
    assert!(
        state.is_credulously_accepted(&alice_arg).unwrap(),
        "new state API should accept Alice's unattacked conclusion"
    );
    assert_eq!(alice_arg, ArgumentId::new("fortify_east"));
}

#[test]
fn new_state_api_rejects_attacked_conclusion_at_zero_intensity() {
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

    let mut state = EncounterArgumentationState::new(registry);
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    let attacker_arg = ArgumentId::new("bob_counter");
    state.add_weighted_attack(&attacker_arg, &alice_arg, 0.5).unwrap();

    // β=0 → the attack binds → fortify_east is not credulously accepted.
    assert!(!state.is_credulously_accepted(&alice_arg).unwrap());
}

#[test]
fn new_state_api_at_high_intensity_tolerates_attack() {
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

    let mut state = EncounterArgumentationState::new(registry)
        .at_intensity(Budget::new(0.6).unwrap());
    let alice_arg = state.add_scheme_instance("alice", alice_instance);
    state.add_weighted_attack(&ArgumentId::new("bob"), &alice_arg, 0.5).unwrap();

    // β=0.6 >= 0.5 → residual drops the attack → accepted credulously.
    assert!(state.is_credulously_accepted(&alice_arg).unwrap());
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p encounter-argumentation --test uc_parity`
Expected: PASS, 3 passed.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter-argumentation/tests/uc_parity.rs
git commit -m "test(encounter-argumentation): UC1 parity between state API and resolve_argument"
```

---

## Task 13: Integration test — relationship modulation (UC3)

**Files:**
- Create: `crates/encounter-argumentation/tests/uc_relationship_modulation.rs`

- [ ] **Step 1: Write the integration test**

Create `crates/encounter-argumentation/tests/uc_relationship_modulation.rs`:

```rust
//! UC3: same scheme instance, same budget, but different relationship
//! snapshots produce different attack weights and therefore different
//! acceptance thresholds.

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use argumentation_weighted::WeightSource;
use encounter_argumentation::{
    ArgumentId, EncounterArgumentationState, RelationshipDims, RelationshipSnapshot,
    RelationshipWeightSource,
};

fn build_state_with_weight(attack_weight: f64, budget: Budget) -> EncounterArgumentationState {
    let mut state = EncounterArgumentationState::new(default_catalog()).at_intensity(budget);
    let alice = ArgumentId::new("alice");
    let bob = ArgumentId::new("bob");
    state.add_weighted_attack(&alice, &bob, attack_weight).unwrap();
    state
}

#[test]
fn high_trust_reduces_effective_attack_weight() {
    // Create two snapshots: neutral, and alice-highly-trusts-bob.
    let neutral = RelationshipSnapshot::new();
    let mut trusting = RelationshipSnapshot::new();
    trusting.set(
        "alice",
        "bob",
        RelationshipDims {
            trust: 1.0,
            fear: 0.0,
            respect: 0.0,
            attraction: 0.0,
            friendship: 0.0,
        },
    );

    let neutral_source = RelationshipWeightSource::new(neutral);
    let trusting_source = RelationshipWeightSource::new(trusting);

    let neutral_w = neutral_source
        .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
        .unwrap();
    let trusting_w = trusting_source
        .weight_for(&ArgumentId::new("alice"), &ArgumentId::new("bob"))
        .unwrap();

    assert!(
        trusting_w < neutral_w,
        "high-trust weight ({}) should be below neutral ({})",
        trusting_w,
        neutral_w,
    );
}

#[test]
fn same_scenario_flips_acceptance_at_different_budgets_for_different_weights() {
    // Low-trust (neutral) scenario: attack weight ~0.5. At β=0.3 the
    // attack binds; at β=0.6 the residual drops it and b is accepted.
    let low_β = Budget::new(0.3).unwrap();
    let high_β = Budget::new(0.6).unwrap();

    let state_low = build_state_with_weight(0.5, low_β);
    let state_high = build_state_with_weight(0.5, high_β);

    let bob = ArgumentId::new("bob");
    assert!(!state_low.is_credulously_accepted(&bob).unwrap());
    assert!(state_high.is_credulously_accepted(&bob).unwrap());

    // Now a "trusting" relationship yields a weight of ~0.35 (neutral
    // 0.5 − 0.15 for trust=1.0). At β=0.3 the attack now binds (0.3 <
    // 0.35 still), at β=0.6 it's tolerated.
    let state_trust_low = build_state_with_weight(0.35, low_β);
    let state_trust_high = build_state_with_weight(0.35, high_β);

    assert!(!state_trust_low.is_credulously_accepted(&bob).unwrap());
    assert!(state_trust_high.is_credulously_accepted(&bob).unwrap());

    // Sanity: an even-lower-weight attack (0.2) would be tolerated
    // already at β=0.3, demonstrating that relationship modulation can
    // flip acceptance at a fixed β.
    let state_very_trust = build_state_with_weight(0.2, low_β);
    assert!(state_very_trust.is_credulously_accepted(&bob).unwrap());
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p encounter-argumentation --test uc_relationship_modulation`
Expected: PASS, 2 passed.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter-argumentation/tests/uc_relationship_modulation.rs
git commit -m "test(encounter-argumentation): UC3 relationship modulation of attack weights"
```

---

## Task 14: Integration test — β intensity sweep (UC4)

**Files:**
- Create: `crates/encounter-argumentation/tests/uc_intensity_sweep.rs`

- [ ] **Step 1: Write the integration test**

Create `crates/encounter-argumentation/tests/uc_intensity_sweep.rs`:

```rust
//! UC4: sweeping scene intensity (β) over a 3-character scene flips
//! credulous acceptance at predictable budgets.

use argumentation_weighted::types::Budget;
use encounter_argumentation::{ArgumentId, EncounterArgumentationState};
use argumentation_schemes::catalog::default_catalog;

#[test]
fn credulous_acceptance_is_monotone_over_intensity_sweep() {
    // Scene: c attacks b (weight 0.4); b attacks a (weight 0.3).
    // Chain: at β=0 a is defended (b is attacked by c so a survives).
    // At β=0.4 a is still defended, b is now a candidate too (c→b
    // droppable), and at β=0.7+ everything tolerates.
    let mut state = EncounterArgumentationState::new(default_catalog());
    let a = ArgumentId::new("a");
    let b = ArgumentId::new("b");
    let c = ArgumentId::new("c");
    state.add_weighted_attack(&c, &b, 0.4).unwrap();
    state.add_weighted_attack(&b, &a, 0.3).unwrap();

    let budgets: Vec<Budget> = [0.0, 0.3, 0.4, 0.7, 1.0]
        .into_iter()
        .map(|v| Budget::new(v).unwrap())
        .collect();

    let mut last_a: Option<bool> = None;
    let mut last_b: Option<bool> = None;
    for &β in &budgets {
        // Fresh state per budget (at_intensity consumes `self`).
        let mut s = EncounterArgumentationState::new(default_catalog()).at_intensity(β);
        s.add_weighted_attack(&c, &b, 0.4).unwrap();
        s.add_weighted_attack(&b, &a, 0.3).unwrap();
        let a_ok = s.is_credulously_accepted(&a).unwrap();
        let b_ok = s.is_credulously_accepted(&b).unwrap();
        if let Some(prev) = last_a {
            assert!(
                !prev || a_ok,
                "credulous monotonicity: a was accepted at previous budget but not at {}",
                β.value()
            );
        }
        if let Some(prev) = last_b {
            assert!(
                !prev || b_ok,
                "credulous monotonicity: b was accepted at previous budget but not at {}",
                β.value()
            );
        }
        last_a = Some(a_ok);
        last_b = Some(b_ok);
    }

    // Final sanity: at β=1.0 every attack is droppable → all three are
    // credulously accepted.
    let mut s_big = EncounterArgumentationState::new(default_catalog())
        .at_intensity(Budget::new(1.0).unwrap());
    s_big.add_weighted_attack(&c, &b, 0.4).unwrap();
    s_big.add_weighted_attack(&b, &a, 0.3).unwrap();
    assert!(s_big.is_credulously_accepted(&a).unwrap());
    assert!(s_big.is_credulously_accepted(&b).unwrap());
    assert!(s_big.is_credulously_accepted(&c).unwrap());
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p encounter-argumentation --test uc_intensity_sweep`
Expected: PASS, 1 passed.

- [ ] **Step 3: Commit**

```bash
git add crates/encounter-argumentation/tests/uc_intensity_sweep.rs
git commit -m "test(encounter-argumentation): UC4 β intensity sweep monotonicity"
```

---

## Task 15: Update `lib.rs` module docs + top-level doctest

**Files:**
- Modify: `crates/encounter-argumentation/src/lib.rs`

- [ ] **Step 1: Read the current module doc**

Run: `head -30 crates/encounter-argumentation/src/lib.rs`

Note the existing `//!` doctest uses `resolve_argument`. We need to ADD a new section describing the `EncounterArgumentationState` path without removing the existing example.

- [ ] **Step 2: Extend the module doc**

In `crates/encounter-argumentation/src/lib.rs`, find the existing `//! # Quick example` section and replace it with:

```rust
//! # Quick example — pairwise resolver (v0.1.x; still supported)
//!
//! ```
//! use argumentation_schemes::catalog::default_catalog;
//! use argumentation_schemes::instantiate;
//! use encounter_argumentation::resolver::{resolve_argument, ArgumentOutcome};
//!
//! let registry = default_catalog();
//! let expert = registry.by_key("argument_from_expert_opinion").unwrap();
//! let instance = instantiate(expert, &[
//!     ("expert".into(), "alice".into()),
//!     ("domain".into(), "military".into()),
//!     ("claim".into(), "fortify_east".into()),
//! ].into_iter().collect()).unwrap();
//!
//! let outcome = resolve_argument(&[instance], &[], &registry);
//! assert!(matches!(outcome, ArgumentOutcome::ProposerWins { .. }));
//! ```
//!
//! # Quick example — state API (v0.2.0)
//!
//! The new `EncounterArgumentationState` unifies scheme reasoning,
//! bipolar graph structure, weighted attack strengths, and a tunable
//! scene-intensity budget:
//!
//! ```
//! use argumentation_schemes::catalog::default_catalog;
//! use argumentation_weighted::types::Budget;
//! use encounter_argumentation::{ArgumentId, EncounterArgumentationState};
//!
//! let registry = default_catalog();
//! let expert = registry.by_key("argument_from_expert_opinion").unwrap();
//! let instance = expert.instantiate(&[
//!     ("expert".into(), "alice".into()),
//!     ("domain".into(), "military".into()),
//!     ("claim".into(), "fortify_east".into()),
//! ].into_iter().collect()).unwrap();
//!
//! let mut state = EncounterArgumentationState::new(registry)
//!     .at_intensity(Budget::new(0.4).unwrap());
//! let alice_arg = state.add_scheme_instance("alice", instance);
//! state
//!     .add_weighted_attack(&ArgumentId::new("bob_counter"), &alice_arg, 0.3)
//!     .unwrap();
//!
//! // At β=0.4 > 0.3 the attack is tolerated: alice's claim is accepted.
//! assert!(state.is_credulously_accepted(&alice_arg).unwrap());
//! ```
```

- [ ] **Step 3: Run the doctest**

Run: `cargo test -p encounter-argumentation --doc`
Expected: PASS, 2 passed (both doctests).

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/src/lib.rs
git commit -m "docs(encounter-argumentation): lib.rs quick example for the new state API"
```

---

## Task 16: CHANGELOG + README + version bump + tag

**Files:**
- Modify: `crates/encounter-argumentation/CHANGELOG.md` (create if absent)
- Modify: `crates/encounter-argumentation/README.md` (create if absent)
- Modify: `crates/encounter-argumentation/Cargo.toml`

- [ ] **Step 1: Check CHANGELOG and README state**

Run:
```bash
ls crates/encounter-argumentation/CHANGELOG.md crates/encounter-argumentation/README.md 2>&1
```

If either file is missing, create it per the content below. If present, prepend/extend.

- [ ] **Step 2: Prepend v0.2.0 entry to CHANGELOG.md**

If `CHANGELOG.md` exists, prepend this block; otherwise create the file with this content:

```markdown
# Changelog

## [0.2.0] - 2026-04-19

### Added
- `EncounterArgumentationState` — unified state object composing
  `argumentation-schemes` + `argumentation-bipolar` +
  `argumentation-weighted` + `argumentation-weighted-bipolar` under one
  encounter-friendly API. Supports:
  - `add_scheme_instance(actor, instance) -> ArgumentId` — adds a
    scheme-backed node; actors with the same conclusion converge on
    the same argument.
  - `add_weighted_attack` / `add_weighted_support` — raw graph
    mutators on the underlying `WeightedBipolarFramework`.
  - `at_intensity(Budget)` — builder setting the scene-intensity β.
  - `is_credulously_accepted` / `is_skeptically_accepted` — acceptance
    queries at the current β.
  - `coalitions()` — Tarjan SCC over the support graph.
- `ArgumentId` newtype over `String` keyed by literal rendering.
- `RelationshipSnapshot` / `RelationshipDims` / `RelationshipWeightSource`
  — Phase-A stubs for relationship-modulated attack weights. **Phase C
  will replace `RelationshipSnapshot` with real societas types.**
- `Error` gained `Bipolar(#[from])`, `Weighted(#[from])`, and
  `WeightedBipolar(#[from])` variants for cross-crate error propagation.

### Preserved (no changes)
- `resolve_argument`, `ArgumentOutcome`, `ArgumentAcceptanceEval`,
  `ArgumentKnowledge`, `StaticKnowledge`, `ArgumentPosition`,
  `SchemeActionScorer`, `scheme_value_argument`, `critical_question_beats`,
  `cq_to_beat` — all v0.1.x API surface retained verbatim.

### Dependencies
- `argumentation-bipolar` — new.
- `argumentation-weighted` — new.
- `argumentation-weighted-bipolar` — new.

## [0.1.0] - prior
- Initial bridge: schemes → ASPIC+ → Dung semantics; pairwise
  resolution; encounter integration via `ArgumentAcceptanceEval`,
  `SchemeActionScorer`, `cq_to_beat`.
```

- [ ] **Step 3: Update README.md**

Prepend (or create) this content:

```markdown
# encounter-argumentation

Bridge crate connecting encounter's social-interaction engine with the
`argumentation` stack.

## What's in the box

### v0.1.x (preserved)
- `resolve_argument` — pairwise ASPIC+ resolution between proposer and
  responder scheme instances.
- `ArgumentAcceptanceEval` — `AcceptanceEval` impl that uses
  argumentation to decide encounter action acceptance.
- `SchemeActionScorer` — wraps an existing `ActionScorer` and boosts
  scores for scheme-backed affordances.
- `ArgumentKnowledge` / `StaticKnowledge` — per-character argumentation
  capabilities.
- `critical_question_beats`, `cq_to_beat` — CQ → encounter Beat mapping.
- `scheme_value_argument` — value-based scheme construction helper.

### v0.2.0 additions — the full-stack state API

`EncounterArgumentationState` composes all four argumentation crates
(schemes, bipolar, weighted, weighted-bipolar) under one encounter-
friendly surface. Use it when you want coalition structure, weighted
attack strength, or a scene-intensity budget — anything beyond pairwise
ASPIC+ resolution.

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::{ArgumentId, EncounterArgumentationState};

let registry = default_catalog();
let expert = registry.by_key("argument_from_expert_opinion").unwrap();
let instance = expert.instantiate(&[
    ("expert".into(), "alice".into()),
    ("domain".into(), "military".into()),
    ("claim".into(), "fortify_east".into()),
].into_iter().collect()).unwrap();

let mut state = EncounterArgumentationState::new(registry)
    .at_intensity(Budget::new(0.4).unwrap());
let alice_arg = state.add_scheme_instance("alice", instance);
state.add_weighted_attack(&ArgumentId::new("bob_counter"), &alice_arg, 0.3).unwrap();

assert!(state.is_credulously_accepted(&alice_arg).unwrap());

for coalition in state.coalitions() {
    println!("coalition size {} members {:?}", coalition.members.len(), coalition.members);
}
```

### Relationship modulation (Phase-A stub)

`RelationshipWeightSource` provides a default mapping from relationship
dimensions (trust, fear, respect, attraction, friendship) to attack
weights. Phase A ships a placeholder `RelationshipSnapshot` type; Phase
C will replace it with a societas adapter.

## Architecture

- Bridge depends on sibling crates via path: `encounter`, `argumentation`,
  `argumentation-schemes`, `argumentation-bipolar`, `argumentation-weighted`,
  `argumentation-weighted-bipolar`.
- `EncounterArgumentationState` internally owns a
  `WeightedBipolarFramework<ArgumentId>`. `ArgumentId` is a newtype
  over the literal's string rendering, so scheme instances with
  identical conclusions converge on a single argument node.
- The existing `resolve_argument` path is unchanged; it still compiles
  scheme instances into an ASPIC+ `StructuredSystem` and runs Dung
  preferred on the result.
```

- [ ] **Step 4: Bump the package version**

In `crates/encounter-argumentation/Cargo.toml`, change `version = "0.1.0"` to `version = "0.2.0"`.

- [ ] **Step 5: Run the full workspace suite**

```bash
cargo test --workspace 2>&1 | tail -10
cargo clippy --workspace -- -D warnings 2>&1 | tail -5
cargo doc --workspace --no-deps 2>&1 | tail -5
```

All three must be clean.

- [ ] **Step 6: Commit and tag**

```bash
git add crates/encounter-argumentation/CHANGELOG.md \
        crates/encounter-argumentation/README.md \
        crates/encounter-argumentation/Cargo.toml
git commit -m "chore(encounter-argumentation): v0.2.0 release — full-stack bridge"
git tag encounter-argumentation-v0.2.0
git log --oneline -3
git tag | grep encounter-argumentation
```

---

## Task 17: Dispatch code review

Follow the `superpowers:requesting-code-review` skill with:
- WHAT_WAS_IMPLEMENTED: Phase A upgrade of `encounter-argumentation` from schemes-only bridge to full-stack bridge.
- PLAN_OR_REQUIREMENTS: Tasks 1-16 from this plan + `ARGUMENTATION_CONSUMERS.md` Phase A scope.
- BASE_SHA: branch point (first commit of `feat/phase-a-encounter-arg-bridge`'s parent) — obtain via `git merge-base HEAD main`.
- HEAD_SHA: `encounter-argumentation-v0.2.0` tag.
- DESCRIPTION: "Added EncounterArgumentationState unifying schemes/bipolar/weighted/wbipolar; ArgumentId newtype; RelationshipSnapshot+WeightSource stubs (Phase C replaces). Existing v0.1.x API preserved verbatim."

Specifically ask the reviewer to verify:
1. The existing API (`resolve_argument` and friends) is bitwise-unchanged — no accidental behaviour drift.
2. `EncounterArgumentationState`'s `coalitions()` correctly handles the empty-framework edge case (no panic on `Budget::zero()` residual when the framework has zero edges).
3. `ArgumentId`'s `From<Literal>` / `From<&Literal>` round-trip fidelity for literals whose names contain unusual characters (consult `argumentation-schemes/src/aif.rs` for the known `¬`-prefix limitation).
4. The `RelationshipWeightSource` scoring recipe is documented as a stub with the right level of "replace me in Phase C" signaling.

Fix any Critical or Important issues before closing out Phase A.

---

## Wrap-up — finishing the branch

After the code review is clean:

1. Announce: "I'm using the finishing-a-development-branch skill to complete this work."
2. **REQUIRED SUB-SKILL:** `superpowers:finishing-a-development-branch`.
3. Verify tests, present merge options, execute the user's choice.

## Self-review notes

- **Spec coverage.** Phase A scope from `ARGUMENTATION_CONSUMERS.md` §3: every bullet maps to a task. `EncounterArgumentationState` (Tasks 4-9), `WeightSource` wiring (Tasks 10-11), UC1/UC3/UC4 integration tests (Tasks 12-14), v0.2.0 release bookkeeping (Tasks 15-16), code review (Task 17). D1-D5 design decisions pinned in pre-flight.
- **No placeholders.** Every code block is complete; every command has an expected result; every test body is spelled out. No "TBD" or "similar to earlier task."
- **Type consistency.** `ArgumentId` is the same newtype across Tasks 3-14. `EncounterArgumentationState` method signatures (`add_scheme_instance`, `add_weighted_attack`, `add_weighted_support`, `at_intensity`, `is_credulously_accepted`, `is_skeptically_accepted`, `coalitions`, `actors_for`, `instances_for`) match between the defining task and all consumer tests. `RelationshipDims`, `RelationshipSnapshot`, `RelationshipWeightSource` spelling is consistent across Tasks 10-11 and their callers.
- **Deferred to later phases.** Real societas wiring (Phase C). Per-character intensity (revisit if pinched). Auto-compilation of scheme-level structure into weighted attacks based on contrariety — Phase A requires the consumer to add attacks explicitly.
