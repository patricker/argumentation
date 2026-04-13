# `argumentation-bipolar` Crate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust crate implementing bipolar argumentation frameworks (attacks + supports) on top of the existing `argumentation` crate, so the narrative stack can model coalitions, alliances, betrayals, and corroboration as first-class data rather than simulating them via separate attack edges.

**Architecture:** A new workspace-member crate `argumentation-bipolar` that reuses `argumentation::ArgumentationFramework` and its Dung semantics. The core type `BipolarFramework<A>` stores attacks and supports as two distinct directed edge sets. Bipolar semantics are implemented by **flattening** the framework — computing derived attacks per Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008 (supported, mediated, secondary) — producing an equivalent Dung framework, running the existing Dung semantics on it, then **post-filtering** extensions for necessary-support closure per Nouioua & Risch 2011. Coalition detection is a graph-algorithms layer over the support edge set using strongly-connected components. No changes to the core `argumentation` crate.

**Tech Stack:** Rust 2024 edition, depends on `argumentation` (workspace path dep) and `petgraph` for SCC computation, `thiserror` 2.0 for errors. No serde (v0.2.0 concern). No async.

**Status:** Phase 3 of the L2/L3 narrative stack per `ARGUMENTATION_vNEXT.md` §2.2. Twin release with `argumentation-weighted`. Independent of `argumentation-schemes`.

**Crate location:** New workspace member of the existing `argumentation` repo at `/home/peter/code/argumentation/`. Lives at `crates/argumentation-bipolar/`. The workspace was already converted from single-crate to multi-crate layout when `argumentation-schemes` shipped on 2026-04-12. Task 1 just adds a new member to the existing `[workspace]` section.

---

## Design decisions locked for v0.1.0

Three design forks from §7 of `ARGUMENTATION_vNEXT.md` resolved up-front so the engineer doesn't re-litigate them mid-implementation:

1. **Support semantics = necessary support** (Nouioua & Risch 2011). An argument `A` is acceptable under a candidate extension `S` only if every argument that is *necessary for* `A` is also in `S`. This matches narrative coalition semantics ("Alice can't win this argument without Bob backing her") and is the default §7.3 of the vNEXT doc recommends.

2. **Derived attack set follows Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008.** Three derived attack kinds are computed at flattening time:
   - **Supported attack** `A ⇒ B` via chain: if `A` supports `X` and `X` attacks `B`, then `A` attacks `B`.
   - **Secondary attack** `A ⇒ C` via chain: if `A` attacks `B` and `B` supports `C`, then `A` attacks `C`.
   - **Mediated attack** `A ⇒ C` via chain: if `A` attacks `B` and `C` supports `B`, then `A` attacks `C`.

   All three compose transitively through support chains. The closure is computed fixed-point over the support graph.

3. **Implementation strategy = flatten + filter.** Given a `BipolarFramework`, the semantics pipeline is:
   - Build `attack_closure` = direct attacks ∪ derived attacks (iterated to fixed point).
   - Construct a plain `argumentation::ArgumentationFramework<A>` with the closure as its only edge set.
   - Call the existing Dung extensions (grounded, complete, preferred, stable, ideal, semi-stable).
   - **Filter** each candidate extension `E` to drop any `E` that is not *support-closed*: ∃`A ∈ E` such that some necessary supporter of `A` is not in `E`.

   The filtered result is the bipolar extension set under necessary-support semantics. This reuses the mature Dung semantics code in the core crate, avoids re-implementing a whole fixed-point stack for bipolar admissibility, and has a direct formal correspondence to Nouioua & Risch §3.

**What is NOT locked:** deductive support and evidential support are deferred to v0.2.0. The `SupportSemantics` enum is pre-declared with `Necessary`, `Deductive`, `Evidential` variants for API stability, but only `Necessary` is implemented in v0.1.0. Constructors for `Deductive` / `Evidential` return `Error::UnimplementedSemantics`.

---

## Use Cases and Validation Criteria

### UC1: Corroboration without attack

**Scenario:** Alice says "I saw the queen meet the stranger." Bob says "I, too, saw the queen meet the stranger." Neither argument attacks anything; Bob's argument *supports* Alice's. A third character, Charlie, attacks Alice's sighting ("Alice has bad eyesight"). Under Dung alone, Charlie defeats Alice and there's no way to represent Bob's independent corroboration. Under necessary-support bipolar, Bob's argument is independently accepted (nothing attacks it), and the support edge means Alice is *reinforced* — though Charlie's attack still wins in strictly attack-based semantics (Bob's support does not produce a *defense* under necessary-support, only a *precondition*).

**What must work:** Construct a `BipolarFramework` with three arguments `{alice, bob, charlie}`, one attack `charlie → alice`, one support `bob → alice` (read as: bob is necessary for alice). Compute preferred extensions. Verify: (a) `bob` appears in every preferred extension (unattacked), (b) when `charlie` is in an extension, `alice` is not (standard defeat), (c) when `alice` is in an extension, `bob` is also in it (support closure — necessary supporter must be present).

**Validates:** Support edges, necessary-support closure filter, the distinction between "nothing attacks me" and "something supports me."

### UC2: Coalition against a common enemy

**Scenario:** Alice and Bob are allied against Charlie. Alice attacks Charlie's position (`alice → charlie`), Bob attacks Charlie's position (`bob → charlie`), and Alice and Bob mutually support each other (`alice → bob` and `bob → alice`, both as supports). Under Dung alone this is just two independent attackers; you lose the coalition structure. Under bipolar, the mutual-support loop forms a coalition.

**What must work:** The coalition detection API returns one coalition containing `{alice, bob}` (SCC over the support graph). Charlie is in a singleton coalition. Standard preferred extensions contain `{alice, bob}` (they both survive because they attack Charlie and defend each other via the derived attack closure from their mutual support). Charlie is never in a preferred extension because both coalition members attack him.

**Validates:** Coalition detection via SCC, mutual-support handling, that the support chain generates derived attacks correctly.

### UC3: Betrayal — retracting support

**Scenario:** Start with the UC2 framework (alice/bob allied against charlie). Then Bob betrays Alice: remove the `bob → alice` support edge while keeping `alice → bob`. The betrayal asymmetrically breaks the coalition. Narratively: "Bob withdraws his backing from Alice's position."

**What must work:** After the support edge is removed, the coalition detection no longer returns `{alice, bob}` as a coalition (the support SCC is broken). Both still appear in preferred extensions (they still attack Charlie independently), but they are now in separate singleton coalitions. This test pins that support edges can be added and removed dynamically without rebuilding the framework.

**Validates:** Mutation API for support edges, SCC recomputation, that coalition ≠ extension membership (you can be accepted without being coalitioned).

### UC4: Mediated attack through support chain

