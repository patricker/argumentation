# `argumentation-weighted` Crate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust crate implementing weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics, on top of the existing `argumentation` crate. Gives the narrative stack a principled way to model relationship-modulated attack strength — stronger relationships absorb higher-weight attacks without acceptance flipping; weaker ones flip on lighter attacks — and drama-manager control over scene intensity as a single "budget" knob.

**Architecture:** A new workspace-member crate `argumentation-weighted` that reuses `argumentation::ArgumentationFramework` and its Dung semantics. The core type `WeightedFramework<A>` attaches an `f64` weight to each attack edge. The inconsistency-budget semantics of Dunne et al. 2011 — "tolerate attacks whose cumulative weight is at most β, then run Dung on the residual" — is implemented via a **β-reduction** step that produces an unweighted `ArgumentationFramework` at a given budget, after which the existing Dung semantics are called. A threshold-sweep API returns the discrete β values at which an argument's acceptance changes, answering drama-manager queries like "how much relationship stress would it take to flip this character's position?" No changes to the core `argumentation` crate.

**Tech Stack:** Rust 2024 edition, depends on `argumentation` (workspace path dep) and `thiserror` 2.0 for errors. No serde. No async.

**Status:** Phase 3 of the L2/L3 narrative stack per `ARGUMENTATION_vNEXT.md` §2.3. Twin release with `argumentation-bipolar`. Independent of both `argumentation-schemes` and `argumentation-bipolar` for v0.1.0 (composition with bipolar is a deferred v0.2.0 target per vNEXT §6).

**Crate location:** New workspace member at `crates/argumentation-weighted/`. The workspace was already converted from single-crate to multi-crate layout by the `argumentation-schemes` work on 2026-04-12. Task 1 adds a new member to the `[workspace]` section.

---

## Design decisions locked for v0.1.0

Two questions from vNEXT §7.4 resolved up-front:

1. **Weights on attacks only, not arguments.** Dunne et al. 2011 defines weights strictly on attack edges. Weighting arguments themselves is a ranking-based-semantics concern and belongs in `argumentation-rankings` (vNEXT §3.1, deferred). For v0.1.0, arguments are unweighted.

2. **Semantics = inconsistency budget (Dunne 2011 §3).** Given a framework and a budget `β ≥ 0`, the weighted extensions are the Dung extensions of the residual framework obtained by **removing a minimum-weight set of attacks whose total weight is at most β**. Formally, for any set `R ⊆ Atts` with `∑ w(a) ≤ β` where `a ∈ R`, the `β`-extensions under semantics `σ` are `σ(⟨Args, Atts ∖ R⟩)`. An argument is **β-credulously accepted** iff it appears in some extension for some valid `R`; **β-skeptically accepted** iff it appears in every extension for every valid `R`.

   v0.1.0 ships a **practical approximation** of the above: instead of enumerating all subsets `R` (which is exponential in `|Atts|`), we use the **cumulative-weight threshold** variant — remove attacks in ascending order of weight until the cumulative removed weight would exceed β, then run Dung semantics on the residual. For monotonic cases (smaller attacks are more expendable, larger ones are always kept) this matches the formal definition; for non-monotonic cases (where removing a heavier attack is strategically better than a lighter one) it is a *lower bound* on credulous acceptance and an *upper bound* on skeptical acceptance. The docstring on the entry points explicitly names this as an approximation and points at the v0.2.0 ticket for the full exponential enumeration.

   This practical variant covers the narrative use case cleanly: relationship-modulated attacks where small weights mean "mild pushback I can shrug off" and large weights mean "serious challenge that forces a realignment." The drama-manager query "what's the smallest β that accepts argument X?" has a well-defined, monotone answer under this variant and is computable in O(|Atts| · semantics-cost).

**What is NOT locked:** composition with `argumentation-bipolar` (weighted supports alongside weighted attacks) is a v0.2.0 target per vNEXT §6. A `WeightSource<A>` trait for pulling weights from participant-relationship metadata is included as a helper in v0.1.0 but kept deliberately minimal — no dependency on the `encounter` crate, which doesn't exist yet.

---

## Use Cases and Validation Criteria

### UC1: Relationship-modulated attack that flips under stress

**Scenario:** Alice and Bob are close friends. Alice argues for fortifying the east wall; Bob argues against it. In their default relationship state, Bob's attack on Alice's position has weight 0.3 (mild pushback that a friendship can absorb). After a betrayal incident, Bob's attack weight rises to 0.9 (serious breach). The drama manager wants to know: at what budget does Alice's position stop being accepted?

