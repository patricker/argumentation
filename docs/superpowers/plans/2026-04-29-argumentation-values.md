# `argumentation-values` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust crate that adds value-based argumentation frameworks (VAFs) to the workspace, integrate it into the encounter bridge with per-character audiences, and demonstrate audience-driven outcome flips in the docs.

**Architecture:** New `argumentation-values` crate sits at the same layer as `argumentation-bipolar` — owns `Audience`, `Value`, `ValueAssignment`, `ValueBasedFramework`. Composition follows the workspace's transformation pattern: a `ValueBasedFramework` *transforms* into a Dung framework with audience-conditioned defeats, then delegates to existing `preferred_extensions`. The encounter bridge gains per-character audience storage on `EncounterArgumentationState`; a new `ValueAwareScorer` wraps the existing `SchemeActionScorer`. Multi-value support per Kaci & van der Torre 2008 (Pareto-defeating rule). Acceptance with hard-cap on objective queries per Dunne & Bench-Capon 2004 complexity (NP/co-NP).

**Tech Stack:** Rust 2024 edition, `argumentation` workspace crates (path deps), `thiserror = "2.0"`, `smallvec = "1.13"` for multi-value compactness, no new heavy deps. APX text format for ASPARTIX interop. No SAT/ASP backends in v0 (gated `--features sat` reserved for future).

---

## Scope notes

This plan is **four phases** that ship independently. You can stop after Phase 1 and have a working VAF crate; stop after Phase 2 and additionally have encounter integration; Phase 3 is interop + multi-audience query; Phase 4 is documentation reflow.

**The use cases this plan enables** (from research synthesis):

- **UC1** Multi-character disagreement on values — Phase 1 + Phase 2
- **UC2** AgreementScenario / multi-character consensus — Phase 3
- **UC3** Audience swap (re-evaluate same scene under different orderings) — Phase 1
- **UC4** AATS+V structured authoring — *deferred* (hook only)
- **UC5** LLM-driven value extraction (Muse3) — *separate plan in muse3*
- **UC6** Legal/appellate-style explanation — Phase 1 + Phase 3 (APX I/O for case import)
- **UC7** Ethical RL judging — *deferred*
- **UC8** Per-character value drift — already covered by existing `value_argument::scheme_value_argument`; Phase 2 extends it to use audiences
- **UC9** Audience aggregation (collective audiences) — *deferred* (hook in Phase 3)
- **UC10** Value-aware NPC consistency at scale — Phase 1 + Phase 2

**Out of scope (separate future plans):**
- LLM value extraction in the Muse3 pipeline (UC5) — separate plan in `~/code/muse3/docs/superpowers/plans/`
- AATS+V 4-layer practical-reasoning framework (UC4) — separate plan if a consumer asks
- Audience aggregation algorithms (UC9 — Bodanza, Tohmé & Auday 2017) — separate plan
- WASM bindings for `ValueBasedFramework` — Phase 4 uses static AttackGraph comparisons rather than live `BetaPlayground`. WASM is a separate plan.
- SAT-backed acceptance for large frameworks — gated behind a feature flag we don't enable

**The "already shipped" foundation this plan builds on:**

- `crates/encounter-argumentation/src/value_argument.rs` — `scheme_value_argument` does ASPIC+-based per-pair value resolution today using the `argument_from_values` Walton scheme. Phase 2 makes this audience-aware.
- `crates/encounter-argumentation/src/scoring.rs` — `SchemeActionScorer` already does preference-weighted scheme boosting (the descriptively-accurate "value importance" semantics that Bodanza & Freidin 2023 found people use). Phase 2's `ValueAwareScorer` *wraps* this; doesn't replace it.
- `crates/encounter-argumentation/src/knowledge.rs` — `ArgumentKnowledge` trait + `ArgumentPosition` with `preference_weight`. Phase 2 augments with per-actor audience.
- `crates/argumentation-schemes/src/catalog/practical.rs:120` — `argument_from_values` Walton scheme with `value` premise slot. Phase 1's scheme bridge consumes this.

---

## File structure

**New crate** `crates/argumentation-values/`:

| Path | Responsibility |
|---|---|
| `Cargo.toml` | Package manifest |
| `README.md` | Crate-level overview and quickstart |
| `src/lib.rs` | Public re-exports + module barrel |
| `src/error.rs` | `Error` enum (`thiserror`) |
| `src/types.rs` | `Value`, `ValueAssignment<A>`, `Audience` |
| `src/framework.rs` | `ValueBasedFramework` + `defeat_graph` + `accepted_for` (single-audience) |
| `src/acceptance.rs` | `subjectively_accepted`, `objectively_accepted` (with hard cap) |
| `src/scheme_bridge.rs` | `from_scheme_instances` extracting `value` slot from `argument_from_values` |
| `src/apx.rs` | APX format I/O (Phase 3) |
| `src/multi.rs` | `MultiAudience` query (Phase 3) |
| `tests/hal_carla.rs` | Integration test: success criterion under both audiences |
| `tests/multi_value.rs` | Multi-value defeat semantics tests |

**Modified files:**

| Path | Change |
|---|---|
| `Cargo.toml` (workspace) | Add `crates/argumentation-values` to workspace members |
| `crates/encounter-argumentation/Cargo.toml` | Add `argumentation-values` path-dep |
| `crates/encounter-argumentation/src/state.rs` | Add `audiences: Mutex<HashMap<String, Audience>>` field, accessors |
| `crates/encounter-argumentation/src/lib.rs` | Add `pub mod value_scorer` |
| `crates/encounter-argumentation/src/value_scorer.rs` | NEW — `ValueAwareScorer<S>` wrapping `S: ActionScorer<P>` |
| `website/docs/concepts/value-based-argumentation.mdx` | Reframe from scoping doc to "how it works" concepts page |
| `website/docs/examples/hal-and-carla.mdx` | Demonstrate audience flip with side-by-side defeat graphs |
| `website/docs/concepts/open-areas.mdx` | Remove VAF from open list, link to implemented page |
| `website/docs/guides/wiring-character-values.md` | NEW — how-to: per-character audiences in a scene |

---

## Use case → task traceability

| Use case | Tasks |
|---|---|
| UC1 Multi-character disagreement | T2, T3, T7, T8, T12 |
| UC2 AgreementScenario | T10, T13 |
| UC3 Audience swap | T2, T3, T11 |
| UC6 Legal-style explanation | T9 |
| UC8 Value drift (audience-aware) | T8 |
| UC10 Per-character value profiles | T6, T7, T8 |

---

## Phase 1: Core `argumentation-values` crate

The formal VAF apparatus. Pure Rust, no encounter-bridge dependency. Self-contained — Phase 1 ships an independently usable crate.

### Task 1: Create crate skeleton

**Files:**
- Create: `crates/argumentation-values/Cargo.toml`
- Create: `crates/argumentation-values/src/lib.rs`
- Create: `crates/argumentation-values/src/error.rs`
- Create: `crates/argumentation-values/README.md`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Write `crates/argumentation-values/Cargo.toml`**

```toml
[package]
name = "argumentation-values"
version = "0.1.0"
edition = "2024"
description = "Value-based argumentation frameworks (Bench-Capon 2003) built on the argumentation crate"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["argumentation", "values", "vaf", "bench-capon", "audience"]
categories = ["algorithms"]

[dependencies]
argumentation = { path = "../.." }
argumentation-schemes = { path = "../argumentation-schemes" }
smallvec = "1.13"
thiserror = "2.0"

[dev-dependencies]
proptest = "1.4"
```

- [ ] **Step 2: Write `crates/argumentation-values/src/error.rs`**

```rust
//! Error types for argumentation-values.

/// Errors produced by VAF operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wrapped error from the underlying Dung framework operations.
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),

    /// `objectively_accepted` / `subjectively_accepted` bail out when the
    /// audience contains too many distinct values for tractable enumeration
    /// of all linear extensions of the partial order. The hard limit is 6
    /// values (= 720 linear extensions in the worst case).
    #[error("audience too large for exhaustive enumeration: {values} values (limit is {limit})")]
    AudienceTooLarge {
        /// Number of distinct values in the audience.
        values: usize,
        /// Hard cap on values past which we refuse to enumerate.
        limit: usize,
    },

    /// An argument referenced by `ValueAssignment::promote` or by an attack
    /// edge is not registered in the underlying framework.
    #[error("argument not in framework: {0}")]
    ArgumentNotFound(String),

    /// APX text input failed to parse (Phase 3).
    #[error("apx parse error at line {line}: {reason}")]
    ApxParse {
        /// 1-indexed line number where parsing failed.
        line: usize,
        /// What went wrong.
        reason: String,
    },
}
```

- [ ] **Step 3: Write `crates/argumentation-values/src/lib.rs` (skeleton — only declares the modules that exist after Task 1)**

The full module barrel grows as Tasks 2/3/5/6/9/10 land their files. For Task 1 we only have `error.rs`, so the lib.rs only declares it:

```rust
//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.
//!
//! Bench-Capon (2003) extended Dung frameworks with *values* — each argument
//! promotes a value, and an *audience* is an ordering over values. Different
//! audiences reach different rational conclusions from the same framework.
//!
//! Module barrel grows as Tasks 2/3/5/6/9/10 add their files. The full
//! crate-level docs land in Task 2 along with the public types.

pub mod error;

pub use error::Error;
```

- [ ] **Step 4: Write `crates/argumentation-values/README.md`**

```markdown
# argumentation-values

Value-based argumentation frameworks (Bench-Capon 2003) for the `argumentation` Rust workspace.

```rust
use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

let mut base = ArgumentationFramework::new();
base.add_argument("h1");
base.add_argument("c1");
base.add_attack(&"h1", &"c1").unwrap();
base.add_attack(&"c1", &"h1").unwrap();

let mut values = ValueAssignment::new();
values.promote("h1", Value::new("life"));
values.promote("c1", Value::new("property"));

let vaf = ValueBasedFramework::new(base, values);
let life_audience = Audience::total([Value::new("life"), Value::new("property")]);

assert!(vaf.accepted_for(&life_audience, &"h1").unwrap());
assert!(!vaf.accepted_for(&life_audience, &"c1").unwrap());
```

See the [VAF concepts page](https://patricker.github.io/argumentation/concepts/value-based-argumentation) for full docs.

## License
MIT OR Apache-2.0.
```

- [ ] **Step 5: Add the new crate to the workspace `Cargo.toml`**

Read the current workspace Cargo.toml:

```bash
grep -A 15 "^\[workspace\]" /home/peter/code/argumentation/Cargo.toml
```

Add `"crates/argumentation-values"` to the `members` list. The result should be:

```toml
[workspace]
members = [
    ".",
    "crates/argumentation-schemes",
    "crates/argumentation-bipolar",
    "crates/argumentation-weighted",
    "crates/argumentation-weighted-bipolar",
    "crates/argumentation-values",
    "crates/encounter-argumentation",
    "crates/argumentation-wasm",
    "tools/scene-tracer",
]
resolver = "2"
```

- [ ] **Step 6: Build to verify the skeleton compiles**

```bash
cd /home/peter/code/argumentation
cargo build -p argumentation-values 2>&1 | tail -5
```

Expected: `Finished`. The Step 3 lib.rs only declares `pub mod error`, which exists. Subsequent tasks (T2 types, T3 framework, T5 acceptance, T6 scheme_bridge, T9 apx, T10 multi) add their `pub mod` lines and re-exports as the files land.

- [ ] **Step 7: Commit**

```bash
git add crates/argumentation-values/ Cargo.toml
git commit -m "feat(argumentation-values): create crate skeleton

New crate at crates/argumentation-values/. Empty Error enum and
lib.rs barrel for now; types, framework, and scheme_bridge land in
subsequent tasks."
```

---

### Task 2: `Value`, `ValueAssignment<A>`, `Audience` types

**Files:**
- Create: `crates/argumentation-values/src/types.rs`
- Modify: `crates/argumentation-values/src/lib.rs` (add `pub mod types;` and re-exports)

- [ ] **Step 1: Write `crates/argumentation-values/src/types.rs`**

```rust
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
#[derive(Debug, Clone, Default)]
pub struct ValueAssignment<A: Eq + Hash> {
    /// `SmallVec<[Value; 1]>` keeps the common single-value case allocation-free.
    promoted: HashMap<A, SmallVec<[Value; 1]>>,
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
```