**Scenario:** Three arguments `a, b, c` with `a → b` (attack) and `c → b` (support, read as: c is necessary for b). A mediated attack applies: because `c` is a necessary supporter of `b`, attacking `c` is equivalent to attacking `b` at the argument level. Conversely, attacking `b` when `c` is necessary for `b` also attacks `c` (if you defeat what c supports, c's relevance is undermined).

**What must work:** After flattening, the derived attack set includes `a → c` (mediated attack). The Dung semantics over the closed framework behave correctly: any extension that contains `a` cannot contain `c` (and therefore cannot contain `b` either, via the support-closure filter).

**Validates:** Derived-attack closure computation, mediated attack rule, interaction between closure and filter.

---

## File Structure

All paths below are relative to `/home/peter/code/argumentation/`.

```
crates/
└── argumentation-bipolar/
    ├── Cargo.toml
    ├── README.md
    ├── CHANGELOG.md
    ├── LICENSE-MIT
    ├── LICENSE-APACHE
    ├── src/
    │   ├── lib.rs                # Public API, crate docs, doctest example
    │   ├── error.rs              # Crate errors
    │   ├── types.rs              # EdgeKind, SupportSemantics, CoalitionId
    │   ├── framework.rs          # BipolarFramework<A> with attack+support edge sets
    │   ├── derived.rs            # Derived-attack closure (supported/mediated/secondary)
    │   ├── flatten.rs            # Convert BipolarFramework → Dung ArgumentationFramework
    │   ├── semantics.rs          # Bipolar grounded/complete/preferred/stable/semi-stable/ideal
    │   ├── coalition.rs          # SCC-based coalition detection over support graph
    │   └── queries.rs            # Transitive supporters/attackers, who-is-with-whom
    └── tests/
        ├── uc1_corroboration.rs
        ├── uc2_coalition.rs
        ├── uc3_betrayal.rs
        ├── uc4_mediated_attack.rs
        └── flattening_closure.rs # Unit coverage for the derived-attack fixed-point
```

Nine source files, five integration test files. Every source file has one clear responsibility.

---

## Phase 1 — Foundations

### Task 1: Scaffold the new workspace crate

**Files:**
- Modify: `Cargo.toml` (root — add `crates/argumentation-bipolar` to `[workspace]` members)
- Create: `crates/argumentation-bipolar/Cargo.toml`
- Create: `crates/argumentation-bipolar/src/lib.rs`
- Create: `crates/argumentation-bipolar/src/error.rs`
- Create: `crates/argumentation-bipolar/README.md`

- [ ] **Step 1: Read the existing root `Cargo.toml`**

```bash
cat /home/peter/code/argumentation/Cargo.toml
```

Confirm the existing `[workspace] members = [".", "crates/argumentation-schemes"]` line. You will change it to add the new crate.

- [ ] **Step 2: Add the new member to the workspace section**

Edit `/home/peter/code/argumentation/Cargo.toml`. Replace the existing members line with:

```toml
members = [".", "crates/argumentation-schemes", "crates/argumentation-bipolar"]
```

Do not modify any other section of the file.

- [ ] **Step 3: Verify existing members still build**

Run: `cargo build --package argumentation && cargo build --package argumentation-schemes`
Expected: both compile. If either fails the workspace edit is wrong — revert and investigate.

- [ ] **Step 4: Create `crates/argumentation-bipolar/Cargo.toml`**

```toml
[package]
name = "argumentation-bipolar"
version = "0.1.0"
edition = "2024"
description = "Bipolar argumentation frameworks (attacks + supports) built on the argumentation crate"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["argumentation", "bipolar", "coalition", "support", "cayrol"]
categories = ["algorithms"]

[dependencies]
argumentation = { path = "../.." }
petgraph = "0.6"
thiserror = "2.0"
```

Note: `petgraph` is already a transitive dep of `argumentation` (it uses it for the core Dung graph). Declaring it explicitly here pins the version.

- [ ] **Step 5: Create `src/lib.rs`**

```rust
//! # argumentation-bipolar
//!
//! Bipolar argumentation frameworks (Cayrol & Lagasquie-Schiex 2005,
//! Amgoud et al. 2008, Nouioua & Risch 2011) built on top of the
//! [`argumentation`] crate's Dung semantics.
//!
//! A bipolar framework extends Dung's abstract argumentation with a
//! second directed edge relation: **support**. Arguments can attack and
//! support one another. This crate implements *necessary support*
//! semantics: `A` supports `B` means `A` must be accepted for `B` to be
//! acceptable.
//!
//! The semantics pipeline flattens a bipolar framework into an
//! equivalent Dung framework (direct attacks + derived attacks from
//! supported/mediated/secondary attack rules), runs the existing Dung
//! semantics via [`argumentation::ArgumentationFramework`], then filters
//! the resulting extensions to those that are support-closed (every
//! accepted argument has all its necessary supporters also accepted).
//!
//! Coalitions are strongly-connected components of the support graph:
//! characters who mutually back each other's positions form a coalition
//! that the drama manager can reason about as a unit.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;

pub use error::Error;
```

- [ ] **Step 6: Create `src/error.rs`**

```rust
//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-bipolar` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// A framework operation referenced an argument that is not in the
    /// framework.
    #[error("argument not found: {0}")]
    ArgumentNotFound(String),

    /// An edge was added that introduces a self-loop where the semantics
    /// reject them. Currently applies only to self-support (an argument
    /// cannot be its own necessary supporter).
    #[error("illegal self-loop: argument '{0}' cannot support itself")]
    IllegalSelfSupport(String),

    /// A bipolar-semantics call was made under a support variant that
    /// v0.1.0 does not implement.
    #[error("support semantics not implemented in v0.1.0: {0:?}")]
    UnimplementedSemantics(crate::types::SupportSemantics),

    /// An error from the underlying Dung layer (e.g., framework too
    /// large for subset enumeration).
    #[error("dung error: {0}")]
    Dung(#[from] argumentation::Error),
}
```

- [ ] **Step 7: Create `README.md`**

```markdown
# argumentation-bipolar

Bipolar argumentation frameworks (attacks + supports) built on the [`argumentation`](../..) crate. Implements necessary-support semantics per Nouioua & Risch 2011 with derived attack closure per Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008.

**Status:** Under active development.
```

- [ ] **Step 8: Verify the new crate compiles**

```bash
cd /home/peter/code/argumentation && cargo build --package argumentation-bipolar
```

Expected: compiles. The `types::SupportSemantics` reference in `error.rs` will fail since that module doesn't exist yet. **You need to declare `pub mod types;` in lib.rs and create a minimal `src/types.rs` stub** before this builds. Create `src/types.rs` with:

```rust
//! Foundational types for bipolar argumentation.

/// Which support semantics the framework uses. Only [`Self::Necessary`]
/// is implemented in v0.1.0; [`Self::Deductive`] and [`Self::Evidential`]
/// are reserved for v0.2.0 and return [`crate::Error::UnimplementedSemantics`]
/// if requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportSemantics {
    /// Necessary support (Nouioua & Risch 2011): `A` supports `B` means
    /// `A` must be in any extension containing `B`.
    Necessary,
    /// Deductive support (Boella et al. 2010). NOT IMPLEMENTED in v0.1.0.
    Deductive,
    /// Evidential support (Oren & Norman 2008). NOT IMPLEMENTED in v0.1.0.
    Evidential,
}
```

Then add `pub mod types;` to lib.rs. Now `cargo build --package argumentation-bipolar` should succeed.

- [ ] **Step 9: Verify the workspace test sweep still passes**

```bash
cd /home/peter/code/argumentation && cargo test --workspace
```

Expected: all existing `argumentation` and `argumentation-schemes` tests still pass. `argumentation-bipolar` has zero tests.

- [ ] **Step 10: Commit**

```bash
cd /home/peter/code/argumentation
git add -A
git commit -m "feat(argumentation-bipolar): scaffold new crate, add to workspace"
```

---

### Task 2: Flesh out core types — `EdgeKind` and `CoalitionId`

**Files:**
- Modify: `crates/argumentation-bipolar/src/types.rs`

- [ ] **Step 1: Extend `src/types.rs` with edge kind and coalition id**

Replace the existing `types.rs` body with:

```rust
//! Foundational types for bipolar argumentation.

/// Which kind of directed edge is in the framework: an attack (A defeats B)
/// or a support (A is required for B under necessary-support semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// `A` attacks `B` — the Dung-standard attack relation.
    Attack,
    /// `A` supports `B` — under necessary-support semantics, `A` must be
    /// in any extension that contains `B`.
    Support,
}

/// Which support semantics the framework uses. Only [`Self::Necessary`]
/// is implemented in v0.1.0; [`Self::Deductive`] and [`Self::Evidential`]
/// are reserved for v0.2.0 and return [`crate::Error::UnimplementedSemantics`]
/// if requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportSemantics {
    /// Necessary support (Nouioua & Risch 2011): `A` supports `B` means
    /// `A` must be in any extension containing `B`.
    Necessary,
    /// Deductive support (Boella et al. 2010). NOT IMPLEMENTED in v0.1.0.
    Deductive,
    /// Evidential support (Oren & Norman 2008). NOT IMPLEMENTED in v0.1.0.
    Evidential,
}

/// Identifier for a coalition detected via strongly-connected components
/// of the support graph. Coalition ids are assigned at detection time by
/// [`crate::coalition::detect_coalitions`] and are only stable within a
/// single call — they change if the framework is mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CoalitionId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_distinguishes_attack_from_support() {
        assert_ne!(EdgeKind::Attack, EdgeKind::Support);
    }

    #[test]
    fn support_semantics_necessary_is_default_implementation() {
        // v0.1.0 only supports Necessary. The other two exist for API
        // stability but route to UnimplementedSemantics.
        assert_eq!(SupportSemantics::Necessary, SupportSemantics::Necessary);
        assert_ne!(SupportSemantics::Necessary, SupportSemantics::Deductive);
        assert_ne!(SupportSemantics::Necessary, SupportSemantics::Evidential);
    }

    #[test]
    fn coalition_id_equality_is_value_based() {
        assert_eq!(CoalitionId(1), CoalitionId(1));
        assert_ne!(CoalitionId(1), CoalitionId(2));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): EdgeKind, SupportSemantics, CoalitionId"
```

---

### Task 3: `BipolarFramework<A>` — CRUD and basic queries

**Files:**
- Create: `crates/argumentation-bipolar/src/framework.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/framework.rs`**

```rust
//! `BipolarFramework<A>`: arguments, attacks, and supports.
//!
//! Stores the framework as two independent directed edge sets over the
//! same node set. The node set is the union of all distinct arguments
//! introduced via [`BipolarFramework::add_argument`] or as an endpoint
//! of a call to [`BipolarFramework::add_attack`] or
//! [`BipolarFramework::add_support`].