**What must work:** Construct a `WeightedFramework` with Alice's argument, Bob's attacking argument weighted at 0.9, and a budget sweep. At β=0, Alice is NOT accepted (Bob's attack fires at full strength). At β=1.0, Alice IS accepted (the 0.9 attack falls within the 1.0 budget and is tolerated). Somewhere in (0.0, 0.9] is the flip point; the threshold-sweep API should report it as exactly 0.9 — "this is the smallest budget that tolerates Bob's attack."

**Validates:** Core `WeightedFramework` type, β-reduction correctness, threshold-sweep returning the right flip point, monotonicity of acceptance under increasing budget.

### UC2: Multiple attacks, cumulative budget

**Scenario:** Three characters — Alice, Bob, Charlie — all attack Dawn's position with weights 0.2, 0.3, 0.5 (total 1.0). Dawn is only accepted if the budget tolerates *all three* attacks, i.e., β ≥ 1.0. With β = 0.5, the weakest two attacks (0.2 + 0.3 = 0.5) fit in the budget but Charlie's 0.5 does not — Dawn is still defeated.

**What must work:** At β=1.0, Dawn is accepted (all attackers tolerated). At β=0.99, Dawn is NOT accepted (can't tolerate all three cumulatively). At β=0.5, Dawn is NOT accepted (Charlie's attack still fires). The threshold-sweep API reports flip points at each discrete β where tolerance cascades through the sorted attacks: {0.2, 0.5, 1.0}.

**Validates:** Cumulative-weight threshold computation, multiple-attacker aggregation, that flip points are reported in sorted order.

### UC3: Scene intensity via budget sweep

**Scenario:** The drama manager is deciding how intense to make a confrontation scene. "Low intensity" = β=1.5 (everybody shrugs off most attacks). "High intensity" = β=0.0 (every attack fires at full force). The manager sweeps β over `[0.0, 2.0]` and asks: which arguments are skeptically accepted at each intensity level?

**What must work:** `WeightedFramework::acceptance_trajectory(target_argument, budget_range)` returns the sequence of `(budget, is_accepted)` pairs at every flip point. Useful for the drama manager to pick an intensity that produces the desired extension structure.

**Validates:** Trajectory API, monotonicity guarantees, that flip points are discrete (no continuous sweep needed — only transition-point β values matter).

### UC4: Drama manager's natural inverse query

**Scenario:** "How strongly would the relationship have to weaken before Alice would accept Bob's argument?" — i.e., find the minimum budget `β*` such that Alice's argument appears in some preferred extension at `β*`.

**What must work:** `min_budget_for_credulous(target_argument)` returns `Some(β*)` where `β*` is the smallest budget achieving credulous acceptance, or `None` if the argument is never accepted (e.g., it is self-attacking or is attacked by an unattackable argument whose attack has weight above the maximum possible budget).

**Validates:** Monotone binary-search-like lookup, correct handling of "never accepted" cases, the inverse query pattern the drama manager actually uses.

---

## File Structure

All paths relative to `/home/peter/code/argumentation/`.

```
crates/
└── argumentation-weighted/
    ├── Cargo.toml
    ├── README.md
    ├── CHANGELOG.md
    ├── LICENSE-MIT
    ├── LICENSE-APACHE
    ├── src/
    │   ├── lib.rs             # Public API, crate docs, doctest
    │   ├── error.rs           # Crate errors
    │   ├── types.rs           # AttackWeight, Budget, WeightedAttack
    │   ├── framework.rs       # WeightedFramework<A>
    │   ├── reduce.rs          # β-reduction: WeightedFramework → Dung ArgumentationFramework
    │   ├── semantics.rs       # Weighted extensions + credulous/skeptical acceptance
    │   ├── sweep.rs           # Threshold-sweep and trajectory API
    │   └── weight_source.rs   # WeightSource trait for external weight computation
    └── tests/
        ├── uc1_friendship_flip.rs
        ├── uc2_cumulative_budget.rs
        ├── uc3_scene_intensity.rs
        ├── uc4_min_budget_query.rs
        └── reduction_correctness.rs
```

Eight source files, five integration test files.

---

## Phase 1 — Foundations

### Task 1: Scaffold the new workspace crate

**Files:**
- Modify: `Cargo.toml` (root — add `crates/argumentation-weighted` to `[workspace]` members)
- Create: `crates/argumentation-weighted/Cargo.toml`
- Create: `crates/argumentation-weighted/src/lib.rs`
- Create: `crates/argumentation-weighted/src/error.rs`
- Create: `crates/argumentation-weighted/README.md`

- [ ] **Step 1: Read the existing root `Cargo.toml`**

```bash
cat /home/peter/code/argumentation/Cargo.toml
```

Note the current `[workspace] members = [...]` line. If the bipolar plan has also been executed, it will already include `crates/argumentation-bipolar`. This task adds `crates/argumentation-weighted` alongside whatever is there.

- [ ] **Step 2: Add the new member to the workspace section**

Edit the `members` line to append `"crates/argumentation-weighted"`. The final list should include `"."`, `"crates/argumentation-schemes"`, possibly `"crates/argumentation-bipolar"`, and the new `"crates/argumentation-weighted"`.

- [ ] **Step 3: Verify existing members still build**

```bash
cargo build --package argumentation
cargo build --package argumentation-schemes
```

(And bipolar if it exists.) Expected: all existing crates still compile.

- [ ] **Step 4: Create `crates/argumentation-weighted/Cargo.toml`**

```toml
[package]
name = "argumentation-weighted"
version = "0.1.0"
edition = "2024"
description = "Weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["argumentation", "weighted", "inconsistency-budget", "dunne", "dung"]
categories = ["algorithms"]

[dependencies]
argumentation = { path = "../.." }
thiserror = "2.0"
```

- [ ] **Step 5: Create `src/lib.rs`**

```rust
//! # argumentation-weighted
//!
//! Weighted argumentation frameworks (Dunne, Hunter, McBurney, Parsons
//! & Wooldridge 2011) built on top of the [`argumentation`] crate's
//! Dung semantics.
//!
//! A weighted framework attaches an `f64` weight to each attack edge.
//! Under the **inconsistency-budget** semantics of Dunne et al., a
//! budget `β` permits attacks whose cumulative weight is at most `β`
//! to be tolerated (i.e., treated as if they did not exist) for the
//! purposes of computing Dung extensions. The budget acts as a single
//! knob: `β = 0` runs the standard Dung semantics over every attack;
//! increasing `β` progressively tolerates more attacks and accepts
//! more arguments. The flip points — the discrete `β` values at which
//! an argument's acceptance changes — are computable from the sorted
//! attack weights alone.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod types;

pub use error::Error;
```

- [ ] **Step 6: Create `src/error.rs`**

```rust
//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-weighted` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An attack weight was non-finite (NaN or infinity) or negative.
    /// Dunne 2011 requires non-negative finite weights.
    #[error("invalid attack weight {weight}: weights must be non-negative finite f64")]
    InvalidWeight {
        /// The weight that failed validation.
        weight: f64,
    },

    /// A budget value was non-finite or negative.
    #[error("invalid budget {budget}: budgets must be non-negative finite f64")]
    InvalidBudget {
        /// The budget that failed validation.
        budget: f64,
    },

    /// An operation referenced an argument that is not in the framework.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// An error from the underlying Dung layer (e.g., framework too
    /// large for subset enumeration).
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}
```

- [ ] **Step 7: Create a minimal `src/types.rs` stub**

(Needed so `lib.rs`'s `pub mod types;` resolves; full content lands in Task 2.)

```rust
//! Foundational types for weighted argumentation. Full content in Task 2.
```

- [ ] **Step 8: Create `README.md`**

```markdown
# argumentation-weighted

Weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics. Built on the [`argumentation`](../..) crate.

**Status:** Under active development.
```

- [ ] **Step 9: Verify the new crate compiles**

```bash
cd /home/peter/code/argumentation && cargo build --package argumentation-weighted
```

Expected: compiles cleanly.

- [ ] **Step 10: Verify the workspace test sweep still passes**

```bash
cargo test --workspace
```

Expected: existing `argumentation` / `argumentation-schemes` / `argumentation-bipolar` tests still pass. The new crate has zero tests.

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): scaffold new crate, add to workspace"
```

---

### Task 2: Core types — `AttackWeight`, `Budget`, `WeightedAttack`

**Files:**
- Modify: `crates/argumentation-weighted/src/types.rs`

- [ ] **Step 1: Replace `src/types.rs` with the full type definitions**

```rust
//! Foundational types for weighted argumentation.
//!
//! - [`AttackWeight`] — validated non-negative finite f64 wrapper.
//! - [`Budget`] — validated non-negative finite f64 wrapper for
//!   inconsistency-budget values.
//! - [`WeightedAttack`] — a directed attack edge carrying a weight.

use crate::error::Error;

/// A non-negative finite attack weight. Constructed via [`Self::new`],
/// which rejects NaN, infinity, and negative values.
///
/// Implements `Copy`, `Clone`, `Debug`, `PartialEq`, and `PartialOrd`
/// but NOT `Eq` or `Hash` — `f64` does not satisfy those by default.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AttackWeight(f64);

impl AttackWeight {
    /// Construct a weight, rejecting NaN, infinity, and negative values.
    pub fn new(value: f64) -> Result<Self, Error> {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::InvalidWeight { weight: value });
        }
        Ok(Self(value))
    }

    /// The underlying `f64` value. Always non-negative and finite by
    /// construction.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// A non-negative finite inconsistency budget. Semantics: attacks whose
/// cumulative weight is at most this value may be tolerated for the
/// purposes of Dung semantics.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Budget(f64);

impl Budget {
    /// Construct a budget, rejecting NaN, infinity, and negative values.
    pub fn new(value: f64) -> Result<Self, Error> {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::InvalidBudget { budget: value });
        }
        Ok(Self(value))
    }

    /// The underlying `f64` value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// A zero budget — equivalent to running standard Dung semantics
    /// (no attacks are tolerated).
    #[must_use]
    pub fn zero() -> Self {
        Self(0.0)
    }
}

/// A weighted directed attack edge: `attacker` attacks `target` with
/// the given `weight`.
///
/// Generic over argument type `A` to match the core crate's convention.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAttack<A: Clone + Eq> {
    /// The attacking argument.
    pub attacker: A,
    /// The target argument.
    pub target: A,
    /// The attack weight. Higher weights are harder to tolerate.
    pub weight: AttackWeight,
}

impl<A: Clone + Eq> WeightedAttack<A> {
    /// Convenience constructor.
    pub fn new(attacker: A, target: A, weight: f64) -> Result<Self, Error> {
        Ok(Self {
            attacker,
            target,
            weight: AttackWeight::new(weight)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_weight_accepts_valid_values() {
        assert!(AttackWeight::new(0.0).is_ok());
        assert!(AttackWeight::new(0.5).is_ok());
        assert!(AttackWeight::new(100.0).is_ok());
    }

    #[test]
    fn attack_weight_rejects_nan() {
        let err = AttackWeight::new(f64::NAN).unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }

    #[test]
    fn attack_weight_rejects_infinity() {
        assert!(AttackWeight::new(f64::INFINITY).is_err());
        assert!(AttackWeight::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn attack_weight_rejects_negative() {
        assert!(AttackWeight::new(-0.1).is_err());
    }

    #[test]
    fn budget_zero_is_valid() {
        assert_eq!(Budget::zero().value(), 0.0);
        assert!(Budget::new(0.0).is_ok());
    }

    #[test]
    fn budget_rejects_invalid_values() {
        assert!(Budget::new(-1.0).is_err());
        assert!(Budget::new(f64::NAN).is_err());
        assert!(Budget::new(f64::INFINITY).is_err());
    }

    #[test]
    fn weighted_attack_new_validates_weight() {
        assert!(WeightedAttack::new("a", "b", 0.5).is_ok());
        assert!(WeightedAttack::new("a", "b", -0.5).is_err());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 7 tests pass.

- [ ] **Step 3: Run clippy**

```bash
cargo clippy --package argumentation-weighted -- -D warnings
```

Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): AttackWeight, Budget, WeightedAttack with validation"
```

