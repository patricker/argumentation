# `argumentation-schemes` Crate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust crate implementing Walton argumentation schemes with critical questions, enabling characters in the narrative stack to use recognizable reasoning patterns (argument from expert opinion, ad hominem, argument from consequences, etc.) that produce structured follow-up options for multi-beat encounters.

**Architecture:** Schemes are compile-time data — each is a `SchemeSpec` struct holding premise slots, a conclusion template (with a `negated` flag for rebut-concluding schemes like ad hominem), critical questions, and metadata. A `CatalogRegistry` collects all schemes and supports lookup by name, ID, or category. An ASPIC+ integration module maps scheme instances (schemes + concrete bindings) into `argumentation::aspic::StructuredSystem` primitives so they can be evaluated via Dung semantics. The crate is a new workspace member of the existing `argumentation` repo. It does NOT depend on `encounter` — the encounter-argumentation bridge crate (separate, future work) owns that mapping.

**Tech Stack:** Rust 2024 edition, depends on `argumentation` (workspace path dep), `thiserror` 2.0 for errors. No serde (schemes are code, not data files). No async.

**Status:** Standalone parallel work for the argumentation team, building on the existing `argumentation` v0.2.0 crate at `/home/peter/code/argumentation/`.

**Crate location:** A new workspace member of the `argumentation` repo. Task 1 converts the repo from a single-crate layout to a Cargo workspace. The existing `argumentation` package stays at the repo root as a workspace member; the new crate lives at `crates/argumentation-schemes/`. Both members share the same repo, CI, release cadence, and test sweep.

**Known MVP limitation — premise encoding:** Scheme instantiation produces synthetic literals like `expert_alice`, `domain_military` that encode "alice fills the expert slot in this scheme instance," NOT "alice is a military expert" as a world fact. This means: (a) two scheme instances that both reference Alice's military expertise produce independent premises that can't be shared or unified, and (b) counter-literals like `¬expert_alice` negate "alice plays the expert role" rather than "alice is competent." For v0.1.0 this is functional — the AF evaluates correctly because literals are unique tokens that get attacked and defended — but it blocks multi-instance fact sharing and proper unification. A v0.2.0 `WorldFact` layer that maps scheme slot bindings to shared knowledge-base literals is the planned fix. The plan calls this out so the implementer knows it's intentional, not an oversight.

---

## Use Cases and Validation Criteria

### UC1: Argument from expert opinion in a council encounter

**Scenario:** Alice is an expert in military strategy. In a council scene, she asserts "we should fortify the eastern wall." The scheme is "argument from expert opinion" with bindings: `expert=alice`, `domain=military`, `claim=fortify_east`.

**What must work:** Instantiate the scheme with bindings → get a `SchemeInstance` with 3 premise literals, a positive conclusion literal, and 6 critical-question instances. Feed it into a `StructuredSystem` via the ASPIC+ integration → get premises and a defeasible rule added to the system. The critical questions enumerate as counter-move candidates with resolved text and concrete counter-literals.

**Validates:** Core instantiation pipeline, ASPIC+ integration, critical question enumeration.

### UC2: Ad hominem in a heated debate

**Scenario:** Bob attacks Alice's character: "You're a coward, why should we listen to you?" This is the ad hominem scheme with bindings: `target=alice`, `flaw=cowardice`, `claim=fortify_east`.

**What must work:** Ad hominem is a *negated-conclusion* scheme — its `ConclusionTemplate` has `negated: true` so the instance's conclusion is `Literal::neg("fortify_east")`, directly contrary to Alice's positive conclusion `Literal::atom("fortify_east")`. When both are added to the same `StructuredSystem`, the ASPIC+ attack detector recognises them as rebutting each other. With Alice's rule preferred, the AF's preferred extension contains Alice's argument and not Bob's. With Bob's rule preferred, vice versa.

**Validates:** Negated-conclusion templates, scheme-to-scheme rebut via ASPIC+ AF generation, end-to-end conflict resolution.

### UC3: Catalog coverage and lookup

**Scenario:** The encounter-argumentation bridge asks "given this encounter context, which schemes are available?" The `CatalogRegistry` returns all schemes tagged with the relevant categories (e.g., `SchemeCategory::Practical` for a war council).

**What must work:** `CatalogRegistry::by_category(Practical)` returns all practical schemes. `CatalogRegistry::by_key("argument_from_consequences")` returns the specific scheme. `CatalogRegistry::all()` returns all 25 schemes. Each scheme has a unique ID and unique key. Every category has at least one scheme.

**Validates:** Registry lookup, category filtering, completeness, uniqueness.

### UC4: Critical questions as follow-up beat candidates

**Scenario:** After Alice uses argument from expert opinion, the encounter engine needs the list of follow-up moves for Bob. Each critical question becomes a candidate action.

**What must work:** `instance.critical_questions` is a `Vec<CriticalQuestionInstance>` with 6 items for expert opinion. Each has a `text` with `?slot` references resolved against the bindings, a `challenge` discriminant indicating what aspect of the scheme it targets, and a `counter_literal` (a `Literal::Neg`) that can be asserted as an ordinary premise to undermine the original argument.

**Validates:** Critical question enumeration, template resolution, counter-literal generation.

---

## File Structure

All paths below are relative to `/home/peter/code/argumentation/`.

```
/home/peter/code/argumentation/
├── Cargo.toml                                  # Modified: add [workspace] section
└── crates/
    └── argumentation-schemes/
        ├── Cargo.toml
        ├── README.md
        ├── CHANGELOG.md
        ├── LICENSE-MIT
        ├── LICENSE-APACHE
        ├── src/
        │   ├── lib.rs                          # Public API, crate docs
        │   ├── error.rs                        # Crate errors
        │   ├── types.rs                        # SchemeId, SchemeCategory, SlotRole, SchemeStrength, Challenge
        │   ├── critical.rs                     # CriticalQuestion type
        │   ├── scheme.rs                       # SchemeSpec, PremiseSlot, ConclusionTemplate, SchemeMetadata
        │   ├── instance.rs                     # SchemeInstance, instantiate(), resolve_template()
        │   ├── registry.rs                     # CatalogRegistry — lookup by id, key, category
        │   ├── aspic.rs                        # ASPIC+ integration: add_scheme_to_system, add_counter_argument
        │   └── catalog/
        │       ├── mod.rs                      # ID offset constants, default_catalog() builder
        │       ├── epistemic.rs                # 3 schemes: expert opinion, witness, position to know
        │       ├── practical.rs                # 7 schemes: consequences (3), values, threat, fear, waste
        │       ├── source.rs                   # 4 schemes: ad hominem (2), bias, ethotic
        │       ├── popular.rs                  # 4 schemes: popular, tradition, precedent, rule
        │       ├── causal.rs                   # 4 schemes: cause→effect, correlation, sign, slippery slope
        │       └── analogy.rs                  # 3 schemes: analogy, classification, commitment
        └── tests/
            ├── instantiation.rs                # UC1: end-to-end instantiation
            ├── scheme_conflict.rs              # UC2: expert vs ad hominem rebut
            ├── catalog_coverage.rs             # UC3: registry coverage and uniqueness
            └── critical_questions.rs           # UC4: CQ enumeration
```

Each catalog module is a collection of `pub fn <scheme_name>() -> SchemeSpec` constructors plus a `pub fn all() -> Vec<SchemeSpec>` collector. `catalog/mod.rs` calls each module's `all()` to build the default catalog.

---

## Phase 1 — Workspace Conversion and Core Types

### Task 1: Convert repo to a Cargo workspace and scaffold the new crate

**Files:**
- Modify: `/home/peter/code/argumentation/Cargo.toml`
- Create: `/home/peter/code/argumentation/crates/argumentation-schemes/Cargo.toml`
- Create: `/home/peter/code/argumentation/crates/argumentation-schemes/src/lib.rs`
- Create: `/home/peter/code/argumentation/crates/argumentation-schemes/src/error.rs`
- Create: `/home/peter/code/argumentation/crates/argumentation-schemes/README.md`

- [ ] **Step 1: Read the existing root `Cargo.toml`**

```bash
cat /home/peter/code/argumentation/Cargo.toml
```

Take note of the existing `[package]` and `[dependencies]` sections — they stay exactly as-is.

- [ ] **Step 2: Add a `[workspace]` section to the root Cargo.toml**

Edit `/home/peter/code/argumentation/Cargo.toml` and add the following at the very top of the file (above the existing `[package]` line):

```toml
[workspace]
members = [".", "crates/argumentation-schemes"]
resolver = "2"

```

The blank line after `resolver = "2"` is intentional — it separates the workspace section from the existing package section. Do not modify any other part of the file.

- [ ] **Step 3: Verify the existing crate still builds**

Run: `cd /home/peter/code/argumentation && cargo build --package argumentation`
Expected: compiles cleanly. If you get an error like "current package believes it's in a workspace when it's not", check that the `members` array includes `"."`.

- [ ] **Step 4: Create the new crate's `Cargo.toml`**

Create `/home/peter/code/argumentation/crates/argumentation-schemes/Cargo.toml`:

```toml
[package]
name = "argumentation-schemes"
version = "0.1.0"
edition = "2024"
description = "Walton argumentation schemes with critical questions, built on the argumentation crate"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["argumentation", "schemes", "walton", "reasoning", "critical-questions"]
categories = ["algorithms"]

[dependencies]
argumentation = { path = "../.." }
thiserror = "2.0"
```

The path `"../.."` works because the workspace root's `Cargo.toml` is itself the `argumentation` package. If the `argumentation` team later moves the root package into `crates/argumentation/`, update this dep to `path = "../argumentation"`.

- [ ] **Step 5: Create `src/lib.rs`**

Create `/home/peter/code/argumentation/crates/argumentation-schemes/src/lib.rs`:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;

pub use error::Error;
```

- [ ] **Step 6: Create `src/error.rs`**

Create `/home/peter/code/argumentation/crates/argumentation-schemes/src/error.rs`:

```rust
//! Crate error types.

use thiserror::Error;

/// Errors that can occur in the `argumentation-schemes` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// A scheme instantiation failed because a required binding was missing.
    #[error("missing binding '{slot}' for scheme '{scheme}'")]
    MissingBinding {
        /// The scheme being instantiated.
        scheme: String,
        /// The slot that was not bound.
        slot: String,
    },

    /// A scheme was not found in the registry.
    #[error("scheme not found: {0}")]
    SchemeNotFound(String),

    /// An error from the underlying ASPIC+ layer.
    #[error("aspic error: {0}")]
    Aspic(#[from] argumentation::Error),
}
```

- [ ] **Step 7: Create `README.md`**

Create `/home/peter/code/argumentation/crates/argumentation-schemes/README.md`:

```markdown
# argumentation-schemes

Walton argumentation schemes with critical questions. Built on the [`argumentation`](../..) crate.

**Status:** Under active development.
```

- [ ] **Step 8: Verify the new crate builds**

Run: `cd /home/peter/code/argumentation && cargo build --package argumentation-schemes`
Expected: compiles cleanly. Resolves `argumentation` via the workspace path dep.

- [ ] **Step 9: Verify the workspace test sweep still passes**

Run: `cd /home/peter/code/argumentation && cargo test --workspace`
Expected: all existing argumentation tests pass; argumentation-schemes has zero tests at this point but compiles.

- [ ] **Step 10: Commit**

```bash
cd /home/peter/code/argumentation
git add -A
git commit -m "feat(argumentation-schemes): convert repo to workspace, scaffold new crate"
```

---

### Task 2: Core types — SchemeId, SchemeCategory, SlotRole, SchemeStrength, Challenge

**Files:**
- Create: `crates/argumentation-schemes/src/types.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