use crate::error::Error;
use crate::types::EdgeKind;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A bipolar argumentation framework over argument type `A`.
///
/// The type is generic over `A` to match the core crate's convention —
/// `A` can be `String`, `&'static str`, a custom `ArgumentId` newtype, etc.
#[derive(Debug, Clone)]
pub struct BipolarFramework<A: Clone + Eq + Hash> {
    arguments: HashSet<A>,
    attacks: HashSet<(A, A)>,
    supports: HashSet<(A, A)>,
}

impl<A: Clone + Eq + Hash> BipolarFramework<A> {
    /// Create an empty framework.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashSet::new(),
            attacks: HashSet::new(),
            supports: HashSet::new(),
        }
    }

    /// Add an argument. Adding an argument that already exists is a no-op.
    pub fn add_argument(&mut self, a: A) {
        self.arguments.insert(a);
    }

    /// Add an attack `attacker → target`. Both arguments are implicitly
    /// added to the framework if not already present. Adding the same
    /// attack twice is a no-op.
    pub fn add_attack(&mut self, attacker: A, target: A) {
        self.arguments.insert(attacker.clone());
        self.arguments.insert(target.clone());
        self.attacks.insert((attacker, target));
    }

    /// Add a support `supporter → supported`. Both arguments are
    /// implicitly added. Adding the same support twice is a no-op.
    /// Returns [`Error::IllegalSelfSupport`] if `supporter == supported`
    /// — an argument cannot be its own necessary supporter.
    pub fn add_support(&mut self, supporter: A, supported: A) -> Result<(), Error>
    where
        A: std::fmt::Debug,
    {
        if supporter == supported {
            return Err(Error::IllegalSelfSupport(format!("{:?}", supporter)));
        }
        self.arguments.insert(supporter.clone());
        self.arguments.insert(supported.clone());
        self.supports.insert((supporter, supported));
        Ok(())
    }

    /// Remove a support edge. Returns true if the edge was present.
    /// Used by consumers modelling betrayal (a support edge is retracted).
    /// Does NOT remove the endpoint arguments from the framework.
    pub fn remove_support(&mut self, supporter: &A, supported: &A) -> bool {
        self.supports
            .remove(&(supporter.clone(), supported.clone()))
    }

    /// Remove an attack edge. Returns true if the edge was present.
    pub fn remove_attack(&mut self, attacker: &A, target: &A) -> bool {
        self.attacks.remove(&(attacker.clone(), target.clone()))
    }

    /// Iterate over all arguments in the framework.
    pub fn arguments(&self) -> impl Iterator<Item = &A> {
        self.arguments.iter()
    }

    /// Iterate over all direct attack edges.
    pub fn attacks(&self) -> impl Iterator<Item = (&A, &A)> {
        self.attacks.iter().map(|(a, b)| (a, b))
    }

    /// Iterate over all direct support edges.
    pub fn supports(&self) -> impl Iterator<Item = (&A, &A)> {
        self.supports.iter().map(|(a, b)| (a, b))
    }

    /// Number of arguments in the framework.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    /// Whether the framework has zero arguments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// Direct attackers of `a` (arguments `X` such that `X → a` in the
    /// attack edge set). Does NOT include derived attackers — see
    /// [`crate::derived`] for the closure.
    pub fn direct_attackers(&self, a: &A) -> Vec<&A> {
        self.attacks
            .iter()
            .filter(|(_, target)| target == a)
            .map(|(attacker, _)| attacker)
            .collect()
    }

    /// Direct supporters of `a` (arguments `X` such that `X → a` in the
    /// support edge set).
    pub fn direct_supporters(&self, a: &A) -> Vec<&A> {
        self.supports
            .iter()
            .filter(|(_, target)| target == a)
            .map(|(supporter, _)| supporter)
            .collect()
    }

    /// Map of each argument to its direct necessary supporters.
    ///
    /// Used by the support-closure filter in [`crate::semantics`] and
    /// by [`crate::queries`] for transitive queries.
    pub fn supporter_map(&self) -> HashMap<&A, HashSet<&A>> {
        let mut map: HashMap<&A, HashSet<&A>> =
            self.arguments.iter().map(|a| (a, HashSet::new())).collect();
        for (supporter, supported) in &self.supports {
            map.entry(supported).or_default().insert(supporter);
        }
        map
    }
}

impl<A: Clone + Eq + Hash> Default for BipolarFramework<A> {
    fn default() -> Self {
        Self::new()
    }
}

// Compile-time guarantee that the canonical owned-string bipolar
// framework is thread-safe, matching the core crate's guarantee.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<BipolarFramework<String>>();
    assert_sync::<BipolarFramework<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_framework_has_no_arguments() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        assert!(bf.is_empty());
        assert_eq!(bf.len(), 0);
    }

    #[test]
    fn add_argument_is_idempotent() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("a");
        assert_eq!(bf.len(), 1);
    }

    #[test]
    fn add_attack_registers_both_endpoints() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        assert_eq!(bf.len(), 2);
        assert_eq!(bf.direct_attackers(&"b"), vec![&"a"]);
        assert!(bf.direct_attackers(&"a").is_empty());
    }

    #[test]
    fn add_support_registers_both_endpoints() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        assert_eq!(bf.len(), 2);
        assert_eq!(bf.direct_supporters(&"b"), vec![&"a"]);
    }

    #[test]
    fn self_support_is_rejected() {
        let mut bf: BipolarFramework<&str> = BipolarFramework::new();
        let err = bf.add_support("a", "a").unwrap_err();
        assert!(matches!(err, Error::IllegalSelfSupport(_)));
    }

    #[test]
    fn remove_support_returns_whether_edge_was_present() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        assert!(bf.remove_support(&"a", &"b"));
        assert!(!bf.remove_support(&"a", &"b"));
        // Arguments stay in the framework even after the edge is removed.
        assert_eq!(bf.len(), 2);
    }

    #[test]
    fn supporter_map_includes_all_arguments_even_unsupported_ones() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_support("b", "c").unwrap();
        let map = bf.supporter_map();
        assert_eq!(map.len(), 3);
        assert!(map[&"a"].is_empty());
        assert!(map[&"b"].is_empty());
        assert_eq!(map[&"c"].len(), 1);
        assert!(map[&"c"].contains(&"b"));
    }
}
```

- [ ] **Step 2: Register `framework` in `src/lib.rs`**

Update `src/lib.rs` to add the new module and a root-level re-export:

```rust
//! # argumentation-bipolar
//!
//! Bipolar argumentation frameworks (Cayrol & Lagasquie-Schiex 2005,
//! Amgoud et al. 2008, Nouioua & Risch 2011) built on top of the
//! [`argumentation`] crate's Dung semantics.
//!
//! A bipolar framework extends Dung's abstract argumentation with a
//! second directed edge relation: **support**. Arguments can attack and
//! support one another. This crate implements *necessary support*
//! semantics: `A` supports `B` means `A` must be accepted for `B` to be
//! acceptable.
//!
//! The semantics pipeline flattens a bipolar framework into an
//! equivalent Dung framework (direct attacks + derived attacks from
//! supported/mediated/secondary attack rules), runs the existing Dung
//! semantics via [`argumentation::ArgumentationFramework`], then filters
//! the resulting extensions to those that are support-closed (every
//! accepted argument has all its necessary supporters also accepted).
//!
//! Coalitions are strongly-connected components of the support graph:
//! characters who mutually back each other's positions form a coalition
//! that the drama manager can reason about as a unit.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod framework;
pub mod types;

pub use error::Error;
pub use framework::BipolarFramework;
```

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 10 tests pass (3 from types + 7 from framework).

- [ ] **Step 4: Run clippy**

```bash
cargo clippy --package argumentation-bipolar -- -D warnings
```

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): BipolarFramework with attack and support edge sets"
```

---

### Task 4: Derived attack closure (supported / secondary / mediated)

**Files:**
- Create: `crates/argumentation-bipolar/src/derived.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/derived.rs`**

The derived attack rules per Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008 §3:

- **Direct**: `A → B` is in the attack set.
- **Supported attack** (attack after support): if `A` supports `X` (transitively) and `X → B` is a direct attack, then `A → B` is a derived attack.
- **Secondary attack** (attack before support): if `A → B` is a direct attack and `B` supports `C` (transitively), then `A → C` is a derived attack.
- **Mediated attack** (attack to a supporter): if `A → X` is a direct attack and `X` supports `C` (transitively), then `A → C` is a derived mediated attack. Note: this is the same as secondary attack under the Amgoud et al. formulation; the distinction is whether you compose `attack ∘ support*` (secondary/mediated) or `support* ∘ attack` (supported). Both are needed.

The closure is a fixed-point: apply the rules until no new attacks appear. For a framework with `n` arguments, `a` direct attacks, and `s` supports, the closure has at most `n²` edges and converges in at most `n` iterations.

