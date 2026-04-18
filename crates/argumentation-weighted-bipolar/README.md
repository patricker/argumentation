# argumentation-weighted-bipolar

Weighted bipolar argumentation frameworks: a composition of
`argumentation-weighted` and `argumentation-bipolar` following Amgoud et
al. 2008, with Dunne 2011 inconsistency-budget semantics applied
uniformly over attacks and supports.

## What it is

A weighted bipolar framework carries two kinds of weighted edges over a
set of arguments: attacks and supports. Given a budget `β ≥ 0`, a
subset `S ⊆ attacks ∪ supports` is **β-inconsistent** iff its
cumulative weight is at most `β`. Acceptance queries iterate every
β-inconsistent subset, drop those edges from the framework, compute
bipolar-preferred extensions on the residual, and aggregate:

- **Credulous**: the argument is in some preferred extension of some residual.
- **Skeptical**: the argument is in every preferred extension of every residual.

## Why compose

`argumentation-bipolar` handles necessary-support semantics (Nouioua &
Risch 2011) by flattening + filtering. `argumentation-weighted` handles
Dunne 2011 over attack subsets. This crate glues them: residuals are
bipolar, not plain Dung, so the aggregation passes through bipolar
semantics instead of plain Dung.

## Example

```rust
use argumentation_weighted_bipolar::{WeightedBipolarFramework, is_credulously_accepted_at, Budget};

let mut wbf = WeightedBipolarFramework::new();
wbf.add_weighted_support("alice", "bob", 0.4).unwrap();
wbf.add_weighted_attack("charlie", "bob", 0.3).unwrap();

let accepted = is_credulously_accepted_at(&wbf, &"bob", Budget::new(0.5).unwrap()).unwrap();
```

## Complexity

Exact enumeration is `O(2^m · g(n))` where `m = |attacks| + |supports|`
and `g(n)` is the bipolar-preferred cost on the residual.
`EDGE_ENUMERATION_LIMIT = 24` caps `m` (~16.8M subsets). Larger
frameworks return `Error::TooManyEdges`.

## References

- Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., & Livet, P. (2008).
  *On bipolarity in argumentation frameworks.* IJIS 23(10).
- Dunne, P. E. et al. (2011). *Weighted argument systems.* AIJ 175(2).
- Nouioua, F. & Risch, V. (2011). *Bipolar argumentation frameworks
  with specialized supports.* ICTAI 2011.
