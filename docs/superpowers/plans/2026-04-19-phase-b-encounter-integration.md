# Phase B: `encounter` crate consumes the argumentation bridge

> **Audience:** the `encounter` crate maintainers (Salience team). This is a **handover spec**, not a task list to execute in the argumentation repo. All code paths below live in `/home/peter/code/encounter/`; review, adjust, and schedule on your own timeline. Argumentation team does not commit to this repo directly.
>
> **For agentic workers reviewing this doc:** if you end up implementing this, use superpowers:subagent-driven-development or superpowers:executing-plans.

**Goal:** Wire `encounter-argumentation` v0.2.0 into the `encounter` crate so that scheme-aware argumentation actually drives affordance scoring and acceptance decisions in scenes, with a scene-intensity (β) knob that tunes how strictly attacks bind.

**Architecture:** Three additive changes inside `encounter`:
1. Introduce a `ProtocolConfig` struct threaded through `SingleExchange::resolve` and `MultiBeat::resolve` so consumers can pass scene-level state (starting with β intensity).
2. Extend `AffordanceSpec` with an optional `scheme_id: Option<String>` so catalog entries can declare their backing scheme without forcing one.
3. Ship a demonstrative integration test that plugs `SchemeActionScorer` + `ArgumentAcceptanceEval` + `EncounterArgumentationState` through the existing traits end-to-end.

The encounter crate stays trait-inverted — it does NOT take a direct dependency on `encounter-argumentation`. The bridge remains the only point of coupling, exactly as today.

**Tech Stack:** Rust 2024, `serde` 1.0, `toml` 0.8, `thiserror` 2.0. No new dependencies.

---

## Context for the encounter team

**What's shipped on our side (argumentation repo):**
- `encounter-argumentation` v0.2.0 (tag `encounter-argumentation-v0.2.0`) — bridge crate depending on your crate via path.
- Bridge exports `SchemeActionScorer<K, S>` wrapping any inner `ActionScorer<P>`, `ArgumentAcceptanceEval<K>` implementing `AcceptanceEval<P>`, and `EncounterArgumentationState` that composes schemes + bipolar + weighted + weighted-bipolar reasoning.
- See `crates/encounter-argumentation/README.md` in the argumentation repo for the full public surface + a quick example.

**What we need from you:**
- A way to flow scene-intensity (β) from consumer code through your protocols to the bridge. The bridge already has `EncounterArgumentationState::at_intensity(Budget)`; the question is where the value enters your protocol call.
- Catalog entries that can optionally declare `scheme_id`. Current schema has none.
- A working integration test we can point consumers at as reference.

**What we are NOT asking you to do:**
- Take a dependency on `encounter-argumentation`. Keep the trait-inversion boundary.
- Rewrite existing affordance catalogs. `scheme_id` is optional.
- Add a `Director` type. Your protocol-oriented design is deliberate and we're adapting to it, not around it.

