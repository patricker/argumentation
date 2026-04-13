# argumentation-weighted

Weighted argumentation frameworks with Dunne et al. 2011 inconsistency-budget semantics. Built on the [`argumentation`](../..) crate.

## What's in the box

- `WeightedFramework<A>` with validated non-negative f64 attack weights.
- β-reduction: produce an unweighted Dung framework at a given budget.
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

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Dunne, Hunter, McBurney, Parsons & Wooldridge (2011). *Weighted argument systems: Basic definitions, algorithms, and complexity results.* AIJ 175(2).
