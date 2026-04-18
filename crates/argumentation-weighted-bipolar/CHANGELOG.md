# Changelog

## [0.1.0] - 2026-04-17

### Added
- `WeightedBipolarFramework<A>` with mutation API for weighted attacks
  and weighted supports (both non-negative finite f64).
- `WeightedSupport<A>` type with `new()` constructor validating
  non-self-support and non-negative finite weights.
- `wbipolar_residuals(framework, budget)` — exact Dunne 2011 enumeration
  of β-inconsistent subsets of `attacks ∪ supports`, returning one
  `BipolarFramework` per residual.
- `is_credulously_accepted_at` and `is_skeptically_accepted_at` —
  acceptance queries that aggregate bipolar-preferred extensions across
  residuals (OR for credulous, AND for skeptical).
- `EDGE_ENUMERATION_LIMIT = 24` guard + `Error::TooManyEdges`.
- Integration tests for UC1 (corroboration) and UC2 (betrayal).
