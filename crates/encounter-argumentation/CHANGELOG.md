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
