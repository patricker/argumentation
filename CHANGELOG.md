# Changelog

## [Unreleased]

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