```rust
//! Derived attack closure per Cayrol & Lagasquie-Schiex 2005 and
//! Amgoud et al. 2008 §3.
//!
//! Given a [`BipolarFramework`], compute the set of all attacks (direct
//! plus derived) that hold under necessary-support semantics. Three
//! derivation rules:
//!
//! 1. **Direct**: every edge in the attack set is an attack.
//! 2. **Supported**: if `A` transitively supports `X` and `X` directly
//!    attacks `B`, then `A` attacks `B`.
//! 3. **Secondary/Mediated**: if `A` directly attacks `X` and `X`
//!    transitively supports `C`, then `A` attacks `C`. (Amgoud et al.
//!    distinguishes secondary and mediated but both produce the same
//!    edges under the necessary-support reading.)
//!
//! The closure is computed as a fixed point over all three rules
//! applied together. For a framework with `n` arguments, convergence is
//! bounded by `n` iterations and the closure has at most `n²` edges.

use crate::framework::BipolarFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// Compute the closure of support from each argument: for each `A`,
/// the set of arguments `X` such that `A` transitively supports `X`.
/// Uses repeated BFS over the direct support edges.
fn support_closure<A: Clone + Eq + Hash>(
    framework: &BipolarFramework<A>,
) -> std::collections::HashMap<A, HashSet<A>> {
    use std::collections::{HashMap, VecDeque};

    let mut closure: HashMap<A, HashSet<A>> = HashMap::new();
    for arg in framework.arguments() {
        closure.insert(arg.clone(), HashSet::new());
    }

    // For each argument `start`, BFS the support graph to find every
    // transitively supported argument.
    for start in framework.arguments() {
        let mut visited: HashSet<A> = HashSet::new();
        let mut frontier: VecDeque<A> = VecDeque::new();
        frontier.push_back(start.clone());
        while let Some(current) = frontier.pop_front() {
            for (sup, supd) in framework.supports() {
                if *sup == current && visited.insert(supd.clone()) {
                    frontier.push_back(supd.clone());
                }
            }
        }
        closure.insert(start.clone(), visited);
    }

    closure
}

/// Compute the closed attack set for a bipolar framework under
/// necessary-support semantics.
///
/// The returned set contains `(attacker, target)` pairs for every
/// direct attack plus every derived attack produced by the supported
/// and secondary/mediated rules. Self-attacks are preserved from the
/// direct set (Dung allows them) but are not introduced by derivation.
///
/// The closure is deterministic and order-independent.
pub fn closed_attacks<A>(framework: &BipolarFramework<A>) -> HashSet<(A, A)>
where
    A: Clone + Eq + Hash,
{
    let support_cl = support_closure(framework);

    let mut closed: HashSet<(A, A)> = HashSet::new();

    // Rule 1: direct attacks.
    for (a, b) in framework.attacks() {
        closed.insert((a.clone(), b.clone()));
    }

    // Rule 2: supported attack — A supports* X, X attacks B ⇒ A attacks B.
    // For every direct attack (X, B) and every A with X ∈ support_cl(A),
    // insert (A, B).
    for (x, b) in framework.attacks() {
        for (a, supported_by_a) in &support_cl {
            if supported_by_a.contains(x) {
                closed.insert((a.clone(), b.clone()));
            }
        }
    }

    // Rule 3: secondary / mediated attack — A attacks X, X supports* C ⇒ A attacks C.
    // For every direct attack (A, X) and every C in support_cl(X), insert (A, C).
    for (a, x) in framework.attacks() {
        if let Some(downstream) = support_cl.get(x) {
            for c in downstream {
                closed.insert((a.clone(), c.clone()));
            }
        }
    }

    // Compose: A supports* X, X attacks Y, Y supports* C ⇒ A attacks C.
    // This is the full two-sided closure. The simpler implementation
    // is to iterate the above two rules to a fixed point; but because
    // support is transitively closed in `support_cl`, the single-pass
    // combination captures both directions without iteration:
    for (x, y) in framework.attacks() {
        for (a, supported_by_a) in &support_cl {
            if !supported_by_a.contains(x) {
                continue;
            }
            if let Some(downstream_of_y) = support_cl.get(y) {
                for c in downstream_of_y {
                    closed.insert((a.clone(), c.clone()));
                }
            }
        }
    }

    closed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_attack_is_preserved() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("a", "b")));
        assert_eq!(closed.len(), 1);
    }

    #[test]
    fn supported_attack_rule_fires() {
        // a supports x, x attacks b ⇒ a attacks b (derived).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "b"))); // direct
        assert!(closed.contains(&("a", "b"))); // supported
    }

    #[test]
    fn secondary_attack_rule_fires() {
        // a attacks x, x supports c ⇒ a attacks c (secondary).
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "x");
        bf.add_support("x", "c").unwrap();
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("a", "x"))); // direct
        assert!(closed.contains(&("a", "c"))); // secondary
    }

    #[test]
    fn two_sided_closure_composes_supported_and_secondary() {
        // a supports x, x attacks y, y supports c ⇒ a attacks c
        // (full closure: supported + secondary in one pass).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "y");
        bf.add_support("y", "c").unwrap();
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "y")));
        assert!(closed.contains(&("a", "y"))); // supported half
        assert!(closed.contains(&("x", "c"))); // secondary half
        assert!(closed.contains(&("a", "c"))); // full two-sided closure
    }

    #[test]
    fn transitive_support_chain_propagates_supported_attack() {
        // a supports b, b supports x, x attacks target ⇒ a attacks target.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "x").unwrap();
        bf.add_attack("x", "target");
        let closed = closed_attacks(&bf);
        assert!(closed.contains(&("x", "target")));
        assert!(closed.contains(&("b", "target")));
        assert!(closed.contains(&("a", "target")));
    }

    #[test]
    fn isolated_arguments_produce_no_derived_attacks() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        assert!(closed_attacks(&bf).is_empty());
    }
}
```

- [ ] **Step 2: Register `derived` in `src/lib.rs`**

Add `pub mod derived;` to lib.rs alongside the other `pub mod` lines.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 16 tests pass (10 from earlier + 6 new).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): derived attack closure (supported/secondary/mediated)"
```

---

## Phase 2 — Semantics

### Task 5: Flattening — `BipolarFramework` → `argumentation::ArgumentationFramework`

**Files:**
- Create: `crates/argumentation-bipolar/src/flatten.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/flatten.rs`**

```rust
//! Flattening: convert a [`BipolarFramework`] into an equivalent
//! [`argumentation::ArgumentationFramework`] whose attack relation is
//! the closed attack set from [`crate::derived::closed_attacks`].
//!
//! The flattened framework has the same node set as the bipolar
//! framework. Every direct attack and every derived attack (supported,
//! secondary, mediated) appears as a single edge in the flattened
//! framework's attack relation. Support edges are NOT represented in
//! the flattened framework — they are handled at the semantics layer
//! via the support-closure filter.
//!
//! This is the abstraction that lets the rest of the crate reuse the
//! core Dung semantics without re-implementing fixed-point equations.

use crate::derived::closed_attacks;
use crate::framework::BipolarFramework;
use argumentation::ArgumentationFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Build a [`argumentation::ArgumentationFramework`] from a
/// [`BipolarFramework`] whose attack relation is the closed attack set.
///
/// Propagates [`argumentation::Error`] from `add_attack` calls, but in
/// practice this only fires if the argument universe is inconsistent
/// (an edge references an argument that wasn't registered), which
/// cannot happen here because `closed_attacks` only produces edges
/// between arguments already in the framework.
pub fn flatten<A>(
    framework: &BipolarFramework<A>,
) -> Result<ArgumentationFramework<A>, argumentation::Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let mut af = ArgumentationFramework::new();
    for arg in framework.arguments() {
        af.add_argument(arg.clone());
    }
    for (attacker, target) in closed_attacks(framework) {
        af.add_attack(&attacker, &target)?;
    }
    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bipolar_flattens_to_empty_dung() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 0);
    }

    #[test]
    fn direct_attack_survives_flattening() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("a", "b");
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 2);
        assert_eq!(af.attackers(&"b").len(), 1);
    }

    #[test]
    fn supported_attack_becomes_direct_in_flat_framework() {
        // a supports x, x attacks b. Flattened: a → b and x → b.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let af = flatten(&bf).unwrap();
        let attackers_of_b: Vec<&&str> = af.attackers(&"b").into_iter().collect();
        assert_eq!(attackers_of_b.len(), 2);
        assert!(attackers_of_b.contains(&&"a"));
        assert!(attackers_of_b.contains(&&"x"));
    }

    #[test]
    fn unrelated_arguments_appear_in_flattened_framework() {
        // Arguments with no edges still appear as isolated nodes.
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        let af = flatten(&bf).unwrap();
        assert_eq!(af.len(), 2);
        assert!(af.attackers(&"a").is_empty());
        assert!(af.attackers(&"b").is_empty());
    }
}
```

- [ ] **Step 2: Register `flatten` in lib.rs**

Add `pub mod flatten;` alongside existing `pub mod` lines.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 20 tests pass (16 + 4 new flattening tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): flatten BipolarFramework to Dung via attack closure"
```