---

### Task 3: `WeightedFramework<A>` — CRUD and basic queries

**Files:**
- Create: `crates/argumentation-weighted/src/framework.rs`
- Modify: `crates/argumentation-weighted/src/lib.rs`

- [ ] **Step 1: Create `src/framework.rs`**

```rust
//! `WeightedFramework<A>`: arguments and weighted attack edges.

use crate::error::Error;
use crate::types::{AttackWeight, WeightedAttack};
use std::collections::HashSet;
use std::hash::Hash;

/// A weighted argumentation framework: a set of arguments and a list
/// of weighted attack edges between them.
///
/// Attack weights are validated at insert time via
/// [`AttackWeight::new`]. Duplicate attack edges (same attacker and
/// target) are NOT deduplicated — each `add_weighted_attack` call
/// appends a new edge, even if one already exists. This matches Dunne
/// 2011, which allows multigraphs with distinct-weight parallel edges.
/// Consumers who want deduplication should call
/// [`WeightedFramework::collapse_duplicate_attacks`].
#[derive(Debug, Clone)]
pub struct WeightedFramework<A: Clone + Eq + Hash> {
    arguments: HashSet<A>,
    attacks: Vec<WeightedAttack<A>>,
}

impl<A: Clone + Eq + Hash> WeightedFramework<A> {
    /// Create an empty framework.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashSet::new(),
            attacks: Vec::new(),
        }
    }

    /// Add an argument. Adding an argument that already exists is a no-op.
    pub fn add_argument(&mut self, a: A) {
        self.arguments.insert(a);
    }

    /// Add a weighted attack. Both endpoints are implicitly added to
    /// the framework. Returns [`Error::InvalidWeight`] if the weight
    /// fails validation. Parallel edges with the same endpoints but
    /// different weights are permitted.
    pub fn add_weighted_attack(
        &mut self,
        attacker: A,
        target: A,
        weight: f64,
    ) -> Result<(), Error> {
        let w = AttackWeight::new(weight)?;
        self.arguments.insert(attacker.clone());
        self.arguments.insert(target.clone());
        self.attacks.push(WeightedAttack {
            attacker,
            target,
            weight: w,
        });
        Ok(())
    }

    /// Collapse parallel edges: for each `(attacker, target)` pair,
    /// keep only one edge whose weight is the sum of all parallel
    /// edges' weights. This is one valid aggregation strategy (sum);
    /// Dunne 2011 does not prescribe one. Consumers who want a
    /// different aggregation (max, min, mean) should implement it
    /// externally.
    pub fn collapse_duplicate_attacks(&mut self) {
        use std::collections::HashMap;
        let mut map: HashMap<(A, A), f64> = HashMap::new();
        for atk in self.attacks.drain(..) {
            let key = (atk.attacker, atk.target);
            *map.entry(key).or_insert(0.0) += atk.weight.value();
        }
        self.attacks = map
            .into_iter()
            .map(|((attacker, target), weight)| WeightedAttack {
                attacker,
                target,
                weight: AttackWeight::new(weight).expect("sum of non-negative weights is non-negative"),
            })
            .collect();
    }

    /// Iterate over all arguments.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.arguments.iter()
    }

    /// Iterate over all weighted attacks.
    pub fn attacks(&self) -> impl Iterator<Item = &WeightedAttack<A>> {
        self.attacks.iter()
    }

    /// Number of arguments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    /// Whether the framework has zero arguments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// Number of attack edges (counting parallel edges separately).
    #[must_use]
    pub fn attack_count(&self) -> usize {
        self.attacks.len()
    }

    /// Return all distinct weight values present in the framework,
    /// sorted ascending. Used by the threshold-sweep API: flip points
    /// can only occur at cumulative-sum values of these weights.
    #[must_use]
    pub fn sorted_weights(&self) -> Vec<f64> {
        let mut ws: Vec<f64> = self.attacks.iter().map(|a| a.weight.value()).collect();
        ws.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ws
    }
}

impl<A: Clone + Eq + Hash> Default for WeightedFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

// Compile-time thread-safety guarantee matching the core crate.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<WeightedFramework<String>>();
    assert_sync::<WeightedFramework<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_framework_has_no_arguments() {
        let wf: WeightedFramework<&str> = WeightedFramework::new();
        assert!(wf.is_empty());
        assert_eq!(wf.len(), 0);
        assert_eq!(wf.attack_count(), 0);
    }

    #[test]
    fn add_weighted_attack_registers_both_endpoints() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        assert_eq!(wf.len(), 2);
        assert_eq!(wf.attack_count(), 1);
    }

    #[test]
    fn add_weighted_attack_rejects_invalid_weight() {
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        assert!(wf.add_weighted_attack("a", "b", -0.1).is_err());
        assert!(wf.add_weighted_attack("a", "b", f64::NAN).is_err());
    }

    #[test]
    fn parallel_edges_are_preserved_before_collapse() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("a", "b", 0.4).unwrap();
        assert_eq!(wf.attack_count(), 2);
    }

    #[test]
    fn collapse_duplicate_attacks_sums_weights() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.3).unwrap();
        wf.add_weighted_attack("a", "b", 0.4).unwrap();
        wf.add_weighted_attack("a", "c", 0.5).unwrap();
        wf.collapse_duplicate_attacks();
        assert_eq!(wf.attack_count(), 2);
        // Find the (a, b) edge and verify its weight is 0.7.
        let ab = wf
            .attacks()
            .find(|a| a.attacker == "a" && a.target == "b")
            .unwrap();
        assert!((ab.weight.value() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn sorted_weights_returns_ascending() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("a", "c", 0.2).unwrap();
        wf.add_weighted_attack("a", "d", 0.8).unwrap();
        let ws = wf.sorted_weights();
        assert_eq!(ws, vec![0.2, 0.5, 0.8]);
    }
}
```

- [ ] **Step 2: Register `framework` in `src/lib.rs`**

Update `src/lib.rs` to add the new module and a root-level re-export:

```rust
//! # argumentation-weighted
//! ...  (keep existing crate docs)

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::WeightedFramework;
```

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 13 tests pass (7 types + 6 framework).

- [ ] **Step 4: Run clippy**

