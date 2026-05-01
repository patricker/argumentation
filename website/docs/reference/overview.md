---
sidebar_position: 1
title: Reference overview
---

Curated entry point into the argumentation workspace. For exhaustive API docs, see [rustdoc](/api/).

## Core types

### `EncounterArgumentationState`
The central state object for the encounter bridge. Composes schemes + bipolar + weighted.
→ [Full docs](/api/encounter_argumentation/state/struct.EncounterArgumentationState.html)

### `StateActionScorer<'a, S>`
Wraps an inner `ActionScorer` and boosts affordances whose argument is credulously accepted at current β.
→ [Full docs](/api/encounter_argumentation/state_scorer/struct.StateActionScorer.html)

### `StateAcceptanceEval<'a>`
Encounter's `AcceptanceEval<P>` impl backed by a live state reference. Rejects on credulously-accepted counter-arguments.
→ [Full docs](/api/encounter_argumentation/state_acceptance/struct.StateAcceptanceEval.html)

### `AffordanceKey`
Canonical `(actor, affordance_name, bindings)` triple used for forward-index lookup.
→ [Full docs](/api/encounter_argumentation/affordance_key/struct.AffordanceKey.html)

### `Budget`
A validated scene-intensity value in [0.0, 1.0]. Construct with `Budget::new(f64)`.
→ [Full docs](/api/argumentation_weighted/types/struct.Budget.html)

### `Scheme` + `SchemeInstance`
A Walton scheme template and its bound instantiation. Instantiate via `Scheme::instantiate(&bindings)`.
→ [Full docs](/api/argumentation_schemes/)

### `WeightedBipolarFramework<A>`
The underlying attack+support+weights graph. Usually accessed through `EncounterArgumentationState`.
→ [Full docs](/api/argumentation_weighted_bipolar/framework/struct.WeightedBipolarFramework.html)

### `Audience`
A strict partial order over values, represented as ranked tiers. `Audience::total([life, property])` for total orders; `Audience::from_tiers(...)` for intra-tier ties. Public `rank(&value)` for consumer code.
→ [Full docs](/api/argumentation_values/types/struct.Audience.html)

### `ValueAssignment<A>`
Maps arguments to the set of values they promote. Multi-value support via `SmallVec<[Value; 1]>` (Kaci & van der Torre 2008).
→ [Full docs](/api/argumentation_values/types/struct.ValueAssignment.html)

### `ValueBasedFramework<A>`
Dung framework + value assignment. `accepted_for(&audience, &arg)` for one audience; `subjectively_accepted` / `objectively_accepted` for queries over the audience space (capped at 6 values).
→ [Full docs](/api/argumentation_values/framework/struct.ValueBasedFramework.html)

### `ValueAwareScorer<S>` (encounter-argumentation)
Wraps any inner `ActionScorer`; reads per-actor audiences from `EncounterArgumentationState` and adds tier-rank-scaled boost to value-promoting affordances.
→ [Full docs](/api/encounter_argumentation/value_scorer/struct.ValueAwareScorer.html)

### `Error` (encounter-argumentation)
Error enum for the bridge. Variants include `MissingProposerBinding` — surfaces when an affordance has no `"self"` binding. Drained via `state.drain_errors()`.
→ [Full docs](/api/encounter_argumentation/error/enum.Error.html)

## Core methods

| Method | What it does |
|---|---|
| `EncounterArgumentationState::new(registry)` | Construct with a scheme catalog. |
| `set_intensity(&self, Budget)` | Set β through a shared reference. |
| `add_scheme_instance_for_affordance(...)` | Seed the forward index. Required before `resolve`. |
| `is_credulously_accepted(&id)` | Acceptance check at current β. |
| `has_accepted_counter_by(responder, &target)` | Per-responder attacker-credulity check. |
| `drain_errors()` | Drain the latched error buffer after resolve. |
| `state.set_audience(actor, audience)` | Set per-actor audience through `&self`. |
| `state.audience_for(actor)` | `Option<Audience>` — borrow per-actor audience. |
| `vaf.accepted_for(&audience, &arg)` | Audience-conditioned credulous acceptance. |

## Crate map

| Crate | Purpose |
|---|---|
| `argumentation` | Dung + ASPIC+ core. |
| `argumentation-bipolar` | Attacks + supports. |
| `argumentation-weighted` | Weighted edges + `Budget`. |
| `argumentation-weighted-bipolar` | Composition; β-residual semantics. |
| `argumentation-schemes` | Walton's 60+ schemes + catalog. |
| `argumentation-values` | Value-based AF (Bench-Capon 2003 + multi-value). |
| `encounter-argumentation` | The bridge crate. |

## What we don't have yet

The library focuses on Dung frameworks, ASPIC+, weighted attacks, bipolar extensions, the encounter bridge, and value-based argumentation. Four formalisms remain on the roadmap; see [open areas](/concepts/open-areas) for the public map (probabilistic AF, ADF, dialogue games, dynamic AF).

## See also

- [Full rustdoc](/api/)
- [Guides](/guides/installation) — how to use these types in practice.
- [Concepts](/concepts/what-is-argumentation) — why these types exist.