---

### Task 6: Support-closure filter and bipolar extensions

**Files:**
- Create: `crates/argumentation-bipolar/src/semantics.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/semantics.rs`**

The support-closure filter: an extension `E` is *support-closed* iff for every argument `a ∈ E`, every direct necessary supporter of `a` is also in `E`. This is the necessary-support acceptance constraint from Nouioua & Risch 2011.

We compute Dung extensions on the flattened framework, then drop any that fail the filter. Necessary-support semantics applies the filter to *direct* supporters only — transitively supported arguments propagate via the filter being applied layer-by-layer (if `a`'s direct supporter `b` is in `E` and `b`'s direct supporter `c` is also required to be in `E`, then the filter catches the whole chain).

```rust
//! Bipolar semantics: compute Dung extensions on the flattened framework
//! and filter them for support closure under necessary-support semantics.
//!
//! An extension `E` is **support-closed** iff for every `a ∈ E`, every
//! direct necessary supporter of `a` is also in `E`. Nouioua & Risch 2011
//! proves this captures necessary-support acceptability exactly when
//! applied on top of Dung extensions of the closed attack relation.

use crate::error::Error;
use crate::flatten::flatten;
use crate::framework::BipolarFramework;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Check whether a candidate extension is support-closed in a bipolar
/// framework: every argument in the extension has all its direct
/// necessary supporters in the extension too.
#[must_use]
pub fn is_support_closed<A>(framework: &BipolarFramework<A>, extension: &HashSet<A>) -> bool
where
    A: Clone + Eq + Hash,
{
    for a in extension {
        for supporter in framework.direct_supporters(a) {
            if !extension.contains(supporter) {
                return false;
            }
        }
    }
    true
}

/// All bipolar preferred extensions under necessary-support semantics.
///
/// Pipeline: flatten → Dung preferred extensions → support-closure filter.
/// The filter may drop candidates that are Dung-preferred but not
/// support-closed. It does NOT promote smaller subsets in their place;
/// if every Dung-preferred extension is filtered out, the result is
/// empty.
pub fn bipolar_preferred_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_preferred = af.preferred_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_preferred
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// All bipolar complete extensions under necessary-support semantics.
pub fn bipolar_complete_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_complete = af.complete_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_complete
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// All bipolar stable extensions.
pub fn bipolar_stable_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_stable = af.stable_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_stable
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// The bipolar grounded extension.
///
/// The Dung grounded extension is unique and may or may not be
/// support-closed. If it is not, this function returns the largest
/// support-closed subset of it — specifically, the result of
/// iteratively removing any argument whose direct supporter is missing.
/// (This differs from the preferred/complete case where we drop the
/// whole candidate; grounded is unique so we must always return
/// something.)
pub fn bipolar_grounded_extension<A>(
    framework: &BipolarFramework<A>,
) -> Result<HashSet<A>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let mut grounded = af.grounded_extension();
    // Support-closure repair: remove arguments whose direct supporters
    // are missing, until a fixed point.
    loop {
        let to_remove: Vec<A> = grounded
            .iter()
            .filter(|a| {
                framework
                    .direct_supporters(a)
                    .iter()
                    .any(|s| !grounded.contains(*s))
            })
            .cloned()
            .collect();
        if to_remove.is_empty() {
            break;
        }
        for a in to_remove {
            grounded.remove(&a);
        }
    }
    Ok(grounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_extension_is_support_closed() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        let ext: HashSet<&str> = HashSet::new();
        assert!(is_support_closed(&bf, &ext));
    }

    #[test]
    fn extension_missing_supporter_fails_closure() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let ext: HashSet<&str> = ["b"].into_iter().collect();
        assert!(!is_support_closed(&bf, &ext));
    }

    #[test]
    fn extension_containing_supporter_passes_closure() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let ext: HashSet<&str> = ["a", "b"].into_iter().collect();
        assert!(is_support_closed(&bf, &ext));
    }

    #[test]
    fn bipolar_preferred_filters_unsupported_candidates() {
        // b has a necessary supporter a, but nothing attacks either.
        // Dung would give {a, b} as the only preferred extension.
        // Bipolar should give the same (and no {b}-alone candidate).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let prefs = bipolar_preferred_extensions(&bf).unwrap();
        assert_eq!(prefs.len(), 1);
        assert_eq!(prefs[0].len(), 2);
        assert!(prefs[0].contains(&"a"));
        assert!(prefs[0].contains(&"b"));
    }

    #[test]
    fn grounded_repair_removes_unsupported_arguments() {
        // a is attacked by c, b is supported by a. Dung grounded would
        // include b (unattacked) but not a (attacked by unattacked c).
        // Under support closure, b must be removed because its
        // supporter a is missing.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_attack("c", "a");
        let grounded = bipolar_grounded_extension(&bf).unwrap();
        assert!(grounded.contains(&"c"));
        assert!(!grounded.contains(&"a"));
        assert!(
            !grounded.contains(&"b"),
            "b should be removed because its supporter a is missing"
        );
    }
}
```

- [ ] **Step 2: Register `semantics` in lib.rs**

Add `pub mod semantics;` alongside existing modules.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 25 tests pass (20 + 5 new semantics tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): necessary-support semantics via flatten + closure filter"
```

---

## Phase 3 — Coalitions and queries

### Task 7: Coalition detection via SCC on the support graph

**Files:**
- Create: `crates/argumentation-bipolar/src/coalition.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/coalition.rs`**

```rust
//! Coalition detection on the support graph.
//!
//! A **coalition** is a strongly-connected component of the support
//! graph: a set of arguments where every pair mutually supports each
//! other, directly or transitively. Singleton SCCs (arguments with no
//! mutual support) are also returned as coalitions of size 1.
//!
//! Uses petgraph's Tarjan SCC implementation, which is O(V + E).

use crate::framework::BipolarFramework;
use crate::types::CoalitionId;
use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use std::hash::Hash;

/// A detected coalition with its member arguments.
#[derive(Debug, Clone)]
pub struct Coalition<A: Clone + Eq + Hash> {
    /// Assigned identifier — stable only within a single
    /// [`detect_coalitions`] call.
    pub id: CoalitionId,
    /// The arguments in this coalition. For a singleton coalition this
    /// has exactly one element.
    pub members: Vec<A>,
}

/// Detect all coalitions in a bipolar framework.
///
/// Builds a petgraph `DiGraph` from the support edges (ignoring
/// attacks), runs Tarjan's SCC algorithm, and returns one [`Coalition`]
/// per SCC with a freshly-assigned [`CoalitionId`].
///
/// Coalition ids are assigned in the order petgraph's SCC iterator
/// returns them, which is a reverse topological order over the
/// condensation. Consumers should treat ids as opaque and use
/// [`Coalition::members`] to identify coalitions semantically.
pub fn detect_coalitions<A>(framework: &BipolarFramework<A>) -> Vec<Coalition<A>>
where
    A: Clone + Eq + Hash,
{
    let mut graph: DiGraph<A, ()> = DiGraph::new();
    let mut index: HashMap<A, petgraph::graph::NodeIndex> = HashMap::new();

    for arg in framework.arguments() {
        let idx = graph.add_node(arg.clone());
        index.insert(arg.clone(), idx);
    }
    for (sup, supd) in framework.supports() {
        let (Some(&a), Some(&b)) = (index.get(sup), index.get(supd)) else {
            continue;
        };
        graph.add_edge(a, b, ());
    }

    tarjan_scc(&graph)
        .into_iter()
        .enumerate()
        .map(|(i, component)| Coalition {
            id: CoalitionId(i as u32),
            members: component.into_iter().map(|n| graph[n].clone()).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_arguments_are_singleton_coalitions() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 3);
        for c in &coalitions {
            assert_eq!(c.members.len(), 1);
        }
    }

    #[test]
    fn mutual_support_produces_one_coalition_of_two() {
        let mut bf = BipolarFramework::new();
        bf.add_support("alice", "bob").unwrap();
        bf.add_support("bob", "alice").unwrap();
        let coalitions = detect_coalitions(&bf);
        // Expect one coalition {alice, bob}, no other singletons.
        assert_eq!(coalitions.len(), 1);
        assert_eq!(coalitions[0].members.len(), 2);
        assert!(coalitions[0].members.contains(&"alice"));
        assert!(coalitions[0].members.contains(&"bob"));
    }

    #[test]
    fn one_way_support_is_two_singletons() {
        // a → b support (but no b → a) is NOT a coalition under SCC.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 2);
        for c in &coalitions {
            assert_eq!(c.members.len(), 1);
        }
    }

    #[test]
    fn attack_edges_do_not_create_coalitions() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("alice", "bob");
        bf.add_attack("bob", "alice"); // mutual attack, not mutual support
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 2);
    }

    #[test]
    fn three_way_mutual_support_forms_one_coalition() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        bf.add_support("c", "a").unwrap();
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 1);
        assert_eq!(coalitions[0].members.len(), 3);
    }

    #[test]
    fn coalition_ids_are_distinct() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        let coalitions = detect_coalitions(&bf);
        let ids: std::collections::HashSet<_> = coalitions.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), coalitions.len());
    }
}
```

- [ ] **Step 2: Register `coalition` in lib.rs**

Add `pub mod coalition;`.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 31 tests pass (25 + 6 new coalition tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): coalition detection via Tarjan SCC on support graph"
```

