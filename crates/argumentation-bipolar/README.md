# argumentation-bipolar

Bipolar argumentation frameworks (attacks + supports) built on the [`argumentation`](../..) crate. Implements necessary-support semantics per Nouioua & Risch 2011 with derived attack closure per Cayrol & Lagasquie-Schiex 2005 / Amgoud et al. 2008.

## What's in the box

- `BipolarFramework<A>` with independent attack and support edge sets.
- Derived attack closure (supported, secondary, mediated rules).
- Flattening to a Dung `ArgumentationFramework` for reuse of the core crate's semantics.
- Necessary-support semantics: grounded, complete, preferred, stable extensions filtered for support-closure.
- Coalition detection via Tarjan SCC on the support graph.
- Transitive query helpers (supporters, attackers, coalition membership).

## Quick example

```rust
use argumentation_bipolar::framework::BipolarFramework;
use argumentation_bipolar::coalition::detect_coalitions;

let mut bf = BipolarFramework::new();
bf.add_support("alice", "bob").unwrap();
bf.add_support("bob", "alice").unwrap();
bf.add_attack("alice", "charlie");
bf.add_attack("bob", "charlie");

let coalitions = detect_coalitions(&bf);
assert!(coalitions.iter().any(|c| c.members.len() == 2));
```

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Cayrol & Lagasquie-Schiex (2005). *On the acceptability of arguments in bipolar argumentation frameworks.* IJAR 23(4).
- Amgoud, Cayrol, Lagasquie-Schiex & Livet (2008). *On bipolarity in argumentation frameworks.* IJIS 23(10).
- Nouioua & Risch (2011). *Bipolar argumentation frameworks with specialized supports.* ICTAI 2011.