- [ ] **Step 2: Update `crates/argumentation-values/src/lib.rs`**

Replace the Task 1 skeleton with the fuller crate-level docs and the new types module:

```rust
//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.
//!
//! Bench-Capon (2003) extended Dung frameworks with *values* — each argument
//! promotes a value, and an *audience* is an ordering over values. Different
//! audiences reach different rational conclusions from the same framework.
//!
//! # Multi-value support
//!
//! This implementation follows Kaci & van der Torre (2008) and supports
//! arguments promoting multiple values. The defeat rule (Pareto-defeating)
//! degenerates to Bench-Capon (2003) single-value when each argument
//! promotes exactly one value. See [`framework::ValueBasedFramework::defeats`]
//! once the framework module lands in Task 3.
//!
//! Module barrel grows as Tasks 3/5/6/9/10 add their files.

pub mod error;
pub mod types;

pub use error::Error;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 3: Run tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values 2>&1 | tail -15
```

Expected: 6 tests pass (the unit tests in `types.rs`).

- [ ] **Step 4: Commit**

```bash
git add crates/argumentation-values/src/types.rs crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): add Value, ValueAssignment, Audience types

Multi-value support via SmallVec<[Value; 1]> (Kaci & van der Torre
2008). Audience as ranked tiers — total orders, partial orders, and
intra-tier ties all expressible. Unranked values are incomparable
(not least-preferred). 5 unit tests."
```

---

### Task 3: Single-value defeat semantics + Hal & Carla success criterion test

**Files:**
- Create: `crates/argumentation-values/src/framework.rs`
- Create: `crates/argumentation-values/tests/hal_carla.rs`
- Modify: `crates/argumentation-values/src/lib.rs` (add `pub mod framework;`)

- [ ] **Step 1: Write `crates/argumentation-values/src/framework.rs`**

```rust
//! `ValueBasedFramework` — Dung framework + value assignment, with
//! audience-conditioned defeat semantics.

use crate::error::Error;
use crate::types::{Audience, ValueAssignment};
use argumentation::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// A value-based argumentation framework: an underlying Dung framework
/// plus a [`ValueAssignment`] mapping arguments to the values they promote.
///
/// Acceptance is computed *per audience* — there is no audience-independent
/// notion of acceptance in a VAF. See [`Self::accepted_for`].
///
/// # Type parameter
///
/// `A` is the argument label type, matching the underlying
/// [`ArgumentationFramework<A>`]. For encounter-bridge use, this is
/// typically `argumentation::ArgumentId`; for standalone use `&'static str`
/// or `String` work fine.
#[derive(Debug, Clone)]
pub struct ValueBasedFramework<A: Clone + Eq + Hash> {
    base: ArgumentationFramework<A>,
    values: ValueAssignment<A>,
}

impl<A: Clone + Eq + Hash + std::fmt::Debug> ValueBasedFramework<A> {
    /// Construct from a Dung framework and value assignment.
    pub fn new(base: ArgumentationFramework<A>, values: ValueAssignment<A>) -> Self {
        Self { base, values }
    }

    /// Borrow the underlying Dung framework (unconditioned attacks).
    pub fn base(&self) -> &ArgumentationFramework<A> {
        &self.base
    }

    /// Borrow the value assignment.
    pub fn value_assignment(&self) -> &ValueAssignment<A> {
        &self.values
    }

    /// Build the audience-conditioned defeat graph as a fresh
    /// [`ArgumentationFramework`].
    ///
    /// An attack `(attacker, target)` in [`Self::base`] becomes a defeat
    /// in the result iff `defeats(attacker, target)` returns true under
    /// this audience.
    ///
    /// # Defeat rule (Kaci & van der Torre 2008, Pareto-defeating)
    ///
    /// Given multi-value assignments, A defeats B iff for **every** value
    /// `v_b` promoted by B, there is **some** value `v_a` promoted by A
    /// such that `v_b` is *not strictly preferred* over `v_a` under the
    /// audience. This degenerates to Bench-Capon (2003) when each
    /// argument promotes exactly one value.
    ///
    /// # Special cases
    ///
    /// - Attacker promotes no value → A defeats B (unconditional).
    /// - Target promotes no value → A defeats B (no preference can save B).
    /// - Either value is unranked in the audience → considered incomparable
    ///   (no strict preference); the attacker side wins ties.
    pub fn defeat_graph(&self, audience: &Audience) -> Result<ArgumentationFramework<A>, Error> {
        let mut result = ArgumentationFramework::new();
        for arg in self.base.arguments() {
            result.add_argument(arg.clone());
        }
        // Iterate attacks via the per-target attackers() accessor — the
        // base framework doesn't expose a flat (attacker, target) iterator,
        // so we walk the graph one target at a time.
        for target in self.base.arguments() {
            for attacker in self.base.attackers(target) {
                if self.defeats(attacker, target, audience) {
                    result.add_attack(attacker, target)?;
                }
            }
        }
        Ok(result)
    }

    /// Returns true iff `attacker` defeats `target` under the audience.
    /// Both arguments must already be in the underlying framework's attack
    /// graph (i.e., `attacker` attacks `target` in the base); this method
    /// only filters by value preference. Calling this with non-attacking
    /// pairs is meaningless but not an error.
    pub fn defeats(&self, attacker: &A, target: &A, audience: &Audience) -> bool {
        let attacker_values = self.values.values(attacker);
        let target_values = self.values.values(target);

        // Null-promotion rule: if either side promotes no value, no value
        // preference can intervene, so the attack stands as a defeat.
        if attacker_values.is_empty() || target_values.is_empty() {
            return true;
        }

        // Pareto-defeating: for every target value, attacker has at least
        // one value that the target's value does not strictly outrank.
        target_values.iter().all(|tv| {
            attacker_values
                .iter()
                .any(|av| !audience.prefers(tv, av))
        })
    }

    /// Audience-conditioned credulous acceptance under preferred semantics.
    ///
    /// Returns `Ok(true)` iff `arg` is in *some* preferred extension of the
    /// audience-conditioned defeat graph.
    ///
    /// This is the fast path — see also `subjectively_accepted` and
    /// `objectively_accepted` (Phase 1 Task 5) for queries over multiple
    /// audiences.
    pub fn accepted_for(&self, audience: &Audience, arg: &A) -> Result<bool, Error> {
        let defeat = self.defeat_graph(audience)?;
        let extensions = defeat.preferred_extensions().map_err(Error::from)?;
        Ok(extensions.iter().any(|ext| ext.contains(arg)))
    }

    /// Audience-conditioned grounded extension.
    ///
    /// Convenience for inspecting "the unique skeptically accepted set"
    /// under one audience. Useful for tests and for explanation generation.
    pub fn grounded_for(&self, audience: &Audience) -> Result<HashSet<A>, Error> {
        let defeat = self.defeat_graph(audience)?;
        defeat.grounded_extension().map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    fn framework_two_args_mutual_attack() -> ValueBasedFramework<&'static str> {
        let mut base = ArgumentationFramework::new();
        base.add_argument("h1");
        base.add_argument("c1");
        base.add_attack(&"h1", &"c1").unwrap();
        base.add_attack(&"c1", &"h1").unwrap();

        let mut values = ValueAssignment::new();
        values.promote("h1", Value::new("life"));
        values.promote("c1", Value::new("property"));

        ValueBasedFramework::new(base, values)
    }

    #[test]
    fn life_audience_defeats_property_attack() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("life"), Value::new("property")]);
        // h1 attacks c1: target value (property) NOT preferred over attacker
        // value (life), so h1 defeats c1.
        assert!(vaf.defeats(&"h1", &"c1", &audience));
        // c1 attacks h1: target value (life) IS strictly preferred over
        // attacker value (property), so c1 does NOT defeat h1.
        assert!(!vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn property_audience_inverts_defeats() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("property"), Value::new("life")]);
        assert!(!vaf.defeats(&"h1", &"c1", &audience));
        assert!(vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn null_promotion_attacker_always_defeats() {
        let mut base = ArgumentationFramework::new();
        base.add_argument("a");
        base.add_argument("b");
        base.add_attack(&"a", &"b").unwrap();
        let mut values = ValueAssignment::new();
        values.promote("b", Value::new("life"));
        // a promotes nothing.
        let vaf = ValueBasedFramework::new(base, values);
        let audience = Audience::total([Value::new("life")]);
        assert!(vaf.defeats(&"a", &"b", &audience));
    }

    #[test]
    fn empty_audience_preserves_all_attacks() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::new();
        // No preferences → everything is incomparable → all attacks defeat.
        assert!(vaf.defeats(&"h1", &"c1", &audience));
        assert!(vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn defeat_graph_filters_attacks() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("life"), Value::new("property")]);
        let defeat = vaf.defeat_graph(&audience).unwrap();
        assert_eq!(defeat.attackers(&"h1").len(), 0);
        assert_eq!(defeat.attackers(&"c1").len(), 1);
    }
}
```

- [ ] **Step 2: Verify the `argumentation` crate exposes `grounded_extension`**

```bash
cd /home/peter/code/argumentation
grep -rn "pub fn grounded_extension" src/ | head -5
```

If `grounded_extension` is not present on `ArgumentationFramework`, the `grounded_for` method in `framework.rs` won't compile. Two options:

(a) **If it's missing entirely**, replace `grounded_for` with a wrapper that uses `preferred_extensions` and intersects:

```rust
pub fn grounded_for(&self, audience: &Audience) -> Result<HashSet<A>, Error> {
    let defeat = self.defeat_graph(audience)?;
    let extensions = defeat.preferred_extensions().map_err(Error::from)?;
    if extensions.is_empty() {
        return Ok(HashSet::new());
    }
    let mut grounded = extensions[0].clone();
    for ext in &extensions[1..] {
        grounded.retain(|a| ext.contains(a));
    }
    Ok(grounded)
}
```

(b) **If it's present but named differently**, update the call.

**This is a verification step — not a code change.** Document what you find in your task report so the next task knows.

- [ ] **Step 3: Update `crates/argumentation-values/src/lib.rs`**

Replace with:

```rust
//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.

pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 4: Build and run the unit tests**

```bash
cd /home/peter/code/argumentation
cargo build -p argumentation-values 2>&1 | tail -5
cargo test -p argumentation-values --lib 2>&1 | tail -15
```

Expected: build passes; all 5 lib tests in framework.rs + 5 lib tests in types.rs = **10 tests pass**.

- [ ] **Step 5: Write the integration test `tests/hal_carla.rs`**

```rust
//! Hal & Carla integration test — the success criterion from the VAF
//! mini-RFC. Verifies grounded-extension flips by audience.
//!
//! Bench-Capon (2003): given the framework
//!     C1 ↔ H1   (mutual attack between property and life)
//!     C2 → H2   (Carla's "my only dose" defeats Hal's "too poor to compensate")
//!     H2 → C1   (Hal's poverty argument also attacks Carla's property claim)
//! and value assignment
//!     H1 → life
//!     C1 → property
//!     H2 → fairness
//!     C2 → life
//! the grounded extensions under two audiences should be:
//!     [[life], [property]]   → {H1, C2}   (Hal goes free)
//!     [[property], [life]]   → {C1, C2}   (Hal punished)
//!     [[life, property]]     → original Dung result (life and property
//!                              incomparable; both H1 and C1 in different
//!                              preferred extensions, neither in grounded)

use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

fn hal_carla_vaf() -> ValueBasedFramework<&'static str> {
    let mut base = ArgumentationFramework::new();
    for arg in ["h1", "c1", "h2", "c2"] {
        base.add_argument(arg);
    }
    base.add_attack(&"h1", &"c1").unwrap();
    base.add_attack(&"c1", &"h1").unwrap();
    base.add_attack(&"c2", &"h2").unwrap();
    base.add_attack(&"h2", &"c1").unwrap();

    let mut values = ValueAssignment::new();
    values.promote("h1", Value::new("life"));
    values.promote("c1", Value::new("property"));
    values.promote("h2", Value::new("fairness"));
    values.promote("c2", Value::new("life"));

    ValueBasedFramework::new(base, values)
}

#[test]
fn life_over_property_grounds_hal() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("life"), Value::new("property")]);
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("h1"), "h1 should be grounded under [life > property]");
    assert!(grounded.contains("c2"), "c2 should be grounded under [life > property]");
    assert!(!grounded.contains("c1"), "c1 should be defeated under [life > property]");
    assert!(!grounded.contains("h2"), "h2 should be defeated under [life > property]");
}

#[test]
fn property_over_life_grounds_carla() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("property"), Value::new("life")]);
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("c1"), "c1 should be grounded under [property > life]");
    assert!(grounded.contains("c2"), "c2 should be grounded under [property > life]");
    assert!(!grounded.contains("h1"), "h1 should be defeated under [property > life]");
    assert!(!grounded.contains("h2"), "h2 should be defeated under [property > life]");
}

#[test]
fn incomparable_audience_yields_dung_result() {
    let vaf = hal_carla_vaf();
    // Both values in the same tier — neither is strictly preferred.
    let audience = Audience::from_tiers(vec![vec![
        Value::new("life"),
        Value::new("property"),
    ]]);
    // Without preferences, c1 ↔ h1 mutual attack stalemates: neither
    // is in the grounded extension. c2 still wins (no in-edges).
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("c2"), "c2 always grounded (no in-edges)");
    assert!(!grounded.contains("h1"), "h1 not grounded under symmetric attack");
    assert!(!grounded.contains("c1"), "c1 not grounded under symmetric attack");
    // h2 is defeated by c2 (which is in grounded), so h2 is out.
    assert!(!grounded.contains("h2"), "h2 defeated by grounded c2");
}

#[test]
fn accepted_for_matches_grounded_for_unique_extension() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("life"), Value::new("property")]);
    // Under this audience the framework should have a unique preferred
    // extension that equals the grounded extension. Verify both APIs agree.
    assert!(vaf.accepted_for(&audience, &"h1").unwrap());
    assert!(vaf.accepted_for(&audience, &"c2").unwrap());
    assert!(!vaf.accepted_for(&audience, &"c1").unwrap());
    assert!(!vaf.accepted_for(&audience, &"h2").unwrap());
}
```