All paths below are relative to `/home/peter/code/argumentation/`.

- [ ] **Step 1: Create `crates/argumentation-schemes/src/types.rs`**

```rust
//! Foundational types for argumentation schemes.

/// Unique identifier for a scheme in a catalog.
///
/// IDs are assigned per category via the offset constants in
/// [`crate::catalog`]. Within a category, IDs are sequential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemeId(pub u32);

/// Category of argumentation scheme. Used for catalog filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeCategory {
    /// Knowledge-based: expert opinion, witness testimony, position to know.
    Epistemic,
    /// Cause and effect: cause to effect, correlation, sign.
    Causal,
    /// Action-oriented: consequences, values, goals, waste.
    Practical,
    /// Attacking the source: ad hominem, bias, credibility.
    SourceBased,
    /// Social proof: popularity, tradition, precedent.
    Popular,
    /// Structural reasoning: analogy, classification, commitment.
    Analogical,
}

/// What role a premise slot plays in the scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotRole {
    /// A person or character (the expert, the witness, the attacker).
    Agent,
    /// A proposition being argued for or against.
    Proposition,
    /// A property or trait being attributed to someone.
    Property,
    /// An action being proposed, evaluated, or warned about.
    Action,
    /// A domain, field, or context constraining the argument.
    Domain,
    /// A consequence or outcome.
    Consequence,
}

/// How strong a scheme's inference typically is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeStrength {
    /// Strong presumption (e.g., argument from established rule).
    Strong,
    /// Moderate presumption (e.g., argument from expert opinion).
    Moderate,
    /// Weak presumption (e.g., argument from popularity).
    Weak,
}

/// What aspect of a scheme a critical question challenges.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Challenge {
    /// Challenges the truth of a specific premise (by slot name).
    PremiseTruth(String),
    /// Challenges the credibility or reliability of the source agent.
    SourceCredibility,
    /// Challenges the validity of the inference rule itself.
    RuleValidity,
    /// Raises a conflicting authority or counter-expert.
    ConflictingAuthority,
    /// Raises an alternative cause or explanation.
    AlternativeCause,
    /// Raises unconsidered consequences.
    UnseenConsequences,
    /// Challenges the relevance or proportionality of the attack.
    Proportionality,
    /// Challenges the analogy's structural similarity.
    DisanalogyClaim,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_id_equality_is_value_based() {
        assert_eq!(SchemeId(1), SchemeId(1));
        assert_ne!(SchemeId(1), SchemeId(2));
    }

    #[test]
    fn challenge_distinguishes_premise_slots() {
        let c1 = Challenge::PremiseTruth("expert".into());
        let c2 = Challenge::PremiseTruth("domain".into());
        assert_ne!(c1, c2);
    }
}
```

- [ ] **Step 2: Register `types` in `crates/argumentation-schemes/src/lib.rs`**

Add `pub mod types;` immediately after the existing `pub mod error;` line. The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod types;

pub use error::Error;
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 2 tests pass (`scheme_id_equality_is_value_based`, `challenge_distinguishes_premise_slots`).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): core types — SchemeId, SchemeCategory, SlotRole, Challenge"
```

---

### Task 3: CriticalQuestion + SchemeSpec + ConclusionTemplate (with negated flag)

**Files:**
- Create: `crates/argumentation-schemes/src/critical.rs`
- Create: `crates/argumentation-schemes/src/scheme.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/critical.rs`**

```rust
//! Critical questions for argumentation schemes.

use crate::types::Challenge;

/// A critical question that probes a scheme's weak points.
///
/// Each Walton scheme carries 2-6 critical questions. When a character
/// uses a scheme in an encounter, these become the available follow-up
/// moves for the opposing party.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalQuestion {
    /// Question number within the scheme (1-based).
    pub number: u32,
    /// Human-readable question text with `?slot` references.
    pub text: String,
    /// What aspect of the scheme this question challenges.
    pub challenge: Challenge,
}

impl CriticalQuestion {
    /// Convenience constructor.
    pub fn new(number: u32, text: impl Into<String>, challenge: Challenge) -> Self {
        Self {
            number,
            text: text.into(),
            challenge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_question_stores_challenge() {
        let cq = CriticalQuestion::new(
            1,
            "Is ?expert really an expert in ?domain?",
            Challenge::PremiseTruth("expert".into()),
        );
        assert_eq!(cq.number, 1);
        assert!(cq.text.contains("?expert"));
        assert!(matches!(cq.challenge, Challenge::PremiseTruth(_)));
    }
}
```

- [ ] **Step 2: Create `crates/argumentation-schemes/src/scheme.rs`**

```rust
//! `SchemeSpec`: the compile-time definition of one argumentation scheme.

use crate::critical::CriticalQuestion;
use crate::types::{SchemeCategory, SchemeId, SchemeStrength, SlotRole};

/// A named premise slot in a scheme.
///
/// When instantiated with bindings, each slot maps to a concrete value
/// (e.g., slot "expert" → "alice").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremiseSlot {
    /// Slot name (used as the binding key, e.g., "expert", "claim", "domain").
    pub name: String,
    /// Human-readable description of what this slot represents.
    pub description: String,
    /// What role this slot plays in the scheme.
    pub role: SlotRole,
}

impl PremiseSlot {
    /// Convenience constructor.
    pub fn new(name: impl Into<String>, description: impl Into<String>, role: SlotRole) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            role,
        }
    }
}

/// Template for the scheme's conclusion.
///
/// `literal_template` is a string with `?slot` references that get resolved
/// against the bindings at instantiation time. `is_negated` controls whether
/// the resulting [`argumentation::aspic::Literal`] is constructed via
/// `Literal::neg` (for rebut-concluding schemes like ad hominem) or
/// `Literal::atom`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConclusionTemplate {
    /// Human-readable description (e.g., "?claim is plausibly true").
    pub description: String,
    /// The literal name template. Slot references prefixed with `?` are
    /// replaced with bound values during instantiation.
    pub literal_template: String,
    /// If true, the conclusion is constructed as a negated literal.
    /// Required for rebuttal-concluding schemes (ad hominem, argument
    /// from negative consequences, slippery slope, etc.).
    pub is_negated: bool,
}

impl ConclusionTemplate {
    /// Convenience constructor for a positive (non-negated) conclusion.
    pub fn positive(description: impl Into<String>, literal_template: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            literal_template: literal_template.into(),
            is_negated: false,
        }
    }

    /// Convenience constructor for a negated conclusion (e.g., ad hominem
    /// concluding ¬claim).
    pub fn negated(description: impl Into<String>, literal_template: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            literal_template: literal_template.into(),
            is_negated: true,
        }
    }
}

/// Metadata about a scheme: citation, tags, strength.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeMetadata {
    /// Citation (e.g., "Walton 2008 p.14").
    pub citation: String,
    /// Domain tags for filtering (e.g., ["epistemic", "authority"]).
    pub domain_tags: Vec<String>,
    /// Whether the scheme is presumptive (virtually all Walton schemes are).
    pub presumptive: bool,
    /// How strong the scheme's inference typically is.
    pub strength: SchemeStrength,
}

/// The complete definition of one argumentation scheme.
///
/// A scheme is a recognisable pattern of reasoning with named premise slots,
/// a conclusion template, and critical questions that probe its weak points.
/// Schemes are compile-time data: each is constructed by a function in the
/// [`crate::catalog`] module. Consumers instantiate schemes with concrete
/// bindings via [`SchemeSpec::instantiate`] or [`crate::instance::instantiate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeSpec {
    /// Unique scheme id.
    pub id: SchemeId,
    /// Canonical name (e.g., "Argument from Expert Opinion").
    pub name: String,
    /// Scheme category for catalog filtering.
    pub category: SchemeCategory,
    /// Named premise slots. Order matters — the first N are the scheme's
    /// "core premises" as defined by Walton.
    pub premises: Vec<PremiseSlot>,
    /// Conclusion template. References premise slot names via `?name` syntax.
    pub conclusion: ConclusionTemplate,
    /// Critical questions that probe the scheme's weak points.
    pub critical_questions: Vec<CriticalQuestion>,
    /// Bibliographic and classification metadata.
    pub metadata: SchemeMetadata,
}

impl SchemeSpec {
    /// Instantiate this scheme with concrete bindings. Convenience method
    /// that delegates to [`crate::instance::instantiate`].
    pub fn instantiate(
        &self,
        bindings: &std::collections::HashMap<String, String>,
    ) -> Result<crate::instance::SchemeInstance, crate::Error> {
        crate::instance::instantiate(self, bindings)
    }

    /// The scheme's canonical name as a snake_case identifier suitable
    /// for lookup keys and affordance mapping.
    pub fn key(&self) -> String {
        self.name
            .to_lowercase()
            .replace(' ', "_")
            .replace('-', "_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_key_is_snake_case() {
        let scheme = SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![PremiseSlot::new("expert", "The expert", SlotRole::Agent)],
            conclusion: ConclusionTemplate::positive("?claim is true", "?claim"),
            critical_questions: vec![],
            metadata: SchemeMetadata {
                citation: "Walton 2008".into(),
                domain_tags: vec!["epistemic".into()],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        };
        assert_eq!(scheme.key(), "argument_from_expert_opinion");
    }

    #[test]
    fn conclusion_template_positive_has_is_negated_false() {
        let t = ConclusionTemplate::positive("desc", "?claim");
        assert!(!t.is_negated);
    }

    #[test]
    fn conclusion_template_negated_has_is_negated_true() {
        let t = ConclusionTemplate::negated("desc", "?claim");
        assert!(t.is_negated);
    }
}
```

> **Note:** `SchemeSpec::instantiate` references `crate::instance::SchemeInstance`, which doesn't exist yet. The `instance` module is created in Task 4. The compile error in this task is expected — Step 4 tells you to skip running tests until Task 4 introduces the missing module. The commit at the end of this task will leave the crate in a non-compiling state, which is fine because the next task fixes it.

- [ ] **Step 3: Register both modules in `crates/argumentation-schemes/src/lib.rs`**

The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod critical;
pub mod error;
pub mod scheme;
pub mod types;

pub use error::Error;
```

- [ ] **Step 4: Skip running tests in this task**

The crate does not compile yet because `SchemeSpec::instantiate` references `crate::instance` which is added in Task 4. This is intentional — splitting Tasks 3 and 4 into a coherent unit would produce one task that's too large for a single subagent dispatch. Move directly to the commit.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): SchemeSpec, PremiseSlot, ConclusionTemplate, CriticalQuestion"
```

The commit message acknowledges the partial state. Task 4 will restore green builds.

---

### Task 4: SchemeInstance and instantiation (with prefix-safe template resolution)

**Files:**
- Create: `crates/argumentation-schemes/src/instance.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/instance.rs`**

The most important detail in this file is `resolve_template`: it must sort bindings by key length **descending** before substituting, so that longer slot names like `threatener` are processed before shorter ones like `threat`. Without this, `String::replace("?threat", X)` would corrupt `?threatener` into `Xener`.

```rust
//! `SchemeInstance`: a scheme instantiated with concrete bindings.

use crate::critical::CriticalQuestion;
use crate::scheme::SchemeSpec;
use crate::Error;
use argumentation::aspic::Literal;
use std::collections::HashMap;

/// A critical question instantiated with concrete bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalQuestionInstance {
    /// Question number (from the parent scheme).
    pub number: u32,
    /// Human-readable text with `?slot` references resolved.
    pub text: String,
    /// The challenge type (from the parent CriticalQuestion).
    pub challenge: crate::types::Challenge,
    /// The literal that, if asserted, would undermine the original argument.
    /// Always negated.
    pub counter_literal: Literal,
}

