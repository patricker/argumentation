# Changelog

## [Unreleased]

## [0.2.0] - 2026-04-12

### Added
- **Premise-level preferences** (`StructuredSystem::prefer_premise`,
  `is_premise_preferred`, `premise_preferences`). Per M&P 2014 Def 3.21,
  falls through when both arguments' last-defeasible-rule frontiers are
  empty. Unlocks encoding the M&P running example.
- **Weakest-link defeat ordering** (`DefeatOrdering::{LastLink, WeakestLink}`
  + `StructuredSystem::with_ordering`). Per M&P 2014 Def 3.23, compares
  the full set of defeasible rules and ordinary premises. Default remains
  LastLink for backwards compatibility.
- **Rationality postulate checks**: `src/aspic/postulates.rs` + new
  `BuildOutput::check_postulates` method. Validates against Caminada-Amgoud
  (2007) postulates 1-4 (sub-argument closure, closure under strict rules,
  direct consistency, indirect consistency).
- **Labelling-primary methods**: `grounded_labelling`, `preferred_labellings`,
  `stable_labellings`, `ideal_labelling`, `semi_stable_labellings` as
  first-class accessors on `ArgumentationFramework<A>`.
- M&P 2014 running example (Example 3.7/3.22) and Whisky example
  (Example 3.26) encoded in `tests/ground_truth_aspic.rs`.

### Changed
- `BuildOutput` now carries a `rules: Vec<Rule>` field. Pre-1.0 additive
  breaking change for code that constructs `BuildOutput` values directly
  (rare; most consumers go through `build_framework`).
- `StructuredSystem` gains `premise_preferences` and `ordering` fields.
  `StructuredSystem::new()` still produces a system with default `LastLink`
  ordering and no premise preferences.

## [0.1.2] - 2026-04-12

### Added
- **Real ICCMA 2019 benchmark fixtures** under `tests/iccma_fixtures/real_2019/`.
  Five small instances (5-8 arguments) from the official
  `iccma-instances.tar.gz` archive plus expected outputs transcribed from the
  matching `reference-results.tar.gz`. A new test harness
  (`tests/ground_truth_iccma.rs`) runs each instance through all six
  semantics (grounded, complete, preferred, stable, ideal, semi-stable)
  and compares against third-party ground truth that we did not compute
  ourselves — 30 fixture/semantic test pairs. See
  `tests/iccma_fixtures/real_2019/PROVENANCE.md` for URLs and format notes.
- **`grounded ⊊ ideal` distinguisher fixture** in `tests/ground_truth_dung.rs`:
  a 5-argument framework (mutual attack `a↔b`, both attacking `c`, mutual
  attack `c↔d`, `d→e`) where grounded = ∅ but ideal = `{d}`. The existing
  invariant test only checks `grounded ⊆ ideal`; this fixture pins the
  strict inclusion that is the whole motivation for ideal semantics.
- **Caminada-style complete labelling ground truth** for the existing
  mutual-attack, three-cycle, and four-chain fixtures, covering the
  characteristic shapes: multiple in/out labellings with an all-undec
  labelling, the unique all-undec labelling on an odd cycle, and the
  unique alternating in/out labelling on a chain.

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