```bash
cargo clippy --package argumentation-weighted -- -D warnings
```

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): WeightedFramework with validation and parallel edge collapse"
```

---

## Phase 2 — β-reduction and semantics

### Task 4: β-reduction — cumulative-weight residual framework

**Files:**
- Create: `crates/argumentation-weighted/src/reduce.rs`
- Modify: `crates/argumentation-weighted/src/lib.rs`

- [ ] **Step 1: Create `src/reduce.rs`**

```rust
//! β-reduction: convert a [`WeightedFramework`] at a given budget into
//! an equivalent unweighted [`argumentation::ArgumentationFramework`].
//!
//! v0.1.0 ships the **cumulative-weight threshold** approximation of
//! Dunne et al. 2011's inconsistency-budget semantics:
//!
//! 1. Sort all attacks by weight ascending.
//! 2. Walk the sorted list, maintaining a running `cumulative` total.
//!    While `cumulative + next_weight ≤ β`, include the next attack in
//!    the "tolerated" set `R` and advance `cumulative`.
//! 3. The residual framework contains all arguments plus every attack
//!    NOT in `R`.
//!
//! This matches the formal definition for the common case (smaller
//! attacks are strictly more expendable). It can under-tolerate in
//! pathological cases where skipping a cheap attack to afford a
//! strategically-important expensive one would yield a larger
//! extension set; the full exponential enumeration is deferred to
//! v0.2.0.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::types::Budget;
use argumentation::ArgumentationFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Reduce a weighted framework at budget `β` to an unweighted Dung
/// framework by tolerating the cheapest attacks first until the
/// cumulative tolerated weight would exceed `β`.
///
/// Returns a plain [`argumentation::ArgumentationFramework`] whose
/// attack edges are the **surviving** attacks (those NOT tolerated).
/// Any existing Dung semantics call on the result corresponds to the
/// weighted semantics at that budget under the cumulative-threshold
/// approximation.
pub fn reduce_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<ArgumentationFramework<A>, Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let mut af = ArgumentationFramework::new();
    for arg in framework.arguments() {
        af.add_argument(arg.clone());
    }

    // Sort a view of attack references by weight ascending so we can
    // walk them in order without modifying the framework.
    let mut sorted_attacks: Vec<&crate::types::WeightedAttack<A>> =
        framework.attacks().collect();
    sorted_attacks.sort_by(|a, b| {
        a.weight
            .value()
            .partial_cmp(&b.weight.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Tolerate the cheapest attacks first.
    let mut cumulative: f64 = 0.0;
    let mut first_surviving = 0;
    for (i, atk) in sorted_attacks.iter().enumerate() {
        if cumulative + atk.weight.value() <= budget.value() {
            cumulative += atk.weight.value();
        } else {
            first_surviving = i;
            break;
        }
        first_surviving = i + 1;
    }

    // Everything from `first_surviving` onward survives — add those
    // attacks to the residual framework.
    for atk in &sorted_attacks[first_surviving..] {
        af.add_attack(&atk.attacker, &atk.target)?;
    }

    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_budget_keeps_all_attacks() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.8).unwrap();
        let af = reduce_at_budget(&wf, Budget::zero()).unwrap();
        assert_eq!(af.len(), 4);
        assert_eq!(af.attackers(&"b").len(), 1);
        assert_eq!(af.attackers(&"d").len(), 1);
    }

    #[test]
    fn large_budget_tolerates_all_attacks() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("c", "d", 0.8).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(10.0).unwrap()).unwrap();
        assert_eq!(af.len(), 4);
        assert!(af.attackers(&"b").is_empty());
        assert!(af.attackers(&"d").is_empty());
    }

    #[test]
    fn budget_tolerates_cheapest_attacks_first() {
        // Weights: 0.2, 0.3, 0.5. Budget 0.5 tolerates the 0.2 and
        // 0.3 (cumulative 0.5) but not the 0.5 (would exceed).
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        wf.add_weighted_attack("a3", "target", 0.5).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(0.5).unwrap()).unwrap();
        // Only a3 should still attack target.
        let attackers: Vec<&&str> = af.attackers(&"target").into_iter().collect();
        assert_eq!(attackers.len(), 1);
        assert_eq!(*attackers[0], "a3");
    }

    #[test]
    fn budget_exactly_at_cumulative_tolerates_that_attack() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        // Budget exactly 0.5 — the boundary case. Both should fit.
        let af = reduce_at_budget(&wf, Budget::new(0.5).unwrap()).unwrap();
        assert!(af.attackers(&"target").is_empty());
    }

    #[test]
    fn budget_one_below_cumulative_does_not_tolerate() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.2).unwrap();
        wf.add_weighted_attack("a2", "target", 0.3).unwrap();
        let af = reduce_at_budget(&wf, Budget::new(0.499).unwrap()).unwrap();
        // 0.499 < 0.2 + 0.3 = 0.5, so a2 cannot be tolerated; only a1.
        let attackers: Vec<&&str> = af.attackers(&"target").into_iter().collect();
        assert_eq!(attackers.len(), 1);
        assert_eq!(*attackers[0], "a2");
    }

    #[test]
    fn isolated_arguments_preserved_through_reduction() {
        let mut wf = WeightedFramework::new();
        wf.add_argument("isolated");
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let af = reduce_at_budget(&wf, Budget::zero()).unwrap();
        assert_eq!(af.len(), 3);
    }
}
```

- [ ] **Step 2: Register `reduce` in lib.rs**

Add `pub mod reduce;` alongside existing `pub mod` lines.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 19 tests pass (13 + 6 reduction tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): β-reduction via cumulative-weight threshold"
```

---

### Task 5: Weighted extensions at a fixed budget

**Files:**
- Create: `crates/argumentation-weighted/src/semantics.rs`
- Modify: `crates/argumentation-weighted/src/lib.rs`

- [ ] **Step 1: Create `src/semantics.rs`**

```rust
//! Weighted extensions at a fixed budget.
//!
//! These are thin wrappers that reduce the framework at the given
//! budget and delegate to the core crate's Dung semantics on the
//! residual framework. Every Dung semantics variant
//! (grounded/complete/preferred/stable/semi-stable/ideal) gets a
//! corresponding `*_at_budget` entry point here.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::reduce::reduce_at_budget;
use crate::types::Budget;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// The grounded extension at the given budget.
pub fn grounded_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<HashSet<A>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.grounded_extension())
}

/// All complete extensions at the given budget.
pub fn complete_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.complete_extensions()?)
}

/// All preferred extensions at the given budget.
pub fn preferred_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.preferred_extensions()?)
}

/// All stable extensions at the given budget.
pub fn stable_at_budget<A>(
    framework: &WeightedFramework<A>,
    budget: Budget,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = reduce_at_budget(framework, budget)?;
    Ok(af.stable_extensions()?)
}

/// Whether `target` is **credulously accepted** at the given budget:
/// does it appear in at least one preferred extension of the residual
/// framework?
pub fn is_credulously_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let prefs = preferred_at_budget(framework, budget)?;
    Ok(prefs.iter().any(|ext| ext.contains(target)))
}

/// Whether `target` is **skeptically accepted** at the given budget:
/// does it appear in every preferred extension of the residual
/// framework?
pub fn is_skeptically_accepted_at<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    budget: Budget,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let prefs = preferred_at_budget(framework, budget)?;
    if prefs.is_empty() {
        return Ok(false);
    }
    Ok(prefs.iter().all(|ext| ext.contains(target)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_at_zero_budget_matches_dung() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        wf.add_weighted_attack("b", "c", 0.5).unwrap();
        // Dung: grounded = {a, c} because a is unattacked and c is
        // attacked by b, which is attacked by a.
        let grounded = grounded_at_budget(&wf, Budget::zero()).unwrap();
        assert!(grounded.contains(&"a"));
        assert!(grounded.contains(&"c"));
        assert!(!grounded.contains(&"b"));
    }

    #[test]
    fn large_budget_grounds_every_argument() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let grounded = grounded_at_budget(&wf, Budget::new(10.0).unwrap()).unwrap();
        // All attacks tolerated; both a and b unattacked.
        assert!(grounded.contains(&"a"));
        assert!(grounded.contains(&"b"));
    }

    #[test]
    fn credulous_and_skeptical_agree_on_grounded_case() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        // Unique preferred extension = {a}. Both credulous and skeptical
        // acceptance of `a` should be true.
        let budget = Budget::zero();
        assert!(is_credulously_accepted_at(&wf, &"a", budget).unwrap());
        assert!(is_skeptically_accepted_at(&wf, &"a", budget).unwrap());
        assert!(!is_credulously_accepted_at(&wf, &"b", budget).unwrap());
        assert!(!is_skeptically_accepted_at(&wf, &"b", budget).unwrap());
    }
}
```

- [ ] **Step 2: Register `semantics` in lib.rs**