/// A scheme instantiated with concrete bindings, ready for ASPIC+ integration.
#[derive(Debug, Clone)]
pub struct SchemeInstance {
    /// The scheme this instance was created from.
    pub scheme_name: String,
    /// The resolved premise literals.
    pub premises: Vec<Literal>,
    /// The resolved conclusion literal.
    pub conclusion: Literal,
    /// Instantiated critical questions with resolved text and counter-literals.
    pub critical_questions: Vec<CriticalQuestionInstance>,
}

/// Resolve a template string by replacing `?slot` references with bound values.
///
/// Bindings are processed in descending key-length order so that longer
/// slot names are substituted before any shorter slot names that happen
/// to be a prefix. Without this, a template `?threatener` containing
/// the substring `?threat` would be corrupted by an earlier substitution
/// of slot `threat`.
fn resolve_template(template: &str, bindings: &HashMap<String, String>) -> String {
    let mut sorted: Vec<(&String, &String)> = bindings.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut result = template.to_string();
    for (key, val) in sorted {
        result = result.replace(&format!("?{}", key), val);
    }
    result
}

/// Instantiate a scheme with concrete bindings.
///
/// Every premise slot in the scheme must have a corresponding entry in
/// `bindings`. Returns [`Error::MissingBinding`] if any required slot is
/// unbound.
///
/// Also available as [`SchemeSpec::instantiate`] (which delegates here).
pub fn instantiate(
    scheme: &SchemeSpec,
    bindings: &HashMap<String, String>,
) -> Result<SchemeInstance, Error> {
    // Validate all slots are bound. Iterate in declared order so the error
    // message names the FIRST missing slot deterministically.
    for slot in &scheme.premises {
        if !bindings.contains_key(&slot.name) {
            return Err(Error::MissingBinding {
                scheme: scheme.name.clone(),
                slot: slot.name.clone(),
            });
        }
    }

    // Build premise literals: for each slot, create an atom encoding
    // "this slot is filled by this value in this scheme instance."
    // E.g., slot "expert" + binding "alice" → Literal::atom("expert_alice").
    let premises: Vec<Literal> = scheme
        .premises
        .iter()
        .map(|slot| {
            let val = &bindings[&slot.name];
            Literal::atom(format!("{}_{}", slot.name, val))
        })
        .collect();

    // Resolve conclusion template, respecting the is_negated flag.
    let conclusion_name = resolve_template(&scheme.conclusion.literal_template, bindings);
    let conclusion = if scheme.conclusion.is_negated {
        Literal::neg(&conclusion_name)
    } else {
        Literal::atom(&conclusion_name)
    };

    // Instantiate critical questions.
    let critical_questions = scheme
        .critical_questions
        .iter()
        .map(|cq| {
            let text = resolve_template(&cq.text, bindings);
            let counter_literal = build_counter_literal(cq, bindings, scheme, &conclusion_name);
            CriticalQuestionInstance {
                number: cq.number,
                text,
                challenge: cq.challenge.clone(),
                counter_literal,
            }
        })
        .collect();

    Ok(SchemeInstance {
        scheme_name: scheme.name.clone(),
        premises,
        conclusion,
        critical_questions,
    })
}