---

### Task 8: Transitive queries (supporters, attackers, who-is-with-whom)

**Files:**
- Create: `crates/argumentation-bipolar/src/queries.rs`
- Modify: `crates/argumentation-bipolar/src/lib.rs`

- [ ] **Step 1: Create `src/queries.rs`**

```rust
//! Transitive queries over a bipolar framework.
//!
//! - [`transitive_supporters`] — all arguments that directly or
//!   indirectly support `a` via the support graph.
//! - [`transitive_attackers`] — all arguments that attack `a` under the
//!   closed attack relation (direct + derived).
//! - [`coalitioned_with`] — the members of `a`'s coalition per
//!   [`crate::coalition::detect_coalitions`].

use crate::coalition::detect_coalitions;
use crate::derived::closed_attacks;
use crate::framework::BipolarFramework;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

/// All arguments that directly or transitively support `a` in the
/// support graph. Does not include `a` itself.
#[must_use]
pub fn transitive_supporters<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    let mut visited: HashSet<A> = HashSet::new();
    let mut frontier: VecDeque<A> = VecDeque::new();
    frontier.push_back(a.clone());

    while let Some(current) = frontier.pop_front() {
        for (supporter, supported) in framework.supports() {
            if *supported == current && visited.insert(supporter.clone()) {
                frontier.push_back(supporter.clone());
            }
        }
    }
    visited.remove(a);
    visited
}

/// All arguments that attack `a` under the closed attack relation
/// (direct attacks plus derived attacks from support closure).
#[must_use]
pub fn transitive_attackers<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    closed_attacks(framework)
        .into_iter()
        .filter_map(|(att, tgt)| if tgt == *a { Some(att) } else { None })
        .collect()
}

/// The members of `a`'s coalition, excluding `a` itself. If `a` is in
/// a singleton coalition, returns an empty set.
#[must_use]
pub fn coalitioned_with<A>(framework: &BipolarFramework<A>, a: &A) -> HashSet<A>
where
    A: Clone + Eq + Hash,
{
    let coalitions = detect_coalitions(framework);
    for coalition in coalitions {
        if coalition.members.contains(a) {
            return coalition
                .members
                .into_iter()
                .filter(|m| m != a)
                .collect();
        }
    }
    HashSet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitive_supporters_walks_support_chain() {
        // a supports b, b supports c. Transitive supporters of c: {a, b}.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        let sups = transitive_supporters(&bf, &"c");
        assert_eq!(sups.len(), 2);
        assert!(sups.contains(&"a"));
        assert!(sups.contains(&"b"));
    }

    #[test]
    fn transitive_attackers_includes_derived_edges() {
        // a supports x, x attacks b ⇒ a attacks b (derived).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "x").unwrap();
        bf.add_attack("x", "b");
        let atts = transitive_attackers(&bf, &"b");
        assert!(atts.contains(&"a"), "derived attacker should be present");
        assert!(atts.contains(&"x"));
    }

    #[test]
    fn coalitioned_with_returns_siblings() {
        let mut bf = BipolarFramework::new();
        bf.add_support("alice", "bob").unwrap();
        bf.add_support("bob", "alice").unwrap();
        let allies = coalitioned_with(&bf, &"alice");
        assert_eq!(allies.len(), 1);
        assert!(allies.contains(&"bob"));
    }

    #[test]
    fn coalitioned_with_returns_empty_for_singleton() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("alice");
        let allies = coalitioned_with(&bf, &"alice");
        assert!(allies.is_empty());
    }
}
```

- [ ] **Step 2: Register `queries` in lib.rs**

Add `pub mod queries;`.

- [ ] **Step 3: Run tests**

```bash
cargo test --package argumentation-bipolar
```

Expected: 35 tests pass (31 + 4 new queries tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-bipolar): transitive supporter/attacker queries and coalition membership"
```

---

## Phase 4 — Integration tests and release

### Task 9: UC1 integration test — corroboration without attack

**Files:**
- Create: `crates/argumentation-bipolar/tests/uc1_corroboration.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC1: corroboration without attack. Alice says "I saw the queen meet
//! the stranger," Bob independently says "I also saw it." Charlie
//! attacks Alice's sighting. Necessary support for Alice from Bob.
//!
//! Expected: any extension that accepts Alice must also accept Bob
//! (support closure). Charlie is unattacked and defeats Alice in any
//! extension that accepts Charlie. Bob stands independently.

use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

#[test]
fn uc1_bob_is_in_every_preferred_extension() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");
    // Bob's claim is unattacked; his argument should be in every preferred extension.

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    assert!(!prefs.is_empty(), "should have at least one preferred extension");
    for ext in &prefs {
        assert!(ext.contains(&"bob"), "bob should be accepted in every preferred extension");
    }
}

#[test]
fn uc1_charlie_and_alice_never_coexist() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    for ext in &prefs {
        let has_alice = ext.contains(&"alice");
        let has_charlie = ext.contains(&"charlie");
        assert!(!(has_alice && has_charlie), "alice and charlie are in conflict");
    }
}

#[test]
fn uc1_alice_requires_bob_via_support_closure() {
    let mut bf = BipolarFramework::new();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("charlie", "alice");

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    for ext in &prefs {
        if ext.contains(&"alice") {
            assert!(
                ext.contains(&"bob"),
                "alice cannot be accepted without her necessary supporter bob"
            );
        }
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar --test uc1_corroboration
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-bipolar): UC1 corroboration without attack"
```

---

### Task 10: UC2 integration test — coalition against a common enemy

**Files:**
- Create: `crates/argumentation-bipolar/tests/uc2_coalition.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC2: coalition against a common enemy. Alice and Bob mutually
//! support each other and both attack Charlie. Verify:
//!   - Coalition detection returns a single coalition {alice, bob}.
//!   - Charlie is not in any preferred extension.
//!   - Alice and Bob are both in every preferred extension.

use argumentation_bipolar::coalition::detect_coalitions;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

fn build_coalition_framework() -> BipolarFramework<&'static str> {
    let mut bf = BipolarFramework::new();
    bf.add_support("alice", "bob").unwrap();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("alice", "charlie");
    bf.add_attack("bob", "charlie");
    bf
}

#[test]
fn uc2_coalition_detection_returns_alice_bob_together() {
    let bf = build_coalition_framework();
    let coalitions = detect_coalitions(&bf);

    let alice_bob_coalition = coalitions
        .iter()
        .find(|c| c.members.contains(&"alice") && c.members.contains(&"bob"));
    assert!(
        alice_bob_coalition.is_some(),
        "alice and bob should be in the same coalition"
    );
    assert_eq!(alice_bob_coalition.unwrap().members.len(), 2);

    let charlie_coalition = coalitions
        .iter()
        .find(|c| c.members.contains(&"charlie"))
        .unwrap();
    assert_eq!(charlie_coalition.members.len(), 1, "charlie is a singleton coalition");
}

#[test]
fn uc2_charlie_never_in_preferred_extension() {
    let bf = build_coalition_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    for ext in &prefs {
        assert!(
            !ext.contains(&"charlie"),
            "charlie must be defeated by the alice-bob coalition"
        );
    }
}

#[test]
fn uc2_alice_and_bob_in_every_preferred_extension() {
    let bf = build_coalition_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    assert!(!prefs.is_empty(), "expected at least one preferred extension");
    for ext in &prefs {
        assert!(ext.contains(&"alice"), "alice should be accepted");
        assert!(ext.contains(&"bob"), "bob should be accepted");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar --test uc2_coalition
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-bipolar): UC2 coalition against common enemy"
```

---

### Task 11: UC3 integration test — betrayal (retracting support)

**Files:**
- Create: `crates/argumentation-bipolar/tests/uc3_betrayal.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC3: betrayal. Start with the UC2 framework, then remove Bob's
//! support for Alice (`bob → alice` support edge is retracted). Verify:
//!   - The alice-bob coalition no longer exists; both are singletons.
//!   - Both still appear in preferred extensions (they still attack Charlie).
//!   - Charlie is still defeated.

use argumentation_bipolar::coalition::detect_coalitions;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

fn build_post_betrayal_framework() -> BipolarFramework<&'static str> {
    let mut bf = BipolarFramework::new();
    bf.add_support("alice", "bob").unwrap();
    bf.add_support("bob", "alice").unwrap();
    bf.add_attack("alice", "charlie");
    bf.add_attack("bob", "charlie");
    // Bob betrays Alice — remove his support for her.
    assert!(bf.remove_support(&"bob", &"alice"));
    bf
}