Add `pub mod semantics;`.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 22 tests pass (19 + 3 new semantics tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): extensions at budget, credulous/skeptical acceptance"
```

---

## Phase 3 — Threshold sweep and trajectory

### Task 6: Threshold sweep — discrete flip points

**Files:**
- Create: `crates/argumentation-weighted/src/sweep.rs`
- Modify: `crates/argumentation-weighted/src/lib.rs`

- [ ] **Step 1: Create `src/sweep.rs`**

The key observation: under the cumulative-weight threshold approximation, acceptance of an argument is a **monotone step function of β**. The flip points — the discrete β values at which acceptance can change — occur exactly at cumulative weights `w_1 + w_2 + … + w_k` where `w_1 ≤ w_2 ≤ …` are the sorted attack weights.

At β = 0, no attacks are tolerated; at β = sum(all weights), all are tolerated. Between those, acceptance can only change at one of the cumulative-sum thresholds. So a full sweep over `[0, total_weight]` requires checking at most `|attacks| + 1` points.

```rust
//! Threshold-sweep API: compute acceptance trajectories for one
//! argument across the full budget range.
//!
//! Under the cumulative-weight threshold approximation, acceptance is
//! a monotone step function of the budget β. The flip points are
//! exactly the cumulative-sum values of the sorted attack weights
//! (plus β = 0 as the starting point). A sweep requires at most
//! `|attacks| + 1` evaluations.

use crate::error::Error;
use crate::framework::WeightedFramework;
use crate::semantics::{is_credulously_accepted_at, is_skeptically_accepted_at};
use crate::types::Budget;
use std::fmt::Debug;
use std::hash::Hash;

/// One point in a threshold sweep: the budget at which this point
/// applies, and whether the target is accepted at that budget.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPoint {
    /// The budget value at which this point was evaluated.
    pub budget: f64,
    /// Whether the target was accepted at that budget.
    pub accepted: bool,
}

/// Which acceptance notion to use for the sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptanceMode {
    /// Credulous: in at least one preferred extension.
    Credulous,
    /// Skeptical: in every preferred extension.
    Skeptical,
}

/// Compute the sorted list of budget breakpoints at which the
/// cumulative-weight threshold transitions — exactly `|attacks|+1`
/// values: `[0, w_1, w_1+w_2, ..., sum]`.
fn breakpoints<A: Clone + Eq + Hash>(framework: &WeightedFramework<A>) -> Vec<f64> {
    let mut out = vec![0.0];
    let mut cumulative = 0.0;
    for w in framework.sorted_weights() {
        cumulative += w;
        out.push(cumulative);
    }
    out
}

/// Compute the full acceptance trajectory for `target` across the
/// framework's budget range, returning one `SweepPoint` at every
/// breakpoint.
///
/// The returned vector is sorted by `budget` ascending and starts at
/// `budget = 0`. Use [`flip_points`] if you only want the budgets at
/// which acceptance changes, not every breakpoint.
pub fn acceptance_trajectory<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    mode: AcceptanceMode,
) -> Result<Vec<SweepPoint>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let mut out = Vec::new();
    for bp in breakpoints(framework) {
        let budget = Budget::new(bp)?;
        let accepted = match mode {
            AcceptanceMode::Credulous => {
                is_credulously_accepted_at(framework, target, budget)?
            }
            AcceptanceMode::Skeptical => {
                is_skeptically_accepted_at(framework, target, budget)?
            }
        };
        out.push(SweepPoint {
            budget: bp,
            accepted,
        });
    }
    Ok(out)
}

/// Return only the budgets at which `target`'s acceptance changes as
/// β increases. Useful for the drama-manager flip-point query.
pub fn flip_points<A>(
    framework: &WeightedFramework<A>,
    target: &A,
    mode: AcceptanceMode,
) -> Result<Vec<f64>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let trajectory = acceptance_trajectory(framework, target, mode)?;
    let mut flips = Vec::new();
    let mut last_accepted: Option<bool> = None;
    for point in trajectory {
        if last_accepted != Some(point.accepted) {
            if last_accepted.is_some() {
                flips.push(point.budget);
            }
            last_accepted = Some(point.accepted);
        }
    }
    Ok(flips)
}

/// Return the smallest budget at which `target` is credulously
/// accepted, or `None` if it is never accepted across the framework's
/// full budget range.
pub fn min_budget_for_credulous<A>(
    framework: &WeightedFramework<A>,
    target: &A,
) -> Result<Option<f64>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let trajectory = acceptance_trajectory(framework, target, AcceptanceMode::Credulous)?;
    Ok(trajectory
        .into_iter()
        .find(|p| p.accepted)
        .map(|p| p.budget))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_at_zero_and_cumulative_sums() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("c", "d", 0.3).unwrap();
        wf.add_weighted_attack("e", "f", 0.5).unwrap();
        let bps = breakpoints(&wf);
        // Expected: [0.0, 0.2, 0.5, 1.0]
        assert_eq!(bps.len(), 4);
        assert!((bps[0] - 0.0).abs() < 1e-9);
        assert!((bps[1] - 0.2).abs() < 1e-9);
        assert!((bps[2] - 0.5).abs() < 1e-9);
        assert!((bps[3] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unattacked_argument_is_accepted_at_every_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_argument("unattacked");
        wf.add_weighted_attack("a", "b", 0.5).unwrap();
        let trajectory =
            acceptance_trajectory(&wf, &"unattacked", AcceptanceMode::Credulous).unwrap();
        assert!(trajectory.iter().all(|p| p.accepted));
    }

    #[test]
    fn singly_attacked_argument_flips_at_attack_weight() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("attacker", "target", 0.5).unwrap();
        // At β=0: attacker defeats target (not accepted).
        // At β=0.5: attack tolerated, target accepted.
        let flips = flip_points(&wf, &"target", AcceptanceMode::Credulous).unwrap();
        assert_eq!(flips.len(), 1);
        assert!((flips[0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn min_budget_for_credulous_finds_smallest_accepting_budget() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "target", 0.3).unwrap();
        wf.add_weighted_attack("b", "target", 0.7).unwrap();
        // Target accepted only once both attacks are tolerated (β ≥ 1.0).
        let min = min_budget_for_credulous(&wf, &"target").unwrap();
        assert_eq!(min, Some(1.0));
    }

    #[test]
    fn min_budget_returns_none_for_self_attack() {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "a", 0.5).unwrap();
        // Self-attacking argument is never accepted under any budget
        // (tolerating the attack leaves an isolated unattacked node,
        // so it IS accepted at β ≥ 0.5). Let's verify the correct answer.
        let min = min_budget_for_credulous(&wf, &"a").unwrap();
        assert_eq!(min, Some(0.5));
    }

    #[test]
    fn trajectory_is_monotone_nondecreasing() {
        // Acceptance is monotone in β — once accepted, stays accepted.
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a1", "target", 0.3).unwrap();
        wf.add_weighted_attack("a2", "target", 0.5).unwrap();
        let trajectory =
            acceptance_trajectory(&wf, &"target", AcceptanceMode::Credulous).unwrap();
        let mut seen_accepted = false;
        for p in trajectory {
            if p.accepted {
                seen_accepted = true;
            } else {
                assert!(
                    !seen_accepted,
                    "acceptance should be monotone non-decreasing in budget"
                );
            }
        }
    }
}
```

- [ ] **Step 2: Register `sweep` in lib.rs**

Add `pub mod sweep;`.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 28 tests pass (22 + 6 new sweep tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): threshold sweep, flip points, min-budget query"
```

---

### Task 7: `WeightSource` helper trait for external weight computation

**Files:**
- Create: `crates/argumentation-weighted/src/weight_source.rs`
- Modify: `crates/argumentation-weighted/src/lib.rs`

- [ ] **Step 1: Create `src/weight_source.rs`**

