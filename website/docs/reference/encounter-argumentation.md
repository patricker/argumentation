---
sidebar_position: 15
title: encounter-argumentation
---

The `encounter-argumentation` crate (v0.5.0) bridges the `encounter` scene engine and the argumentation framework. It provides `EncounterArgumentationState` — a single object that composes scheme reasoning, bipolar graph structure, weighted attack strengths, and a tunable scene-intensity budget (β) — along with two trait implementations (`StateAcceptanceEval` and `StateActionScorer`) that plug directly into encounter's `AcceptanceEval` and `ActionScorer` hooks. As of v0.5.0, `SocietasRelationshipSource` and `NameResolver` have moved to the `societas-encounter` crate (enable the `argumentation` feature there to get relationship-modulated attack weights).

**Crate:** `encounter-argumentation` ([crates.io](https://crates.io/crates/encounter-argumentation) · [rustdoc](/api/encounter_argumentation/))

## Key types

### `EncounterArgumentationState`
The central encounter-level state object. Wraps a `WeightedBipolarFramework<ArgumentId>` internally and adds actor attribution, affordance indexing, error latching, and β management. Construct with `EncounterArgumentationState::new(registry)`, configure β via `.at_intensity(budget)` (builder) or `.set_intensity(budget)` (through `&self`). The state is `Send + Sync` — required by encounter's trait bounds.
→ [Full docs](/api/encounter_argumentation/state/struct.EncounterArgumentationState.html)

### `StateAcceptanceEval`
Implements encounter's `AcceptanceEval<P>` trait. For each affordance proposed by an actor, it evaluates whether any other actor has a credulously accepted counter-argument at the current scene β. Cannot propagate `Result`; records errors in the state's latch — always call `state.drain_errors()` after encounter's `resolve` returns.
→ [Full docs](/api/encounter_argumentation/state_acceptance/struct.StateAcceptanceEval.html)

### `StateActionScorer`
Implements encounter's `ActionScorer<P>` trait. Scores proposed affordances by the acceptance strength of the proposer's argument under the current scene β. Like `StateAcceptanceEval`, it latches errors rather than propagating them.
→ [Full docs](/api/encounter_argumentation/state_scorer/struct.StateActionScorer.html)

### `AffordanceKey`
Canonical identifier for an `(actor, affordance_name, bindings)` triple. Bindings are normalised into a `BTreeMap` internally so the key's hash is insertion-order-independent. Used to look up which `ArgumentId` was seeded for a given affordance via `state.argument_id_for(&key)`.
→ [Full docs](/api/encounter_argumentation/affordance_key/struct.AffordanceKey.html)

### `ArgumentId`
An opaque string-backed identifier for an argument node in the framework. Derived automatically from a scheme instance's conclusion literal; can also be constructed directly with `ArgumentId::new("name")`.
→ [Full docs](/api/encounter_argumentation/arg_id/struct.ArgumentId.html)

### `ArgumentKnowledge` / `StaticKnowledge`
Knowledge base types for seeding the state with pre-known argument positions before a scene starts. `StaticKnowledge` holds a fixed set of `ArgumentKnowledge` entries.
→ [Full docs](/api/encounter_argumentation/knowledge/)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `EncounterArgumentationState::new(registry)` | `Self` | Creates a new state with the given scheme catalogue and β = 0. |
| `state.at_intensity(budget)` | `Self` | Builder: sets scene β, returns the state by value for chaining. |
| `state.set_intensity(budget)` | `()` | Mutates β through `&self` (uses internal `Mutex`). |
| `state.add_scheme_instance(actor, instance)` | `ArgumentId` | Adds a scheme instance asserted by `actor`; returns the argument's id. |
| `state.add_scheme_instance_for_affordance(actor, affordance_name, &bindings, instance)` | `ArgumentId` | As above, plus indexes the argument under an `AffordanceKey` for later lookup. |
| `state.add_weighted_attack(&attacker, &target, weight)` | `Result<()>` | Adds a weighted attack edge to the framework. |
| `state.add_weighted_support(&supporter, &supported, weight)` | `Result<()>` | Adds a weighted support edge (rejects self-support). |
| `state.is_credulously_accepted(&arg)` | `Result<bool>` | Credulous acceptance at current β via bipolar-weighted semantics. |
| `state.is_skeptically_accepted(&arg)` | `Result<bool>` | Skeptical acceptance at current β. |
| `state.has_accepted_counter_by(responder, &target)` | `Result<bool>` | True iff `responder` has a credulously accepted argument that directly attacks `target`. |
| `state.attackers_of(&target)` | `Vec<ArgumentId>` | Direct attackers of `target` in the current framework (structural query, β-independent). |
| `state.actors_by_argument()` | `&HashMap<ArgumentId, Vec<String>>` | Full actor attribution map; used by weight sources to resolve argument nodes back to actor names. |
| `state.coalitions()` | `Result<Vec<Coalition<ArgumentId>>>` | SCC-based coalition detection on the support graph. |
| `state.drain_errors()` | `Vec<Error>` | Drains the error latch. Always call this after encounter's `resolve` returns. |

## Errors

### `Error::SchemeNotFound(String)`
A requested scheme key was not found in the registry.

### `Error::MissingBinding { scheme, slot }`
A required slot binding was missing during scheme instantiation.

### `Error::MissingProposerBinding { affordance_name }`
`StateAcceptanceEval` could not find a `"self"` binding on an affordance, so it could not identify the proposer. The eval defaulted to *accept* and latched this error. Surfaces via `drain_errors`. Consumers using a non-`"self"` proposer slot name should wrap `StateAcceptanceEval` with a custom implementation.

### `Error::Scheme(argumentation_schemes::Error)`
Propagated from the schemes layer.

### `Error::Dung(argumentation::Error)`
Propagated from the core Dung layer.

### `Error::Bipolar(argumentation_bipolar::Error)`
Propagated from the bipolar layer.

### `Error::Weighted(argumentation_weighted::Error)`
Propagated from the weighted layer.

### `Error::WeightedBipolar(argumentation_weighted_bipolar::Error)`
Propagated from the weighted-bipolar layer (e.g., too many edges for enumeration, self-support).

## See also

- [Encounter integration](/concepts/encounter-integration) — trait-impl architecture and scene lifecycle
- [Implementing an acceptance eval](/guides/implementing-acceptance-eval) — how to use `StateAcceptanceEval`
- [Implementing an action scorer](/guides/implementing-action-scorer) — how to use `StateActionScorer`
- [Societas-modulated weights](/guides/societas-modulated-weights) — relationship-driven attack weights via `societas-encounter`
- [Tune β](/guides/tuning-beta) — picking a scene-appropriate intensity budget
- [Debug acceptance](/guides/debugging-acceptance) — diagnosing unexpected verdicts
- [argumentation-weighted-bipolar reference](/reference/argumentation-weighted-bipolar) — the underlying graph crate