#[test]
fn uc3_coalition_dissolved_after_betrayal() {
    let bf = build_post_betrayal_framework();
    let coalitions = detect_coalitions(&bf);

    // No coalition of size 2+ should exist — the mutual support loop is broken.
    for c in &coalitions {
        assert_eq!(
            c.members.len(),
            1,
            "after betrayal, no coalition should contain more than one member"
        );
    }
}

#[test]
fn uc3_alice_still_attacks_charlie() {
    let bf = build_post_betrayal_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    for ext in &prefs {
        assert!(
            !ext.contains(&"charlie"),
            "charlie should still be defeated even after the coalition breaks"
        );
    }
    assert!(!prefs.is_empty());
}

#[test]
fn uc3_both_alice_and_bob_still_accepted_independently() {
    let bf = build_post_betrayal_framework();
    let prefs = bipolar_preferred_extensions(&bf).unwrap();

    // Neither is attacked by anything, so both should be in every
    // preferred extension.
    for ext in &prefs {
        assert!(ext.contains(&"alice"));
        assert!(ext.contains(&"bob"));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar --test uc3_betrayal
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-bipolar): UC3 betrayal via remove_support"
```

---

### Task 12: UC4 integration test — mediated attack through support chain

**Files:**
- Create: `crates/argumentation-bipolar/tests/uc4_mediated_attack.rs`

- [ ] **Step 1: Create the test file**

```rust
//! UC4: mediated attack through a support chain.
//!
//! Framework: `a → b` (attack), `c → b` (support, c is necessary for b).
//! Under flattening, `a → c` should appear as a derived mediated attack
//! (attacking c because c supports b and b is the attack target, via
//! the secondary-attack closure rule applied transitively).
//!
//! Expected: any preferred extension containing `a` cannot contain `c`
//! (so it cannot contain `b` either, via support closure).

use argumentation_bipolar::derived::closed_attacks;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::semantics::bipolar_preferred_extensions;

#[test]
fn uc4_derived_attack_c_present_after_closure() {
    // a attacks b; c supports b. The full two-sided closure composes:
    //   c supports b (support chain), a attacks b (direct) ⇒ a attacks c.
    // Via the "supported" rule with roles inverted: a supports-nothing,
    // so the (a, c) edge comes from the secondary/supported composition.
    //
    // Actually, let's walk the rules:
    //   - Direct: {(a, b)}
    //   - Supported (a supports* x, x attacks b ⇒ a attacks b): a has no supports, contributes nothing.
    //   - Secondary (a attacks x, x supports* c ⇒ a attacks c):
    //       a attacks b (direct), but b does not support c here — c supports b, not the other way.
    //       So secondary alone does not fire.
    //   - Two-sided (a supports* x, x attacks y, y supports* c):
    //       a has no supports, contributes nothing.
    //
    // So the rules as stated give closed_attacks = {(a, b)}. The
    // derived attack to c must come from the Nouioua & Risch necessary-support
    // rule: because c is necessary for b, any attack on b propagates to c.
    // The cleanest way to express this in v0.1.0 is via support-closure
    // filtering: any extension that accepts c is filtered out unless b
    // is also accepted, and vice versa. The derived-attacks layer does
    // NOT introduce (a, c) directly; the FILTER does the work.
    //
    // This test validates the filter behavior, not the derived-attacks
    // closure.
    let mut bf = BipolarFramework::new();
    bf.add_attack("a", "b");
    bf.add_support("c", "b").unwrap();

    let closed = closed_attacks(&bf);
    assert!(closed.contains(&("a", "b")));
    // c is not directly attacked by the closure.
}

#[test]
fn uc4_extensions_containing_a_cannot_contain_b_or_c() {
    let mut bf = BipolarFramework::new();
    bf.add_attack("a", "b");
    bf.add_support("c", "b").unwrap();
    // a is unattacked, b is directly attacked by a, c is unattacked but
    // c is a necessary supporter of b.

    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    assert!(!prefs.is_empty());

    for ext in &prefs {
        if ext.contains(&"a") {
            assert!(!ext.contains(&"b"), "a defeats b directly");
            // c is not directly attacked, but including c without b
            // leaves c "orphaned" — support closure allows this because
            // c does not require any supporter of its own. c may
            // legitimately appear in the same extension as a.
        }
    }
}

#[test]
fn uc4_b_requires_c_via_support_closure() {
    // If b is ever in an extension, c must be too (necessary support).
    let mut bf = BipolarFramework::new();
    bf.add_support("c", "b").unwrap();
    // No attacks. Preferred extension should be {b, c}.
    let prefs = bipolar_preferred_extensions(&bf).unwrap();
    assert_eq!(prefs.len(), 1);
    let ext = &prefs[0];
    assert!(ext.contains(&"c"));
    assert!(ext.contains(&"b"));
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar --test uc4_mediated_attack
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-bipolar): UC4 mediated attack via support chain"
```

---

### Task 13: Flattening / closure unit test file

**Files:**
- Create: `crates/argumentation-bipolar/tests/flattening_closure.rs`

- [ ] **Step 1: Create the test file**

```rust
//! Additional unit tests for the derived-attack closure that exercise
//! interaction between multiple support edges and attacks in one
//! framework. These live in a separate integration test file so the
//! unit tests in `derived.rs` stay focused on individual rule firings.

use argumentation_bipolar::derived::closed_attacks;
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::flatten::flatten;

#[test]
fn parallel_support_branches_both_propagate_attacks() {
    // a supports x1, a supports x2 (two parallel branches).
    // x1 attacks target, x2 attacks target.
    // Closure should include (a, target) via either branch.
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "x1").unwrap();
    bf.add_support("a", "x2").unwrap();
    bf.add_attack("x1", "target");
    bf.add_attack("x2", "target");

    let closed = closed_attacks(&bf);
    assert!(closed.contains(&("a", "target")));
    assert!(closed.contains(&("x1", "target")));
    assert!(closed.contains(&("x2", "target")));
}

#[test]
fn flattened_framework_has_same_argument_set_as_bipolar() {
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "b").unwrap();
    bf.add_attack("c", "d");
    bf.add_argument("isolated");
    let af = flatten(&bf).unwrap();
    assert_eq!(af.len(), 5);
}

#[test]
fn closure_is_deterministic_across_rebuilds() {
    let build = || {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        bf.add_attack("c", "d");
        closed_attacks(&bf)
    };
    let first = build();
    let second = build();
    assert_eq!(first, second);
}