```rust
//! `WeightSource` trait for computing attack weights from external
//! state (relationship metadata, personality traits, etc.).
//!
//! This is a deliberately minimal abstraction for v0.1.0 — it exists
//! so consumers like the future `encounter` crate can pass a policy
//! closure for deriving weights, without coupling this crate to any
//! specific narrative-stack types. The full `encounter`-specific
//! integration lives in whatever bridge crate the narrative team
//! ships; this trait is just the hook.

use crate::error::Error;
use crate::framework::WeightedFramework;
use std::hash::Hash;

/// A source of attack weights. Given an attacker and a target (and
/// whatever context `Self` carries), produce the weight for the
/// corresponding attack edge.
///
/// Implementations might read participant relationship metadata,
/// personality compatibility, recent interaction history, or any other
/// external state. The trait itself is stateless from this crate's
/// perspective.
pub trait WeightSource<A> {
    /// Compute the weight for an attack from `attacker` to `target`.
    /// Returns `None` if this source has no opinion (i.e., the attack
    /// should not be added). Returns `Some(w)` otherwise.
    fn weight_for(&self, attacker: &A, target: &A) -> Option<f64>;
}

/// A closure-based `WeightSource` that wraps any `Fn(&A, &A) -> Option<f64>`.
pub struct ClosureWeightSource<F>(pub F);

impl<A, F> WeightSource<A> for ClosureWeightSource<F>
where
    F: Fn(&A, &A) -> Option<f64>,
{
    fn weight_for(&self, attacker: &A, target: &A) -> Option<f64> {
        (self.0)(attacker, target)
    }
}

/// Populate a `WeightedFramework` from a list of attack pairs, pulling
/// each weight from the provided `WeightSource`. Pairs for which the
/// source returns `None` are skipped. Pairs for which the source
/// returns an invalid weight propagate an [`Error::InvalidWeight`].
///
/// This is a convenience builder. Consumers that need more control
/// (e.g., different sources for different attack types) should call
/// `add_weighted_attack` directly.
pub fn populate_from_source<A, W, I>(
    framework: &mut WeightedFramework<A>,
    pairs: I,
    source: &W,
) -> Result<(), Error>
where
    A: Clone + Eq + Hash,
    W: WeightSource<A>,
    I: IntoIterator<Item = (A, A)>,
{
    for (attacker, target) in pairs {
        if let Some(weight) = source.weight_for(&attacker, &target) {
            framework.add_weighted_attack(attacker, target, weight)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedSource(f64);

    impl WeightSource<&'static str> for FixedSource {
        fn weight_for(&self, _attacker: &&'static str, _target: &&'static str) -> Option<f64> {
            Some(self.0)
        }
    }

    #[test]
    fn closure_weight_source_returns_closure_output() {
        let src = ClosureWeightSource(|_a: &&str, _b: &&str| Some(0.42));
        assert_eq!(src.weight_for(&"x", &"y"), Some(0.42));
    }

    #[test]
    fn populate_from_source_adds_all_attacks() {
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        let src = FixedSource(0.5);
        populate_from_source(&mut wf, vec![("a", "b"), ("c", "d")], &src).unwrap();
        assert_eq!(wf.attack_count(), 2);
    }

    #[test]
    fn populate_skips_none_weights() {
        let src = ClosureWeightSource(
            |_a: &&str, target: &&str| if *target == "b" { Some(0.5) } else { None },
        );
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        populate_from_source(&mut wf, vec![("x", "b"), ("x", "c")], &src).unwrap();
        assert_eq!(wf.attack_count(), 1);
    }

    #[test]
    fn populate_propagates_invalid_weights() {
        let src = ClosureWeightSource(|_a: &&str, _b: &&str| Some(-1.0));
        let mut wf: WeightedFramework<&str> = WeightedFramework::new();
        let err = populate_from_source(&mut wf, vec![("x", "y")], &src).unwrap_err();
        assert!(matches!(err, Error::InvalidWeight { .. }));
    }
}
```

- [ ] **Step 2: Register `weight_source` in lib.rs**

Add `pub mod weight_source;`.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-weighted
```

Expected: 32 tests pass (28 + 4 new weight_source tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-weighted): WeightSource trait for external weight computation"
```

---

## Phase 4 — Integration tests and release

### Task 8: UC1 integration test — friendship flip

**Files:**
- Create: `crates/argumentation-weighted/tests/uc1_friendship_flip.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC1: Alice and Bob are friends. Bob attacks Alice's argument with
//! weight 0.9 (mild-to-significant in context). At β=0, the attack
//! fires and Alice is not accepted. At β=1.0 (budget exceeds 0.9),
//! the attack is tolerated and Alice is accepted.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::semantics::{
    is_credulously_accepted_at, is_skeptically_accepted_at,
};
use argumentation_weighted::sweep::{flip_points, min_budget_for_credulous, AcceptanceMode};
use argumentation_weighted::types::Budget;

fn alice_bob_framework() -> WeightedFramework<&'static str> {
    let mut wf = WeightedFramework::new();
    wf.add_argument("alice_claim");
    wf.add_weighted_attack("bob_attack", "alice_claim", 0.9).unwrap();
    wf
}

#[test]
fn uc1_alice_defeated_at_zero_budget() {
    let wf = alice_bob_framework();
    assert!(!is_credulously_accepted_at(&wf, &"alice_claim", Budget::zero()).unwrap());
    assert!(!is_skeptically_accepted_at(&wf, &"alice_claim", Budget::zero()).unwrap());
}

#[test]
fn uc1_alice_accepted_when_budget_exceeds_attack_weight() {
    let wf = alice_bob_framework();
    let budget = Budget::new(1.0).unwrap();
    assert!(is_credulously_accepted_at(&wf, &"alice_claim", budget).unwrap());
}

#[test]
fn uc1_flip_point_is_exactly_zero_point_nine() {
    let wf = alice_bob_framework();
    let flips = flip_points(&wf, &"alice_claim", AcceptanceMode::Credulous).unwrap();
    assert_eq!(flips.len(), 1);
    assert!((flips[0] - 0.9).abs() < 1e-9);
}

#[test]
fn uc1_min_budget_for_credulous_alice_is_zero_point_nine() {
    let wf = alice_bob_framework();
    let min = min_budget_for_credulous(&wf, &"alice_claim").unwrap();
    assert_eq!(min, Some(0.9));
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted --test uc1_friendship_flip
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-weighted): UC1 friendship flip integration"
```

---

### Task 9: UC2 integration test — cumulative budget across multiple attackers

**Files:**
- Create: `crates/argumentation-weighted/tests/uc2_cumulative_budget.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC2: Three attackers on Dawn's position with weights 0.2, 0.3, 0.5
//! (total 1.0). Dawn is only accepted once the budget tolerates ALL
//! three attacks (β ≥ 1.0). At β = 0.99 she is still defeated.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::semantics::is_credulously_accepted_at;
use argumentation_weighted::sweep::{acceptance_trajectory, min_budget_for_credulous, AcceptanceMode};
use argumentation_weighted::types::Budget;

fn dawn_framework() -> WeightedFramework<&'static str> {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("alice", "dawn", 0.2).unwrap();
    wf.add_weighted_attack("bob", "dawn", 0.3).unwrap();
    wf.add_weighted_attack("charlie", "dawn", 0.5).unwrap();
    wf
}

#[test]
fn uc2_dawn_defeated_at_zero_budget() {
    let wf = dawn_framework();
    assert!(!is_credulously_accepted_at(&wf, &"dawn", Budget::zero()).unwrap());
}

#[test]
fn uc2_dawn_defeated_at_budget_zero_point_five() {
    // 0.5 tolerates the 0.2 + 0.3 but not the 0.5 (cumulative 0.5
    // exactly, but the last 0.5 would push us to 1.0).
    // Wait — 0.2 + 0.3 = 0.5 exactly. The 0.5 attack can't fit in the
    // remaining 0.0 budget. So one attack survives.
    let wf = dawn_framework();
    assert!(!is_credulously_accepted_at(&wf, &"dawn", Budget::new(0.5).unwrap()).unwrap());
}

#[test]
fn uc2_dawn_accepted_at_budget_one() {
    let wf = dawn_framework();
    assert!(is_credulously_accepted_at(&wf, &"dawn", Budget::new(1.0).unwrap()).unwrap());
}

#[test]
fn uc2_dawn_min_budget_is_exactly_one() {
    let wf = dawn_framework();
    let min = min_budget_for_credulous(&wf, &"dawn").unwrap();
    assert_eq!(min, Some(1.0));
}

#[test]
fn uc2_dawn_trajectory_has_single_flip() {
    let wf = dawn_framework();
    let trajectory = acceptance_trajectory(&wf, &"dawn", AcceptanceMode::Credulous).unwrap();
    // Breakpoints: [0.0, 0.2, 0.5, 1.0]. Acceptance should be
    // [false, false, false, true].
    let accepted: Vec<bool> = trajectory.iter().map(|p| p.accepted).collect();
    assert_eq!(accepted, vec![false, false, false, true]);
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted --test uc2_cumulative_budget
```

