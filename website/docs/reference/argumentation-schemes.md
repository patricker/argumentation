---
sidebar_position: 14
title: argumentation-schemes
---

The `argumentation-schemes` crate provides Walton's argumentation schemes (Walton, Reed & Macagno 2008): a catalogue of presumptive reasoning patterns — argument from expert opinion, ad hominem, argument from consequences, and more — each with named premise slots, a conclusion template, and critical questions that probe its weak points. Scheme instances compile to ASPIC+ premises and defeasible rules, which can then be evaluated via the `argumentation` crate's Dung semantics. The crate also includes AIF (Argument Interchange Format) serialisation and helpers for wiring schemes directly into `StructuredSystem`.

**Crate:** `argumentation-schemes` ([crates.io](https://crates.io/crates/argumentation-schemes) · [rustdoc](/api/argumentation_schemes/))

## Key types

### `SchemeSpec`
The compile-time definition of one argumentation scheme: a unique id, canonical name, premise slots, conclusion template, critical questions, and bibliographic metadata. Instantiate with concrete bindings via `SchemeSpec::instantiate(&bindings)` or the free function `instantiate(scheme, &bindings)`.
→ [Full docs](/api/argumentation_schemes/scheme/struct.SchemeSpec.html)

### `SchemeInstance`
The result of instantiating a `SchemeSpec` with concrete bindings. Holds the resolved premises (as `Literal` values), the resolved conclusion `Literal`, and the instantiated critical questions. Pass it to `add_scheme_to_system` to wire it into an ASPIC+ `StructuredSystem`.
→ [Full docs](/api/argumentation_schemes/instance/struct.SchemeInstance.html)

### `CatalogRegistry`
An in-memory collection of schemes indexed by id, snake_case key, and category. Obtain the full Walton 2008 catalogue via `default_catalog()` or `CatalogRegistry::with_walton_catalog()`. Use `by_key("argument_from_expert_opinion")` for lookup; keys are derived by lowercasing the scheme name and replacing spaces and hyphens with underscores.
→ [Full docs](/api/argumentation_schemes/registry/struct.CatalogRegistry.html)

### `PremiseSlot`
A named premise slot in a scheme definition. Has a `name` (used as the binding key), `description`, and `SlotRole` (Agent, Proposition, Domain, etc.).
→ [Full docs](/api/argumentation_schemes/scheme/struct.PremiseSlot.html)

### `ConclusionTemplate`
Template for a scheme's conclusion literal. Slot references use `?slot_name` syntax and are resolved at instantiation time. `is_negated` controls whether the conclusion literal is constructed as a negation (for rebuttal-concluding schemes like ad hominem).
→ [Full docs](/api/argumentation_schemes/scheme/struct.ConclusionTemplate.html)

### `CriticalQuestionInstance`
An instantiated critical question — a follow-up challenge a competent reasoner would raise. Holds the question text and the `Challenge` type (PremiseTruth, ExpertReliability, etc.).
→ [Full docs](/api/argumentation_schemes/instance/struct.CriticalQuestionInstance.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `catalog::default_catalog()` | `CatalogRegistry` | Returns a registry preloaded with the full Walton 2008 scheme catalogue. |
| `instantiate(scheme, &bindings)` | `Result<SchemeInstance>` | Free-function instantiation of a scheme with a `HashMap<String, String>` of bindings. Fails if any required slot is missing. |
| `SchemeSpec::instantiate(&self, &bindings)` | `Result<SchemeInstance>` | Convenience method delegating to the free function above. |
| `CatalogRegistry::by_key(&self, key)` | `Option<&SchemeSpec>` | Lookup by snake_case key (e.g., `"argument_from_expert_opinion"`). |
| `CatalogRegistry::by_category(&self, category)` | `Vec<&SchemeSpec>` | Filter schemes by `SchemeCategory` (Epistemic, Practical, Causal, etc.). |
| `aspic::add_scheme_to_system(&mut sys, &instance)` | `()` | Adds the scheme instance's premises and defeasible conclusion rule to a `StructuredSystem`. |
| `aspic::add_counter_argument(&mut sys, &instance, &target)` | `()` | Adds a scheme instance as a counter-argument (rebut) attacking `target` in a `StructuredSystem`. |
| `aif::instance_to_aif(&instance)` | `AifDocument` | Serialises a scheme instance to AIF (Argument Interchange Format). |
| `aif::aif_to_instance(&doc, &registry)` | `Result<SchemeInstance>` | Deserialises an AIF document back to a scheme instance, looking up the scheme in `registry`. |

## Errors

### `Error::MissingBinding { scheme, slot }`
A required premise slot had no binding provided at instantiation time.

### `Error::SchemeNotFound(String)`
A scheme key or id was not found in the registry.

### `Error::Aspic(String)`
An ASPIC+ structural error when adding a scheme to a `StructuredSystem`.

## See also

- [Walton schemes](/concepts/walton-schemes) — conceptual guide to scheme-based reasoning
- [Catalog authoring](/guides/catalog-authoring) — how to add custom schemes to the registry
- [East Wall example](/examples/east-wall) — end-to-end scene using `argument_from_expert_opinion`
- [encounter-argumentation reference](/reference/encounter-argumentation) — the bridge that wires schemes into encounter scenes
