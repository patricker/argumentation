---
sidebar_position: 99
title: Changelog
---

Per-crate release history. The full per-crate `CHANGELOG.md` files in the repo are the source of truth; this page mirrors them for discoverability and cross-version comparison.

## `encounter-argumentation`

### 0.5.0 — 2026-04-25 (breaking)

**Removed.** `SocietasRelationshipSource`, `NameResolver`, and the six coefficient constants (`BASELINE_WEIGHT`, `TRUST_COEF`, `FEAR_COEF`, `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF`) moved to the `societas-encounter` crate (under the `argumentation` feature). See the [migration guide](/guides/migration-v0.4-to-v0.5).

**Removed dependencies.** `societas-core`, `societas-relations`, `societas-memory`.

**Preserved.** `EncounterArgumentationState` (including `actors_by_argument()`), `StateAcceptanceEval`, `StateActionScorer`, `AffordanceKey`, error variants, scheme/CQ APIs.

### 0.4.0 — 2026-04-24

**Added.** `SocietasRelationshipSource<'a, R>` (a `WeightSource<ArgumentId>` reading societas-relations dimensions), `NameResolver` trait + HashMap blanket impl, six coefficient `pub const`s, `EncounterArgumentationState::actors_by_argument()` accessor.

**Removed.** Phase A `RelationshipDims` / `RelationshipSnapshot` / `RelationshipWeightSource` stubs — these had a soundness bug (`ArgumentId` treated as actor name).

(Superseded by 0.5.0 — see migration guide.)

### 0.3.0 — 2026-04-20

**Added.** Bridge state types: `StateAcceptanceEval` (per-responder credulous-counter check), `StateActionScorer` (proposer-side credulous boost), `AffordanceKey` ((actor, affordance, bindings) forward index key).

**Added.** State APIs: `add_scheme_instance_for_affordance`, `argument_id_for`, `attackers_of`, `has_accepted_counter_by`, `set_intensity` (shared-ref setter), `drain_errors`, `Error::MissingProposerBinding`.

**Changed.** `intensity` and the error latch use `std::sync::Mutex` internally — required by encounter's `AcceptanceEval: Send + Sync` bound.

### 0.2.0 — 2026-04-19

**Added.** `EncounterArgumentationState` — unified state composing schemes + bipolar + weighted + weighted-bipolar with a tunable scene-intensity `Budget`.

## `societas-encounter`

### Unreleased — argumentation feature

**Added.** Optional feature `argumentation` (default off) — adds `argumentation-weighted` and `encounter-argumentation` as deps.

**Added.** `argumentation::SocietasRelationshipSource<'a>` — `WeightSource<ArgumentId>` impl reading five societas-relations dimensions and applying a coefficient recipe. Resolves `ArgumentId → actor names` via `EncounterArgumentationState::actors_by_argument`, then `name → EntityId` via the existing `crate::names::NameResolver` trait.

**Added.** Six public coefficient constants: `BASELINE_WEIGHT`, `TRUST_COEF`, `FEAR_COEF`, `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF`. Same values as `encounter-argumentation v0.4.0`.

**Notes.** Multi-actor arguments aggregate per-pair weights by arithmetic mean across the (attacker × target) Cartesian product. Unresolvable actor names are silently skipped (not promoted to baseline pairs). Calibration is provisional — pending gameplay telemetry.

## `argumentation-values`

### 0.1.0 — 2026-04-30 (initial release)

**Added.** First release of the `argumentation-values` crate.

- `Value`, `ValueAssignment<A>` (multi-value via SmallVec), `Audience` (ranked tiers with `rank()` accessor) — Bench-Capon (2003) types extended with Kaci & van der Torre (2008) multi-value support.
- `ValueBasedFramework<A>` with `defeat_graph(audience)`, `defeats(attacker, target, audience)`, `accepted_for(audience, arg)`, `grounded_for(audience)`. Pareto-defeating rule.
- `subjectively_accepted` and `objectively_accepted` (capped at 6 distinct values per Dunne & Bench-Capon 2004 complexity).
- `MultiAudience` for multi-character consensus queries (DiArg-style AgreementScenario).
- APX format I/O (`from_apx` / `to_apx`) for ASPARTIX interop.
- `from_scheme_instances` bridge for populating `ValueAssignment` from instantiated `argument_from_values` Walton schemes.
- 40 tests including the Hal & Carla integration test under three audiences as the success criterion.

**Companion change in `encounter-argumentation`** (no version bump — same crate v0.5):

- `EncounterArgumentationState` gains per-actor `audiences: Mutex<HashMap<String, Audience>>` storage, mirroring `intensity`.
- `ValueAwareScorer<S>` wraps any inner scorer and adds audience-conditioned value-preference boost.

## See also

- [`encounter-argumentation` CHANGELOG](https://github.com/patricker/argumentation/blob/main/crates/encounter-argumentation/CHANGELOG.md) (canonical)
- [`societas-encounter` CHANGELOG](https://github.com/patricker/societas/blob/main/crates/encounter/CHANGELOG.md) (canonical)
- [Migration v0.4 → v0.5](/guides/migration-v0.4-to-v0.5)