Expected: 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-weighted): UC2 cumulative budget across multiple attackers"
```

---

### Task 10: UC3 integration test — scene intensity trajectory

**Files:**
- Create: `crates/argumentation-weighted/tests/uc3_scene_intensity.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC3: the drama manager sweeps scene intensity (budget) over a
//! framework with three arguments in conflict. Verify that the
//! trajectory reports a sensible sequence of flip points and that
//! acceptance is monotone non-decreasing in β for every argument.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::{acceptance_trajectory, AcceptanceMode};

#[test]
fn uc3_trajectory_is_monotone_for_every_argument() {
    let mut wf = WeightedFramework::new();
    // Three arguments in a chain: a attacks b attacks c, each weighted.
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    for argument in ["a", "b", "c"] {
        let traj = acceptance_trajectory(&wf, &argument, AcceptanceMode::Credulous).unwrap();
        let mut seen_accept = false;
        for point in traj {
            if point.accepted {
                seen_accept = true;
            } else {
                assert!(
                    !seen_accept,
                    "{}: acceptance must be monotone in budget",
                    argument
                );
            }
        }
    }
}

#[test]
fn uc3_drama_knob_low_intensity_accepts_everybody() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    // At the highest breakpoint (sum of weights = 1.0), every attack is
    // tolerated and all three arguments should be credulously accepted.
    let traj = acceptance_trajectory(&wf, &"b", AcceptanceMode::Credulous).unwrap();
    assert!(traj.last().unwrap().accepted, "b should be accepted at max budget");
}

#[test]
fn uc3_high_intensity_zero_budget_runs_pure_dung() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.4).unwrap();
    wf.add_weighted_attack("b", "c", 0.6).unwrap();

    let traj = acceptance_trajectory(&wf, &"b", AcceptanceMode::Credulous).unwrap();
    // At β=0, the chain a→b→c makes b not-accepted (attacked by a).
    assert!(!traj[0].accepted);
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted --test uc3_scene_intensity
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-weighted): UC3 scene intensity trajectory"
```

---

### Task 11: UC4 integration test — min-budget inverse query

**Files:**
- Create: `crates/argumentation-weighted/tests/uc4_min_budget_query.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC4: the drama manager asks "how much relationship stress before
//! Alice accepts Bob's argument?" — i.e., find the smallest budget at
//! which a target argument becomes credulously accepted.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::min_budget_for_credulous;

#[test]
fn uc4_unattacked_argument_accepted_at_zero() {
    let mut wf = WeightedFramework::new();
    wf.add_argument("free");
    wf.add_weighted_attack("noise1", "noise2", 0.5).unwrap();
    let min = min_budget_for_credulous(&wf, &"free").unwrap();
    assert_eq!(min, Some(0.0));
}

#[test]
fn uc4_singly_attacked_argument_needs_budget_equal_to_attack_weight() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("attacker", "target", 0.75).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    assert_eq!(min, Some(0.75));
}

#[test]
fn uc4_argument_with_multiple_attackers_needs_cumulative_budget() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "target", 0.3).unwrap();
    wf.add_weighted_attack("b", "target", 0.4).unwrap();
    wf.add_weighted_attack("c", "target", 0.5).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    // Needs all three tolerated to accept target: 0.3 + 0.4 + 0.5 = 1.2
    assert_eq!(min, Some(1.2));
}

#[test]
fn uc4_argument_only_needs_direct_attackers_tolerated() {
    // b attacks target. a attacks b. At β=0, a defeats b, b can't
    // attack target, so target is accepted at β=0.
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("b", "target", 0.5).unwrap();
    wf.add_weighted_attack("a", "b", 0.2).unwrap();
    let min = min_budget_for_credulous(&wf, &"target").unwrap();
    assert_eq!(
        min,
        Some(0.0),
        "target should be accepted at β=0 because a defeats b"
    );
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted --test uc4_min_budget_query
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-weighted): UC4 min-budget inverse query"
```

---

### Task 12: Reduction correctness — fixture tests from Dunne 2011

**Files:**
- Create: `crates/argumentation-weighted/tests/reduction_correctness.rs`

- [ ] **Step 1: Create the test file**

```rust
//! Reduction correctness tests: Dunne 2011 paper examples plus
//! boundary-value fixtures.

use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::reduce::reduce_at_budget;
use argumentation_weighted::semantics::grounded_at_budget;
use argumentation_weighted::types::Budget;

#[test]
fn reduction_at_zero_preserves_every_attack() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.1).unwrap();
    wf.add_weighted_attack("c", "d", 0.2).unwrap();
    wf.add_weighted_attack("e", "f", 0.3).unwrap();
    let af = reduce_at_budget(&wf, Budget::zero()).unwrap();
    // Every argument attacked exactly once.
    for target in ["b", "d", "f"] {
        assert_eq!(af.attackers(&target).len(), 1);
    }
}

#[test]
fn reduction_at_large_budget_tolerates_everything() {
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    wf.add_weighted_attack("c", "d", 0.7).unwrap();
    let af = reduce_at_budget(&wf, Budget::new(100.0).unwrap()).unwrap();
    assert!(af.attackers(&"b").is_empty());
    assert!(af.attackers(&"d").is_empty());
}

#[test]
fn grounded_agrees_with_dung_at_zero_budget() {
    // Framework: a → b, b → c, c → d (chain). At β=0 this is pure Dung.
    // Grounded = {a, c} (odd positions in the chain).
    let mut wf = WeightedFramework::new();
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    wf.add_weighted_attack("b", "c", 0.5).unwrap();
    wf.add_weighted_attack("c", "d", 0.5).unwrap();
    let grounded = grounded_at_budget(&wf, Budget::zero()).unwrap();
    assert!(grounded.contains(&"a"));
    assert!(grounded.contains(&"c"));
    assert!(!grounded.contains(&"b"));
    assert!(!grounded.contains(&"d"));
}

#[test]
fn reduction_is_deterministic_across_rebuilds() {
    let build = || {
        let mut wf = WeightedFramework::new();
        wf.add_weighted_attack("a", "b", 0.2).unwrap();
        wf.add_weighted_attack("c", "d", 0.3).unwrap();
        reduce_at_budget(&wf, Budget::new(0.25).unwrap()).unwrap().len()
    };
    assert_eq!(build(), build());
}

