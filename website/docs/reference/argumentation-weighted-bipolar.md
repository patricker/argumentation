---
sidebar_position: 13
title: argumentation-weighted-bipolar
---

The `argumentation-weighted-bipolar` crate composes `argumentation-bipolar` and `argumentation-weighted` into a single graph type: `WeightedBipolarFramework<A>`. Every edge — whether an attack or a support — carries a non-negative finite weight. Under β-residual semantics, a budget β permits any subset of edges whose cumulative weight is at most β to be dropped (tolerated); acceptance queries iterate every such residual, run bipolar preferred-extension semantics on each, and aggregate credulously or skeptically. This is the primary graph type consumed by `encounter-argumentation`.

**Crate:** `argumentation-weighted-bipolar` ([crates.io](https://crates.io/crates/argumentation-weighted-bipolar) · [rustdoc](/api/argumentation_weighted_bipolar/))

## Key types

### `WeightedBipolarFramework<A>`
A weighted bipolar argumentation framework. Stores arguments, weighted attack edges, and weighted support edges. Both endpoint arguments are implicitly added on each `add_weighted_attack` / `add_weighted_support` call. Self-support is rejected; attack self-loops are permitted.
→ [Full docs](/api/argumentation_weighted_bipolar/framework/struct.WeightedBipolarFramework.html)

### `types::Budget`
Re-exported from `argumentation-weighted`. The inconsistency budget β — at β = 0 all edges bind; increasing β tolerates progressively more edges.
→ [Full docs](/api/argumentation_weighted_bipolar/types/struct.Budget.html)

### `types::AttackWeight`
Re-exported validated non-negative finite `f64` wrapper for attack edge weights.
→ [Full docs](/api/argumentation_weighted_bipolar/types/struct.AttackWeight.html)

### `types::WeightedAttack<A>` / `types::WeightedSupport<A>`
Directed weighted edge structs. Fields `attacker`/`supporter`, `target`/`supported`, and `weight` are public.
→ [Full docs](/api/argumentation_weighted_bipolar/types/)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `is_credulously_accepted_at(&wbf, arg, beta)` | `Result<bool>` | Returns true if `arg` is in some preferred extension of some β-residual (under bipolar semantics). |
| `is_skeptically_accepted_at(&wbf, arg, beta)` | `Result<bool>` | Returns true if `arg` is in every preferred extension of every β-residual. |
| `wbipolar_residuals(&wbf, beta)` | `Result<Vec<BipolarFramework<A>>>` | Enumerates all β-inconsistent residual bipolar frameworks. Each residual is a `BipolarFramework` with the tolerated edges removed. |
| `WeightedBipolarFramework::add_weighted_attack(&mut self, attacker, target, weight)` | `Result<()>` | Adds a weighted attack edge, validating the weight. Both endpoints are auto-added. |
| `WeightedBipolarFramework::add_weighted_support(&mut self, supporter, supported, weight)` | `Result<()>` | Adds a weighted support edge, validating the weight. Rejects self-support. |
| `WeightedBipolarFramework::argument_count(&self)` | `usize` | Number of argument nodes. |
| `WeightedBipolarFramework::edge_count(&self)` | `usize` | Total edge count (attacks + supports combined). |

## Errors

### `Error::InvalidWeight { weight }`
An edge weight was non-finite or negative.

### `Error::InvalidBudget { budget }`
A budget value was non-finite or negative.

### `Error::TooManyEdges { edges, limit }`
The framework has more total edges (attacks + supports) than `EDGE_ENUMERATION_LIMIT` allows. The exact semantics enumerate 2^(edge_count) residuals.

### `Error::IllegalSelfSupport`
A support edge was added from an argument to itself.

### `Error::Bipolar(argumentation_bipolar::Error)`
An error propagated from the bipolar layer during residual semantic evaluation.

### `Error::Dung(argumentation::Error)`
An error propagated from the core Dung layer.

## See also

- [Attacks and supports](/concepts/attacks-and-supports) — bipolar framework concepts
- [Weighted frameworks and β](/concepts/weighted-and-beta) — inconsistency-budget model
- [argumentation-bipolar reference](/reference/argumentation-bipolar) — unweighted bipolar layer
- [argumentation-weighted reference](/reference/argumentation-weighted) — unipolar weighted layer
- [encounter-argumentation reference](/reference/encounter-argumentation) — the bridge that wraps this crate
