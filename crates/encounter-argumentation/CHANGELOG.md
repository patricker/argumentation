# Changelog

## [0.5.0] - 2026-04-25

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

**Signature change:** the generic `R: NameResolver` type parameter on
`SocietasRelationshipSource<'a, R>` is gone — the new home uses
`SocietasRelationshipSource<'a>` with `&'a dyn NameResolver`. The
v0.4.0 `HashMap<String, EntityId>` blanket impl is NOT carried over;
consumers that constructed the source directly with a HashMap must
switch to `StaticNameResolver` (or another `NameResolver` implementor):

```rust
// v0.4.0:
let resolver: HashMap<String, EntityId> = ...;
let source = SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, tick);

// v0.5.0:
let mut resolver = StaticNameResolver::new();
resolver.add("alice", EntityId::from_u64(1));
let source = SocietasRelationshipSource::new(&registry, &store, &resolver, &actors, tick);
```

(The `&resolver` borrow at the call site implicitly coerces to `&dyn NameResolver` — no API change beyond the type swap.)

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

## [0.4.0] - 2026-04-24

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
  bridge consumers can look it up at encounter `resolve` time without
  round-tripping through scheme instantiation.
- `argument_id_for(&AffordanceKey) -> Option<ArgumentId>`.
- `attackers_of(&ArgumentId) -> Vec<ArgumentId>` — structural query.
- `has_accepted_counter_by(responder, &target) -> Result<bool, Error>`
  — per-responder credulous-counter query.
- `set_intensity(&self, Budget)` — **shared-ref** setter (replaces the
  prior by-value `at_intensity` requirement for mid-scene β change).
- `drain_errors() -> Vec<Error>` — drain the bridge's error buffer;
  trait impls default to permissive on internal errors and append to
  this buffer for later retrieval.
- `Error::MissingProposerBinding { affordance_name }` — surfaces when
  `StateAcceptanceEval` encounters an affordance without a `"self"`
  binding; the eval defaults to *accept* AND records this error so
  consumers can diagnose via `drain_errors`.

### Changed
- `EncounterArgumentationState::intensity` and the new error latch
  are now stored in `std::sync::Mutex` internally. `intensity()` still
  returns `Budget` by value; `at_intensity(self, Budget) -> Self`
  (by-value builder) still works. The `Mutex` choice (over `Cell` /
  `RefCell` as originally planned) is forced by encounter's
  `AcceptanceEval<P>: Send + Sync` trait bound, which requires
  `EncounterArgumentationState: Sync`. Mutex poisoning is tolerated
  via `unwrap_or_else(|e| e.into_inner())` — a prior panic elsewhere
  must not turn into a panic inside encounter's resolve loop.

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
`/home/peter/code/encounter/` is not modified. This contrasts with an
earlier plan iteration that proposed changes to encounter's `AffordanceSpec`,
`SingleExchange`, and `MultiBeat` — all that complexity is absorbed here.

### Deferred to v0.3.1
- `seed_from_bindings` — a bulk-seeding helper (take a list of
  `(actor, affordance_name, bindings, instance)` tuples) was discussed
  in the design but not shipped. Current consumers should loop over
  `add_scheme_instance_for_affordance` calls manually; a batch helper
  will land in v0.3.1 if the verbosity becomes a pain point.

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