/// Build the counter-literal for a critical question.
///
/// The counter-literal is what would be asserted to undermine the scheme.
/// Different challenge types target different aspects:
/// - `PremiseTruth(slot)` negates the premise literal for that slot.
/// - `SourceCredibility` negates `credible_<agent>` for the relevant agent.
/// - Others negate a synthetic marker derived from the conclusion or scheme key.
fn build_counter_literal(
    cq: &CriticalQuestion,
    bindings: &HashMap<String, String>,
    scheme: &SchemeSpec,
    conclusion_name: &str,
) -> Literal {
    use crate::types::Challenge;
    match &cq.challenge {
        Challenge::PremiseTruth(slot_name) => {
            let val = bindings
                .get(slot_name)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            Literal::neg(format!("{}_{}", slot_name, val))
        }
        Challenge::SourceCredibility => {
            let agent = bindings
                .get("expert")
                .or_else(|| bindings.get("witness"))
                .or_else(|| bindings.get("source"))
                .or_else(|| bindings.get("person"))
                .or_else(|| bindings.get("target"))
                .map(|s| s.as_str())
                .unwrap_or("source");
            Literal::neg(format!("credible_{}", agent))
        }
        Challenge::RuleValidity => {
            Literal::neg(format!("valid_rule_{}", scheme.key()))
        }
        Challenge::ConflictingAuthority => {
            Literal::neg(format!("consensus_on_{}", conclusion_name))
        }
        Challenge::AlternativeCause => {
            Literal::neg(format!("sole_cause_{}", conclusion_name))
        }
        Challenge::UnseenConsequences => {
            Literal::neg(format!("all_consequences_considered_{}", conclusion_name))
        }
        Challenge::Proportionality => {
            let target = bindings
                .get("target")
                .or_else(|| bindings.get("threatener"))
                .map(|s| s.as_str())
                .unwrap_or("target");
            Literal::neg(format!("proportionate_attack_{}", target))
        }
        Challenge::DisanalogyClaim => {
            Literal::neg(format!("analogy_holds_{}", conclusion_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critical::CriticalQuestion;
    use crate::scheme::{ConclusionTemplate, PremiseSlot, SchemeMetadata, SchemeSpec};
    use crate::types::*;

    fn expert_opinion_scheme() -> SchemeSpec {
        SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![
                PremiseSlot::new("expert", "The claimed expert", SlotRole::Agent),
                PremiseSlot::new("domain", "Field of expertise", SlotRole::Domain),
                PremiseSlot::new("claim", "The asserted proposition", SlotRole::Proposition),
            ],
            conclusion: ConclusionTemplate::positive("?claim is plausibly true", "?claim"),
            critical_questions: vec![
                CriticalQuestion::new(
                    1,
                    "Is ?expert an expert in ?domain?",
                    Challenge::PremiseTruth("expert".into()),
                ),
                CriticalQuestion::new(
                    2,
                    "Is ?expert credible?",
                    Challenge::SourceCredibility,
                ),
            ],
            metadata: SchemeMetadata {
                citation: "Walton 2008 p.14".into(),
                domain_tags: vec!["epistemic".into()],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        }
    }

    fn full_bindings() -> HashMap<String, String> {
        [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn instantiate_produces_three_premises_and_positive_conclusion() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(instance.premises.len(), 3);
        assert_eq!(instance.conclusion, Literal::atom("fortify_east"));
    }

    #[test]
    fn instantiate_resolves_critical_question_text() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert!(instance.critical_questions[0].text.contains("alice"));
        assert!(instance.critical_questions[0].text.contains("military"));
    }

    #[test]
    fn instantiate_fails_on_missing_binding() {
        let scheme = expert_opinion_scheme();
        let mut bindings = full_bindings();
        bindings.remove("domain");
        let err = instantiate(&scheme, &bindings).unwrap_err();
        match err {
            Error::MissingBinding { slot, .. } => assert_eq!(slot, "domain"),
            other => panic!("expected MissingBinding, got {:?}", other),
        }
    }

    #[test]
    fn counter_literals_for_premise_truth_match_premise_encoding() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        // CQ1 challenges PremiseTruth("expert") → counter is ¬expert_alice,
        // which is the contrary of the premise literal expert_alice.
        let cq1 = &instance.critical_questions[0];
        assert_eq!(cq1.counter_literal, Literal::neg("expert_alice"));
        assert!(cq1.counter_literal.is_contrary_of(&instance.premises[0]));
    }

    #[test]
    fn counter_literal_for_source_credibility_uses_agent_binding() {
        let scheme = expert_opinion_scheme();
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        let cq2 = &instance.critical_questions[1];
        assert_eq!(cq2.counter_literal, Literal::neg("credible_alice"));
    }

    #[test]
    fn negated_conclusion_template_produces_negated_literal() {
        let mut scheme = expert_opinion_scheme();
        scheme.conclusion = ConclusionTemplate::negated("¬?claim", "?claim");
        let instance = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(instance.conclusion, Literal::neg("fortify_east"));
    }

    #[test]
    fn resolve_template_handles_prefix_overlapping_slot_names() {
        // Regression for the threat scheme: slots "threatener" and
        // "threat" share a prefix. Without length-descending sort,
        // ?threat would match inside ?threatener and corrupt it.
        let bindings: HashMap<String, String> = [
            ("threatener".to_string(), "darth_vader".to_string()),
            ("threat".to_string(), "destroy_planet".to_string()),
            ("demand".to_string(), "join_dark_side".to_string()),
        ]
        .into_iter()
        .collect();

        let template = "Does ?threatener carry out ?threat to force ?demand?";
        let resolved = resolve_template(template, &bindings);
        assert_eq!(
            resolved,
            "Does darth_vader carry out destroy_planet to force join_dark_side?"
        );
    }

    #[test]
    fn schemespec_instantiate_method_delegates_to_free_function() {
        let scheme = expert_opinion_scheme();
        let via_method = scheme.instantiate(&full_bindings()).unwrap();
        let via_free_fn = instantiate(&scheme, &full_bindings()).unwrap();
        assert_eq!(via_method.premises, via_free_fn.premises);
        assert_eq!(via_method.conclusion, via_free_fn.conclusion);
    }
}
```

- [ ] **Step 2: Register `instance` in `crates/argumentation-schemes/src/lib.rs`**

The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod critical;
pub mod error;
pub mod instance;
pub mod scheme;
pub mod types;

pub use error::Error;
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 14 tests pass total (2 from Task 2 + 1 from `critical` + 3 from `scheme` + 8 from `instance`). Watch specifically for `resolve_template_handles_prefix_overlapping_slot_names` to pass — that's the regression test for the prefix bug.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): SchemeInstance with prefix-safe template resolution"
```

---

### Task 5: CatalogRegistry — lookup by id, key, category

**Files:**
- Create: `crates/argumentation-schemes/src/registry.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/registry.rs`**

```rust
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
```

- [ ] **Step 2: Register `registry` in `crates/argumentation-schemes/src/lib.rs`**

The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod critical;
pub mod error;
pub mod instance;
pub mod registry;
pub mod scheme;
pub mod types;

pub use error::Error;
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 18 tests pass (14 from prior tasks + 4 new registry tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): CatalogRegistry with lookup by id, key, category"
```

---

### Task 6: ASPIC+ integration — add_scheme_to_system, add_counter_argument

**Files:**
- Create: `crates/argumentation-schemes/src/aspic.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/aspic.rs`**

```rust
//! ASPIC+ integration: feed a [`SchemeInstance`] into an
//! [`argumentation::aspic::StructuredSystem`] as ordinary premises plus
//! a defeasible rule.

use crate::instance::SchemeInstance;
use argumentation::aspic::{Literal, RuleId, StructuredSystem};

/// Feed a scheme instance into a `StructuredSystem` as ordinary premises
/// and a defeasible rule (premises → conclusion).
///
/// Returns the [`RuleId`] of the defeasible rule that was added, which
/// can be used for preference ordering via
/// [`StructuredSystem::prefer_rule`].
///
/// The instance's `premises` are added as ordinary (defeasible) premises
/// via [`StructuredSystem::add_ordinary`]. The instance's `conclusion`
/// (already polarised by the scheme's [`crate::scheme::ConclusionTemplate`])
/// becomes the rule's conclusion.
pub fn add_scheme_to_system(
    instance: &SchemeInstance,
    system: &mut StructuredSystem,
) -> RuleId {
    for premise in &instance.premises {
        system.add_ordinary(premise.clone());
    }
    system.add_defeasible_rule(instance.premises.clone(), instance.conclusion.clone())
}

/// Feed a critical question's counter-argument into a `StructuredSystem`
/// as an ordinary premise asserting the counter-literal, plus a defeasible
/// rule concluding the contrary of the original scheme's conclusion (rebut).
///
/// Returns the [`RuleId`] of the counter-rule.
pub fn add_counter_argument(
    counter_literal: &Literal,
    target_conclusion: &Literal,
    system: &mut StructuredSystem,
) -> RuleId {
    system.add_ordinary(counter_literal.clone());
    let neg_conclusion = target_conclusion.contrary();
    system.add_defeasible_rule(vec![counter_literal.clone()], neg_conclusion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critical::CriticalQuestion;
    use crate::instance::instantiate;
    use crate::scheme::*;
    use crate::types::*;
    use std::collections::HashMap;

    fn expert_scheme() -> SchemeSpec {
        SchemeSpec {
            id: SchemeId(1),
            name: "Argument from Expert Opinion".into(),
            category: SchemeCategory::Epistemic,
            premises: vec![
                PremiseSlot::new("expert", "The expert", SlotRole::Agent),
                PremiseSlot::new("domain", "Field", SlotRole::Domain),
                PremiseSlot::new("claim", "The claim", SlotRole::Proposition),
            ],
            conclusion: ConclusionTemplate::positive("?claim is true", "?claim"),
            critical_questions: vec![CriticalQuestion::new(
                1,
                "Is ?expert an expert?",
                Challenge::SourceCredibility,
            )],
            metadata: SchemeMetadata {
                citation: "Walton 2008".into(),
                domain_tags: vec![],
                presumptive: true,
                strength: SchemeStrength::Moderate,
            },
        }
    }

    fn alice_bindings() -> HashMap<String, String> {
        [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn add_scheme_creates_argument_with_expected_conclusion() {
        let scheme = expert_scheme();
        let instance = instantiate(&scheme, &alice_bindings()).unwrap();

        let mut system = StructuredSystem::new();
        let _rule_id = add_scheme_to_system(&instance, &mut system);

        assert!(!system.rules().is_empty());
        let built = system.build_framework().unwrap();
        let conclusion_args =
            built.arguments_with_conclusion(&Literal::atom("fortify_east"));
        assert!(
            !conclusion_args.is_empty(),
            "expected at least one argument concluding fortify_east"
        );
    }

    #[test]
    fn counter_argument_filters_original_when_preferred() {
        let scheme = expert_scheme();
        let instance = instantiate(&scheme, &alice_bindings()).unwrap();

        let mut system = StructuredSystem::new();
        let main_rule = add_scheme_to_system(&instance, &mut system);

        // Bob presses CQ1 by asserting the counter-literal.
        let cq = &instance.critical_questions[0];
        let counter_rule =
            add_counter_argument(&cq.counter_literal, &instance.conclusion, &mut system);
        // Prefer Bob's counter so it strictly defeats Alice's argument.
        system.prefer_rule(counter_rule, main_rule).unwrap();

        let built = system.build_framework().unwrap();
        let preferred = built.framework.preferred_extensions().unwrap();
        assert_eq!(preferred.len(), 1);

        let has_fortify = built
            .arguments_with_conclusion(&Literal::atom("fortify_east"))
            .iter()
            .any(|a| preferred[0].contains(&a.id));
        assert!(
            !has_fortify,
            "original claim should be defeated when counter-rule is preferred"
        );
    }
}
```

- [ ] **Step 2: Register `aspic` in `crates/argumentation-schemes/src/lib.rs`**

The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod aspic;
pub mod critical;
pub mod error;
pub mod instance;
pub mod registry;
pub mod scheme;
pub mod types;

pub use error::Error;
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 20 tests pass (18 from prior tasks + 2 new aspic tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): ASPIC+ integration — add_scheme_to_system, add_counter_argument"
```

---

## Phase 2 — Scheme Catalog (25 schemes across 6 categories)

### Task 7: Catalog scaffolding + epistemic schemes (3)

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/mod.rs`
- Create: `crates/argumentation-schemes/src/catalog/epistemic.rs`
- Modify: `crates/argumentation-schemes/src/lib.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/mod.rs`**

```rust
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

pub mod epistemic;

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
pub const ANALOGICAL_ID_OFFSET: u32 = 500;

/// Build the default catalog containing all built-in schemes.
pub fn default_catalog() -> CatalogRegistry {
    let mut reg = CatalogRegistry::new();
    for scheme in epistemic::all() {
        reg.register(scheme);
    }
    reg
}
```

- [ ] **Step 2: Create `crates/argumentation-schemes/src/catalog/epistemic.rs`**

```rust
//! Epistemic schemes: reasoning about knowledge and expertise.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 4 + Appendix 1.

use crate::catalog::EPISTEMIC_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all epistemic schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_expert_opinion(),
        argument_from_witness_testimony(),
        argument_from_position_to_know(),
    ]
}

/// Argument from Expert Opinion (Walton 2008 p.14, Scheme 1).
///
/// E is an expert in domain D. E asserts that proposition A is true.
/// Therefore, A may plausibly be taken to be true.
pub fn argument_from_expert_opinion() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET),
        name: "Argument from Expert Opinion".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("expert", "Source E is an expert in subject domain D", SlotRole::Agent),
            PremiseSlot::new("domain", "The field of expertise containing the claim", SlotRole::Domain),
            PremiseSlot::new("claim", "The proposition E asserts", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim may plausibly be taken to be true",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "How credible is ?expert as an expert source?", Challenge::SourceCredibility),
            CriticalQuestion::new(2, "Is ?expert an expert in the field that ?claim is in?", Challenge::PremiseTruth("domain".into())),
            CriticalQuestion::new(3, "What did ?expert assert that implies ?claim?", Challenge::PremiseTruth("claim".into())),
            CriticalQuestion::new(4, "Is ?expert personally reliable as a source?", Challenge::SourceCredibility),
            CriticalQuestion::new(5, "Is ?claim consistent with what other experts assert?", Challenge::ConflictingAuthority),
            CriticalQuestion::new(6, "Is ?expert's assertion based on evidence?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.14 (Scheme 1)".into(),
            domain_tags: vec!["epistemic".into(), "authority".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Witness Testimony (Walton 2008 p.309, Scheme 2).
pub fn argument_from_witness_testimony() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET + 1),
        name: "Argument from Witness Testimony".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("witness", "The person who claims to have observed the event", SlotRole::Agent),
            PremiseSlot::new("event", "The event that was allegedly observed", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?event occurred as ?witness described",
            "?event",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?witness telling the truth (not lying)?", Challenge::SourceCredibility),
            CriticalQuestion::new(2, "Was ?witness in a position to observe ?event?", Challenge::PremiseTruth("witness".into())),
            CriticalQuestion::new(3, "Is ?witness's account of ?event consistent with other evidence?", Challenge::ConflictingAuthority),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.309 (Scheme 2)".into(),
            domain_tags: vec!["epistemic".into(), "testimony".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Position to Know (Walton 2008 p.310, Scheme 3).
pub fn argument_from_position_to_know() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(EPISTEMIC_ID_OFFSET + 2),
        name: "Argument from Position to Know".into(),
        category: SchemeCategory::Epistemic,
        premises: vec![
            PremiseSlot::new("source", "The person in a position to know", SlotRole::Agent),
            PremiseSlot::new("domain", "The domain of knowledge", SlotRole::Domain),
            PremiseSlot::new("claim", "The asserted proposition within that domain", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is plausibly true",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?source in a position to know whether ?claim is true?", Challenge::PremiseTruth("source".into())),
            CriticalQuestion::new(2, "Is ?source a truthful and reliable reporter?", Challenge::SourceCredibility),
            CriticalQuestion::new(3, "Did ?source actually assert ?claim?", Challenge::PremiseTruth("claim".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.310 (Scheme 3)".into(),
            domain_tags: vec!["epistemic".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expert_opinion_has_six_critical_questions() {
        let scheme = argument_from_expert_opinion();
        assert_eq!(scheme.critical_questions.len(), 6);
        assert_eq!(scheme.premises.len(), 3);
    }

    #[test]
    fn all_returns_three_epistemic_schemes() {
        let schemes = all();
        assert_eq!(schemes.len(), 3);
        assert!(schemes.iter().all(|s| s.category == SchemeCategory::Epistemic));
    }

    #[test]
    fn epistemic_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= EPISTEMIC_ID_OFFSET);
            assert!(s.id.0 < EPISTEMIC_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 3: Register `catalog` in `crates/argumentation-schemes/src/lib.rs`**

The full lib.rs after this edit:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable pattern of argument
//! (e.g., "argument from expert opinion," "ad hominem," "argument from
//! consequences") that carries its own critical questions — the follow-up
//! challenges a competent reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod aspic;
pub mod catalog;
pub mod critical;
pub mod error;
pub mod instance;
pub mod registry;
pub mod scheme;
pub mod types;

pub use error::Error;
```

- [ ] **Step 4: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 23 tests pass (20 from prior tasks + 3 new epistemic tests).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): epistemic catalog — expert opinion, witness, position to know"
```

---

### Task 8: Practical schemes (7) — consequences, values, threat, fear, waste

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/practical.rs`
- Modify: `crates/argumentation-schemes/src/catalog/mod.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/practical.rs`**

```rust
//! Practical schemes: reasoning about actions, consequences, and values.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::PRACTICAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all practical schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_positive_consequences(),
        argument_from_negative_consequences(),
        argument_from_values(),
        argument_from_threat(),
        argument_from_fear_appeal(),
        argument_from_waste(),
        argument_from_sunk_cost(),
    ]
}

/// Argument from Positive Consequences (Walton 2008 p.332).
///
/// If action A is brought about, good consequence G will occur.
/// Therefore A should be brought about.
pub fn argument_from_positive_consequences() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET),
        name: "Argument from Positive Consequences".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being proposed", SlotRole::Action),
            PremiseSlot::new("good_consequence", "The beneficial outcome", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be brought about because ?good_consequence will result",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Will ?action actually lead to ?good_consequence?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?good_consequence actually good on balance?", Challenge::PremiseTruth("good_consequence".into())),
            CriticalQuestion::new(3, "Are there negative consequences that offset ?good_consequence?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.332".into(),
            domain_tags: vec!["practical".into(), "consequences".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Negative Consequences (Walton 2008 p.333).
///
/// Negated-conclusion scheme: concludes ¬do_?action.
pub fn argument_from_negative_consequences() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 1),
        name: "Argument from Negative Consequences".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being warned against", SlotRole::Action),
            PremiseSlot::new("bad_consequence", "The harmful outcome", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::negated(
            "?action should not be brought about because ?bad_consequence will result",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Will ?action actually lead to ?bad_consequence?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?bad_consequence actually bad on balance?", Challenge::PremiseTruth("bad_consequence".into())),
            CriticalQuestion::new(3, "Are there positive consequences that offset ?bad_consequence?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.333".into(),
            domain_tags: vec!["practical".into(), "consequences".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Values (Walton 2008 p.321).
pub fn argument_from_values() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 2),
        name: "Argument from Values".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action being evaluated", SlotRole::Action),
            PremiseSlot::new("value", "The value being promoted", SlotRole::Property),
            PremiseSlot::new("agent", "The agent whose values are at stake", SlotRole::Agent),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be carried out because it promotes ?value for ?agent",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?action really promote ?value?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is ?value relevant to the current context?", Challenge::PremiseTruth("value".into())),
            CriticalQuestion::new(3, "Are there competing values that take precedence over ?value?", Challenge::UnseenConsequences),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.321".into(),
            domain_tags: vec!["practical".into(), "values".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Threat (Walton 2008 p.335).
///
/// Note: slot names `threatener` and `threat` overlap as prefixes —
/// the `resolve_template` function in `instance.rs` sorts bindings by
/// length descending to handle this correctly.
pub fn argument_from_threat() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 3),
        name: "Argument from Threat".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("threatener", "The person making the threat", SlotRole::Agent),
            PremiseSlot::new("threat", "The bad thing that will happen", SlotRole::Consequence),
            PremiseSlot::new("demand", "The action being demanded", SlotRole::Action),
        ],
        conclusion: ConclusionTemplate::positive(
            "?demand should be complied with to avoid ?threat",
            "comply_?demand",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?threatener have the ability to carry out ?threat?", Challenge::PremiseTruth("threatener".into())),
            CriticalQuestion::new(2, "Is ?threat proportionate to ?demand?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Is there an alternative to complying with ?demand?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.335".into(),
            domain_tags: vec!["practical".into(), "coercion".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Fear Appeal (Walton 2008 p.336).
pub fn argument_from_fear_appeal() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 4),
        name: "Argument from Fear Appeal".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The recommended action", SlotRole::Action),
            PremiseSlot::new("fearful_outcome", "The feared consequence of inaction", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be taken to avoid ?fearful_outcome",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?fearful_outcome truly likely if ?action is not taken?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Is the fear of ?fearful_outcome proportionate to the actual risk?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Is ?action the only way to avoid ?fearful_outcome?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.336".into(),
            domain_tags: vec!["practical".into(), "emotion".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Waste (Walton 2008 p.339).
pub fn argument_from_waste() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 5),
        name: "Argument from Waste".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The action already started", SlotRole::Action),
            PremiseSlot::new("investment", "What has been invested so far", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be continued to avoid wasting ?investment",
            "continue_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "How much has actually been invested in ?action?", Challenge::PremiseTruth("investment".into())),
            CriticalQuestion::new(2, "Would continuing ?action actually recoup ?investment?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.339".into(),
            domain_tags: vec!["practical".into(), "sunk_cost".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Sunk Cost (Walton 2008 p.340) — distinct from Waste:
/// emphasises the prior commitment as a reason for continuation.
pub fn argument_from_sunk_cost() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(PRACTICAL_ID_OFFSET + 6),
        name: "Argument from Sunk Cost".into(),
        category: SchemeCategory::Practical,
        premises: vec![
            PremiseSlot::new("action", "The committed action", SlotRole::Action),
            PremiseSlot::new("commitment", "The prior commitment that locks the agent in", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be continued to honour ?commitment",
            "continue_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?commitment still binding given current circumstances?", Challenge::PremiseTruth("commitment".into())),
            CriticalQuestion::new(2, "Does honouring ?commitment actually require continuing ?action?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.340".into(),
            domain_tags: vec!["practical".into(), "sunk_cost".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_seven_practical_schemes() {
        let schemes = all();
        assert_eq!(schemes.len(), 7);
        assert!(schemes.iter().all(|s| s.category == SchemeCategory::Practical));
    }

    #[test]
    fn negative_consequences_has_negated_conclusion() {
        assert!(argument_from_negative_consequences().conclusion.is_negated);
    }

    #[test]
    fn positive_consequences_has_non_negated_conclusion() {
        assert!(!argument_from_positive_consequences().conclusion.is_negated);
    }

    #[test]
    fn threat_scheme_has_three_premises() {
        assert_eq!(argument_from_threat().premises.len(), 3);
    }

    #[test]
    fn practical_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= PRACTICAL_ID_OFFSET);
            assert!(s.id.0 < PRACTICAL_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 2: Update `crates/argumentation-schemes/src/catalog/mod.rs`**

Replace the entire file with:

```rust
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

pub mod epistemic;
pub mod practical;

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
    reg
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 28 tests pass (23 from prior tasks + 5 new practical tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): practical catalog — consequences, values, threat, fear, waste, sunk cost"
```

---

### Task 9: Source-based schemes (4) — ad hominem (2), bias, ethotic

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/source.rs`
- Modify: `crates/argumentation-schemes/src/catalog/mod.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/source.rs`**

```rust
//! Source-based schemes: attacking or bolstering the person, not the argument.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 5 + Appendix 1.

use crate::catalog::SOURCE_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

/// Return all source-based schemes.
pub fn all() -> Vec<SchemeSpec> {
    vec![
        ad_hominem(),
        ad_hominem_circumstantial(),
        argument_from_bias(),
        ethotic_argument(),
    ]
}

/// Ad Hominem — generic (Walton 2008 p.141).
///
/// Negated-conclusion scheme: target has flaw F, therefore ¬claim.
pub fn ad_hominem() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET),
        name: "Ad Hominem".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("target", "The person being attacked", SlotRole::Agent),
            PremiseSlot::new("flaw", "The character flaw alleged", SlotRole::Property),
            PremiseSlot::new("claim", "The claim being challenged via the attack", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim should be rejected because ?target has ?flaw",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?target really have ?flaw?", Challenge::PremiseTruth("flaw".into())),
            CriticalQuestion::new(2, "Does ?flaw actually bear on the credibility of ?claim?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Is the attack on ?target proportionate to ?flaw?", Challenge::Proportionality),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.141".into(),
            domain_tags: vec!["source".into(), "attack".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Ad Hominem — circumstantial (Walton 2008 p.143).
///
/// Negated-conclusion scheme.
pub fn ad_hominem_circumstantial() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 1),
        name: "Ad Hominem Circumstantial".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("target", "The person whose circumstances are cited", SlotRole::Agent),
            PremiseSlot::new("inconsistency", "How target's circumstances conflict with the claim", SlotRole::Property),
            PremiseSlot::new("claim", "The claim being undermined", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim is undermined because ?target's circumstances (?inconsistency) are inconsistent with it",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?target actually have the alleged ?inconsistency?", Challenge::PremiseTruth("inconsistency".into())),
            CriticalQuestion::new(2, "Is the inconsistency relevant to ?claim?", Challenge::Proportionality),
            CriticalQuestion::new(3, "Could ?target's ?claim still be valid despite the inconsistency?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.143".into(),
            domain_tags: vec!["source".into(), "attack".into(), "hypocrisy".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Bias (Walton 2008 p.340).
///
/// Negated-conclusion scheme.
pub fn argument_from_bias() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 2),
        name: "Argument from Bias".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("source", "The biased source", SlotRole::Agent),
            PremiseSlot::new("bias", "The alleged bias", SlotRole::Property),
            PremiseSlot::new("claim", "The claim made by the biased source", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::negated(
            "?claim should be treated with suspicion because ?source has ?bias",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?source actually have the alleged ?bias?", Challenge::PremiseTruth("bias".into())),
            CriticalQuestion::new(2, "Does ?bias actually affect ?source's assertion of ?claim?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Even if ?source is biased, might ?claim still be true?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.340".into(),
            domain_tags: vec!["source".into(), "attack".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Ethotic Argument — positive (Walton 2008 p.146).
pub fn ethotic_argument() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(SOURCE_ID_OFFSET + 3),
        name: "Ethotic Argument".into(),
        category: SchemeCategory::SourceBased,
        premises: vec![
            PremiseSlot::new("person", "The person whose character is cited", SlotRole::Agent),
            PremiseSlot::new("good_character", "The positive character trait", SlotRole::Property),
            PremiseSlot::new("claim", "The claim bolstered by good character", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is more plausible because ?person has ?good_character",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?person actually have ?good_character?", Challenge::PremiseTruth("good_character".into())),
            CriticalQuestion::new(2, "Does ?good_character make ?claim more plausible?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.146".into(),
            domain_tags: vec!["source".into(), "support".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_source_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn ad_hominem_has_negated_conclusion() {
        assert!(ad_hominem().conclusion.is_negated, "ad hominem must conclude ¬claim");
    }

    #[test]
    fn ad_hominem_circumstantial_has_negated_conclusion() {
        assert!(ad_hominem_circumstantial().conclusion.is_negated);
    }

    #[test]
    fn bias_has_negated_conclusion() {
        assert!(argument_from_bias().conclusion.is_negated);
    }

    #[test]
    fn ethotic_has_positive_conclusion() {
        assert!(!ethotic_argument().conclusion.is_negated);
    }

    #[test]
    fn source_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= SOURCE_ID_OFFSET);
            assert!(s.id.0 < SOURCE_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 2: Register `source` in `crates/argumentation-schemes/src/catalog/mod.rs`**

Add `pub mod source;` after `pub mod practical;` and add the registration loop in `default_catalog`. The full updated `mod.rs`:

```rust
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

pub mod epistemic;
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
    reg
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 34 tests pass (28 from prior tasks + 6 new source tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): source-based catalog — ad hominem, bias, ethotic"
```

---

### Task 10: Popular schemes (4) — popular opinion, tradition, precedent, established rule

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/popular.rs`
- Modify: `crates/argumentation-schemes/src/catalog/mod.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/popular.rs`**

```rust
//! Popular schemes: social proof, tradition, precedent.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::POPULAR_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_popular_opinion(),
        argument_from_tradition(),
        argument_from_precedent(),
        argument_from_established_rule(),
    ]
}

/// Argument from Popular Opinion (Walton 2008 p.311).
pub fn argument_from_popular_opinion() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET),
        name: "Argument from Popular Opinion".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("claim", "The claim widely accepted", SlotRole::Proposition),
            PremiseSlot::new("population", "The group that accepts the claim", SlotRole::Agent),
        ],
        conclusion: ConclusionTemplate::positive(
            "?claim is plausible based on popular acceptance by ?population",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "What evidence supports that ?population actually accepts ?claim?", Challenge::PremiseTruth("population".into())),
            CriticalQuestion::new(2, "Is ?population's acceptance based on good reasoning?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Is ?claim the type of claim that popular acceptance makes more plausible?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.311".into(),
            domain_tags: vec!["popular".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Tradition (Walton 2008 p.316).
pub fn argument_from_tradition() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 1),
        name: "Argument from Tradition".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("practice", "The traditional practice", SlotRole::Action),
            PremiseSlot::new("tradition", "Evidence of longstanding tradition", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?practice should be continued based on ?tradition",
            "continue_?practice",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Has ?practice actually been a longstanding tradition?", Challenge::PremiseTruth("tradition".into())),
            CriticalQuestion::new(2, "Were the circumstances that justified ?practice still applicable?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Have conditions changed such that ?practice is no longer appropriate?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.316".into(),
            domain_tags: vec!["popular".into(), "tradition".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Precedent (Walton 2008 p.319).
pub fn argument_from_precedent() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 2),
        name: "Argument from Precedent".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("precedent_case", "The precedent case", SlotRole::Property),
            PremiseSlot::new("current_case", "The current situation", SlotRole::Property),
            PremiseSlot::new("action", "The action taken in the precedent", SlotRole::Action),
        ],
        conclusion: ConclusionTemplate::positive(
            "?action should be taken in ?current_case as it was in ?precedent_case",
            "do_?action",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?current_case sufficiently similar to ?precedent_case?", Challenge::DisanalogyClaim),
            CriticalQuestion::new(2, "Was ?action the right decision in ?precedent_case?", Challenge::PremiseTruth("precedent_case".into())),
            CriticalQuestion::new(3, "Are there relevant differences between ?precedent_case and ?current_case?", Challenge::DisanalogyClaim),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.319".into(),
            domain_tags: vec!["popular".into(), "legal".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Established Rule (Walton 2008 p.318).
pub fn argument_from_established_rule() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(POPULAR_ID_OFFSET + 3),
        name: "Argument from Established Rule".into(),
        category: SchemeCategory::Popular,
        premises: vec![
            PremiseSlot::new("rule", "The established rule or law", SlotRole::Property),
            PremiseSlot::new("case", "The case the rule applies to", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "The outcome prescribed by ?rule applies to ?case",
            "rule_applies_?case",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?rule actually apply to ?case?", Challenge::PremiseTruth("case".into())),
            CriticalQuestion::new(2, "Is ?rule still valid and in force?", Challenge::PremiseTruth("rule".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.318".into(),
            domain_tags: vec!["popular".into(), "legal".into(), "normative".into()],
            presumptive: true,
            strength: SchemeStrength::Strong,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_popular_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn established_rule_has_strong_strength() {
        assert_eq!(argument_from_established_rule().metadata.strength, SchemeStrength::Strong);
    }

    #[test]
    fn popular_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= POPULAR_ID_OFFSET);
            assert!(s.id.0 < POPULAR_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 2: Register `popular` in `crates/argumentation-schemes/src/catalog/mod.rs`**

Add `pub mod popular;` and the registration loop. Full updated `mod.rs`:

```rust
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
    reg
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 37 tests pass (34 from prior tasks + 3 new popular tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): popular catalog — popular opinion, tradition, precedent, rule"
```

---

### Task 11: Causal schemes (4) — cause→effect, correlation, sign, slippery slope

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/causal.rs`
- Modify: `crates/argumentation-schemes/src/catalog/mod.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/causal.rs`**

```rust
//! Causal schemes: reasoning about causes, effects, signs, and chains.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::CAUSAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_cause_to_effect(),
        argument_from_correlation_to_cause(),
        argument_from_sign(),
        argument_from_gradual_slippery_slope(),
    ]
}

/// Argument from Cause to Effect (Walton 2008 p.327).
pub fn argument_from_cause_to_effect() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET),
        name: "Argument from Cause to Effect".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("cause", "The causal event or condition", SlotRole::Action),
            PremiseSlot::new("effect", "The effect that follows", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?effect will occur because ?cause has occurred",
            "?effect",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a strong causal link between ?cause and ?effect?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Has ?cause actually occurred or will it occur?", Challenge::PremiseTruth("cause".into())),
            CriticalQuestion::new(3, "Could something else prevent ?effect despite ?cause?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.327".into(),
            domain_tags: vec!["causal".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Correlation to Cause (Walton 2008 p.328).
pub fn argument_from_correlation_to_cause() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 1),
        name: "Argument from Correlation to Cause".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("antecedent", "The first correlated event", SlotRole::Action),
            PremiseSlot::new("consequent", "The second correlated event", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::positive(
            "?antecedent causes ?consequent",
            "causes_?antecedent_to_?consequent",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a genuine correlation between ?antecedent and ?consequent?", Challenge::PremiseTruth("antecedent".into())),
            CriticalQuestion::new(2, "Could both ?antecedent and ?consequent be caused by a third factor?", Challenge::AlternativeCause),
            CriticalQuestion::new(3, "Could the causal direction be reversed (?consequent causes ?antecedent)?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.328".into(),
            domain_tags: vec!["causal".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Sign (Walton 2008 p.329).
pub fn argument_from_sign() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 2),
        name: "Argument from Sign".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("sign", "The observed sign or indicator", SlotRole::Property),
            PremiseSlot::new("indicated", "What the sign indicates", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?indicated is plausible based on ?sign",
            "?indicated",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is ?sign a reliable indicator of ?indicated?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Could ?sign indicate something other than ?indicated?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.329".into(),
            domain_tags: vec!["causal".into(), "abductive".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

/// Argument from Gradual Slippery Slope (Walton 2008 p.338).
///
/// Negated-conclusion scheme.
pub fn argument_from_gradual_slippery_slope() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(CAUSAL_ID_OFFSET + 3),
        name: "Argument from Gradual Slippery Slope".into(),
        category: SchemeCategory::Causal,
        premises: vec![
            PremiseSlot::new("first_step", "The initial innocuous action", SlotRole::Action),
            PremiseSlot::new("final_outcome", "The undesirable end state", SlotRole::Consequence),
        ],
        conclusion: ConclusionTemplate::negated(
            "?first_step should not be taken because it leads to ?final_outcome",
            "do_?first_step",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Is there a plausible chain from ?first_step to ?final_outcome?", Challenge::RuleValidity),
            CriticalQuestion::new(2, "Can the chain be stopped at some intermediate point?", Challenge::AlternativeCause),
            CriticalQuestion::new(3, "Is ?final_outcome really as bad as claimed?", Challenge::PremiseTruth("final_outcome".into())),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.338".into(),
            domain_tags: vec!["causal".into(), "slippery_slope".into()],
            presumptive: true,
            strength: SchemeStrength::Weak,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_four_causal_schemes() {
        assert_eq!(all().len(), 4);
    }

    #[test]
    fn slippery_slope_has_negated_conclusion() {
        assert!(argument_from_gradual_slippery_slope().conclusion.is_negated);
    }

    #[test]
    fn causal_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= CAUSAL_ID_OFFSET);
            assert!(s.id.0 < CAUSAL_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 2: Register `causal` in `crates/argumentation-schemes/src/catalog/mod.rs`**

Add `pub mod causal;` and the registration loop. Full updated `mod.rs`:

```rust
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
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 40 tests pass (37 from prior tasks + 3 new causal tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): causal catalog — cause-to-effect, correlation, sign, slippery slope"
```

---

### Task 12: Analogical schemes (3) + close out the catalog

**Files:**
- Create: `crates/argumentation-schemes/src/catalog/analogy.rs`
- Modify: `crates/argumentation-schemes/src/catalog/mod.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/src/catalog/analogy.rs`**

```rust
//! Analogical schemes: analogy, classification, commitment.
//!
//! Ref: Walton, Reed & Macagno 2008, Chapter 9 + Appendix 1.

use crate::catalog::ANALOGICAL_ID_OFFSET;
use crate::critical::CriticalQuestion;
use crate::scheme::*;
use crate::types::*;

pub fn all() -> Vec<SchemeSpec> {
    vec![
        argument_from_analogy(),
        argument_from_verbal_classification(),
        argument_from_commitment(),
    ]
}

/// Argument from Analogy (Walton 2008 p.315).
pub fn argument_from_analogy() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET),
        name: "Argument from Analogy".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new("similar_case", "The analogous case", SlotRole::Property),
            PremiseSlot::new("current_case", "The case being reasoned about", SlotRole::Property),
            PremiseSlot::new("property", "The property that holds in the analogous case", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?property also holds in ?current_case because it holds in ?similar_case",
            "?property",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Are ?similar_case and ?current_case truly similar in relevant respects?", Challenge::DisanalogyClaim),
            CriticalQuestion::new(2, "Is ?property the kind of thing that transfers between analogous cases?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Are there relevant differences between ?similar_case and ?current_case that block the analogy?", Challenge::DisanalogyClaim),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.315".into(),
            domain_tags: vec!["analogical".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Verbal Classification (Walton 2008 p.320).
pub fn argument_from_verbal_classification() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET + 1),
        name: "Argument from Verbal Classification".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new("subject", "The entity being classified", SlotRole::Agent),
            PremiseSlot::new("classification", "The classification being applied", SlotRole::Property),
        ],
        conclusion: ConclusionTemplate::positive(
            "?subject has the properties associated with ?classification",
            "is_a_?classification_?subject",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Does ?subject actually fit the definition of ?classification?", Challenge::PremiseTruth("classification".into())),
            CriticalQuestion::new(2, "Is ?classification the right category for this context?", Challenge::RuleValidity),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.320".into(),
            domain_tags: vec!["analogical".into(), "definition".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

/// Argument from Commitment (Walton 2008 p.322).
pub fn argument_from_commitment() -> SchemeSpec {
    SchemeSpec {
        id: SchemeId(ANALOGICAL_ID_OFFSET + 2),
        name: "Argument from Commitment".into(),
        category: SchemeCategory::Analogical,
        premises: vec![
            PremiseSlot::new("agent", "The person who made the commitment", SlotRole::Agent),
            PremiseSlot::new("commitment", "The commitment that was made", SlotRole::Action),
            PremiseSlot::new("claim", "The claim that follows from the commitment", SlotRole::Proposition),
        ],
        conclusion: ConclusionTemplate::positive(
            "?agent should act consistently with ?commitment, therefore ?claim",
            "?claim",
        ),
        critical_questions: vec![
            CriticalQuestion::new(1, "Did ?agent actually make ?commitment?", Challenge::PremiseTruth("commitment".into())),
            CriticalQuestion::new(2, "Does ?claim actually follow from ?commitment?", Challenge::RuleValidity),
            CriticalQuestion::new(3, "Have circumstances changed such that ?commitment no longer applies?", Challenge::AlternativeCause),
        ],
        metadata: SchemeMetadata {
            citation: "Walton 2008 p.322".into(),
            domain_tags: vec!["analogical".into(), "social_contract".into()],
            presumptive: true,
            strength: SchemeStrength::Moderate,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_three_analogical_schemes() {
        assert_eq!(all().len(), 3);
    }

    #[test]
    fn analogical_ids_are_in_offset_range() {
        for s in all() {
            assert!(s.id.0 >= ANALOGICAL_ID_OFFSET);
            assert!(s.id.0 < ANALOGICAL_ID_OFFSET + 100);
        }
    }
}
```

- [ ] **Step 2: Register `analogy` in `crates/argumentation-schemes/src/catalog/mod.rs`**

The full updated `mod.rs`:

```rust
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

pub mod analogy;
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
    for scheme in analogy::all() {
        reg.register(scheme);
    }
    reg
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes`
Expected: 42 tests pass (40 from prior tasks + 2 new analogical tests). Total scheme count in `default_catalog()` is now 25 (3+7+4+4+4+3).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(argumentation-schemes): analogical catalog — analogy, classification, commitment"
```

---

## Phase 3 — Integration Tests and Release

### Task 13: UC1 — instantiation integration test

**Files:**
- Create: `crates/argumentation-schemes/tests/instantiation.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/tests/instantiation.rs`**

```rust
//! UC1: end-to-end instantiation pipeline. Uses the public API exclusively
//! (no internal modules) to validate that consumers can perform the full
//! scheme → instance → ASPIC+ flow without touching crate internals.

use argumentation::aspic::{Literal, StructuredSystem};
use argumentation_schemes::aspic::add_scheme_to_system;
use argumentation_schemes::catalog::default_catalog;
use std::collections::HashMap;

#[test]
fn uc1_expert_opinion_full_pipeline() {
    let catalog = default_catalog();
    let scheme = catalog
        .by_key("argument_from_expert_opinion")
        .expect("expert opinion scheme should be in default catalog");

    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();

    let instance = scheme.instantiate(&bindings).expect("instantiation should succeed");
    assert_eq!(instance.premises.len(), 3, "expert opinion has 3 premise slots");
    assert_eq!(instance.conclusion, Literal::atom("fortify_east"));
    assert_eq!(instance.critical_questions.len(), 6, "expert opinion has 6 CQs");

    // Feed into ASPIC+ and verify the AF contains an argument concluding fortify_east.
    let mut system = StructuredSystem::new();
    add_scheme_to_system(&instance, &mut system);
    let built = system.build_framework().expect("framework build should succeed");
    let conclusion_args = built.arguments_with_conclusion(&Literal::atom("fortify_east"));
    assert!(
        !conclusion_args.is_empty(),
        "AF should contain at least one argument concluding fortify_east"
    );

    let preferred = built
        .framework
        .preferred_extensions()
        .expect("preferred enumeration should succeed");
    assert!(
        !preferred.is_empty(),
        "an unattacked argument should appear in some preferred extension"
    );
}

#[test]
fn uc1_missing_binding_returns_error() {
    let catalog = default_catalog();
    let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
    let bindings: HashMap<String, String> = [("expert".to_string(), "alice".to_string())]
        .into_iter()
        .collect();
    let err = scheme.instantiate(&bindings).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("domain") || msg.contains("claim"));
}
```

- [ ] **Step 2: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes --test instantiation`
Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-schemes): UC1 end-to-end instantiation integration"
```

---

### Task 14: UC2 — scheme conflict via negated-conclusion rebut

**Files:**
- Create: `crates/argumentation-schemes/tests/scheme_conflict.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/tests/scheme_conflict.rs`**

```rust
//! UC2: Alice uses argument from expert opinion. Bob uses ad hominem against
//! Alice. Both schemes use the same `claim` slot binding (`fortify_east`).
//! The expert opinion scheme has a positive conclusion template, so Alice's
//! instance concludes `Literal::atom("fortify_east")`. The ad hominem scheme
//! has a negated conclusion template, so Bob's instance concludes
//! `Literal::neg("fortify_east")`. These are direct contraries — ASPIC+
//! detects them as rebutting each other and the AF is fully populated.
//!
//! With Alice's rule preferred, Alice wins. With Bob's rule preferred, Bob
//! wins. With neither preferred, both arguments survive in different
//! preferred extensions.

use argumentation::aspic::{Literal, StructuredSystem};
use argumentation_schemes::aspic::add_scheme_to_system;
use argumentation_schemes::catalog::epistemic::argument_from_expert_opinion;
use argumentation_schemes::catalog::source::ad_hominem;
use std::collections::HashMap;

fn alice_instance() -> argumentation_schemes::instance::SchemeInstance {
    let bindings: HashMap<String, String> = [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    argument_from_expert_opinion().instantiate(&bindings).unwrap()
}

fn bob_instance() -> argumentation_schemes::instance::SchemeInstance {
    let bindings: HashMap<String, String> = [
        ("target".to_string(), "alice".to_string()),
        ("flaw".to_string(), "cowardice".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect();
    ad_hominem().instantiate(&bindings).unwrap()
}

#[test]
fn ad_hominem_concludes_negated_claim_directly() {
    let bob = bob_instance();
    assert_eq!(
        bob.conclusion,
        Literal::neg("fortify_east"),
        "ad hominem must conclude ¬claim, not claim"
    );
}

#[test]
fn alice_and_bob_rebut_each_other_in_the_af() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    add_scheme_to_system(&alice, &mut system);
    add_scheme_to_system(&bob, &mut system);

    let built = system.build_framework().unwrap();
    let alice_args = built.arguments_with_conclusion(&Literal::atom("fortify_east"));
    let bob_args = built.arguments_with_conclusion(&Literal::neg("fortify_east"));
    assert!(!alice_args.is_empty(), "Alice's argument should be constructed");
    assert!(!bob_args.is_empty(), "Bob's argument should be constructed");

    // With no preferences set, both should survive in some preferred extension
    // (one each), since rebut is mutual and neither strictly defeats the other.
    let preferred = built.framework.preferred_extensions().unwrap();
    let alice_in_some = alice_args
        .iter()
        .any(|a| preferred.iter().any(|ext| ext.contains(&a.id)));
    let bob_in_some = bob_args
        .iter()
        .any(|a| preferred.iter().any(|ext| ext.contains(&a.id)));
    assert!(alice_in_some, "Alice's argument must appear in some preferred extension");
    assert!(bob_in_some, "Bob's argument must appear in some preferred extension");
}

#[test]
fn alice_wins_when_her_rule_is_preferred() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    let alice_rule = add_scheme_to_system(&alice, &mut system);
    let bob_rule = add_scheme_to_system(&bob, &mut system);
    system.prefer_rule(alice_rule, bob_rule).unwrap();

    let built = system.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 1, "with strict preference, expect a unique preferred extension");

    let alice_wins = built
        .arguments_with_conclusion(&Literal::atom("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    let bob_wins = built
        .arguments_with_conclusion(&Literal::neg("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    assert!(alice_wins, "Alice should win when her rule is preferred");
    assert!(!bob_wins, "Bob should be defeated when Alice's rule is preferred");
}

#[test]
fn bob_wins_when_his_rule_is_preferred() {
    let alice = alice_instance();
    let bob = bob_instance();

    let mut system = StructuredSystem::new();
    let alice_rule = add_scheme_to_system(&alice, &mut system);
    let bob_rule = add_scheme_to_system(&bob, &mut system);
    system.prefer_rule(bob_rule, alice_rule).unwrap();

    let built = system.build_framework().unwrap();
    let preferred = built.framework.preferred_extensions().unwrap();
    assert_eq!(preferred.len(), 1);

    let alice_wins = built
        .arguments_with_conclusion(&Literal::atom("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    let bob_wins = built
        .arguments_with_conclusion(&Literal::neg("fortify_east"))
        .iter()
        .any(|a| preferred[0].contains(&a.id));
    assert!(!alice_wins, "Alice should be defeated when Bob's rule is preferred");
    assert!(bob_wins, "Bob should win when his rule is preferred");
}
```

- [ ] **Step 2: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes --test scheme_conflict`
Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-schemes): UC2 expert vs ad hominem rebut via negated conclusions"
```

---

### Task 15: UC3 — catalog coverage and uniqueness

**Files:**
- Create: `crates/argumentation-schemes/tests/catalog_coverage.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/tests/catalog_coverage.rs`**

```rust
//! UC3: catalog coverage. Pins the count, the per-category presence, the
//! uniqueness of ids and keys, and the presence of the schemes the
//! encounter bridge will look up by name.

use argumentation_schemes::catalog::default_catalog;
use argumentation_schemes::types::SchemeCategory;

#[test]
fn default_catalog_has_exactly_25_schemes() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.len(),
        25,
        "v0.1.0 ships with exactly 25 schemes; got {}",
        catalog.len()
    );
}

#[test]
fn every_category_has_at_least_one_scheme() {
    let catalog = default_catalog();
    let categories = [
        SchemeCategory::Epistemic,
        SchemeCategory::Causal,
        SchemeCategory::Practical,
        SchemeCategory::SourceBased,
        SchemeCategory::Popular,
        SchemeCategory::Analogical,
    ];
    for cat in &categories {
        assert!(
            !catalog.by_category(*cat).is_empty(),
            "category {:?} has no schemes",
            cat
        );
    }
}

#[test]
fn every_scheme_has_at_least_one_critical_question() {
    let catalog = default_catalog();
    for scheme in catalog.all() {
        assert!(
            !scheme.critical_questions.is_empty(),
            "scheme '{}' has no critical questions",
            scheme.name
        );
    }
}

#[test]
fn every_scheme_has_at_least_one_premise() {
    let catalog = default_catalog();
    for scheme in catalog.all() {
        assert!(
            !scheme.premises.is_empty(),
            "scheme '{}' has no premises",
            scheme.name
        );
    }
}

#[test]
fn scheme_keys_are_unique() {
    let catalog = default_catalog();
    let mut keys: Vec<String> = catalog.all().iter().map(|s| s.key()).collect();
    let total = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), total, "duplicate scheme keys found");
}

#[test]
fn scheme_ids_are_unique() {
    let catalog = default_catalog();
    let mut ids: Vec<u32> = catalog.all().iter().map(|s| s.id.0).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate scheme ids found");
}

#[test]
fn narrative_relevant_schemes_are_present() {
    let catalog = default_catalog();
    let expected = [
        "argument_from_expert_opinion",
        "argument_from_witness_testimony",
        "argument_from_positive_consequences",
        "argument_from_negative_consequences",
        "argument_from_threat",
        "ad_hominem",
        "argument_from_bias",
        "argument_from_tradition",
        "argument_from_precedent",
        "argument_from_cause_to_effect",
        "argument_from_analogy",
        "argument_from_commitment",
    ];
    for key in &expected {
        assert!(
            catalog.by_key(key).is_some(),
            "missing narrative-relevant scheme: {}",
            key
        );
    }
}

#[test]
fn category_counts_match_expected() {
    let catalog = default_catalog();
    assert_eq!(catalog.by_category(SchemeCategory::Epistemic).len(), 3);
    assert_eq!(catalog.by_category(SchemeCategory::Practical).len(), 7);
    assert_eq!(catalog.by_category(SchemeCategory::SourceBased).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Popular).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Causal).len(), 4);
    assert_eq!(catalog.by_category(SchemeCategory::Analogical).len(), 3);
}
```

- [ ] **Step 2: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes --test catalog_coverage`
Expected: 8 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-schemes): UC3 catalog coverage, uniqueness, narrative-relevant schemes"
```

---

### Task 16: UC4 — critical question enumeration

**Files:**
- Create: `crates/argumentation-schemes/tests/critical_questions.rs`

- [ ] **Step 1: Create `crates/argumentation-schemes/tests/critical_questions.rs`**

```rust
//! UC4: critical questions as follow-up beat candidates. The encounter
//! engine will iterate `instance.critical_questions` and offer each one
//! as a candidate move for the opposing party.

use argumentation::aspic::Literal;
use argumentation_schemes::catalog::epistemic::argument_from_expert_opinion;
use std::collections::HashMap;

fn alice_bindings() -> HashMap<String, String> {
    [
        ("expert".to_string(), "alice".to_string()),
        ("domain".to_string(), "military".to_string()),
        ("claim".to_string(), "fortify_east".to_string()),
    ]
    .into_iter()
    .collect()
}

#[test]
fn expert_opinion_produces_six_follow_up_candidates() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    assert_eq!(instance.critical_questions.len(), 6);
}

#[test]
fn every_critical_question_text_resolves_at_least_one_binding() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    for cq in &instance.critical_questions {
        let mentions_binding = cq.text.contains("alice")
            || cq.text.contains("military")
            || cq.text.contains("fortify_east");
        assert!(
            mentions_binding,
            "CQ{} text should contain at least one resolved binding, got: {}",
            cq.number, cq.text
        );
    }
}

#[test]
fn every_critical_question_counter_literal_is_negated() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    for cq in &instance.critical_questions {
        assert!(
            matches!(cq.counter_literal, Literal::Neg(_)),
            "CQ{} counter-literal should be negated, got: {:?}",
            cq.number, cq.counter_literal
        );
    }
}

#[test]
fn critical_questions_span_multiple_challenge_types() {
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();
    let unique_challenges: std::collections::HashSet<_> = instance
        .critical_questions
        .iter()
        .map(|cq| std::mem::discriminant(&cq.challenge))
        .collect();
    assert!(
        unique_challenges.len() >= 2,
        "expert opinion CQs should span multiple challenge types"
    );
}

#[test]
fn premise_truth_counter_literal_is_contrary_of_premise() {
    use argumentation_schemes::types::Challenge;
    let scheme = argument_from_expert_opinion();
    let instance = scheme.instantiate(&alice_bindings()).unwrap();

    // Find a CQ with a PremiseTruth challenge.
    let cq = instance
        .critical_questions
        .iter()
        .find(|cq| matches!(cq.challenge, Challenge::PremiseTruth(_)))
        .expect("expert opinion has at least one PremiseTruth CQ");

    // The counter-literal should be the contrary of one of the premise literals.
    let counter_is_contrary_of_some_premise = instance
        .premises
        .iter()
        .any(|p| cq.counter_literal.is_contrary_of(p));
    assert!(
        counter_is_contrary_of_some_premise,
        "PremiseTruth counter-literal should be the contrary of one of the instance's premise literals"
    );
}
```

- [ ] **Step 2: Run tests**

Run: `cd /home/peter/code/argumentation && cargo test --package argumentation-schemes --test critical_questions`
Expected: 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(argumentation-schemes): UC4 critical question enumeration and counter literals"
```

---

### Task 17: Public API surface, docs, and v0.1.0 release prep

**Files:**
- Modify: `crates/argumentation-schemes/src/lib.rs`
- Create: `crates/argumentation-schemes/CHANGELOG.md`
- Create: `crates/argumentation-schemes/LICENSE-MIT`
- Create: `crates/argumentation-schemes/LICENSE-APACHE`
- Modify: `crates/argumentation-schemes/README.md`

- [ ] **Step 1: Finalize `crates/argumentation-schemes/src/lib.rs`**

Replace the file with:

```rust
//! # argumentation-schemes
//!
//! Walton argumentation schemes with critical questions for structured
//! social reasoning. Each scheme is a recognisable reasoning pattern
//! (expert opinion, ad hominem, argument from consequences) that carries
//! its own critical questions — the follow-up challenges a competent
//! reasoner would raise.
//!
//! Builds on the [`argumentation`] crate's ASPIC+ layer: scheme instances
//! compile to ASPIC+ premises and defeasible rules that can be evaluated
//! via Dung semantics.
//!
//! ## Quick example
//!
//! ```
//! use argumentation_schemes::catalog::default_catalog;
//! use std::collections::HashMap;
//!
//! let catalog = default_catalog();
//! let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
//!
//! let bindings: HashMap<String, String> = [
//!     ("expert".to_string(), "alice".to_string()),
//!     ("domain".to_string(), "military".to_string()),
//!     ("claim".to_string(), "fortify_east".to_string()),
//! ]
//! .into_iter()
//! .collect();
//!
//! let instance = scheme.instantiate(&bindings).unwrap();
//! assert_eq!(instance.premises.len(), 3);
//! assert_eq!(instance.critical_questions.len(), 6);
//! ```
//!
//! ## References
//!
//! - Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.*
//!   Cambridge University Press.
//! - Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework.*
//!   Argument & Computation 5(1).

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod aspic;
pub mod catalog;
pub mod critical;
pub mod error;
pub mod instance;
pub mod registry;
pub mod scheme;
pub mod types;

pub use error::Error;
pub use instance::{instantiate, CriticalQuestionInstance, SchemeInstance};
pub use registry::CatalogRegistry;
pub use scheme::{ConclusionTemplate, PremiseSlot, SchemeMetadata, SchemeSpec};
pub use types::{Challenge, SchemeCategory, SchemeId, SchemeStrength, SlotRole};
```

- [ ] **Step 2: Create `crates/argumentation-schemes/CHANGELOG.md`**

```markdown
# Changelog

## [0.1.0] - 2026-04-12

### Added
- Core types: `SchemeSpec`, `PremiseSlot`, `ConclusionTemplate` (with
  `is_negated` flag), `CriticalQuestion`, `SchemeInstance`, `CatalogRegistry`.
- Prefix-safe template resolution: `resolve_template` sorts bindings by
  key length descending so that overlapping slot names like `threatener`
  and `threat` substitute correctly.
- 25 built-in Walton schemes across 6 categories:
  - Epistemic (3): expert opinion, witness testimony, position to know
  - Practical (7): positive/negative consequences, values, threat,
    fear appeal, waste, sunk cost
  - Source-based (4): ad hominem, ad hominem circumstantial, bias, ethotic
  - Popular (4): popular opinion, tradition, precedent, established rule
  - Causal (4): cause-to-effect, correlation, sign, slippery slope
  - Analogical (3): analogy, verbal classification, commitment
- Per-category ID offset constants prevent cross-category collisions.
- ASPIC+ integration: `add_scheme_to_system`, `add_counter_argument`.
- `SchemeSpec::instantiate` convenience method.
- 40+ unit tests across modules, plus 4 integration test suites covering
  UC1 (instantiation), UC2 (scheme conflict), UC3 (catalog coverage),
  UC4 (critical questions).

### Known limitations
- Scheme instantiation produces synthetic per-instance literals
  (e.g., `expert_alice`) that encode "this slot is filled by this value
  in this scheme instance" rather than world-fact propositions. This
  blocks multi-instance fact sharing; a `WorldFact` layer is planned for
  v0.2.0.
```

- [ ] **Step 3: Create `crates/argumentation-schemes/LICENSE-MIT`**

```
MIT License

Copyright (c) 2026 The argumentation-schemes contributors

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

- [ ] **Step 4: Create `crates/argumentation-schemes/LICENSE-APACHE`**

Use the standard Apache 2.0 license text from <https://www.apache.org/licenses/LICENSE-2.0.txt>. Copy the full text into the file. Set the copyright year to 2026.

- [ ] **Step 5: Update `crates/argumentation-schemes/README.md`**

Replace the file with:

```markdown
# argumentation-schemes

Walton argumentation schemes with critical questions for structured social
reasoning. Builds on the [`argumentation`](../..) crate's ASPIC+ layer.

## What's in the box

- 25 built-in Walton schemes across 6 categories: epistemic, practical,
  source-based, popular, causal, analogical.
- A `CatalogRegistry` for lookup by id, key, or category.
- Scheme instantiation with binding resolution and critical question
  enumeration.
- Direct ASPIC+ integration: scheme instances compile to ordinary premises
  plus a defeasible rule, ready to feed into a `StructuredSystem`.

## Quick example

```rust
use argumentation_schemes::catalog::default_catalog;
use std::collections::HashMap;

let catalog = default_catalog();
let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();

let bindings: HashMap<String, String> = [
    ("expert".to_string(), "alice".to_string()),
    ("domain".to_string(), "military".to_string()),
    ("claim".to_string(), "fortify_east".to_string()),
]
.into_iter()
.collect();

let instance = scheme.instantiate(&bindings).unwrap();
assert_eq!(instance.critical_questions.len(), 6);
```

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.*
  Cambridge University Press.
- Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework.*
  Argument & Computation 5(1).
```

- [ ] **Step 6: Full verification sweep**

Run each command in turn from `/home/peter/code/argumentation`:

```bash
cargo test --package argumentation-schemes
```
Expected: 42 unit tests + 19 integration tests (2 + 4 + 8 + 5) all pass.

```bash
cargo test --workspace
```
Expected: existing argumentation tests still pass; new crate's tests pass.

```bash
cargo clippy --package argumentation-schemes -- -D warnings
```
Expected: zero warnings.

```bash
cargo fmt --package argumentation-schemes -- --check
```
Expected: no formatting drift. If any, run `cargo fmt --package argumentation-schemes` and re-stage.

```bash
cargo doc --package argumentation-schemes --no-deps
```
Expected: docs build cleanly with no missing-docs warnings (since `#![deny(missing_docs)]` is in effect).

- [ ] **Step 7: Commit and tag**

```bash
git add -A
git commit -m "chore(argumentation-schemes): v0.1.0 release prep"
git tag -a argumentation-schemes-v0.1.0 -m "argumentation-schemes v0.1.0"
```

---

## Out of scope for v0.1.0

- **AIF (Argument Interchange Format) import/export.** Deferred to v0.2.0. The scheme data model is compatible with AIF; adding serde + XML/JSON support is ~1 week of additional work.
- **Natural-language parsing into schemes.** That's argument mining, a separate research field.
- **Visualisation / GUI for scheme authoring.** Carneades' niche, not ours.
- **Additional schemes beyond the initial 25.** Walton 2008 lists 65. The remaining ~40 can be added incrementally as encounter use cases demand them.
- **`encounter-argumentation` bridge crate.** The bridge that maps scheme instances to encounter's `AffordanceSpec` and wires critical questions into multi-beat follow-up beats. Separate crate, separate plan.
- **`WorldFact` layer.** The v0.2.0 fix for the synthetic-literal MVP limitation: a layer that maps scheme slot bindings onto shared knowledge-base literals so that two scheme instances about the same agent share evidence.
- **Domain-specific scheme extensions** (legal proof standards, medical reasoning patterns). Consumers can implement custom schemes via the `SchemeSpec` type without modifying this crate.
- **Scheme-to-scheme relationship metadata** (e.g., "ad hominem is a counter-scheme to argument from expert opinion"). Walton discusses these but formalising them is v0.2.0 scope.

## References

- Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.* Cambridge University Press. Appendix 1 contains the full scheme catalogue this crate draws from.
- Walton, D. (1996). *Argumentation Schemes for Presumptive Reasoning.* Lawrence Erlbaum.
- Chesñevar, C. et al. (2006). *Towards an argument interchange format.* Knowledge Engineering Review 21(4).
- Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework for structured argumentation.* Argument & Computation 5(1) — the ASPIC+ layer this crate builds on.

---

**End of plan.**
