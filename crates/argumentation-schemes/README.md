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

## AIF round-trip (v0.2.0)

Schemes round-trip through [AIFdb](http://corpora.aifdb.org) JSON:

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_schemes::{aif_to_instance, instance_to_aif};
use std::collections::HashMap;

let catalog = default_catalog();
let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
let bindings: HashMap<String, String> = [
    ("expert".into(), "alice".into()),
    ("domain".into(), "military".into()),
    ("claim".into(), "fortify_east".into()),
].into_iter().collect();

let instance = scheme.instantiate(&bindings).unwrap();
let aif = instance_to_aif(&instance);
let json = aif.to_json().unwrap();

// ... consume with external tooling or round-trip back:
let registry = CatalogRegistry::with_walton_catalog();
let recovered = aif_to_instance(&aif, &registry).unwrap();
assert_eq!(recovered.premises, instance.premises);
```

**Not preserved through AIF.** Critical-question counter-literals and
`Challenge` tags are not part of the AIF format; on import they are
re-derived by number-matching against the catalog's scheme definition.

## License

Dual-licensed under MIT or Apache-2.0 at your option.

## References

- Walton, D., Reed, C., & Macagno, F. (2008). *Argumentation Schemes.*
  Cambridge University Press.
- Modgil, S. & Prakken, H. (2014). *The ASPIC+ framework.*
  Argument & Computation 5(1).
