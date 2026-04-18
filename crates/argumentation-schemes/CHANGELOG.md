# Changelog

## [0.2.0] - 2026-04-17

### Added
- `aif` module providing AIF (AIFdb JSON) round-trip:
  - `AifDocument`, `AifNode`, `AifEdge` serde data model.
  - `instance_to_aif(&SchemeInstance) -> AifDocument` export.
  - `aif_to_instance(&AifDocument, &CatalogRegistry) -> SchemeInstance`
    import.
  - `AifDocument::from_json` / `to_json` string helpers.
- `CatalogRegistry::with_default()` and `CatalogRegistry::by_name()`
  convenience methods for AIF import use.
- `Error::AifParse` and `Error::AifUnknownScheme` variants.

### Dependencies
- `serde` 1.0 (with derive) — new.
- `serde_json` 1.0 — new.

### Notes
- Critical-question `Challenge` tags and `counter_literal` values are
  not part of the AIF format and are re-derived on import from the
  catalog's scheme definition.

## [0.1.0] - 2026-04-12

### Added
- Core types: `SchemeSpec`, `PremiseSlot`, `ConclusionTemplate` (with
  `is_negated` flag), `CriticalQuestion`, `SchemeInstance`, `CatalogRegistry`.
- Prefix-safe template resolution: `resolve_template` sorts bindings by
  key length descending so that overlapping slot names like `threatener`
  and `threat` substitute correctly.
- 25 built-in Walton schemes across 6 categories:
  - Epistemic (3): expert opinion, witness testimony, position to know
  - Practical (7): positive/negative consequences, values, threat,
    fear appeal, waste, sunk cost
  - Source-based (4): ad hominem, ad hominem circumstantial, bias, ethotic
  - Popular (4): popular opinion, tradition, precedent, established rule
  - Causal (4): cause-to-effect, correlation, sign, slippery slope
  - Analogical (3): analogy, verbal classification, commitment
- Per-category ID offset constants prevent cross-category collisions.
- ASPIC+ integration: `add_scheme_to_system`, `add_counter_argument`.
- `SchemeSpec::instantiate` convenience method.
- 40+ unit tests across modules, plus 4 integration test suites covering
  UC1 (instantiation), UC2 (scheme conflict), UC3 (catalog coverage),
  UC4 (critical questions).

### Known limitations
- Scheme instantiation produces synthetic per-instance literals
  (e.g., `expert_alice`) that encode "this slot is filled by this value
  in this scheme instance" rather than world-fact propositions. This
  blocks multi-instance fact sharing; a `WorldFact` layer is planned for
  v0.2.0.