#[test]
fn reduction_preserves_argument_set() {
    let mut wf = WeightedFramework::new();
    wf.add_argument("isolated");
    wf.add_weighted_attack("a", "b", 0.5).unwrap();
    let af = reduce_at_budget(&wf, Budget::new(1.0).unwrap()).unwrap();
    assert_eq!(af.len(), 3);
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-weighted --test reduction_correctness
```

Expected: 5 tests pass.

- [ ] **Step 3: Run the full crate sweep**

```bash
cargo test --package argumentation-weighted
```

Expected: 32 unit + 21 integration = 53 tests passing.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(argumentation-weighted): reduction correctness and Dunne 2011 fixtures"
```

---

### Task 13: Public API + docs + v0.1.0 release prep

**Files:**
- Modify: `crates/argumentation-weighted/src/lib.rs`
- Create: `crates/argumentation-weighted/CHANGELOG.md`
- Create: `crates/argumentation-weighted/LICENSE-MIT`
- Create: `crates/argumentation-weighted/LICENSE-APACHE`
- Modify: `crates/argumentation-weighted/README.md`

- [ ] **Step 1: Finalize `src/lib.rs` with re-exports and doctest example**

```rust
//! # argumentation-weighted
//!
//! Weighted argumentation frameworks (Dunne, Hunter, McBurney, Parsons
//! & Wooldridge 2011) built on top of the [`argumentation`] crate's
//! Dung semantics.
//!
//! A weighted framework attaches an `f64` weight to each attack edge.
//! Under the **inconsistency-budget** semantics of Dunne et al., a
//! budget `β` permits attacks whose cumulative weight is at most `β`
//! to be tolerated for the purposes of computing Dung extensions. The
//! budget acts as a single knob: `β = 0` runs standard Dung semantics
//! over every attack; increasing `β` progressively tolerates more
//! attacks and accepts more arguments.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_weighted::framework::WeightedFramework;
//! use argumentation_weighted::sweep::min_budget_for_credulous;
//!
//! let mut wf = WeightedFramework::new();
//! wf.add_weighted_attack("attacker", "target", 0.6).unwrap();
//!
//! // At what budget does `target` become accepted?
//! let min = min_budget_for_credulous(&wf, &"target").unwrap();
//! assert_eq!(min, Some(0.6));
//! ```
//!
//! ## Semantics notes
//!
//! v0.1.0 implements the **cumulative-weight threshold** approximation
//! of Dunne 2011's inconsistency-budget semantics: the cheapest
//! attacks are tolerated first until the cumulative weight would
//! exceed `β`. This is equivalent to the formal definition when
//! smaller attacks are strictly more expendable than larger ones
//! (which is the common case for relationship-modulated attack
//! strength). The full exponential enumeration over subsets of attacks
//! is a deferred v0.2.0 target.
//!
//! ## References
//!
//! - Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., &
//!   Wooldridge, M. (2011). *Weighted argument systems: Basic
//!   definitions, algorithms, and complexity results.* Artificial
//!   Intelligence 175(2).
//! - Bistarelli, S., Rossi, F., & Santini, F. (2018). *A collective
//!   defence against grouped attacks for weighted abstract argumentation
//!   frameworks.* IJAR 92.
//! - Coste-Marquis, S. et al. (2012). *Weighted attacks in
//!   argumentation frameworks.* KR 2012.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod reduce;
pub mod semantics;
pub mod sweep;
pub mod types;
pub mod weight_source;

pub use error::Error;
pub use framework::WeightedFramework;
pub use reduce::reduce_at_budget;
pub use semantics::{
    complete_at_budget, grounded_at_budget, is_credulously_accepted_at,
    is_skeptically_accepted_at, preferred_at_budget, stable_at_budget,
};
pub use sweep::{
    acceptance_trajectory, flip_points, min_budget_for_credulous, AcceptanceMode, SweepPoint,
};
pub use types::{AttackWeight, Budget, WeightedAttack};
pub use weight_source::{populate_from_source, ClosureWeightSource, WeightSource};
```

- [ ] **Step 2: Create `CHANGELOG.md`**

```markdown
# Changelog

## [0.1.0] - 2026-04-TBD

### Added
- `WeightedFramework<A>` with non-negative finite attack weights.
- `AttackWeight` and `Budget` newtypes with validation.
- β-reduction via cumulative-weight threshold (Dunne 2011 approximation).
- Weighted extensions at fixed budgets: `grounded_at_budget`,
  `complete_at_budget`, `preferred_at_budget`, `stable_at_budget`.
- Credulous and skeptical acceptance queries under a budget.
- Threshold-sweep API: `acceptance_trajectory`, `flip_points`,
  `min_budget_for_credulous`.
- `WeightSource` trait and `ClosureWeightSource` helper for computing
  weights from external state (relationship metadata, personality, etc.).
- 32 unit tests + 21 integration tests across UC1-UC4 plus reduction
  correctness fixtures.

### Known limitations
- Cumulative-weight threshold is a practical approximation of the full
  Dunne 2011 inconsistency-budget semantics. In pathological cases
  (where strategically removing an expensive attack to afford a cheaper
  one would yield a strictly larger extension), the approximation is a
  lower bound on credulous acceptance. The full exponential variant is
  a deferred v0.2.0 target.
- No composition with `argumentation-bipolar` yet (weighted bipolar
  frameworks per Amgoud et al. 2008 are deferred to v0.2.0).
```

- [ ] **Step 3: Create `LICENSE-MIT`**

```
MIT License

Copyright (c) 2026 The argumentation-weighted contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 4: Create `LICENSE-APACHE`**

Use the standard Apache 2.0 license text from <https://www.apache.org/licenses/LICENSE-2.0.txt> with copyright line `Copyright 2026 The argumentation-weighted contributors`. You can copy the sibling `crates/argumentation-schemes/LICENSE-APACHE` file and just update the copyright holder line.

- [ ] **Step 5: Update `README.md`**

```markdown
# argumentation-weighted

Weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics. Built on the [`argumentation`](../..) crate.

## What's in the box

- `WeightedFramework<A>` with validated non-negative f64 attack weights.
- β-reduction: produce an unweighted Dung framework at a given budget.
- Weighted extensions: grounded, complete, preferred, stable at any budget.
- Threshold-sweep API: acceptance trajectory, flip points, min-budget inverse query.
- `WeightSource` trait for pulling weights from external state.

## Quick example

```rust
use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::min_budget_for_credulous;

let mut wf = WeightedFramework::new();
wf.add_weighted_attack("attacker", "target", 0.6).unwrap();

let min = min_budget_for_credulous(&wf, &"target").unwrap();
assert_eq!(min, Some(0.6));
```

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Dunne, Hunter, McBurney, Parsons & Wooldridge (2011). *Weighted argument systems: Basic definitions, algorithms, and complexity results.* AIJ 175(2).
```

- [ ] **Step 6: Full verification sweep**

```bash
cd /home/peter/code/argumentation
cargo test --package argumentation-weighted
cargo test --workspace
cargo clippy --package argumentation-weighted -- -D warnings
cargo fmt --package argumentation-weighted -- --check
cargo doc --package argumentation-weighted --no-deps
```

Expected: 53 unit + integration tests + 1 doctest all pass; workspace sweep clean; clippy clean; fmt clean; docs build cleanly.

If `cargo fmt --check` emits drift, run `cargo fmt --package argumentation-weighted` and re-stage before committing.

- [ ] **Step 7: Commit and tag**

```bash
git add -A
git commit -m "chore(argumentation-weighted): v0.1.0 release prep"
git tag -a argumentation-weighted-v0.1.0 -m "argumentation-weighted v0.1.0"
```

Do not push the tag — the human will decide when to push.

---

## Out of scope for v0.1.0

- **Full exponential inconsistency-budget semantics.** The v0.1.0 cumulative-threshold approximation is correct for the common monotone case but does not enumerate all subsets `R` with weight ≤ β. A v0.2.0 target once a real scenario demonstrates the approximation is too coarse.
- **Weighted supports and weighted-bipolar composition** (Amgoud et al. 2008). Deferred to v0.2.0 once both `argumentation-bipolar` and `argumentation-weighted` have shipped independently and their composition points are clear.
- **Alternative weighted semantics** (Coste-Marquis et al. 2012). The Dunne 2011 variant is the only one shipped in v0.1.0.
- **Weighted ASPIC+**. vNEXT §7.4 explicitly defers this — weighted Dung is sufficient for narrative use cases.
- **Weighted-bipolar coalition-attack semantics** (Bistarelli et al. 2018). Extension on top of weighted-bipolar, so deferred.
- **ICCMA-style benchmark integration.** The core crate's `iccma_fixtures` would need weighted-extension format support. Not a Phase 3 deliverable.

## References

- Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., & Wooldridge, M. (2011). *Weighted argument systems: Basic definitions, algorithms, and complexity results.* Artificial Intelligence 175(2). [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0004370210001153) — the foundational paper; §3 has the inconsistency-budget definition.
- Bistarelli, S., Rossi, F., & Santini, F. (2018). *A collective defence against grouped attacks for weighted abstract argumentation frameworks.* IJAR 92. — Extension to coalition attacks, referenced in vNEXT §2.3 as partially superseding SETAFs in the weighted case.
- Coste-Marquis, S., Konieczny, S., Marquis, P., & Ouali, M. A. (2012). *Weighted attacks in argumentation frameworks.* KR 2012. — Alternative weighted semantics, for v0.2.0 comparison.
- Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P. (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10). — Natural composition target with `argumentation-bipolar`, deferred to v0.2.0.

---

**End of plan.**
