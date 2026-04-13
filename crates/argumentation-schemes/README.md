# argumentation-schemes

Walton argumentation schemes with critical questions for structured social
reasoning. Builds on the [`argumentation`](../..) crate's ASPIC+ layer.

## What's in the box

- 25 built-in Walton schemes across 6 categories: epistemic, practical,
  source-based, popular, causal, analogical.
- A `CatalogRegistry` for lookup by id, key, or category.
- Scheme instantiation with binding resolution and critical question
  enumeration.
- Direct ASPIC+ integration: scheme instances compile to ordinary premises
  plus a defeasible rule, ready to feed into a `StructuredSystem`.

## Quick example

```rust
use argumentation_schemes::catalog::default_catalog;
use std::collections::HashMap;

let catalog = default_catalog();
let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();

let bindings: HashMap<String, String> = [
    ("expert".to_string(), "alice".to_string()),
    ("domain".to_string(), "military".to_string()),
    ("claim".to_string(), "fortify_east".to_string()),
]
.into_iter()
.collect();

let instance = scheme.instantiate(&bindings).unwrap();
assert_eq!(instance.critical_questions.len(), 6);
```

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.*
  Cambridge University Press.
- Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework.*
  Argument & Computation 5(1).
