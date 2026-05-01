---
sidebar_position: 16
title: argumentation-values
---

The `argumentation-values` crate (v0.1.0) implements **value-based argumentation frameworks** (Bench-Capon 2003) extended with multi-value support per Kaci & van der Torre (2008). It adds `Value`, `ValueAssignment<A>`, `Audience`, and `ValueBasedFramework<A>` on top of the core `argumentation` crate, plus subjective/objective acceptance, a Walton-scheme→audience bridge, APX format I/O for ASPARTIX interop, and `MultiAudience` consensus queries for multi-character scenes.

**Crate:** `argumentation-values` ([crates.io](https://crates.io/crates/argumentation-values) · [rustdoc](/api/argumentation_values/))

## Key types

### `Value`
A newtype around `String` representing a value an argument can promote (e.g., `Value::new("life")`). Implements `Display`, `From<&str>`, `From<String>`, `Hash`, `Ord`.
→ [Full docs](/api/argumentation_values/types/struct.Value.html)

### `ValueAssignment<A>`
Maps each argument (of label type `A`) to the *set* of values it promotes via `SmallVec<[Value; 1]>` — the single-value common case is allocation-free; multi-value is supported per Kaci & van der Torre 2008. Empty set means "promotes no value" (defeats unconditionally under VAF semantics).
→ [Full docs](/api/argumentation_values/types/struct.ValueAssignment.html)

### `Audience`
A strict partial order over values, represented as ranked tiers. `Audience::total([life, property])` produces a total order; `Audience::from_tiers(vec![vec![life, liberty], vec![property]])` allows intra-tier ties. Public `rank(&value) -> Option<usize>` for consumer code.
→ [Full docs](/api/argumentation_values/types/struct.Audience.html)

### `ValueBasedFramework<A>`
A Dung framework (`ArgumentationFramework<A>`) plus a `ValueAssignment<A>`. Audience-conditioned acceptance via `defeat_graph(audience)`, `accepted_for(audience, arg)`, `grounded_for(audience)`. Pareto-defeating rule: A defeats B iff for every value B promotes, some value A promotes is not strictly less-preferred under the audience.
→ [Full docs](/api/argumentation_values/framework/struct.ValueBasedFramework.html)

### `MultiAudience`
Multi-character consensus queries. `MultiAudience::new(&[alice_audience, bob_audience])` then `common_grounded(&vaf)` returns arguments grounded under *every* audience — the council-style consensus answer.
→ [Full docs](/api/argumentation_values/multi/struct.MultiAudience.html)

### `Error`
Error enum: `Dung(#[from] argumentation::Error)`, `AudienceTooLarge { values, limit }` (returned past 6 distinct values for subjective/objective acceptance), `ArgumentNotFound(String)`, `ApxParse { line, reason }`.
→ [Full docs](/api/argumentation_values/error/enum.Error.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `ValueBasedFramework::new(base, values)` | `Self` | Construct from a Dung framework and a value assignment. |
| `vaf.defeat_graph(&audience)` | `Result<ArgumentationFramework<A>, Error>` | Build the audience-conditioned defeat graph. |
| `vaf.defeats(&attacker, &target, &audience)` | `bool` | Pareto-defeating rule check. |
| `vaf.accepted_for(&audience, &arg)` | `Result<bool, Error>` | Credulous acceptance under one audience (preferred-extension membership). |
| `vaf.grounded_for(&audience)` | `Result<HashSet<A>, Error>` | Grounded extension under one audience (delegates to upstream `grounded_extension()`). |
| `vaf.subjectively_accepted(&arg)` | `Result<bool, Error>` | Accepted by *some* audience (NP-complete; capped at 6 values). |
| `vaf.objectively_accepted(&arg)` | `Result<bool, Error>` | Accepted by *every* audience (co-NP-complete; capped at 6 values). |
| `MultiAudience::new(&audiences)` | `Self` | Construct from a slice of audiences. |
| `multi.common_grounded(&vaf)` | `Result<HashSet<A>, Error>` | Intersection of grounded extensions across all audiences. |
| `multi.common_credulous(&vaf)` | `Result<HashSet<A>, Error>` | Intersection of credulous-acceptance sets across all audiences. |
| `from_scheme_instances(instances, to_arg)` | `ValueAssignment<A>` | Extract `value` bindings from `argument_from_values` Walton-scheme instances. |
| `apx::from_apx(input)` | `Result<(ValueBasedFramework<String>, Audience), Error>` | Parse ASPARTIX-compatible APX text. |
| `apx::to_apx(&vaf, &audience)` | `String` | Serialise to APX. |

## Constants

### `acceptance::ENUMERATION_LIMIT`
The hard cap (currently `6`) on distinct values for subjective/objective acceptance. Past this, methods return `Error::AudienceTooLarge` per Dunne & Bench-Capon (2004) complexity bounds.

### `scheme_bridge::DEFAULT_VALUES_SCHEME_NAME`
The default-catalog name (`"Argument from Values"`) compared against `SchemeInstance.scheme_name`. Use `from_scheme_instances_with_name` to target a custom scheme name.

## Errors

### `Error::Dung(argumentation::Error)`
Wrapped error from underlying Dung framework operations (e.g., framework-too-large for subset enumeration).

### `Error::AudienceTooLarge { values, limit }`
`subjectively_accepted`/`objectively_accepted` bail out when distinct values exceed `ENUMERATION_LIMIT`. Use a fixed-audience query (`accepted_for`) instead.

### `Error::ArgumentNotFound(String)`
An argument referenced in an attack edge is not registered in the underlying framework.

### `Error::ApxParse { line, reason }`
APX text input failed to parse. `line` is 1-indexed.

## See also

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — formalism and design rationale.
- [Hal & Carla](/examples/hal-and-carla) — the engine-driven scene this implementation was built around.
- [Wiring per-character values](/guides/wiring-character-values) — how-to for encounter bridge integration.
- [Import/export APX](/guides/import-export-apx) — how-to for ASPARTIX interop.
- [Multi-character consensus](/guides/multi-character-consensus) — how-to for `MultiAudience` queries.
- [Reference overview](/reference/overview) — workspace-level types overview.
