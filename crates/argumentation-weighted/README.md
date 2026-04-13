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

## Known limitation — non-monotonicity

v0.1.0 ships a practical approximation of Dunne 2011's inconsistency-budget semantics: the cheapest attacks are tolerated first, in ascending weight order, until the budget would be exceeded. This approximation is **not globally monotone** in β. A chained-defense framework like `a→b (0.4), b→c (0.6)` flips `c`'s acceptance from true (at β=0, defended by `a`) to false (once `a→b` is tolerated and `b` starts attacking `c` unopposed) to true again (once `b→c` is also tolerated). The witness fixture lives at `tests/uc3_scene_intensity.rs`.

`min_budget_for_credulous` returns the *first* budget at which the target is accepted — not a stable threshold. Use `acceptance_trajectory` if you need the full picture. The full Dunne 2011 existential-subset semantics would be monotone but requires enumeration over 2^|attacks| subsets; that's a deferred v0.2.0 target.

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Dunne, Hunter, McBurney, Parsons & Wooldridge (2011). *Weighted argument systems: Basic definitions, algorithms, and complexity results.* AIJ 175(2).
