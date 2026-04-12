# Changelog

## [Unreleased]

## [0.1.1] - 2026-04-12

### Fixed
- **ASPIC+ strict-wrap rebut propagation**: `compute_attacks` now propagates
  rebut attacks from defeasible sub-arguments to their strict-topped parents
  (M&P 2014 §3.3.1 Def 3.10). Before this fix, frameworks using strict rules
  to wrap defeasible conclusions (e.g. the classical Married/Bachelor example)
  produced logically incoherent extensions. See commit `e387259`.
- TGF parser now accepts `%`-prefixed comment lines to match APX.

### Added
- `StructuredSystem` exposed at the crate root via `pub use aspic::StructuredSystem`.
- `pub const ENUMERATION_LIMIT: usize` at the crate root (was `pub(crate)`).
- `Literal::atom` / `Literal::neg` reject the reserved `__applicable_` prefix
  via `debug_assert!`; `Literal::undercut_marker` is the single sanctioned
  constructor for the reserved namespace.
- `prefer_rule` now returns `Result` and rejects reflexive (`a > a`) and
  cyclic (`a > b > a`) preferences at insertion time.
- `BuildOutput::conclusions_in`, `argument_by_conclusion`, `arguments_with_conclusion`
  convenience accessors for mapping `ArgumentId` back to user-visible literals.
- `Display` impls for `ArgumentId`, `RuleId`, `Label`, and `AttackKind`.
- Cross-semantics stress test suite (`tests/stress.rs`): 9 invariants at
  2000 proptest cases each.

### Changed
- `ENUMERATION_LIMIT` lowered from 30 to 22 to match the "practical up to
  ~20 arguments" claim in the crate-level docs. The hard limit and the
  documented recommendation now agree.
- `compute_attacks` precomputes per-target caches once outside the outer
  loop, reducing complexity from `O(n³ × depth)` to `O(n² + n × depth)`.
- `tests/iccma_benchmarks.rs` renamed to `tests/fixture_format_smoke.rs`
  to reflect what it actually tests.

### Breaking changes (pre-1.0)
- `StructuredSystem::prefer_rule` signature changed from `fn(&mut self, RuleId, RuleId)`
  to `fn(&mut self, RuleId, RuleId) -> Result<(), Error>`. Existing callers
  must add `.unwrap()` or propagate the `Result`.

## [0.1.0] - 2026-04-11

### Added
- `ArgumentationFramework<A>` with Dung 1995 semantics:
  grounded, complete, preferred, stable, ideal, semi-stable extensions.
- Caminada three-valued labellings and extension↔labelling conversions.
- ICCMA APX and TGF parsers.
- Integration tests against fixture instances.
- Property-based universal invariants via proptest.
- ASPIC+ structured argumentation: `StructuredSystem`, propositional language,
  knowledge base (necessary/ordinary premises), strict and defeasible rules,
  argument construction via forward chaining, undercut/undermine/rebut attack
  detection, last-link defeat resolution with Elitist preference comparison,
  automatic AF generation. Cyclic rule sets are rejected up front.
- Worked example: the classical penguin-does-not-fly scenario.

### Known limitations
- Extension enumeration is subset-based (exponential). Use for frameworks up to ~20 arguments.
- ASPIC+ Elitist ordering only; Democratic and weakest-link deferred to future versions.
- No SAT/ASP back-end.