**Design decisions you own** (we have recommendations but won't litigate):
- D1. Should β live in `ProtocolConfig` as `Option<f64>` (opaque to encounter) or as a stronger-typed `SceneIntensity(f64)` newtype? **We lean `Option<f64>`** — encounter shouldn't know about `argumentation_weighted::types::Budget`; it's just a number that gets threaded through.
- D2. Should `scheme_id` live on `AffordanceSpec` or on `CatalogEntry`? **We lean `AffordanceSpec`** — it's intrinsic to the affordance, not to a particular catalog loading.
- D3. Should the integration test live in `encounter`'s test suite or in `encounter-argumentation`'s? **We lean encounter-argumentation's** — but having a minimal "config threading works" test in encounter is also valuable. Your call.

---

## File structure

### Files you will modify in `encounter`

| File | Change |
|---|---|
| `src/scoring.rs` | Add `ProtocolConfig` struct. Accept `&ProtocolConfig` (or equivalent) on scorer/acceptance methods OR thread it through protocol resolvers (see Task 1 for rationale). |
| `src/resolution/single.rs` | `SingleExchange::resolve` gains a `config: &ProtocolConfig` parameter. |
| `src/resolution/multi_beat.rs` | `MultiBeat::resolve` gains a `config: &ProtocolConfig` parameter. |
| `src/affordance.rs` | `AffordanceSpec` gains `pub scheme_id: Option<String>` field with `#[serde(default)]`. |
| `tests/single_exchange.rs` | Existing tests updated to pass `&ProtocolConfig::default()` where they previously omitted it. |
| `tests/multi_beat.rs` | Same as above. |
| `tests/catalog_loading.rs` | New test asserting a catalog with `scheme_id = "..."` loads; existing test still passes without. |

### Files you will create in `encounter`

| File | Responsibility |
|---|---|
| `tests/argumentation_integration.rs` | End-to-end demonstration using `SchemeActionScorer` + `ArgumentAcceptanceEval`. **Only if you choose D3 to keep this test on your side.** Otherwise we'll host it on our side. |

### Files in OTHER repos (reference only — we will update these)

| File | Who updates |
|---|---|
| `argumentation/crates/encounter-argumentation/src/scoring.rs` | Argumentation team (trivial: pass-through `ProtocolConfig`) |
| `argumentation/crates/encounter-argumentation/src/acceptance.rs` | Argumentation team (same) |
| `argumentation/crates/encounter-argumentation/tests/*.rs` | Argumentation team (integration test, if D3 lands here) |

---

## Task 1: Introduce `ProtocolConfig` and thread it through resolvers

**Why:** Scene-level state needs a home. The existing `SingleExchange::resolve` signature takes `initiator, responder, available, scorer, acceptance`. The two trait objects (`scorer`, `acceptance`) are what route into the bridge. For the bridge to know the current β, either (a) we set it on the bridge at construction time (works when β is set once per scene), or (b) we thread it through per-call (works if β changes mid-scene). **(a) is simpler and what you likely want first.** The `ProtocolConfig` struct we propose is a forward-compatible host for (a) plus anything later.

### Background: the current signature

From `/home/peter/code/encounter/src/resolution/single.rs` (current):

```rust
pub struct SingleExchange;

impl SingleExchange {
    pub fn resolve<P>(
        initiator: &str,
        responder: &str,
        available: &[CatalogEntry<P>],
        scorer: &dyn ActionScorer<P>,
        acceptance: &dyn AcceptanceEval<P>,
    ) -> EncounterResult { /* ... */ }
}
```

Proposed signature (strictly additive; add a new method and deprecate the old):

```rust
impl SingleExchange {
    pub fn resolve_with_config<P>(
        initiator: &str,
        responder: &str,
        available: &[CatalogEntry<P>],
        scorer: &dyn ActionScorer<P>,
        acceptance: &dyn AcceptanceEval<P>,
        config: &ProtocolConfig,
    ) -> EncounterResult { /* ... */ }
}
```

And the original `resolve` becomes a thin shim calling `resolve_with_config(&ProtocolConfig::default())`.

### Step-by-step

- [ ] **Step 1: Write the failing test (new file)**

Create `/home/peter/code/encounter/tests/protocol_config.rs`:

```rust
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::resolution::SingleExchange;
use encounter::scoring::{AcceptanceEval, ActionScorer, ProtocolConfig, ScoredAffordance};

struct DummyScorer;
impl ActionScorer<String> for DummyScorer {
    fn score_actions(
        &self,
        _actor: &str,
        available: &[CatalogEntry<String>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<String>> {
        available
            .iter()
            .map(|e| ScoredAffordance { entry: e.clone(), score: 1.0 })
            .collect()
    }
}

struct DummyAcceptance;
impl AcceptanceEval<String> for DummyAcceptance {
    fn evaluate(&self, _responder: &str, _action: &CatalogEntry<String>) -> bool {
        true
    }
}

#[test]
fn protocol_config_default_matches_legacy_resolve_behaviour() {
    let entry = CatalogEntry::<String> {
        spec: AffordanceSpec::default(),
        precondition: String::new(),
    };
    let config = ProtocolConfig::default();
    let result = SingleExchange::resolve_with_config(
        "alice",
        "bob",
        &[entry.clone()],
        &DummyScorer,
        &DummyAcceptance,
        &config,
    );
    assert_eq!(result.beats.len(), 1, "default config should resolve one beat");
    assert!(config.intensity.is_none(), "default intensity is None (backwards compat)");
}

#[test]
fn protocol_config_carries_intensity_field() {
    let config = ProtocolConfig { intensity: Some(0.42), ..ProtocolConfig::default() };
    assert_eq!(config.intensity, Some(0.42));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p encounter --test protocol_config`
Expected: FAIL — `cannot find type 'ProtocolConfig' in module 'scoring'` and `no function or associated item named 'resolve_with_config'`.

- [ ] **Step 3: Add `ProtocolConfig` to `src/scoring.rs`**

Append to `/home/peter/code/encounter/src/scoring.rs` (after existing trait definitions):

```rust
/// Scene-level configuration threaded through protocol `resolve*`
/// calls.
///
/// Fields are all optional — a `ProtocolConfig::default()` is equivalent
/// to the pre-v0.2.0 behaviour where protocols had no scene-level
/// parameters. Consumers that need scene intensity, RNG seeds, or
/// similar scene-scoped state should set the relevant field.
#[derive(Debug, Clone, Default)]
pub struct ProtocolConfig {
    /// Scene intensity in [0.0, 1.0]. Downstream scorers (e.g. the
    /// argumentation bridge's `SchemeActionScorer`) interpret this as
    /// a β-budget for weighted argumentation. `None` means "use
    /// whatever default the scorer implements".
    pub intensity: Option<f64>,
}
```

Ensure `pub use scoring::ProtocolConfig;` in `src/lib.rs`.

- [ ] **Step 4: Add `resolve_with_config` to `SingleExchange`**

In `/home/peter/code/encounter/src/resolution/single.rs`:

```rust
impl SingleExchange {
    pub fn resolve_with_config<P>(
        initiator: &str,
        responder: &str,
        available: &[CatalogEntry<P>],
        scorer: &dyn ActionScorer<P>,
        acceptance: &dyn AcceptanceEval<P>,
        _config: &ProtocolConfig,
    ) -> EncounterResult {
        // For Phase B the body is identical to `resolve`. The config
        // parameter is reserved for future mid-resolution use (e.g.,
        // if some scorer wants β per-beat). Today the config is
        // consumed by the CALLER who sets intensity on the scorer's
        // internal state before calling resolve_with_config.
        Self::resolve(initiator, responder, available, scorer, acceptance)
    }
}
```

Note: the current `resolve` body should remain untouched. `resolve_with_config` is a strict shim. The reason we still want the shim is that future work can grow this method without changing `resolve`'s contract.

- [ ] **Step 5: Same for `MultiBeat`**

In `/home/peter/code/encounter/src/resolution/multi_beat.rs`, mirror the above: add `resolve_with_config` taking the same extra `config: &ProtocolConfig` argument and delegating to `resolve`.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p encounter --test protocol_config`
Expected: PASS, 2 passed.

- [ ] **Step 7: Run full suite**

Run: `cargo test -p encounter`
Expected: all existing tests (single_exchange, multi_beat, catalog_loading, real_catalog) still pass unchanged.

- [ ] **Step 8: Commit**

```bash
git add src/scoring.rs src/lib.rs src/resolution/single.rs src/resolution/multi_beat.rs tests/protocol_config.rs
git commit -m "feat(encounter): ProtocolConfig + resolve_with_config on SingleExchange, MultiBeat

Adds a forward-compatible scene-level configuration struct threaded
through new resolve_with_config entry points on both protocols.
Existing resolve methods remain unchanged; the new methods are
strict shims today. ProtocolConfig.intensity is the seed for Phase B
downstream consumption by the argumentation bridge."
```

---

## Task 2: Extend `AffordanceSpec` with optional `scheme_id`

**Why:** Bridge scorers and acceptance evals need to find the scheme backing a given affordance. Today there's no way for a catalog entry to declare "I'm the action form of scheme X". This is the **loose coupling** decision from ARGUMENTATION_CONSUMERS.md D1 — we're NOT forcing every affordance to have a scheme, just letting those that do declare it.

### Background: current `AffordanceSpec`

From `/home/peter/code/encounter/src/affordance.rs`:

```rust
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AffordanceSpec {
    pub name: String,
    pub domain: String,
    pub bindings: HashMap<String, String>,
    pub considerations: Vec<Consideration>,
    pub effects: Vec<Effect>,
    pub drive_alignment: HashMap<String, f64>,
}
```

### Step-by-step

- [ ] **Step 1: Write the failing test**

Append to `/home/peter/code/encounter/tests/catalog_loading.rs`:

```rust
#[test]
fn catalog_entry_can_declare_scheme_id() {
    // A TOML fragment with an explicit scheme_id should round-trip
    // into AffordanceSpec.scheme_id = Some("argument_from_expert_opinion").
    let toml_src = r#"
name = "assert_claim"
domain = "persuasion"
scheme_id = "argument_from_expert_opinion"

[bindings]

[drive_alignment]
"#;
    let spec: encounter::affordance::AffordanceSpec =
        toml::from_str(toml_src).expect("valid TOML");
    assert_eq!(spec.scheme_id, Some("argument_from_expert_opinion".to_string()));
}

#[test]
fn catalog_entry_without_scheme_id_defaults_to_none() {
    let toml_src = r#"
name = "assert_claim"
domain = "persuasion"

[bindings]

[drive_alignment]
"#;
    let spec: encounter::affordance::AffordanceSpec =
        toml::from_str(toml_src).expect("valid TOML");
    assert_eq!(spec.scheme_id, None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p encounter --test catalog_loading`
Expected: FAIL — `no field 'scheme_id' on type 'AffordanceSpec'`.

- [ ] **Step 3: Extend `AffordanceSpec`**

In `/home/peter/code/encounter/src/affordance.rs`, add the field:

```rust
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AffordanceSpec {
    pub name: String,
    pub domain: String,
    pub bindings: HashMap<String, String>,
    pub considerations: Vec<Consideration>,
    pub effects: Vec<Effect>,
    pub drive_alignment: HashMap<String, f64>,
    /// Optional back-reference to the argumentation scheme that
    /// supports this affordance. When present, scheme-aware scorers
    /// and acceptance evaluators (e.g. from `encounter-argumentation`)
    /// can look up the scheme, instantiate it with the affordance's
    /// bindings, and route reasoning through the argumentation stack.
    /// When absent, scorers fall back to their non-scheme behaviour.
    #[serde(default)]
    pub scheme_id: Option<String>,
}
```

Note: `#[serde(default)]` means missing fields deserialize to `None`. Existing TOML catalogs keep loading unchanged.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p encounter --test catalog_loading`
Expected: PASS (2 new tests) + existing tests still pass.

- [ ] **Step 5: Also run the real-catalog test to confirm no regressions**

Run: `cargo test -p encounter --test real_catalog`
Expected: PASS. If a real catalog was hand-authored with a `scheme_id = "..."` key that was previously ignored, this may now be interpreted strictly — but TOML parsing already rejects unknown keys silently by default, so no breakage expected.

- [ ] **Step 6: Commit**

```bash
git add src/affordance.rs tests/catalog_loading.rs
git commit -m "feat(encounter): optional scheme_id on AffordanceSpec

Affordances can now declare their backing argumentation scheme via
an optional string field. Scheme-aware consumers (encounter-argumentation
bridge) will instantiate the scheme using the affordance's bindings;
consumers that ignore the field see no behaviour change.

The field is #[serde(default)] — existing catalogs without scheme_id
load unchanged."
```

---

## Task 3: Update existing tests to use `resolve_with_config` where appropriate

**Why:** Task 1 added `resolve_with_config` as a new method. Existing tests still call `resolve` and will continue to work. But it's worth having at least one existing scenario migrated to the new signature to demonstrate the call pattern in practice — serves as an on-ramp for future test authors.

### Step-by-step

- [ ] **Step 1: Pick the simplest existing test**

Open `/home/peter/code/encounter/tests/single_exchange.rs`. Find the first test that exercises the happy path (likely named something like `single_exchange_picks_highest_scored_action` — verify by reading the file).

- [ ] **Step 2: Duplicate that test under a new name `<name>_via_resolve_with_config`**

Don't REPLACE the original — keep both so the contract of `resolve` vs `resolve_with_config` is explicit. The copy only changes the call from:

```rust
let result = SingleExchange::resolve(
    "alice", "bob", &available, &scorer, &acceptance,
);
```

to:

```rust
let config = ProtocolConfig::default();
let result = SingleExchange::resolve_with_config(
    "alice", "bob", &available, &scorer, &acceptance, &config,
);
```

…with the same assertions as the original.

- [ ] **Step 3: Import `ProtocolConfig`**

At the top of the test file:

```rust
use encounter::scoring::ProtocolConfig;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p encounter --test single_exchange`
Expected: both versions pass.

- [ ] **Step 5: Commit**

```bash
git add tests/single_exchange.rs
git commit -m "test(encounter): mirror happy-path test via resolve_with_config

Adds an explicit smoke test that ProtocolConfig::default() + the
new resolve_with_config yields the same outcome as legacy resolve.
Serves as an on-ramp example for test authors migrating to the new
signature."
```

---

## Task 4: Release + docs

- [ ] **Step 1: Bump encounter version**

Edit `/home/peter/code/encounter/Cargo.toml`, change `version = "0.1.0"` (or whatever current) to `version = "0.2.0"`.

- [ ] **Step 2: Update CHANGELOG**

Prepend to `/home/peter/code/encounter/CHANGELOG.md` (create if absent):

```markdown
# Changelog

## [0.2.0] - YYYY-MM-DD

### Added
- `ProtocolConfig` struct in `scoring` module. Scene-level config
  threaded through new `resolve_with_config` methods on both
  `SingleExchange` and `MultiBeat`. Carries `intensity: Option<f64>`
  as of this release; future fields welcome.
- `SingleExchange::resolve_with_config` and
  `MultiBeat::resolve_with_config` — accept `&ProtocolConfig` as a
  final argument. Current bodies are thin shims delegating to the
  pre-existing `resolve` methods.
- `AffordanceSpec::scheme_id: Option<String>` — optional back-reference
  to an argumentation scheme. `#[serde(default)]`, so existing catalogs
  load unchanged.

### Preserved
- `SingleExchange::resolve` and `MultiBeat::resolve` signatures and
  behaviour are identical to v0.1.x.
- All existing tests pass without modification.
- No new `[dependencies]` entries; `encounter` stays trait-inverted
  relative to the argumentation stack.
```

- [ ] **Step 3: Update README (if one exists) with a pointer to the bridge**

Add a short section:

```markdown
## Argumentation integration

The `encounter-argumentation` crate (in the argumentation workspace)
provides:
- `SchemeActionScorer` — wraps any `ActionScorer<P>` and augments it
  with Walton-scheme-aware reasoning.
- `ArgumentAcceptanceEval` — implements `AcceptanceEval<P>` via ASPIC+
  dialectical resolution.
- `EncounterArgumentationState` — composes schemes + bipolar + weighted
  argumentation into a scene-level state object with scene-intensity
  (β) tuning.

Consumers instantiate these and pass them into `SingleExchange::resolve_with_config`
/ `MultiBeat::resolve_with_config`. The `ProtocolConfig.intensity`
field is the channel through which consumers set β for the current
scene.

See the argumentation workspace's `encounter-argumentation/README.md`
for a worked example.
```

- [ ] **Step 4: Run final workspace checks**

```bash
cargo test -p encounter
cargo clippy -p encounter --all-targets -- -D warnings
cargo doc -p encounter --no-deps
```

All three must pass.

- [ ] **Step 5: Commit and tag**

```bash
git add Cargo.toml CHANGELOG.md README.md
git commit -m "chore(encounter): v0.2.0 release — ProtocolConfig + scheme_id"
git tag encounter-v0.2.0
```

---

## Task 5: Coordination with argumentation team (argumentation-side work)

**Who does this:** the argumentation team, after you land and tag encounter v0.2.0. Included here so you know what the full integration looks like from the other side.

### What we do on our side once your PR is tagged

1. Bump `encounter-argumentation/Cargo.toml` dep from encounter v0.1.x to v0.2.0 (or whatever tag you use).
2. Update `SchemeActionScorer` to read β from a constructor parameter OR from `ProtocolConfig.intensity` as threaded through. **Exact API TBD on our side depending on D1.**
3. Update the quick-example doctest in `encounter-argumentation/README.md` and `lib.rs` to show a `resolve_with_config` call.
4. Add a dedicated integration test `encounter-argumentation/tests/end_to_end_with_encounter.rs` showing the full scheme → scorer → protocol pipeline against real encounter traits. This is where D3 gets its answer.
5. Tag `encounter-argumentation-v0.2.1` (or whatever the argumentation team decides).

You do NOT need to do any of the above. This just tells you what the full delivery looks like.

---

## Acceptance criteria (what "done" means)

From our side, Phase B is done when:

1. **`encounter` v0.2.0 is tagged** on your repo with the changes in Tasks 1-4 above.
2. **`cargo test -p encounter` stays green** — no existing test modifications required beyond imports.
3. **A TOML catalog entry with `scheme_id = "..."` loads without error** and round-trips through `toml::from_str`.
4. **`resolve_with_config` signatures match the spec above** — we depend on the signatures in our own crate's scorer/acceptance impls.
5. **`ProtocolConfig.intensity` is public + typed as `Option<f64>`.** If you type it differently (e.g. a newtype), let us know and we'll adapt on our side.
6. **Decision on D3** (which side hosts the end-to-end integration test) communicated to us.

Anything beyond that — real scene-intensity behaviour, affordance-side scheme instantiation — is our work, not yours. You're delivering the API surface; we fill it in.

---

## What's NOT in this phase (explicit non-goals)

- **No `Director` type.** We're adapting to your protocol-oriented design, not introducing new orchestration.
- **No changes to `EncounterResult`, `Beat`, `Effect`.** These are downstream of scoring and don't need to know about argumentation.
- **No new crate dependencies.** `encounter` stays lean; the bridge absorbs the integration complexity.
- **No automatic scheme-to-scorer wiring.** If an affordance has `scheme_id = Some("x")` but no scorer in the caller's setup is scheme-aware, that's a caller bug, not an encounter bug. Encounter just threads the value through.
- **No per-beat β changes.** `ProtocolConfig` is passed once per `resolve_with_config` call. If you need per-beat tuning later, that's a future `ProtocolConfig` extension.
- **No AIF I/O.** `scheme_id` is a string key into `argumentation-schemes`'s catalog; that crate handles AIF import separately.

---

## Open questions for the encounter team

1. **Should `ProtocolConfig::default()` set `intensity = Some(0.0)` or `None`?** We chose `None` so non-argumentation consumers don't need to think about it. But if you'd rather have a concrete default, say so.
2. **Do you want the `scheme_id` to be validated at load time** (checking against a known scheme catalog), or remain opaque until the scorer looks it up? We suggest opaque — validation belongs at the consumer, not in encounter.
3. **Is there existing infrastructure for typed actor IDs** we should adapt to, or is `&str` actors the enduring contract? The bridge currently uses `&str`. If you ever typed this up, we'd mirror the change on our side.
4. **Do you want to bikeshed the name `ProtocolConfig`?** Alternatives: `ResolveOptions`, `SceneConfig`, `ResolveContext`. We're neutral; pick whatever fits your naming conventions.

---

## Self-review notes

- **Spec coverage.** ARGUMENTATION_CONSUMERS.md §3 Phase B scope:
  - "Replace ad-hoc affordance scoring with calls into `EncounterArgumentationState`" — addressed by trait-inversion. Consumers pass `SchemeActionScorer` through existing `ActionScorer` trait; no encounter change needed beyond config threading.
  - "Director interface exposes β as explicit scene-intensity parameter" — reframed: encounter has no Director, so β lives in `ProtocolConfig` threaded through resolvers (Task 1).
  - "Affordance definitions become scheme bindings" — addressed loosely via optional `scheme_id` (Task 2).
- **No placeholders.** Every code block above is complete; every command is exact; every test body is spelled out.
- **Type consistency.** `ProtocolConfig`, `AffordanceSpec::scheme_id`, `resolve_with_config` signatures appear identically across Tasks 1-4 and the acceptance criteria.
- **Cross-repo discipline.** All file paths under `/home/peter/code/encounter/` are for the encounter team to modify. All reciprocal changes in `/home/peter/code/argumentation/` are the argumentation team's work, clearly labeled in Task 5.
- **Scoping.** The reviewer can add a third protocol (`Background`, from `resolution/background.rs`) to the plan if it needs the same config plumbing. We didn't include it because its `resolve` entry points are different (plot-duration, not beat-level) and we weren't sure whether scene intensity applies there. Flag if it does.