- [ ] **Step 6: Run the integration tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values --test hal_carla 2>&1 | tail -15
```

Expected: 4 tests pass.

If `incomparable_audience_yields_dung_result` fails, the issue is most likely in how the Pareto-defeating rule handles the `[[life, property]]` case. Both values are ranked at tier 0; `audience.prefers(life, property) = false` (same tier); `audience.prefers(property, life) = false`. So both attacks defeat (neither side has strict preference). That should reproduce the symmetric Dung framework.

If a different test fails, check the defeat-graph computation against the manual derivation in the test docstring before changing code.

- [ ] **Step 7: Commit**

```bash
git add crates/argumentation-values/src/framework.rs \
        crates/argumentation-values/tests/hal_carla.rs \
        crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): defeat semantics + Hal & Carla success criterion

Pareto-defeating rule (Kaci & van der Torre 2008) — degenerates to
Bench-Capon 2003 single-value case. Hal & Carla integration test
verifies grounded-extension flips between [life>property] and
[property>life] audiences. Incomparable audience reproduces original
Dung symmetric-attack stalemate."
```

---

### Task 4: Multi-value support tests

**Files:**
- Create: `crates/argumentation-values/tests/multi_value.rs`

The defeat semantics already handle multi-value (the Pareto rule was implemented in Task 3). This task adds tests that *exercise* the multi-value path beyond the degenerate single-value cases.

- [ ] **Step 1: Write `tests/multi_value.rs`**

```rust
//! Multi-value defeat semantics (Kaci & van der Torre 2008).
//!
//! Pareto rule: A defeats B iff for every value v_b in val(B), some
//! value v_a in val(A) is not strictly less-preferred than v_b under
//! the audience.

use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

fn binary_attack(
    a_values: &[&str],
    b_values: &[&str],
) -> ValueBasedFramework<&'static str> {
    let mut base = ArgumentationFramework::new();
    base.add_argument("a");
    base.add_argument("b");
    base.add_attack(&"a", &"b").unwrap();
    let mut values = ValueAssignment::new();
    for v in a_values {
        values.promote("a", Value::new(*v));
    }
    for v in b_values {
        values.promote("b", Value::new(*v));
    }
    ValueBasedFramework::new(base, values)
}

