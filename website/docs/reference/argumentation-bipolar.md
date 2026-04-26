---
sidebar_position: 11
title: argumentation-bipolar
---

The `argumentation-bipolar` crate extends Dung's attack-only model with a second directed edge relation: **support**. It implements *necessary support* semantics (Cayrol & Lagasquie-Schiex 2005, Nouioua & Risch 2011): if `A` supports `B`, then `A` must be accepted for `B` to be acceptable. The crate builds on top of `argumentation`'s Dung semantics by flattening bipolar frameworks — computing the closed attack relation and filtering extensions that are not support-closed — and provides SCC-based coalition detection over the support graph.

**Crate:** `argumentation-bipolar` ([crates.io](https://crates.io/crates/argumentation-bipolar) · [rustdoc](/api/argumentation_bipolar/))

## Key types

### `BipolarFramework<A>`
A bipolar argumentation framework generic over argument type `A`. Stores arguments and two independent directed edge sets (attacks and supports). Self-support is rejected; self-attack is allowed (matches the core crate's convention). Provides `add_attack`, `add_support`, `remove_support`, `direct_attackers`, `direct_supporters`, and `supporter_map`.
→ [Full docs](/api/argumentation_bipolar/framework/struct.BipolarFramework.html)

### `Coalition<A>`
A strongly-connected component of the support graph — a group of arguments that mutually support each other. Returned by `detect_coalitions`.
→ [Full docs](/api/argumentation_bipolar/coalition/struct.Coalition.html)

### `EdgeKind`
Discriminates attack edges from support edges. Used in derived-relation and flatten queries.
→ [Full docs](/api/argumentation_bipolar/types/enum.EdgeKind.html)

### `CoalitionId`
An opaque identifier for a detected coalition, stable for the lifetime of a `detect_coalitions` call.
→ [Full docs](/api/argumentation_bipolar/types/struct.CoalitionId.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `detect_coalitions(&bf)` | `Vec<Coalition<A>>` | Runs Tarjan SCC on the support graph to find mutually-supporting groups. |
| `bipolar_preferred_extensions(&bf)` | `Result<Vec<HashSet<A>>>` | Computes preferred extensions under necessary-support semantics: flattens the closed attack relation, runs Dung preferred semantics, then filters non-support-closed extensions. |
| `bipolar_grounded_extension(&bf)` | `Result<HashSet<A>>` | Computes the grounded extension of the flattened framework, then removes arguments whose necessary supporters are absent. |
| `bipolar_stable_extensions(&bf)` | `Result<Vec<HashSet<A>>>` | Stable extensions of the flattened framework, filtered for support-closure. |
| `bipolar_complete_extensions(&bf)` | `Result<Vec<HashSet<A>>>` | Complete extensions of the flattened framework, filtered for support-closure. |
| `is_support_closed(ext, bf)` | `bool` | Returns true iff every argument in `ext` has all its necessary supporters also in `ext`. |
| `derived::closed_attacks(&bf)` | `HashSet<(A, A)>` | Computes the full closed attack relation (direct + supported + secondary + mediated) per C&LS 2005 §3. |
| `flatten::flatten(&bf)` | `ArgumentationFramework<A>` | Produces an equivalent plain Dung AF whose edges are the closed attack relation. |

## Errors

### `Error::ArgumentNotFound(String)`
An operation referenced an argument not in the framework.

### `Error::IllegalSelfSupport(String)`
`add_support` was called with identical supporter and supported arguments. Self-support is not valid under necessary-support semantics.

### `Error::Dung(argumentation::Error)`
An error propagated from the underlying Dung layer (e.g., framework too large for subset enumeration).

## See also

- [Attacks and supports](/concepts/attacks-and-supports) — bipolar framework concepts
- [argumentation crate reference](/reference/argumentation) — the Dung layer this crate builds on
- [argumentation-weighted-bipolar reference](/reference/argumentation-weighted-bipolar) — adds weights to both attacks and supports
