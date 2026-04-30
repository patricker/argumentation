# argumentation-values

Value-based argumentation frameworks (Bench-Capon 2003) for the `argumentation` Rust workspace.

```rust
use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

let mut base = ArgumentationFramework::new();
base.add_argument("h1");
base.add_argument("c1");
base.add_attack(&"h1", &"c1").unwrap();
base.add_attack(&"c1", &"h1").unwrap();

let mut values = ValueAssignment::new();
values.promote("h1", Value::new("life"));
values.promote("c1", Value::new("property"));

let vaf = ValueBasedFramework::new(base, values);
let life_audience = Audience::total([Value::new("life"), Value::new("property")]);

assert!(vaf.accepted_for(&life_audience, &"h1").unwrap());
assert!(!vaf.accepted_for(&life_audience, &"c1").unwrap());
```

See the [VAF concepts page](https://patricker.github.io/argumentation/concepts/value-based-argumentation) for full docs.

## License
MIT OR Apache-2.0.
