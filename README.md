# argumentation

Formal argumentation in Rust: Dung abstract argumentation frameworks (1995) and ASPIC+ structured argumentation (Modgil & Prakken 2014).

[![Crates.io](https://img.shields.io/crates/v/argumentation.svg)](https://crates.io/crates/argumentation)
[![Docs.rs](https://docs.rs/argumentation/badge.svg)](https://docs.rs/argumentation)

## What it is

A pure-Rust implementation of two canonical argumentation frameworks from the AI literature:

1. **Dung 1995 abstract argumentation** — arguments are opaque nodes, attacks are directed edges, and the library computes grounded, complete, preferred, stable, ideal, and semi-stable extensions, plus Caminada three-valued labellings.
2. **ASPIC+ structured argumentation** — arguments are trees built from a knowledge base and strict/defeasible rules, with preference-based defeat resolution. The ASPIC+ layer automatically emits an abstract AF that the Dung layer evaluates.

Both layers are independently usable. Ships with parsers for the ICCMA APX and TGF benchmark formats.

## What it isn't

- Not a natural-language argument miner.
- Not an LLM-powered debater.
- Not an ICCMA-competition-winning solver (subset enumeration, not SAT-based — see Performance).
- Not a theorem prover.

## Quick start

```rust
use argumentation::ArgumentationFramework;

let mut af = ArgumentationFramework::new();
af.add_argument("a");
af.add_argument("b");
af.add_argument("c");
af.add_attack(&"a", &"b").unwrap();
af.add_attack(&"b", &"c").unwrap();

let grounded = af.grounded_extension();
assert!(grounded.contains(&"a") && grounded.contains(&"c"));
```

See `src/lib.rs` for the ASPIC+ quick example.

## Performance

Current implementation uses subset-enumeration for extension semantics, which is exponential in the number of arguments. Practical up to ~20 arguments. Larger instances will need SAT/ASP solvers in a future version.

## References

- Dung, P.M. (1995). [*On the acceptability of arguments...*](https://www.cs.utexas.edu/~mooney/cs395t/papers/dung1995.pdf) AIJ 77(2).
- Modgil, S. & Prakken, H. (2014). [*The ASPIC+ framework for structured argumentation.*](https://www.tandfonline.com/doi/abs/10.1080/19462166.2013.869766) Argument & Computation 5(1).
- Baroni, P., Caminada, M., Giacomin, M. (2011). *An introduction to argumentation semantics.* KER 26(4).
- [ICCMA](http://argumentationcompetition.org/) benchmark instances.

## License

Dual-licensed under MIT or Apache-2.0 at your option.
