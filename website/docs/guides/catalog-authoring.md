---
sidebar_position: 2
title: Author an affordance catalog
---

Define a scene's affordance catalog in TOML so content authors can add new actions without touching Rust, then load it at runtime.

**Learning objective:** create a valid TOML catalog for a scene, load it in Rust, and have each affordance backed by a Walton scheme via `scheme_id` bindings.

## Prerequisites

- The library installed ([installation guide](/guides/installation)).
- The `encounter` crate's `AffordanceSpec` type — it has `serde::Deserialize` derived.

## Step 1: Write the catalog file

Create `catalog/persuasion.toml`:

```toml
[[affordance]]
name = "argue_expert_opinion"
domain = "persuasion"
bindings = ["self", "expert", "domain", "claim"]

[[affordance.effects_on_accept]]
kind = "relationship_delta"
dimension = "agreement"
amount = 0.15

[[affordance.effects_on_reject]]
kind = "relationship_delta"
dimension = "agreement"
amount = -0.05

[[affordance]]
name = "argue_counter_opinion"
domain = "persuasion"
bindings = ["self", "expert", "domain", "claim"]
```

Keep affordance names stable — the bridge's forward index keys off them.

## Step 2: Load in Rust

```rust
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use std::fs;

#[derive(serde::Deserialize)]
struct CatalogFile {
    affordance: Vec<AffordanceSpec>,
}

fn load_catalog(path: &str) -> Vec<CatalogEntry<String>> {
    let text = fs::read_to_string(path).expect("catalog not found");
    let file: CatalogFile = toml::from_str(&text).expect("catalog parse failure");
    file.affordance
        .into_iter()
        .map(|spec| CatalogEntry { spec, precondition: String::new() })
        .collect()
}
```

## Step 3: Associate schemes at seed time

The `encounter` crate's `AffordanceSpec` doesn't carry a `scheme_id` field — our bridge is "loosely coupled" (see [Phase B plan](https://github.com/patricker/argumentation/blob/main/docs/superpowers/plans/2026-04-20-phase-b-state-scorer-bridge.md)). Associate schemes in code when seeding:

```rust
let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
let instance = scheme.instantiate(&bindings).unwrap();
state.add_scheme_instance_for_affordance(
    actor,
    &catalog_entry.spec.name,  // affordance_name from TOML
    &bindings,
    instance,
);
```

## Verify it worked

Load + seed two affordances, then run `MultiBeat`. The catalog entries drive action selection; the bridge adds argument-theoretic scoring + acceptance on top.

## Related

- [Implementing ActionScorer](/guides/implementing-action-scorer) — wire the inner scorer that produces base scores.
- [Reference overview](/reference/overview) — `AffordanceSpec` / `CatalogEntry` signatures.