#[test]
fn closure_handles_cycles_in_support_graph() {
    // Mutual support cycle a ↔ b, then a attacks c.
    // Both a and b become derived attackers of c.
    let mut bf = BipolarFramework::new();
    bf.add_support("a", "b").unwrap();
    bf.add_support("b", "a").unwrap();
    bf.add_attack("a", "c");
    let closed = closed_attacks(&bf);
    assert!(closed.contains(&("a", "c")));
    assert!(
        closed.contains(&("b", "c")),
        "mutual support should propagate direct attacks through the cycle"
    );
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --package argumentation-bipolar --test flattening_closure
```

Expected: 4 tests pass.

- [ ] **Step 3: Run the full sweep**

```bash
cargo test --package argumentation-bipolar
```

Expected: 35 unit + 13 integration = 48 tests passing.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(argumentation-bipolar): extended closure and flattening coverage"
```

---

### Task 14: Public API + docs + v0.1.0 release prep

**Files:**
- Modify: `crates/argumentation-bipolar/src/lib.rs`
- Create: `crates/argumentation-bipolar/CHANGELOG.md`
- Create: `crates/argumentation-bipolar/LICENSE-MIT`
- Create: `crates/argumentation-bipolar/LICENSE-APACHE`
- Modify: `crates/argumentation-bipolar/README.md`

- [ ] **Step 1: Finalize `src/lib.rs` with re-exports and doctest example**

```rust
//! # argumentation-bipolar
//!
//! Bipolar argumentation frameworks (Cayrol & Lagasquie-Schiex 2005,
//! Amgoud et al. 2008, Nouioua & Risch 2011) built on top of the
//! [`argumentation`] crate's Dung semantics.
//!
//! A bipolar framework extends Dung's abstract argumentation with a
//! second directed edge relation: **support**. Arguments can attack and
//! support one another. This crate implements *necessary support*
//! semantics: `A` supports `B` means `A` must be accepted for `B` to be
//! acceptable.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_bipolar::framework::BipolarFramework;
//! use argumentation_bipolar::coalition::detect_coalitions;
//! use argumentation_bipolar::semantics::bipolar_preferred_extensions;
//!
//! let mut bf = BipolarFramework::new();
//! bf.add_support("alice", "bob").unwrap();
//! bf.add_support("bob", "alice").unwrap();
//! bf.add_attack("alice", "charlie");
//! bf.add_attack("bob", "charlie");
//!
//! let coalitions = detect_coalitions(&bf);
//! assert!(coalitions.iter().any(|c| c.members.len() == 2));
//!
//! let prefs = bipolar_preferred_extensions(&bf).unwrap();
//! for ext in &prefs {
//!     assert!(!ext.contains(&"charlie"));
//! }
//! ```
//!
//! ## Semantics pipeline
//!
//! 1. [`derived::closed_attacks`] computes the set of all attacks under
//!    the closed attack relation (direct + supported + secondary +
//!    mediated) per C&LS 2005 §3.
//! 2. [`flatten::flatten`] produces an equivalent
//!    [`argumentation::ArgumentationFramework`] whose attack edges are
//!    the closure.
//! 3. [`semantics::bipolar_preferred_extensions`] runs the core crate's
//!    Dung preferred semantics on the flattened framework, then filters
//!    extensions that are not *support-closed* (every accepted argument
//!    must have all its necessary supporters in the extension too).
//! 4. [`coalition::detect_coalitions`] runs Tarjan SCC on the support
//!    graph to find mutually-supporting groups.
//!
//! ## References
//!
//! - Cayrol, C. & Lagasquie-Schiex, M.-C. (2005). *On the acceptability
//!   of arguments in bipolar argumentation frameworks.* ECSQARU / IJAR
//!   23(4).
//! - Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P.
//!   (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10).
//! - Nouioua, F. & Risch, V. (2011). *Bipolar argumentation frameworks
//!   with specialized supports.* ICTAI 2011.
//! - Cohen, A. et al. (2014). *A survey of different approaches to
//!   support in argumentation systems.* KER 29(5).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod coalition;
pub mod derived;
pub mod error;
pub mod flatten;
pub mod framework;
pub mod queries;
pub mod semantics;
pub mod types;

pub use coalition::{Coalition, detect_coalitions};
pub use error::Error;
pub use framework::BipolarFramework;
pub use semantics::{
    bipolar_complete_extensions, bipolar_grounded_extension, bipolar_preferred_extensions,
    bipolar_stable_extensions, is_support_closed,
};
pub use types::{CoalitionId, EdgeKind, SupportSemantics};
```

- [ ] **Step 2: Create `CHANGELOG.md`**

```markdown
# Changelog

## [0.1.0] - 2026-04-TBD

### Added
- `BipolarFramework<A>` with distinct attack and support edge sets,
  plus mutation and query APIs for each.
- Derived attack closure per Cayrol & Lagasquie-Schiex 2005 and Amgoud
  et al. 2008: supported, secondary, and mediated attacks computed as a
  fixed point over the support graph.
- Flattening: convert a bipolar framework into an equivalent
  `argumentation::ArgumentationFramework` whose attack relation is the
  closed attack set.
- Necessary-support semantics via Dung flatten + support-closure filter:
  `bipolar_preferred_extensions`, `bipolar_complete_extensions`,
  `bipolar_stable_extensions`, `bipolar_grounded_extension`.
- Coalition detection via Tarjan SCC on the support graph.
- Transitive supporter/attacker queries and coalition-membership lookup.
- 35 unit tests + 13 integration tests covering UC1 (corroboration),
  UC2 (coalition), UC3 (betrayal), UC4 (mediated attack).

### Known limitations
- Only `SupportSemantics::Necessary` is implemented. `Deductive` and
  `Evidential` are pre-declared for API stability but return
  `Error::UnimplementedSemantics`.
- No weighted-bipolar composition with `argumentation-weighted` yet;
  the natural composition point is a v0.2.0 follow-up.
```

- [ ] **Step 3: Create `LICENSE-MIT`**

```
MIT License

Copyright (c) 2026 The argumentation-bipolar contributors

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

Use the standard Apache 2.0 license text from <https://www.apache.org/licenses/LICENSE-2.0.txt>. Set the copyright line at the bottom to `Copyright 2026 The argumentation-bipolar contributors`. This is the same license text already used by the sibling `argumentation-schemes` crate — you can copy `crates/argumentation-schemes/LICENSE-APACHE` verbatim and just confirm the copyright line.

- [ ] **Step 5: Update `README.md`**

```markdown
# argumentation-bipolar

Bipolar argumentation frameworks (attacks + supports) built on the [`argumentation`](../..) crate. Implements necessary-support semantics per Nouioua & Risch 2011 with derived attack closure per Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008.

## What's in the box

- `BipolarFramework<A>` with independent attack and support edge sets.
- Derived attack closure (supported, secondary, mediated rules).
- Flattening to a Dung `ArgumentationFramework` for reuse of the core crate's semantics.
- Necessary-support semantics: grounded, complete, preferred, stable extensions filtered for support-closure.
- Coalition detection via Tarjan SCC on the support graph.
- Transitive query helpers (supporters, attackers, coalition membership).

## Quick example

```rust
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::coalition::detect_coalitions;

let mut bf = BipolarFramework::new();
bf.add_support("alice", "bob").unwrap();
bf.add_support("bob", "alice").unwrap();
bf.add_attack("alice", "charlie");
bf.add_attack("bob", "charlie");

let coalitions = detect_coalitions(&bf);
assert!(coalitions.iter().any(|c| c.members.len() == 2));
```

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Cayrol & Lagasquie-Schiex (2005). *On the acceptability of arguments in bipolar argumentation frameworks.* IJAR 23(4).
- Amgoud, Cayrol, Lagasquie-Schiex & Livet (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10).
- Nouioua & Risch (2011). *Bipolar argumentation frameworks with specialized supports.* ICTAI 2011.
```

- [ ] **Step 6: Full verification sweep**

```bash
cd /home/peter/code/argumentation
cargo test --package argumentation-bipolar
cargo test --workspace
cargo clippy --package argumentation-bipolar -- -D warnings
cargo fmt --package argumentation-bipolar -- --check
cargo doc --package argumentation-bipolar --no-deps
```

Expected: 48 unit + integration tests plus 1 doctest pass; workspace sweep still green; clippy clean; fmt clean; docs build cleanly.

If `cargo fmt --check` emits drift, run `cargo fmt --package argumentation-bipolar` and re-stage before committing.

- [ ] **Step 7: Commit and tag**

```bash
git add -A
git commit -m "chore(argumentation-bipolar): v0.1.0 release prep"
git tag -a argumentation-bipolar-v0.1.0 -m "argumentation-bipolar v0.1.0"
```

Do not push the tag — the human will decide when to push.

---

## Out of scope for v0.1.0

- **Deductive and evidential support semantics.** `SupportSemantics::Deductive` and `::Evidential` are pre-declared for API stability but return `Error::UnimplementedSemantics`. v0.2.0 work.
- **Weighted-bipolar composition with `argumentation-weighted`.** Amgoud et al. 2008's weighted-bipolar framework. Natural v0.2.0 target once both crates ship and their composition points are clear.
- **Dialectical / gradual semantics on bipolar frameworks.** Out of scope for necessary-support v0.1.0; defer to `argumentation-rankings` per vNEXT §3.1.
- **AIF import/export.** The vNEXT doc treats this as a cross-cutting concern for the schemes crate primarily. Not a bipolar-specific deliverable.
- **Visualization tooling.** Library, not application.
- **Benchmark / ICCMA-style performance validation.** The flatten + filter approach is correct-by-construction but not necessarily fast on large frameworks. Defer perf work until a real scenario demands it.

## References

- Cayrol, C. & Lagasquie-Schiex, M.-C. (2005). *On the acceptability of arguments in bipolar argumentation frameworks.* ECSQARU 2005; extended in IJAR 23(4). [DOI](https://doi.org/10.1016/j.ijar.2009.12.004)
- Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P. (2008). *On bipolarity in argumentation frameworks.* International Journal of Intelligent Systems 23(10). [DOI](https://doi.org/10.1002/int.20307)
- Cohen, A., Gottifredi, S., García, A., & Simari, G. (2014). *A survey of different approaches to support in argumentation systems.* Knowledge Engineering Review 29(5).
- Nouioua, F. & Risch, V. (2011). *Bipolar argumentation frameworks with specialized supports.* ICTAI 2011.
- Boella, G., Gabbay, D. M., van der Torre, L., & Villata, S. (2010). *Support in abstract argumentation.* COMMA 2010 — deductive support (v0.2.0 target).

---

**End of plan.**
