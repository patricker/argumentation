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
