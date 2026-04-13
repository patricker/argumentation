# Changelog

## [0.1.0] - 2026-04-13

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
  Dunne 2011 inconsistency-budget semantics: attacks are tolerated in
  ascending weight order, cheapest first, until the budget would be
  exceeded. This approximation is NOT globally monotone in β —
  a chained-defense framework like `a→b→c` can flip `c`'s acceptance
  from true (at β=0, via `a`'s defense) to false (when `a→b` is
  tolerated) to true again (when `b→c` is also tolerated). The full
  existential-subset semantics would be monotone but requires
  enumeration over `2^|attacks|` subsets and is deferred to v0.2.0.
  See `tests/uc3_scene_intensity.rs` for the witness fixture.
- No composition with `argumentation-bipolar` yet (weighted bipolar
  frameworks per Amgoud et al. 2008 are deferred to v0.2.0).