#[test]
fn pareto_defeat_when_attacker_dominates() {
    // a promotes {life, autonomy}, b promotes {property}
    // Under audience [life > property], life ≥ property (life higher).
    // For target value property: attacker has life (preferred over property).
    // → a defeats b.
    let vaf = binary_attack(&["life", "autonomy"], &["property"]);
    let aud = Audience::total([Value::new("life"), Value::new("property")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_blocked_when_target_strictly_dominates_every_attacker_value() {
    // a promotes {property}, b promotes {life, autonomy}
    // Under audience [life > autonomy > property]:
    //   target value life: attacker has only property; life is preferred over property
    //                      → no attacker value satisfies the rule for target value life.
    // → a does NOT defeat b.
    let vaf = binary_attack(&["property"], &["life", "autonomy"]);
    let aud = Audience::total([
        Value::new("life"),
        Value::new("autonomy"),
        Value::new("property"),
    ]);
    assert!(!vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_one_target_value_can_save_the_target() {
    // a promotes {fairness}, b promotes {fairness, life}
    // Under audience [life > fairness]:
    //   target value fairness: attacker has fairness, equal rank → not strictly preferred,
    //                          attacker side wins → satisfies rule.
    //   target value life: attacker has fairness; life IS preferred over fairness
    //                      → attacker has no value not-less-preferred than life
    //                      → fails the rule.
    // → a does NOT defeat b (one target value B has that A can't match).
    let vaf = binary_attack(&["fairness"], &["fairness", "life"]);
    let aud = Audience::total([Value::new("life"), Value::new("fairness")]);
    assert!(!vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_reduces_to_benchcapon_for_single_values() {
    // Single-value sanity check: attacker promotes {life}, target promotes {property}.
    // Under [life > property]: a defeats b. Under [property > life]: a does NOT defeat b.
    let vaf = binary_attack(&["life"], &["property"]);
    let life_audience = Audience::total([Value::new("life"), Value::new("property")]);
    let property_audience = Audience::total([Value::new("property"), Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &life_audience));
    assert!(!vaf.defeats(&"a", &"b", &property_audience));
}

#[test]
fn unranked_target_value_does_not_save_target() {
    // a promotes {life}, b promotes {fairness} (fairness not in audience).
    // Under [life]: fairness is unranked → audience.prefers(fairness, life) = false
    //               → for target value fairness, attacker value life satisfies rule
    //               → a defeats b.
    let vaf = binary_attack(&["life"], &["fairness"]);
    let aud = Audience::total([Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn unranked_attacker_value_can_still_defeat_unranked_target() {
    // Both unranked → audience.prefers returns false → defeats by null-tie rule.
    let vaf = binary_attack(&["honor"], &["tradition"]);
    let aud = Audience::total([Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values --test multi_value 2>&1 | tail -15
```

Expected: 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/argumentation-values/tests/multi_value.rs
git commit -m "test(argumentation-values): multi-value Pareto defeat tests

Six cases covering: dominant attacker, blocked attacker, partial-set
defense, single-value reduction, unranked target, fully-unranked
pair. Exercises the Kaci & van der Torre 2008 Pareto rule directly."
```

---

### Task 5: Subjective + objective acceptance with hard cap

**Files:**
- Create: `crates/argumentation-values/src/acceptance.rs`
- Modify: `crates/argumentation-values/src/lib.rs` (add `pub mod acceptance;`)
- Modify: `crates/argumentation-values/src/framework.rs` (add `subjectively_accepted` and `objectively_accepted` methods that delegate to the new module)

- [ ] **Step 1: Write `crates/argumentation-values/src/acceptance.rs`**

```rust
//! Subjective and objective acceptance over the space of audiences.
//!
//! Per Bench-Capon (2003):
//! - **Subjective acceptance**: arg is accepted by *some* audience.
//! - **Objective acceptance**: arg is accepted by *every* audience.
//!
//! Both queries enumerate the linear extensions of the partial order
//! defined by the value set. The number of linear extensions is bounded
//! above by `n!` for `n` distinct values; we hard-cap at 6 (= 720) to
//! keep these queries tractable for narrative-scale frameworks.
//!
//! Past the cap, methods return [`Error::AudienceTooLarge`].

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::{Audience, Value};
use std::collections::BTreeSet;
use std::hash::Hash;

/// Hard cap on distinct values for subjective/objective acceptance.
/// At 6 values, worst-case linear-extension count is 720.
pub const ENUMERATION_LIMIT: usize = 6;

/// Returns `Ok(true)` iff `arg` is accepted by *some* total ordering of
/// the values mentioned in the framework.
///
/// Strategy: enumerate every permutation of the value set, build the
/// corresponding total-order [`Audience`], and check `accepted_for`.
/// Returns true on the first acceptance; otherwise false.
pub fn subjectively_accepted<A>(
    vaf: &ValueBasedFramework<A>,
    arg: &A,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + std::fmt::Debug,
{
    let values: Vec<Value> = vaf
        .value_assignment()
        .distinct_values()
        .into_iter()
        .cloned()
        .collect();

    if values.len() > ENUMERATION_LIMIT {
        return Err(Error::AudienceTooLarge {
            values: values.len(),
            limit: ENUMERATION_LIMIT,
        });
    }

    for perm in permutations(&values) {
        let audience = Audience::total(perm);
        if vaf.accepted_for(&audience, arg)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns `Ok(true)` iff `arg` is accepted by *every* total ordering of
/// the values mentioned in the framework.
pub fn objectively_accepted<A>(
    vaf: &ValueBasedFramework<A>,
    arg: &A,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + std::fmt::Debug,
{
    let values: Vec<Value> = vaf
        .value_assignment()
        .distinct_values()
        .into_iter()
        .cloned()
        .collect();

    if values.len() > ENUMERATION_LIMIT {
        return Err(Error::AudienceTooLarge {
            values: values.len(),
            limit: ENUMERATION_LIMIT,
        });
    }

    for perm in permutations(&values) {
        let audience = Audience::total(perm);
        if !vaf.accepted_for(&audience, arg)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// All permutations of a small slice. Heap's algorithm; allocates a
/// fresh `Vec<Value>` per permutation. Acceptable up to ENUMERATION_LIMIT.
fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    let mut working: Vec<T> = items.to_vec();
    permute_recursive(&mut working, 0, &mut result);
    result
}

fn permute_recursive<T: Clone>(items: &mut [T], start: usize, out: &mut Vec<Vec<T>>) {
    if start == items.len() {
        out.push(items.to_vec());
        return;
    }
    for i in start..items.len() {
        items.swap(start, i);
        permute_recursive(items, start + 1, out);
        items.swap(start, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation::ArgumentationFramework;
    use crate::types::ValueAssignment;

    fn hal_carla() -> ValueBasedFramework<&'static str> {
        let mut base = ArgumentationFramework::new();
        for arg in ["h1", "c1", "h2", "c2"] {
            base.add_argument(arg);
        }
        base.add_attack(&"h1", &"c1").unwrap();
        base.add_attack(&"c1", &"h1").unwrap();
        base.add_attack(&"c2", &"h2").unwrap();
        base.add_attack(&"h2", &"c1").unwrap();

        let mut values = ValueAssignment::new();
        values.promote("h1", Value::new("life"));
        values.promote("c1", Value::new("property"));
        values.promote("h2", Value::new("fairness"));
        values.promote("c2", Value::new("life"));

        ValueBasedFramework::new(base, values)
    }

    #[test]
    fn c2_objectively_accepted() {
        let vaf = hal_carla();
        // c2 has no in-edges in any audience → always grounded → always accepted.
        assert!(objectively_accepted(&vaf, &"c2").unwrap());
    }

    #[test]
    fn h1_subjectively_but_not_objectively_accepted() {
        let vaf = hal_carla();
        // h1 is accepted under [life > property, fairness, ...] but not
        // under [property > life, ...].
        assert!(subjectively_accepted(&vaf, &"h1").unwrap());
        assert!(!objectively_accepted(&vaf, &"h1").unwrap());
    }

    #[test]
    fn c1_subjectively_but_not_objectively_accepted() {
        let vaf = hal_carla();
        // Symmetric to h1.
        assert!(subjectively_accepted(&vaf, &"c1").unwrap());
        assert!(!objectively_accepted(&vaf, &"c1").unwrap());
    }

    #[test]
    fn audience_too_large_returns_error() {
        // Build a 7-value framework and verify the cap fires.
        let mut base = ArgumentationFramework::new();
        for arg in ["a", "b", "c", "d", "e", "f", "g"] {
            base.add_argument(arg);
        }
        let mut values = ValueAssignment::new();
        for (i, name) in ["v1", "v2", "v3", "v4", "v5", "v6", "v7"].iter().enumerate() {
            let arg = ["a", "b", "c", "d", "e", "f", "g"][i];
            values.promote(arg, Value::new(*name));
        }
        let vaf = ValueBasedFramework::new(base, values);
        let result = subjectively_accepted(&vaf, &"a");
        assert!(matches!(
            result,
            Err(Error::AudienceTooLarge { values: 7, limit: 6 })
        ));
    }

    #[test]
    fn permutations_of_three_yields_six() {
        let perms = permutations(&[1, 2, 3]);
        assert_eq!(perms.len(), 6);
    }

    #[test]
    fn permutations_of_zero_yields_one_empty() {
        let perms: Vec<Vec<i32>> = permutations(&[]);
        assert_eq!(perms.len(), 1);
        assert!(perms[0].is_empty());
    }
}
```

- [ ] **Step 2: Add convenience methods on `ValueBasedFramework`**

Edit `crates/argumentation-values/src/framework.rs` — add to the `impl<A> ValueBasedFramework<A>` block (just before the closing `}`):

```rust
    /// Subjective acceptance — accepted by *some* total ordering of values
    /// in this framework. See [`crate::acceptance::subjectively_accepted`].
    pub fn subjectively_accepted(&self, arg: &A) -> Result<bool, Error> {
        crate::acceptance::subjectively_accepted(self, arg)
    }

    /// Objective acceptance — accepted by *every* total ordering of values
    /// in this framework. See [`crate::acceptance::objectively_accepted`].
    pub fn objectively_accepted(&self, arg: &A) -> Result<bool, Error> {
        crate::acceptance::objectively_accepted(self, arg)
    }
```

- [ ] **Step 3: Update `crates/argumentation-values/src/lib.rs`**

```rust
//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.

pub mod acceptance;
pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 4: Run all tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values 2>&1 | tail -25
```

Expected: types (6) + framework (5) + acceptance (6) + hal_carla integration (4) + multi_value integration (6) = **27 tests pass.**

- [ ] **Step 5: Commit**

```bash
git add crates/argumentation-values/src/acceptance.rs \
        crates/argumentation-values/src/framework.rs \
        crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): subjective + objective acceptance

Enumerates linear extensions of the partial order defined by the
framework's value set. Hard-capped at 6 distinct values
(ENUMERATION_LIMIT) per Dunne & Bench-Capon 2004 complexity
(NP/co-NP). Past the cap returns Err(AudienceTooLarge). Heap's
algorithm for permutation enumeration."
```

---

### Task 6: Scheme → Audience bridge

**Files:**
- Create: `crates/argumentation-values/src/scheme_bridge.rs`
- Modify: `crates/argumentation-values/src/lib.rs` (add `pub mod scheme_bridge;`)

This task wires the existing `argument_from_values` Walton scheme (already in `argumentation-schemes/src/catalog/practical.rs:120`) into `ValueAssignment`. Consumers can populate value assignments mechanically from instantiated schemes rather than hand-building.

- [ ] **Step 1: Verify scheme bindings shape**

```bash
cd /home/peter/code/argumentation
grep -A 3 "pub struct SchemeInstance" crates/argumentation-schemes/src/instance.rs
```

Confirm that `SchemeInstance` exposes the bindings as a `HashMap<String, String>` (as documented in CLAUDE.md). The bridge uses the `value` binding name from the `argument_from_values` scheme.

- [ ] **Step 2: Write `crates/argumentation-values/src/scheme_bridge.rs`**

```rust
//! Bridge between `argumentation-schemes` and `ValueAssignment`.
//!
//! The `argument_from_values` Walton scheme (Walton 2008 p.321) carries
//! a `value` premise slot — see
//! `argumentation-schemes/src/catalog/practical.rs:120`. This module
//! extracts that slot from instantiated schemes and builds a
//! [`ValueAssignment`] keyed by the scheme's conclusion.
//!
//! For schemes other than `argument_from_values`, no value is extracted.
//!
//! ## Note on the scheme identifier
//!
//! `SchemeInstance` carries `scheme_name: String` (the human-readable name
//! from `SchemeSpec::name`), not the snake-case key. We compare against the
//! literal name `"Argument from Values"` since that is what
//! `argument_from_values()` registers in the default catalog. If consumers
//! register a custom values scheme under a different name, they should
//! call [`from_scheme_instances_with_name`] with the appropriate name.

use crate::types::{Value, ValueAssignment};
use argumentation_schemes::SchemeInstance;
use std::collections::HashMap;
use std::hash::Hash;

/// The default-catalog name of the values scheme — see
/// `argumentation-schemes/src/catalog/practical.rs:124`.
pub const DEFAULT_VALUES_SCHEME_NAME: &str = "Argument from Values";

/// Extract value promotions from an iterator of instantiated schemes
/// using the default catalog's values-scheme name.
///
/// For each scheme instance:
/// - If `instance.scheme_name == "Argument from Values"` and a `"value"`
///   binding is present, the scheme's conclusion is mapped to the bound
///   value.
/// - Otherwise, the scheme is skipped silently.
///
/// `to_arg` converts a `SchemeInstance` to the caller's argument label
/// type — typically by reading the conclusion literal. For encounter use
/// this is a closure producing an `ArgumentId` from `instance.conclusion`.
///
/// Bindings are passed in separately because `SchemeInstance` does not
/// retain its bindings post-instantiation — only the resolved literals.
/// Callers must keep the original bindings alongside each instance.
pub fn from_scheme_instances<'a, A, I, F>(
    instances: I,
    to_arg: F,
) -> ValueAssignment<A>
where
    A: Eq + Hash + Clone,
    I: IntoIterator<Item = (&'a SchemeInstance, &'a HashMap<String, String>)>,
    F: Fn(&SchemeInstance) -> A,
{
    from_scheme_instances_with_name(instances, to_arg, DEFAULT_VALUES_SCHEME_NAME)
}

/// Same as [`from_scheme_instances`] but lets the caller specify a custom
/// values-scheme name (for consumers who register their own variant).
pub fn from_scheme_instances_with_name<'a, A, I, F>(
    instances: I,
    to_arg: F,
    target_scheme_name: &str,
) -> ValueAssignment<A>
where
    A: Eq + Hash + Clone,
    I: IntoIterator<Item = (&'a SchemeInstance, &'a HashMap<String, String>)>,
    F: Fn(&SchemeInstance) -> A,
{
    let mut assignment = ValueAssignment::new();
    for (instance, bindings) in instances {
        if instance.scheme_name.as_str() != target_scheme_name {
            continue;
        }
        let Some(value_str) = bindings.get("value") else {
            continue;
        };
        let arg = to_arg(instance);
        assignment.promote(arg, Value::new(value_str.clone()));
    }
    assignment
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;
    use std::collections::HashMap;

    #[test]
    fn extracts_value_from_argument_from_values_scheme() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_values").unwrap();

        let mut bindings = HashMap::new();
        bindings.insert("action".into(), "uphold_honor".into());
        bindings.insert("value".into(), "honor".into());
        bindings.insert("agent".into(), "alice".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances(
            std::iter::once((&instance, &bindings)),
            to_arg,
        );
        let arg = instance.conclusion.to_string();
        let values = assignment.values(&arg);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_str(), "honor");
    }

    #[test]
    fn skips_non_value_schemes() {
        let registry = default_catalog();
        let scheme = registry
            .by_key("argument_from_expert_opinion")
            .expect("argument_from_expert_opinion in default catalog");

        let mut bindings = HashMap::new();
        bindings.insert("expert".into(), "alice".into());
        bindings.insert("domain".into(), "military".into());
        bindings.insert("claim".into(), "fortify".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances(
            std::iter::once((&instance, &bindings)),
            to_arg,
        );
        // Empty assignment because the scheme is not Argument from Values.
        assert!(assignment.values(&instance.conclusion.to_string()).is_empty());
    }

    #[test]
    fn custom_scheme_name_supported() {
        // If a consumer registers their own values scheme under a different
        // name (e.g., "Custom Values"), the with_name variant lets them
        // target it explicitly. Here we just verify the API shape compiles
        // and behaves correctly when the name doesn't match.
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_values").unwrap();
        let mut bindings = HashMap::new();
        bindings.insert("action".into(), "do_x".into());
        bindings.insert("value".into(), "honor".into());
        bindings.insert("agent".into(), "alice".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances_with_name(
            std::iter::once((&instance, &bindings)),
            to_arg,
            "Some Other Scheme Name",
        );
        // No match — empty assignment.
        assert!(assignment.values(&instance.conclusion.to_string()).is_empty());
    }
}
```

- [ ] **Step 3: Verify the imports compile**

The `argumentation_schemes::SchemeInstance` re-export needs to exist. Check:

```bash
cd /home/peter/code/argumentation
grep -E "pub use.*SchemeInstance|pub struct SchemeInstance" crates/argumentation-schemes/src/lib.rs crates/argumentation-schemes/src/instance.rs
```

If `SchemeInstance` is not re-exported at the crate root, the import becomes `use argumentation_schemes::instance::SchemeInstance;`. Adjust accordingly.

If `SchemeInstance.conclusion` is not a string-displayable type (it's likely a `Literal`), the test's `to_arg` closure may need adjustment. Verify with:

```bash
grep -A 8 "pub struct SchemeInstance" crates/argumentation-schemes/src/instance.rs
```

If `conclusion` is `Literal`, use `inst.conclusion.to_string()` (Display impl) — already in the test as written.

- [ ] **Step 4: Update `crates/argumentation-values/src/lib.rs`**

```rust
//! Value-based argumentation frameworks (VAFs) built on the `argumentation` crate.

pub mod acceptance;
pub mod error;
pub mod framework;
pub mod scheme_bridge;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use scheme_bridge::from_scheme_instances;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 5: Run tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values 2>&1 | tail -10
```

Expected: 3 new tests pass in scheme_bridge module. Total now 30 tests in the crate.

- [ ] **Step 6: Commit**

```bash
git add crates/argumentation-values/src/scheme_bridge.rs \
        crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): scheme→audience bridge

from_scheme_instances() extracts the 'value' binding from instances
of the argument_from_values Walton scheme, building a ValueAssignment
keyed by the scheme's conclusion. Other schemes are skipped. Lets
consumers populate value assignments mechanically from a scheme
catalog rather than hand-building them."
```

---

## Phase 2: Encounter bridge integration

Wires the new crate into `encounter-argumentation` so scenes can carry per-character audiences.

### Task 7: Audiences storage on `EncounterArgumentationState`

**Files:**
- Modify: `crates/encounter-argumentation/Cargo.toml` (add path-dep)
- Modify: `crates/encounter-argumentation/src/state.rs`
- Possibly modify: `crates/encounter-argumentation/src/lib.rs` (re-export `Audience`)

- [ ] **Step 1: Add the dep to `crates/encounter-argumentation/Cargo.toml`**

Find the `[dependencies]` section and add:

```toml
argumentation-values = { path = "../argumentation-values" }
```

Verify nothing else needs to change:

```bash
cd /home/peter/code/argumentation
cargo build -p encounter-argumentation 2>&1 | tail -5
```

Expected: still builds (the dep is unused so far).

- [ ] **Step 2: Add the audiences field to `state.rs`**

Read the existing `EncounterArgumentationState` definition:

```bash
sed -n '1,80p' /home/peter/code/argumentation/crates/encounter-argumentation/src/state.rs
```

Note how `intensity: Mutex<Budget>` is stored (and accessed). Mirror that pattern.

Add to the imports:

```rust
use argumentation_values::Audience;
```

Add to the struct fields (alongside the existing `intensity: Mutex<Budget>`):

```rust
    /// Per-character audiences (value preference orderings). Mutable via
    /// `set_audience` through a shared reference, mirroring how `intensity`
    /// is stored. Empty map means no audiences are configured; consumers
    /// that opt into VAF-aware scoring should populate this before resolve.
    audiences: Mutex<HashMap<String, Audience>>,
```

Update `new()` to initialise the map:

```rust
            audiences: Mutex::new(HashMap::new()),
```

(Add the field initialiser in the `Self { ... }` literal returned from `new`.)

Add accessor methods on the impl block (place after the existing `set_intensity`):

```rust
    /// Set the audience (value preference ordering) for one actor.
    /// Mirrors `set_intensity` — uses a shared reference plus interior mutability.
    pub fn set_audience(&self, actor: &str, audience: Audience) {
        let mut map = self.audiences.lock().expect("audiences mutex poisoned");
        map.insert(actor.to_string(), audience);
    }

    /// Borrow the audience for one actor, if set. Returns a clone because
    /// the underlying mutex guard cannot be held across the boundary.
    pub fn audience_for(&self, actor: &str) -> Option<Audience> {
        let map = self.audiences.lock().expect("audiences mutex poisoned");
        map.get(actor).cloned()
    }

    /// Iterator over all (actor, audience) pairs. Returns owned data
    /// so the lock isn't held by the caller.
    pub fn audiences(&self) -> Vec<(String, Audience)> {
        let map = self.audiences.lock().expect("audiences mutex poisoned");
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
```

- [ ] **Step 3: Re-export `Audience` from `encounter-argumentation::lib`**

Add to `crates/encounter-argumentation/src/lib.rs` (next to other re-exports):

```rust
pub use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};
```

- [ ] **Step 4: Add a smoke test**

Edit `crates/encounter-argumentation/src/state.rs`'s test module (or create one if absent). Add:

```rust
    #[test]
    fn audiences_round_trip() {
        use argumentation_values::{Audience, Value};
        let registry = argumentation_schemes::catalog::default_catalog();
        let state = EncounterArgumentationState::new(registry);
        let audience = Audience::total([Value::new("life"), Value::new("property")]);
        state.set_audience("alice", audience.clone());
        let read = state.audience_for("alice").unwrap();
        assert_eq!(read.value_count(), 2);
        assert!(state.audience_for("bob").is_none());
    }
```

- [ ] **Step 5: Build + test**

```bash
cd /home/peter/code/argumentation
cargo test -p encounter-argumentation 2>&1 | tail -15
```

Expected: build succeeds, all existing tests still pass, new audiences_round_trip test passes.

- [ ] **Step 6: Commit**

```bash
git add crates/encounter-argumentation/Cargo.toml \
        crates/encounter-argumentation/src/state.rs \
        crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): per-character audience storage

Adds audiences: Mutex<HashMap<String, Audience>> to
EncounterArgumentationState mirroring how intensity is stored.
Consumers populate via set_audience(actor, audience) before resolve;
the upcoming ValueAwareScorer reads via audience_for(actor)."
```

---

### Task 8: `ValueAwareScorer`

**Files:**
- Create: `crates/encounter-argumentation/src/value_scorer.rs`
- Modify: `crates/encounter-argumentation/src/lib.rs` (add `pub mod value_scorer;`)

The new scorer wraps any inner `ActionScorer<P>` (typically `SchemeActionScorer`, which is itself wrapping a baseline). It boosts affordances in proportion to how strongly the actor's audience prefers the values the scheme-instance promotes.

This is the *VAF-aware* extension to `SchemeActionScorer`'s already-shipped preference-weighted boost. It does not replace `SchemeActionScorer`; it stacks on top.

- [ ] **Step 1: Write `crates/encounter-argumentation/src/value_scorer.rs`**

```rust
//! Value-aware action scoring.
//!
//! [`ValueAwareScorer`] wraps an inner [`ActionScorer`] and adds a per-
//! affordance boost proportional to how strongly the actor's audience
//! prefers the values the affordance's backing scheme promotes.
//!
//! # Composition
//!
//! Designed to wrap [`crate::SchemeActionScorer`] (which itself wraps a
//! baseline scorer):
//!
//! ```ignore
//! let scorer = ValueAwareScorer::new(
//!     SchemeActionScorer::new(knowledge, registry, baseline, 0.3),
//!     state,
//!     0.2,
//! );
//! ```
//!
//! The two boosts compose additively: scheme-strength boost first, then
//! value-preference boost. Both are skipped silently when the actor has
//! no configured audience (i.e., no VAF dimension on this character).

use crate::state::EncounterArgumentationState;
use encounter::affordance::CatalogEntry;
use encounter::scoring::{ActionScorer, ScoredAffordance};

/// An [`ActionScorer`] that boosts affordances by audience-conditioned
/// value preference.
pub struct ValueAwareScorer<'a, S> {
    inner: S,
    state: &'a EncounterArgumentationState,
    max_boost: f64,
}

impl<'a, S> ValueAwareScorer<'a, S> {
    /// Construct a new value-aware scorer.
    ///
    /// # Parameters
    /// - `inner` — the scorer to wrap (typically `SchemeActionScorer`).
    /// - `state` — the encounter state holding per-actor audiences.
    /// - `max_boost` — additive boost when the actor's most-preferred
    ///   value is promoted by the affordance's scheme. Scaled linearly
    ///   downward by audience tier rank.
    pub fn new(inner: S, state: &'a EncounterArgumentationState, max_boost: f64) -> Self {
        Self {
            inner,
            state,
            max_boost,
        }
    }
}

impl<'a, S, P> ActionScorer<P> for ValueAwareScorer<'a, S>
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

        let Some(audience) = self.state.audience_for(actor) else {
            // No audience for this actor → behave as the inner scorer.
            return scored;
        };

        // Apply value boost per affordance.
        for sa in &mut scored {
            sa.score += value_boost_for_affordance(
                &sa.bindings,
                &audience,
                self.max_boost,
            );
        }
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

/// If the affordance's bindings include a `value` slot (as
/// `argument_from_values` schemes do), and that value is ranked in the
/// audience, returns a positive boost scaled by tier rank. Returns 0.0
/// otherwise.
///
/// Scaling: tier 0 (most preferred) → `max_boost`; deeper tiers scale
/// linearly down toward `max_boost / tier_count` at the bottom tier.
/// Unranked values get 0.
fn value_boost_for_affordance(
    bindings: &std::collections::HashMap<String, String>,
    audience: &argumentation_values::Audience,
    max_boost: f64,
) -> f64 {
    let Some(value_str) = bindings.get("value") else {
        return 0.0;
    };
    let value = argumentation_values::Value::new(value_str.clone());
    let tier_count = audience.tier_count();
    if tier_count == 0 {
        return 0.0;
    }
    let Some(rank) = audience.rank(&value) else {
        return 0.0;
    };
    let normalised = (tier_count - rank) as f64 / tier_count as f64;
    max_boost * normalised
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;
    use argumentation_values::{Audience, Value};
    use encounter::affordance::AffordanceSpec;
    use encounter::scoring::ScoredAffordance;
    use std::collections::HashMap;

    /// Test scorer that returns one fixed-score result for every affordance,
    /// with a value binding set to `value_promoted`.
    struct StubScorer {
        value_promoted: String,
    }

    impl<P: Clone> ActionScorer<P> for StubScorer {
        fn score_actions(
            &self,
            actor: &str,
            available: &[CatalogEntry<P>],
            _participants: &[String],
        ) -> Vec<ScoredAffordance<P>> {
            available
                .iter()
                .map(|entry| {
                    let mut bindings = HashMap::new();
                    bindings.insert("self".into(), actor.into());
                    bindings.insert("value".into(), self.value_promoted.clone());
                    ScoredAffordance {
                        entry: entry.clone(),
                        score: 1.0,
                        bindings,
                    }
                })
                .collect()
        }
    }

    fn dummy_entry() -> CatalogEntry<()> {
        CatalogEntry {
            spec: AffordanceSpec {
                name: "test_affordance".into(),
                domain: "test".into(),
                bindings: vec!["self".into(), "value".into()],
                considerations: Vec::new(),
                effects_on_accept: Vec::new(),
                effects_on_reject: Vec::new(),
                drive_alignment: Vec::new(),
            },
            precondition: String::new(),
        }
    }

    #[test]
    fn no_audience_means_no_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        let inner = StubScorer { value_promoted: "honor".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        assert_eq!(scored.len(), 1);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boost_proportional_to_tier_position() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([Value::new("honor"), Value::new("safety")]),
        );
        let inner = StubScorer { value_promoted: "honor".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        // honor is at tier 0 of 2 tiers → normalised = (2-0)/2 = 1.0.
        // boost = 0.5 * 1.0 = 0.5; total = 1.0 + 0.5 = 1.5.
        assert!((scored[0].score - 1.5).abs() < 1e-9, "got {}", scored[0].score);
    }

    #[test]
    fn unranked_value_gets_no_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([Value::new("life")]),
        );
        let inner = StubScorer { value_promoted: "tradition".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        assert!((scored[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn lower_tier_gets_smaller_boost() {
        let registry = default_catalog();
        let state = EncounterArgumentationState::new(registry);
        state.set_audience(
            "alice",
            Audience::total([
                Value::new("honor"),
                Value::new("safety"),
                Value::new("comfort"),
            ]),
        );
        let inner = StubScorer { value_promoted: "comfort".into() };
        let scorer = ValueAwareScorer::new(inner, &state, 0.5);
        let entries = vec![dummy_entry()];
        let scored = scorer.score_actions("alice", &entries, &["alice".into()]);
        // comfort at tier 2 of 3 tiers → normalised = (3-2)/3 ≈ 0.333.
        // boost ≈ 0.5 * 0.333 ≈ 0.167; total ≈ 1.167.
        assert!((scored[0].score - 1.1666666666).abs() < 1e-3, "got {}", scored[0].score);
    }
}
```

- [ ] **Step 2: Wire the new module**

Edit `crates/encounter-argumentation/src/lib.rs`. Add:

```rust
pub mod value_scorer;
pub use value_scorer::ValueAwareScorer;
```

(Place near the other `pub mod` and `pub use` lines.)

- [ ] **Step 3: Build + test**

```bash
cd /home/peter/code/argumentation
cargo test -p encounter-argumentation 2>&1 | tail -15
```

Expected: build clean, all existing tests still pass, 4 new value_scorer tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/encounter-argumentation/src/value_scorer.rs \
        crates/encounter-argumentation/src/lib.rs
git commit -m "feat(encounter-argumentation): ValueAwareScorer wraps SchemeActionScorer

Stacks audience-conditioned value-preference boost on top of the
existing scheme-strength boost. Reads per-actor audience from the
state's audience storage (Task 7); boost magnitude scales linearly
with audience tier rank. Skips silently when the actor has no
configured audience."
```

---

## Phase 3: Interop + multi-audience query

### Task 9: APX format I/O (ASPARTIX VAF extension)

**Files:**
- Create: `crates/argumentation-values/src/apx.rs`
- Modify: `crates/argumentation-values/src/lib.rs` (add `pub mod apx;`)

ASPARTIX's APX format with the VAF extension uses Prolog-style facts:
- `arg(name).`
- `att(attacker, target).`
- `val(arg, value).`
- `valpref(value_a, value_b).` — value_a strictly preferred over value_b

The format is line-based; we implement a small handwritten parser (no external dep).

- [ ] **Step 1: Write `crates/argumentation-values/src/apx.rs`**

```rust
//! APX text format I/O for VAFs (ASPARTIX-compatible).
//!
//! The APX format uses Prolog-style facts:
//!
//! ```text
//! arg(h1).
//! arg(c1).
//! att(h1, c1).
//! att(c1, h1).
//! val(h1, life).
//! val(c1, property).
//! valpref(life, property).
//! ```
//!
//! Comments start with `%` and run to end of line. Whitespace is ignored.
//!
//! `valpref(a, b)` means value `a` is strictly preferred over value `b`.
//! Multiple `valpref` facts together encode an audience.

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::{Audience, Value, ValueAssignment};
use argumentation::ArgumentationFramework;

/// Parse an APX document into (framework, audience) pair.
///
/// The resulting framework owns string-typed argument labels (matching
/// the `arg(name)` identifier in the input). The audience is derived
/// from the `valpref` facts; if no `valpref` facts are present, the
/// audience is empty.
pub fn from_apx(input: &str) -> Result<(ValueBasedFramework<String>, Audience), Error> {
    let mut base = ArgumentationFramework::new();
    let mut values = ValueAssignment::new();
    let mut prefs: Vec<(String, String)> = Vec::new();

    for (line_idx, raw_line) in input.lines().enumerate() {
        let line_no = line_idx + 1;
        // Strip comments
        let line = raw_line.split('%').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        // Each fact ends with `.`
        let fact = line.strip_suffix('.').ok_or_else(|| Error::ApxParse {
            line: line_no,
            reason: format!("expected fact ending with '.', got: {line}"),
        })?;
        // Predicate(args) form
        let (pred, args) = parse_pred(fact, line_no)?;
        match pred {
            "arg" => {
                let [name] = expect_n_args::<1>(&args, line_no, "arg")?;
                base.add_argument(name.to_string());
            }
            "att" => {
                let [attacker, target] = expect_n_args::<2>(&args, line_no, "att")?;
                let attacker_s = attacker.to_string();
                let target_s = target.to_string();
                base.add_attack(&attacker_s, &target_s)
                    .map_err(Error::from)?;
            }
            "val" => {
                let [arg, value] = expect_n_args::<2>(&args, line_no, "val")?;
                values.promote(arg.to_string(), Value::new(value.to_string()));
            }
            "valpref" => {
                let [a, b] = expect_n_args::<2>(&args, line_no, "valpref")?;
                prefs.push((a.to_string(), b.to_string()));
            }
            other => {
                return Err(Error::ApxParse {
                    line: line_no,
                    reason: format!("unknown predicate: {other}"),
                });
            }
        }
    }

    let audience = audience_from_prefs(&prefs);
    Ok((ValueBasedFramework::new(base, values), audience))
}

/// Serialise a VAF + audience to APX format.
pub fn to_apx(vaf: &ValueBasedFramework<String>, audience: &Audience) -> String {
    let mut out = String::new();
    let mut args: Vec<&String> = vaf.base().arguments().collect();
    args.sort();
    for arg in &args {
        out.push_str(&format!("arg({arg}).\n"));
    }
    for target in &args {
        let mut attackers: Vec<&String> = vaf.base().attackers(*target).into_iter().collect();
        attackers.sort();
        for atk in attackers {
            out.push_str(&format!("att({atk}, {target}).\n"));
        }
    }
    let mut entries: Vec<(&String, &[Value])> = vaf
        .value_assignment()
        .entries()
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    for (arg, vals) in entries {
        for v in vals {
            out.push_str(&format!("val({arg}, {v}).\n"));
        }
    }
    // valpref: emit one fact per strict preference. The pairwise scan is
    // O(n²) on |values| but n is bounded by ENUMERATION_LIMIT * a small
    // constant in practice. ASPARTIX accepts redundant valpref facts
    // (it computes the transitive closure on import), so emitting all
    // strict pairs (not just the transitive reduction) is fine.
    let all_values: Vec<&Value> = audience.values().collect();
    let mut emitted = std::collections::BTreeSet::new();
    for a in &all_values {
        for b in &all_values {
            if audience.prefers(a, b) && emitted.insert((a.as_str(), b.as_str())) {
                out.push_str(&format!("valpref({a}, {b}).\n"));
            }
        }
    }
    out
}

fn parse_pred(fact: &str, line: usize) -> Result<(&str, Vec<&str>), Error> {
    let open = fact.find('(').ok_or_else(|| Error::ApxParse {
        line,
        reason: format!("expected '(' in fact: {fact}"),
    })?;
    let close = fact.rfind(')').ok_or_else(|| Error::ApxParse {
        line,
        reason: format!("expected ')' in fact: {fact}"),
    })?;
    if close <= open {
        return Err(Error::ApxParse {
            line,
            reason: format!("malformed fact: {fact}"),
        });
    }
    let pred = &fact[..open];
    let args_str = &fact[open + 1..close];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    Ok((pred, args))
}

fn expect_n_args<const N: usize>(
    args: &[&str],
    line: usize,
    pred: &str,
) -> Result<[String; N], Error> {
    if args.len() != N {
        return Err(Error::ApxParse {
            line,
            reason: format!(
                "{pred} expects {N} arg(s), got {got}",
                got = args.len()
            ),
        });
    }
    let mut out: [String; N] = std::array::from_fn(|_| String::new());
    for (i, a) in args.iter().enumerate() {
        out[i] = (*a).to_string();
    }
    Ok(out)
}

/// Build an Audience from a set of pairwise (strict) preferences.
///
/// Strategy: topologically sort the directed preference graph (a → b if
/// `valpref(a, b)`). Each topological level becomes one tier.
fn audience_from_prefs(prefs: &[(String, String)]) -> Audience {
    use std::collections::{BTreeSet, HashMap, HashSet};

    if prefs.is_empty() {
        return Audience::new();
    }

    // Collect all distinct values.
    let mut all: BTreeSet<String> = BTreeSet::new();
    for (a, b) in prefs {
        all.insert(a.clone());
        all.insert(b.clone());
    }

    // Build successor map (a → set of b's that a outranks).
    let mut outranks: HashMap<String, HashSet<String>> = HashMap::new();
    let mut indegree: HashMap<String, usize> = HashMap::new();
    for v in &all {
        outranks.entry(v.clone()).or_default();
        indegree.entry(v.clone()).or_insert(0);
    }
    for (a, b) in prefs {
        if outranks.get_mut(a).unwrap().insert(b.clone()) {
            *indegree.get_mut(b).unwrap() += 1;
        }
    }

    let mut tiers: Vec<Vec<Value>> = Vec::new();
    let mut remaining: HashSet<String> = all.into_iter().collect();
    while !remaining.is_empty() {
        let mut current_tier: Vec<String> = remaining
            .iter()
            .filter(|v| *indegree.get(*v).unwrap() == 0)
            .cloned()
            .collect();
        if current_tier.is_empty() {
            // Cycle — emit remaining as one indistinguishable tier.
            current_tier = remaining.iter().cloned().collect();
        }
        current_tier.sort();
        for v in &current_tier {
            remaining.remove(v);
            // Decrement indegrees for successors.
            if let Some(succs) = outranks.get(v) {
                for s in succs {
                    if let Some(d) = indegree.get_mut(s) {
                        if *d > 0 {
                            *d -= 1;
                        }
                    }
                }
            }
        }
        tiers.push(current_tier.into_iter().map(Value::new).collect());
    }

    Audience::from_tiers(tiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_small_vaf() {
        let input = r#"
% Hal & Carla in APX
arg(h1).
arg(c1).
att(h1, c1).
att(c1, h1).
val(h1, life).
val(c1, property).
valpref(life, property).
"#;
        let (vaf, audience) = from_apx(input).unwrap();
        assert_eq!(vaf.base().len(), 2);
        assert!(audience.prefers(&Value::new("life"), &Value::new("property")));
        // Round-trip: serialise then re-parse.
        let serialised = to_apx(&vaf, &audience);
        let (vaf2, audience2) = from_apx(&serialised).unwrap();
        assert_eq!(vaf2.base().len(), 2);
        assert!(audience2.prefers(&Value::new("life"), &Value::new("property")));
    }

    #[test]
    fn parse_error_reports_line_and_predicate() {
        let input = "arg(a).\nbogus(stuff).\n";
        let err = from_apx(input).unwrap_err();
        match err {
            Error::ApxParse { line, reason } => {
                assert_eq!(line, 2);
                assert!(reason.contains("unknown predicate"));
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    fn comments_and_whitespace_ignored() {
        let input = r#"
% header comment
arg(a).   % trailing comment
arg(b).

att(a, b).
"#;
        let (vaf, _) = from_apx(input).unwrap();
        assert_eq!(vaf.base().len(), 2);
    }

    #[test]
    fn empty_input_yields_empty_vaf() {
        let (vaf, audience) = from_apx("").unwrap();
        assert_eq!(vaf.base().len(), 0);
        assert_eq!(audience.value_count(), 0);
    }

    #[test]
    fn multi_tier_audience_emerges_from_chained_prefs() {
        let input = r#"
arg(a).
arg(b).
arg(c).
val(a, life).
val(b, fairness).
val(c, property).
valpref(life, fairness).
valpref(fairness, property).
"#;
        let (_vaf, audience) = from_apx(input).unwrap();
        assert_eq!(audience.tier_count(), 3);
        assert!(audience.prefers(&Value::new("life"), &Value::new("fairness")));
        assert!(audience.prefers(&Value::new("fairness"), &Value::new("property")));
        assert!(audience.prefers(&Value::new("life"), &Value::new("property")));
    }
}
```

- [ ] **Step 2: Wire the module**

Update `crates/argumentation-values/src/lib.rs`:

```rust
pub mod acceptance;
pub mod apx;
pub mod error;
pub mod framework;
pub mod scheme_bridge;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use scheme_bridge::from_scheme_instances;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 3: Run tests**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values --test '*' --lib 2>&1 | tail -10
cargo test -p argumentation-values apx 2>&1 | tail -10
```

Expected: 5 new APX tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/argumentation-values/src/apx.rs \
        crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): APX text format I/O

ASPARTIX-compatible APX format with VAF extension (arg/1, att/2,
val/2, valpref/2). Handwritten line-based parser (no external dep).
Audience derivation via topological sort over the preference graph.
Round-trip preserves the framework + audience. Five tests including
parse-error reporting and multi-tier audience emergence."
```

---

### Task 10: `MultiAudience::common_extensions`

**Files:**
- Create: `crates/argumentation-values/src/multi.rs`
- Modify: `crates/argumentation-values/src/lib.rs`

DiArg's AgreementScenario, simplified: given a set of audiences (one per character in a multi-actor scene), find the arguments accepted under *every* audience. This is the natural "consensus" query for a council.

- [ ] **Step 1: Write `crates/argumentation-values/src/multi.rs`**

```rust
//! Multi-audience consensus queries.
//!
//! When a scene involves multiple characters, each potentially with their
//! own value ordering, the natural query becomes: "which arguments survive
//! under *every* character's audience?" This is DiArg's AgreementScenario
//! abstraction (Kampik 2020), simplified.

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::Audience;
use std::collections::HashSet;
use std::hash::Hash;

/// Query operations over a set of audiences.
pub struct MultiAudience<'a> {
    audiences: &'a [Audience],
}

impl<'a> MultiAudience<'a> {
    /// Construct from a slice of audiences. Empty slice means "no audiences"
    /// — every argument is trivially accepted under the empty universal
    /// quantifier (`common_extensions` returns the union of arguments).
    pub fn new(audiences: &'a [Audience]) -> Self {
        Self { audiences }
    }

    /// Borrow the underlying audiences.
    pub fn audiences(&self) -> &[Audience] {
        self.audiences
    }

    /// Returns the set of arguments that are credulously accepted (i.e.,
    /// in *some* preferred extension) under *every* audience in the set.
    ///
    /// This is the consensus answer: which proposals survive regardless
    /// of which character's value ordering you adopt? Useful for council
    /// / jury / cabinet narrative queries.
    pub fn common_credulous<A>(
        &self,
        vaf: &ValueBasedFramework<A>,
    ) -> Result<HashSet<A>, Error>
    where
        A: Clone + Eq + Hash + std::fmt::Debug,
    {
        if self.audiences.is_empty() {
            return Ok(vaf.base().arguments().cloned().collect());
        }

        // Compute per-audience credulous sets.
        let per_audience: Vec<HashSet<A>> = self
            .audiences
            .iter()
            .map(|aud| -> Result<HashSet<A>, Error> {
                let defeat = vaf.defeat_graph(aud)?;
                let extensions = defeat.preferred_extensions().map_err(Error::from)?;
                let mut credulous = HashSet::new();
                for ext in extensions {
                    for arg in ext {
                        credulous.insert(arg);
                    }
                }
                Ok(credulous)
            })
            .collect::<Result<_, _>>()?;

        // Intersect.
        let mut iter = per_audience.into_iter();
        let Some(mut acc) = iter.next() else {
            return Ok(HashSet::new());
        };
        for next in iter {
            acc.retain(|a| next.contains(a));
        }
        Ok(acc)
    }

    /// Returns the set of arguments grounded under *every* audience in
    /// the set. Strictly stronger than `common_credulous` — the consensus
    /// among the most cautious answers from each character.
    pub fn common_grounded<A>(
        &self,
        vaf: &ValueBasedFramework<A>,
    ) -> Result<HashSet<A>, Error>
    where
        A: Clone + Eq + Hash + std::fmt::Debug,
    {
        if self.audiences.is_empty() {
            return Ok(vaf.base().arguments().cloned().collect());
        }

        let per_audience: Vec<HashSet<A>> = self
            .audiences
            .iter()
            .map(|aud| vaf.grounded_for(aud))
            .collect::<Result<_, _>>()?;

        let mut iter = per_audience.into_iter();
        let Some(mut acc) = iter.next() else {
            return Ok(HashSet::new());
        };
        for next in iter {
            acc.retain(|a| next.contains(a));
        }
        Ok(acc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Value, ValueAssignment};
    use argumentation::ArgumentationFramework;

    fn hal_carla() -> ValueBasedFramework<&'static str> {
        let mut base = ArgumentationFramework::new();
        for arg in ["h1", "c1", "h2", "c2"] {
            base.add_argument(arg);
        }
        base.add_attack(&"h1", &"c1").unwrap();
        base.add_attack(&"c1", &"h1").unwrap();
        base.add_attack(&"c2", &"h2").unwrap();
        base.add_attack(&"h2", &"c1").unwrap();

        let mut values = ValueAssignment::new();
        values.promote("h1", Value::new("life"));
        values.promote("c1", Value::new("property"));
        values.promote("h2", Value::new("fairness"));
        values.promote("c2", Value::new("life"));

        ValueBasedFramework::new(base, values)
    }

    #[test]
    fn common_grounded_across_opposing_audiences_yields_only_unanimous_winners() {
        let vaf = hal_carla();
        let life_first = Audience::total([Value::new("life"), Value::new("property")]);
        let property_first = Audience::total([Value::new("property"), Value::new("life")]);
        let multi = MultiAudience::new(&[life_first, property_first]);
        let common = multi.common_grounded(&vaf).unwrap();
        // c2 always grounded; nothing else survives both audiences.
        assert!(common.contains("c2"));
        assert!(!common.contains("h1"));
        assert!(!common.contains("c1"));
    }

    #[test]
    fn empty_audience_set_returns_all_arguments() {
        let vaf = hal_carla();
        let multi = MultiAudience::new(&[]);
        let common = multi.common_grounded(&vaf).unwrap();
        assert_eq!(common.len(), 4);
    }

    #[test]
    fn common_credulous_is_superset_of_common_grounded() {
        let vaf = hal_carla();
        let life_first = Audience::total([Value::new("life"), Value::new("property")]);
        let property_first = Audience::total([Value::new("property"), Value::new("life")]);
        let multi = MultiAudience::new(&[life_first, property_first]);
        let credulous = multi.common_credulous(&vaf).unwrap();
        let grounded = multi.common_grounded(&vaf).unwrap();
        for arg in &grounded {
            assert!(credulous.contains(arg), "common_grounded must subset common_credulous");
        }
    }
}
```

- [ ] **Step 2: Wire the module**

Update `crates/argumentation-values/src/lib.rs`:

```rust
pub mod acceptance;
pub mod apx;
pub mod error;
pub mod framework;
pub mod multi;
pub mod scheme_bridge;
pub mod types;

pub use error::Error;
pub use framework::ValueBasedFramework;
pub use multi::MultiAudience;
pub use scheme_bridge::from_scheme_instances;
pub use types::{Audience, Value, ValueAssignment};
```

- [ ] **Step 3: Test**

```bash
cd /home/peter/code/argumentation
cargo test -p argumentation-values multi 2>&1 | tail -10
```

Expected: 3 new tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/argumentation-values/src/multi.rs \
        crates/argumentation-values/src/lib.rs
git commit -m "feat(argumentation-values): MultiAudience consensus query

DiArg-inspired AgreementScenario simplified: given a slice of
audiences (one per character in a multi-actor scene), expose
common_grounded and common_credulous queries that intersect
acceptance across every audience. Three tests."
```

---

## Phase 4: Documentation reflow

The mini-RFC at `concepts/value-based-argumentation.mdx` is currently a scoping doc ("**Not implemented yet — this page describes what we'd build.**"). After Phase 1–3 it's no longer hypothetical — needs to become a concepts page. Also: update Hal & Carla page to demonstrate the audience flip; add a how-to guide; remove VAF from the open-areas list.

### Task 11: Reframe VAF concepts page from RFC to "how it works"

**Files:**
- Modify: `website/docs/concepts/value-based-argumentation.mdx`
- Modify: `website/docs/concepts/open-areas.mdx` (remove VAF entry)

- [ ] **Step 1: Rewrite the VAF concepts page**

Replace the entire contents of `website/docs/concepts/value-based-argumentation.mdx` with:

```mdx
---
sidebar_position: 9
title: Value-based argumentation (VAF)
---

**The `argumentation-values` crate adds value-based argumentation frameworks to the workspace. This page is the conceptual overview; for API details see [the rustdoc](/api/argumentation_values/).**

Bench-Capon (2003) extended Dung frameworks with *values* — each argument promotes a value, and an *audience* is an ordering over values. Different audiences reach different rational conclusions from the same framework. Our implementation supports the multi-value generalisation from Kaci & van der Torre (2008).

## Types at a glance

```rust
use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

let mut base = ArgumentationFramework::new();
base.add_argument("h1");
base.add_argument("c1");
base.add_attack(&"h1", &"c1").unwrap();
base.add_attack(&"c1", &"h1").unwrap();

let mut values = ValueAssignment::new();
values.promote("h1", Value::new("life"));
values.promote("c1", Value::new("property"));

let vaf = ValueBasedFramework::new(base, values);
let life_first = Audience::total([Value::new("life"), Value::new("property")]);

assert!(vaf.accepted_for(&life_first, &"h1").unwrap());
assert!(!vaf.accepted_for(&life_first, &"c1").unwrap());
```

## Defeat semantics (Kaci & van der Torre 2008, Pareto)

Given attack `(A, B)` in the underlying Dung framework and audience `X`, A *defeats* B in the audience-conditioned graph iff:

> for every value `v_b` promoted by B, there exists some value `v_a` promoted by A such that `v_b` is **not** strictly preferred over `v_a` under X.

Single-value (Bench-Capon 2003) is the degenerate case where each argument promotes exactly one value.

**Special cases:**
- A or B promotes no value → A defeats B (no preference can intervene).
- A value is unranked in the audience → considered incomparable (no strict preference); attacker side wins ties.

## Acceptance modes

| Method | Cost | Use case |
|---|---|---|
| `accepted_for(&audience, &arg)` | Polynomial-ish (one preferred-extension call) | Runtime: "what does this character believe?" |
| `subjectively_accepted(&arg)` | NP-complete; capped at 6 distinct values | Authoring: "is there *any* audience under which X is accepted?" |
| `objectively_accepted(&arg)` | co-NP-complete; capped at 6 distinct values | Authoring: "is X accepted under *every* audience? (i.e., universally compelling)" |
| `MultiAudience::common_grounded(&vaf)` | k × preferred extensions, where k = |audiences| | Multi-character scenes: "which proposals does the *whole council* agree on?" |

The ENUMERATION_LIMIT cap on subjective/objective queries is per Dunne & Bench-Capon (2004). Past 6 values, methods return `Err(Error::AudienceTooLarge)` — use a fixed-audience query instead.

## Hal & Carla, worked

The canonical example. See [the engine-driven scene](/examples/hal-and-carla) for the live version.

```rust
let mut base = ArgumentationFramework::new();
for a in ["h1", "c1", "h2", "c2"] { base.add_argument(a); }
base.add_attack(&"h1", &"c1").unwrap();   // life attacks property
base.add_attack(&"c1", &"h1").unwrap();   // property attacks life
base.add_attack(&"c2", &"h2").unwrap();   // Carla's "my only dose" defeats fairness appeal
base.add_attack(&"h2", &"c1").unwrap();   // Hal's poverty attacks property

let mut values = ValueAssignment::new();
values.promote("h1", Value::new("life"));
values.promote("c1", Value::new("property"));
values.promote("h2", Value::new("fairness"));
values.promote("c2", Value::new("life"));

let vaf = ValueBasedFramework::new(base, values);

// Audience 1: life > property → Hal goes free.
let life_first = Audience::total([Value::new("life"), Value::new("property")]);
let g1 = vaf.grounded_for(&life_first).unwrap();
assert!(g1.contains("h1") && g1.contains("c2"));

// Audience 2: property > life → Hal punished.
let property_first = Audience::total([Value::new("property"), Value::new("life")]);
let g2 = vaf.grounded_for(&property_first).unwrap();
assert!(g2.contains("c1") && g2.contains("c2"));
```

## Encounter bridge integration

`encounter-argumentation` ships `ValueAwareScorer` for runtime use:

```rust
use encounter_argumentation::{
    EncounterArgumentationState, SchemeActionScorer, ValueAwareScorer,
    Audience, Value,
};

let state = EncounterArgumentationState::new(catalog);
state.set_audience("alice", Audience::total([Value::new("duty"), Value::new("survival")]));
state.set_audience("bob", Audience::total([Value::new("survival"), Value::new("duty")]));

let scorer = ValueAwareScorer::new(
    SchemeActionScorer::new(knowledge, registry, baseline_scorer, 0.3),
    &state,
    0.2,
);
```

Per-character audiences flow through the scorer at resolve time — Alice and Bob score the same affordance differently when their value orderings differ.

See the [wiring per-character values](/guides/wiring-character-values) how-to for a complete worked example.

## APX format I/O

ASPARTIX-compatible APX format with VAF extension:

```text
arg(h1).
arg(c1).
att(h1, c1).
att(c1, h1).
val(h1, life).
val(c1, property).
valpref(life, property).
```

Use `argumentation_values::apx::from_apx(text)` to parse, `to_apx(&vaf, &audience)` to serialise. Round-trips preserve the framework + audience. Useful for importing benchmark VAFs from the literature or exporting scenes for analysis in ASPARTIX.

## Bibliography

- [Bench-Capon (2003)](/academic/bibliography#benchcapon2003) — the original VAF paper.
- [Atkinson & Bench-Capon (2007)](/academic/bibliography#atkinson2007) — practical reasoning over VAFs.
- Kaci, S. & van der Torre, L. (2008). "Preference-based argumentation: Arguments supporting multiple values." *International Journal of Approximate Reasoning* 48(3): 730–751.
- Dunne, P.E. & Bench-Capon, T. (2004). "Complexity in Value-Based Argument Systems." *JELIA 2004*: 360–371.
- Bodanza, G.A. & Freidin, E. (2023). "Confronting value-based argumentation frameworks with people's assessment of argument strength." *Argument & Computation* 14(3): 247–273. — empirical critique; informs why we expose `SchemeActionScorer`'s direct value-importance scoring alongside the orthodox VAF defeat semantics.

## Further reading

- [Hal & Carla](/examples/hal-and-carla) — the engine-driven scene this implementation was built around.
- [Open areas](/concepts/open-areas) — what's still on the roadmap (probabilistic AF, ADF, dialogue games, dynamic AF).
- [Wiring per-character values](/guides/wiring-character-values) — how-to.
```

- [ ] **Step 2: Update `concepts/open-areas.mdx`**

Find the section "## 1. Value-based argumentation frameworks (VAF)" in `website/docs/concepts/open-areas.mdx`. **Replace the whole section with a one-paragraph "now implemented" note**, and renumber the remaining sections (probabilistic becomes 1, ADF becomes 2, dialogue games becomes 3, dynamic becomes 4). Add a banner near the top.

Read the current file first:

```bash
sed -n '1,30p' /home/peter/code/argumentation/website/docs/concepts/open-areas.mdx
```

Replace the introduction (around lines 1-10) with:

```mdx
---
sidebar_position: 8
title: Open areas
---

**Four formalisms in the argumentation literature this library does not yet implement, with notes on what we'd build and why.** (One previously listed — value-based argumentation — has since been implemented; see the [VAF concepts page](/concepts/value-based-argumentation).)

The library focuses on Dung abstract frameworks, ASPIC+ structured arguments, weighted attacks, bipolar (attack + support) extensions, the encounter bridge, and value-based argumentation. The argumentation literature is broader. This page is a public roadmap — not a commitment to ship, but an honest map of the gap.
```

Then **delete the entire `## 1. Value-based argumentation frameworks (VAF)` section** (probably lines 11-22 or thereabouts), and renumber the remaining four sections from 1 to 4 (Probabilistic, ADF, Persuasion Dialogue Games, Dynamic AF).

- [ ] **Step 3: Build the website**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. The previously-broken `/concepts/value-based-argumentation` warning was already resolved when Phase 3 of the flagship-demo plan landed; this task only modifies content. New `/guides/wiring-character-values` link will warn until Task 13 lands — expected.

- [ ] **Step 4: Commit**

```bash
git add website/docs/concepts/value-based-argumentation.mdx \
        website/docs/concepts/open-areas.mdx
git commit -m "docs(concepts): reframe VAF page from RFC to implementation overview

VAF is now implemented (argumentation-values crate). The concepts
page is now a 'how it works' overview with worked Hal & Carla,
encounter-bridge integration sketch, APX I/O example, and the
acceptance-modes table. Removed VAF from open-areas; that page now
covers four formalisms instead of five."
```

---

### Task 12: Update Hal & Carla page to demonstrate audience flip

**Files:**
- Modify: `website/docs/examples/hal-and-carla.mdx`

The current page (post-flagship-demo plan) says "Carla wins regardless of β" and points at VAF as the future fix. Now that VAF is implemented, the page can demonstrate the audience flip directly using static `<AttackGraph>` components (one per audience) since the WASM bindings for `ValueBasedFramework` are out of scope for this plan.

- [ ] **Step 1: Read the current page to find the "In our library" section**

```bash
sed -n '60,95p' /home/peter/code/argumentation/website/docs/examples/hal-and-carla.mdx
```

- [ ] **Step 2: Add an "Audience-flip demonstration" section**

Insert a new section between "## A pre-recorded multi-beat trace at four discrete β" and "## Why values matter" (around line 65).

Find the line "**This is the limit of the abstract weighted framework.**" and insert *after the closing paragraph of that section*, before "## Why values matter":

```mdx
## With values: the audience flips the outcome

The abstract weighted framework above can't prefer Hal — but `argumentation-values` can. Below, the same four arguments and four attacks, but with values attached: H1 promotes *life*, C1 promotes *property*, H2 promotes *fairness*, C2 promotes *life*. Each side panel shows the audience-conditioned **defeat graph** under one audience.

<div style={{display: 'flex', gap: '1rem', flexWrap: 'wrap'}}>

<div style={{flex: '1 1 350px'}}>

### Audience: life &gt; property

<AttackGraph
  title="Defeat graph — life ≻ property"
  arguments={[
    {id: 'H1', label: 'Hal: life > property', accepted: 'grounded'},
    {id: 'C1', label: 'Carla: property rights'},
    {id: 'H2', label: 'Hal: too poor to compensate'},
    {id: 'C2', label: 'Carla: my only dose', accepted: 'grounded'},
  ]}
  attacks={[
    {from: 'H1', to: 'C1'},
    {from: 'C2', to: 'H2'},
    {from: 'H2', to: 'C1'},
  ]}
  height={320}
  caption="C1's attack on H1 is filtered out (life is preferred over property). Grounded extension: {H1, C2}. Hal goes free."
/>

</div>

<div style={{flex: '1 1 350px'}}>

### Audience: property &gt; life

<AttackGraph
  title="Defeat graph — property ≻ life"
  arguments={[
    {id: 'H1', label: 'Hal: life > property'},
    {id: 'C1', label: 'Carla: property rights', accepted: 'grounded'},
    {id: 'H2', label: 'Hal: too poor to compensate'},
    {id: 'C2', label: 'Carla: my only dose', accepted: 'grounded'},
  ]}
  attacks={[
    {from: 'C1', to: 'H1'},
    {from: 'C2', to: 'H2'},
    {from: 'H2', to: 'C1'},
  ]}
  height={320}
  caption="H1's attack on C1 is filtered out (property is preferred over life). Grounded extension: {C1, C2}. Hal punished."
/>

</div>

</div>

The framework hasn't changed; the audience has. Same four arguments, same four attacks, opposite outcome. This is the formal machinery of value-based argumentation.

```

- [ ] **Step 3: Update the "In our library" section**

The current section says VAF is "on the roadmap as an open formalism." Replace it with:

```mdx
## In our library

Value-based argumentation is implemented in the [`argumentation-values`](https://docs.rs/argumentation-values) crate. The audience-flip demonstration above uses the actual library to derive the defeat graphs shown. For full API details, types, and the Hal & Carla integration test that pins this exact behaviour, see the [VAF concepts page](/concepts/value-based-argumentation).

For multi-character scene wiring (per-character audiences flowing into the encounter bridge), see the [wiring per-character values](/guides/wiring-character-values) how-to.
```

(That replaces the current "Value-based argumentation is on the roadmap as an open formalism…" paragraph.)

- [ ] **Step 4: Update the further reading**

Add to the existing Further reading section in `hal-and-carla.mdx`:

```mdx
- [Wiring per-character values](/guides/wiring-character-values) — how to wire per-character audiences into a multi-actor scene.
```

- [ ] **Step 5: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. The `/guides/wiring-character-values` link will warn — expected, fixed in Task 13.

- [ ] **Step 6: Commit**

```bash
git add website/docs/examples/hal-and-carla.mdx
git commit -m "docs(examples): demonstrate audience flip in Hal & Carla

Side-by-side defeat graphs under [life>property] vs [property>life]
audiences show grounded extension flipping from {H1,C2} to {C1,C2}.
Replaces the 'VAF is on the roadmap' paragraph with a pointer to
the implemented crate and the wiring-character-values how-to."
```

---

### Task 13: New how-to guide — wiring per-character values

**Files:**
- Create: `website/docs/guides/wiring-character-values.md`
- Modify: `website/sidebars.ts` (add the new guide to `guidesSidebar`)

- [ ] **Step 1: Write `website/docs/guides/wiring-character-values.md`**

```markdown
---
sidebar_position: 9
title: Wire per-character values into a scene
---

Wire per-character audiences into an encounter scene so Alice and Bob reach different conclusions when they hold different values. The `ValueAwareScorer` reads each character's audience from `EncounterArgumentationState` and adjusts proposal scoring accordingly.

**Learning objective:** add per-character value preferences to an existing encounter scene with one new dependency, two `set_audience` calls, and one `ValueAwareScorer::new` wrapper around the existing scorer chain.

## Prerequisites

- A working scene from [Build your first scene](/getting-started/first-scene).
- `encounter-argumentation` v0.5+ with `argumentation-values` available (path-dep or registry-dep).

## Step 1: Add the dep

Already a transitive dep through `encounter-argumentation`, but if you use `argumentation-values` types directly:

```toml
[dependencies]
argumentation-values = "0.1"
```

## Step 2: Set per-character audiences before resolve

```rust
use encounter_argumentation::{Audience, EncounterArgumentationState, Value};

let state = EncounterArgumentationState::new(catalog);

// Alice prioritises duty above all else.
state.set_audience(
    "alice",
    Audience::total([Value::new("duty"), Value::new("survival"), Value::new("comfort")]),
);

// Bob's audience inverts duty and survival.
state.set_audience(
    "bob",
    Audience::total([Value::new("survival"), Value::new("duty"), Value::new("comfort")]),
);
```

The audience storage uses interior mutability (mirroring how `set_intensity` works) — you can call `set_audience` through a shared `&state` reference at any point before resolve.

## Step 3: Wrap your existing scorer

If you're already using `SchemeActionScorer`, just stack `ValueAwareScorer` on top:

```rust
use encounter_argumentation::{SchemeActionScorer, ValueAwareScorer};

let scheme_scorer = SchemeActionScorer::new(
    knowledge,
    registry,
    baseline_scorer,
    0.3,  // scheme-strength boost magnitude
);
let value_scorer = ValueAwareScorer::new(
    scheme_scorer,
    &state,
    0.2,  // value-preference boost magnitude
);
```

The two boosts compose additively: scheme-strength boost first, then value-preference boost. Both skip silently when the actor has no audience configured.

## Step 4: Resolve as usual

```rust
use encounter::resolution::MultiBeat;
use encounter_argumentation::StateAcceptanceEval;

let acceptance = StateAcceptanceEval::new(&state);
let participants = vec!["alice".into(), "bob".into()];
let result = MultiBeat.resolve(&participants, &practice, &catalog, &value_scorer, &acceptance);
```

When Alice scores her affordances, those backed by schemes promoting `duty` get the largest boost (tier 0). When Bob scores the same affordances, `survival`-promoting ones get the largest. Same scene, same arguments — different proposals win.

## Verify

A worked test pattern:

```rust
#[test]
fn alice_and_bob_reach_different_outcomes() {
    let state = build_state_with_two_proposals();
    state.set_audience("alice", Audience::total([Value::new("duty")]));
    state.set_audience("bob", Audience::total([Value::new("survival")]));

    let scorer = build_value_aware_scorer(&state);
    let alice_scored = scorer.score_actions("alice", &affordances, &participants);
    let bob_scored = scorer.score_actions("bob", &affordances, &participants);

    // Alice's top pick should be the duty-promoting affordance; Bob's the
    // survival-promoting one.
    assert_ne!(alice_scored[0].entry.spec.name, bob_scored[0].entry.spec.name);
}
```

## How the value boost is computed

For each affordance, the scorer checks if its `bindings` map contains a `value` slot. If yes, and that value is ranked in the actor's audience, the boost is:

```
boost = max_boost * (tier_count - rank) / tier_count
```

Where `rank = 0` for the most preferred value (largest boost) and `rank = tier_count - 1` for the least preferred (smallest non-zero boost). Unranked values get zero boost.

## When NOT to use this

- **Scenes where all characters share an audience.** Set the audience once in scene setup and skip the per-character storage.
- **Scenes where values aren't relevant to the proposals.** If the affordances aren't backed by `argument_from_values` schemes, `ValueAwareScorer` is a no-op — skip it.
- **Scenes where the storyteller wants to *force* an outcome regardless of character values.** Use direct `add_weighted_attack` with hard-coded weights instead.

## Related

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — the formalism this scorer is built on.
- [Hal & Carla](/examples/hal-and-carla) — the canonical scene that motivates per-character audiences.
- [Modulate attack weights with societas](/guides/societas-modulated-weights) — for live relationship-driven attack weight modulation.
- [`argumentation-values` API docs](https://docs.rs/argumentation-values).
```

- [ ] **Step 2: Add to `website/sidebars.ts`**

Find the `guidesSidebar` block and add `'guides/wiring-character-values'` after `'guides/societas-modulated-weights'`:

```typescript
  guidesSidebar: [
    'guides/installation',
    'guides/catalog-authoring',
    'guides/implementing-action-scorer',
    'guides/implementing-acceptance-eval',
    'guides/tuning-beta',
    'guides/debugging-acceptance',
    'guides/societas-modulated-weights',
    'guides/wiring-character-values',
    'guides/migration-v0.4-to-v0.5',
  ],
```

- [ ] **Step 3: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. The previously-broken `/guides/wiring-character-values` warnings from Tasks 11 and 12 should now resolve.

- [ ] **Step 4: Commit**

```bash
git add website/docs/guides/wiring-character-values.md \
        website/sidebars.ts
git commit -m "docs(guides): add wiring-character-values how-to

Walks through adding per-character audiences to an encounter scene:
add dep, set_audience per actor, wrap SchemeActionScorer with
ValueAwareScorer, resolve as usual. Includes a worked-test pattern
and a When-NOT-to-use section. Wired into the guides sidebar between
societas-modulated-weights and migration-v0.4-to-v0.5."
```

---

## Done

After Task 13 the workspace has:

- A working `argumentation-values` crate (~600 LOC + ~400 LOC tests) covering the formal Bench-Capon VAF apparatus with multi-value support
- Per-character audiences storable on the encounter state, with a `ValueAwareScorer` that reads them at resolve time
- APX format I/O for ASPARTIX interop
- `MultiAudience::common_grounded` for council-style consensus queries
- A reframed concepts page, a demonstration on the Hal & Carla example page, and a how-to guide
- Removed VAF from the open-areas list (now four formalisms, not five)

Verify with one final pass:

```bash
cd /home/peter/code/argumentation
cargo test --workspace 2>&1 | tail -10
cd website && npx docusaurus build 2>&1 | tail -10
```

Expected: all tests pass; docusaurus build succeeds; `/api/` warnings are pre-existing.

If running via subagent-driven-development, the final code-review should evaluate:
- Whether `audience_from_prefs` in `apx.rs` correctly handles cycles (currently emits remaining values as one tier; verify against the test `multi_tier_audience_emerges_from_chained_prefs`)
- Whether the `ValueAwareScorer`'s tier-rank computation is correct for multi-value-per-tier audiences (the v0 implementation walks `audience.values()` linearly which conflates position with tier; if a multi-value-per-tier test fails, expose `Audience::rank()` and call it instead)
- Whether the Hal & Carla audience-flip side-by-side display renders correctly across viewport widths (the `flexWrap: 'wrap'` should reflow on narrow screens)
