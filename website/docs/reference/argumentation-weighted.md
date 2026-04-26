---
sidebar_position: 12
title: argumentation-weighted
---

The `argumentation-weighted` crate adds real-valued weights to attack edges and introduces a `Budget` β for inconsistency tolerance, following the weighted argument systems of Dunne, Hunter, McBurney, Parsons & Wooldridge 2011. A budget β permits any subset of attacks whose cumulative weight is at most β to be tolerated (dropped) when computing Dung extensions. At β = 0 the semantics reduce to standard Dung; increasing β progressively accepts more arguments by tolerating more attacks. The crate also provides sweep utilities for finding the minimum β at which an argument becomes accepted, and the `WeightSource<A>` trait for plugging in external weight computation.

**Crate:** `argumentation-weighted` ([crates.io](https://crates.io/crates/argumentation-weighted) · [rustdoc](/api/argumentation_weighted/))

## Key types

### `WeightedFramework<A>`
A weighted argumentation framework. Stores arguments and weighted directed attack edges (no support edges — see `argumentation-weighted-bipolar` for that). Accepts only non-negative finite weights.
→ [Full docs](/api/argumentation_weighted/framework/struct.WeightedFramework.html)

### `types::Budget`
A validated non-negative finite `f64` wrapper representing the inconsistency budget β. Construct with `Budget::new(value)` (rejects NaN, infinity, negative) or `Budget::zero()` for standard Dung semantics.
→ [Full docs](/api/argumentation_weighted/types/struct.Budget.html)

### `types::AttackWeight`
A validated non-negative finite `f64` wrapper for individual attack weights. Higher weight = harder to tolerate.
→ [Full docs](/api/argumentation_weighted/types/struct.AttackWeight.html)

### `types::WeightedAttack<A>`
A directed attack edge carrying a validated `AttackWeight`. Fields `attacker`, `target`, and `weight` are public.
→ [Full docs](/api/argumentation_weighted/types/struct.WeightedAttack.html)

### `weight_source::WeightSource<A>` (trait)
A pluggable interface for computing attack weights from external context (e.g., relationship strengths, domain models). Implement this trait to wire in a custom weight source; use `populate_from_source` to apply it to a framework.
→ [Full docs](/api/argumentation_weighted/weight_source/trait.WeightSource.html)

### `weight_source::ClosureWeightSource<A>`
A simple `WeightSource` implementation backed by a closure. Useful for tests and one-off weight mappings.
→ [Full docs](/api/argumentation_weighted/weight_source/struct.ClosureWeightSource.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `is_credulously_accepted_at(&wf, arg, beta)` | `Result<bool>` | Returns true if `arg` is in some preferred extension of some β-inconsistent residual. |
| `is_skeptically_accepted_at(&wf, arg, beta)` | `Result<bool>` | Returns true if `arg` is in every preferred extension of every β-inconsistent residual. |
| `preferred_at_budget(&wf, beta)` | `Result<Vec<HashSet<A>>>` | Enumerates preferred extensions across all β-inconsistent residuals. |
| `grounded_at_budget(&wf, beta)` | `Result<Vec<HashSet<A>>>` | Grounded extensions of all β-inconsistent residuals. |
| `sweep::min_budget_for_credulous(&wf, arg)` | `Result<Option<f64>>` | Finds the minimum β at which `arg` becomes credulously accepted. Returns `None` if `arg` is never accepted regardless of budget. |
| `sweep::acceptance_trajectory(&wf, arg)` | `Result<Vec<SweepPoint>>` | Samples credulous acceptance across the full weight range, returning (β, accepted) pairs. |
| `sweep::flip_points(&wf, arg)` | `Result<Vec<f64>>` | Returns the β values at which `arg`'s credulous acceptance status flips. Each flip point equals an attack weight. |
| `reduce::dunne_residuals(&wf, beta)` | `Result<Vec<ArgumentationFramework<A>>>` | Enumerates all β-inconsistent residual frameworks (subsets of attacks with cumulative weight ≤ β dropped). |
| `weight_source::populate_from_source(&mut wf, source)` | `Result<()>` | Walks the framework's attacks and fills weights from the provided `WeightSource`. |

## Errors

### `Error::InvalidWeight { weight }`
An attack weight was non-finite (NaN or infinity) or negative. Dunne 2011 requires non-negative finite weights.

### `Error::InvalidBudget { budget }`
A budget value was non-finite or negative.

### `Error::TooManyAttacks { attacks, limit }`
The framework exceeded `ATTACK_ENUMERATION_LIMIT` attacks. The exact semantics enumerate the power set of attacks (O(2^m)); this limit caps memory and time.

### `Error::Dung(argumentation::Error)`
An error propagated from the underlying Dung layer.

## See also

- [Weighted frameworks and β](/concepts/weighted-and-beta) — conceptual explanation of the inconsistency-budget model
- [Tune β](/guides/tuning-beta) — practical guidance on picking a scene-appropriate β
- [argumentation-weighted-bipolar reference](/reference/argumentation-weighted-bipolar) — adds support edges to this model
- [encounter-argumentation reference](/reference/encounter-argumentation) — uses `Budget` as the scene-intensity knob
