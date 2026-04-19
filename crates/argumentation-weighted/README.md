# argumentation-weighted

Weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics. Built on the [`argumentation`](../..) crate.

## What's in the box

- `WeightedFramework<A>` with validated non-negative f64 attack weights.
- Exact Dunne 2011 β-inconsistent subset enumeration via `dunne_residuals`.
- Weighted extensions: grounded, complete, preferred, stable at any budget.
- Threshold-sweep API: acceptance trajectory, flip points, min-budget inverse query.
- `WeightSource` trait for pulling weights from external state.

## Quick example

```rust
use argumentation_weighted::framework::WeightedFramework;
use argumentation_weighted::sweep::min_budget_for_credulous;

let mut wf = WeightedFramework::new();
wf.add_weighted_attack("attacker", "target", 0.6).unwrap();

let min = min_budget_for_credulous(&wf, &"target").unwrap();
assert_eq!(min, Some(0.6));
```

## Semantics

Implements Dunne et al. 2011 inconsistency-budget semantics via exact
subset enumeration. For a budget `β`, a set `S` of attacks is
**β-inconsistent** iff `Σ w(a) ≤ β for a ∈ S`. An argument is
β-credulously accepted iff it belongs to some preferred extension of
`(AF \ S)` for some β-inconsistent `S`; β-skeptically accepted iff it
belongs to every preferred extension of every β-inconsistent `S`.

Monotonicity: credulous acceptance is monotone non-decreasing in β.

## Complexity

Exact enumeration is `O(2^m · f(n))` where `m` is the number of attacks
and `f(n)` is the Dung enumeration cost on the residual. The
`ATTACK_ENUMERATION_LIMIT` constant caps `m` at 24 (~16.8M subsets).
Larger frameworks return `Error::TooManyAttacks`.

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Dunne, Hunter, McBurney, Parsons & Wooldridge (2011). *Weighted argument systems: Basic definitions, algorithms, and complexity results.* AIJ 175(2).
