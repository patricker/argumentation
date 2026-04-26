---
sidebar_position: 7
title: Modulate attack weights with societas
---

Wire `societas_encounter::SocietasRelationshipSource` into an encounter scene so attack weights reflect live relationship state. Trust dampens attacks; fear amplifies them.

**Learning objective:** add societas-relationship modulation to an existing encounter scene with one new dependency, one resolver registration, and one weight-computation call per attack edge.

## Prerequisites

- The library installed and a working scene from [Build your first scene](/getting-started/first-scene).
- The `societas-encounter` crate available as a path-dep or registry dep (with the `argumentation` feature).

## Step 1: Add the dep

```toml
[dependencies]
societas-encounter = { version = "0.1", features = ["argumentation"] }
societas-core = "0.1"
societas-relations = "0.1"
societas-memory = "0.1"   # or any other SocialStore impl
```

## Step 2: Build the resolver

Map your scene's actor names to societas `EntityId`s.

```rust
use societas_core::EntityId;
use societas_encounter::names::StaticNameResolver;

let mut resolver = StaticNameResolver::new();
resolver.add("alice", EntityId::from_u64(1));
resolver.add("bob", EntityId::from_u64(2));
```

`StaticNameResolver::add` panics on names that collide with affordance role-binding slots (`"self"`, `"target"`, `"subject"`, `"initiator"`, `"aggressor"`, `"actor"`, `"recipient"`, `"witness"`). Use character names that don't conflict.

## Step 3: Seed relationship state

Attack weights derive from five dimensions: trust, fear, respect, attraction, friendship. Seed any subset — unseeded dimensions sit at zero (neutral).

```rust
use societas_core::{ModifierSource, Tick};
use societas_memory::MemStore;
use societas_relations::{Dimension, RelationshipRegistry};

let mut store = MemStore::new();
let registry = RelationshipRegistry::new();

// Bob trusts Alice → Bob's attack on Alice's claim is dampened.
registry.add_modifier(
    &mut store,
    EntityId::from_u64(2),  // bob
    EntityId::from_u64(1),  // alice
    Dimension::Trust,
    1.0,                    // magnitude in [-1, 1]
    0.0,                    // decay rate; 0.0 = permanent
    ModifierSource::Personality,
    Tick(0),
);
```

## Step 4: Build the source

```rust
use societas_encounter::SocietasRelationshipSource;

let source = SocietasRelationshipSource::new(
    &registry,
    &store,
    &resolver,
    state.actors_by_argument(),
    Tick(0),
);
```

`Tick` is owned and fixed at construction. To advance time mid-scene, build a fresh source.

## Step 5: Compute and seed weights

For each attack edge in your framework, ask the source for a weight, then wire it via `add_weighted_attack`:

```rust
use argumentation_weighted::WeightSource;

let w = source.weight_for(&bob_id, &alice_id).unwrap();
state.add_weighted_attack(&bob_id, &alice_id, w)?;
```

`weight_for` always returns `Some(_)` — unresolved actors or unseeded arguments fall back to `BASELINE_WEIGHT` (0.5).

## How the weight gets computed

For one attacker–target actor pair:

```
weight = clamp(
  BASELINE_WEIGHT
    + TRUST_COEF       * trust       // -0.15 — high trust dampens
    + FEAR_COEF        * fear        // +0.25 — high fear amplifies
    + RESPECT_COEF     * respect     // -0.05
    + ATTRACTION_COEF  * attraction  // -0.05
    + FRIENDSHIP_COEF  * friendship, // -0.10
  0.0, 1.0,
)
```

Multi-actor arguments (when one `ArgumentId` was asserted by two or more actors) average the per-pair weights across the (attacker × target) Cartesian product.

## Verify

Construct the same scene twice — once with high trust, once with high fear — and check the weights:

```rust
assert!(w_trust < 0.5);   // trust dampens below baseline
assert!(w_fear  > 0.5);   // fear amplifies above baseline
```

For an end-to-end credulous-acceptance flip, see the integration test in `societas-encounter/tests/relationship_source.rs`.

## When NOT to use this

- **Static scenes** where the dramatic outcome should be deterministic regardless of relationship state — wire weights directly via `add_weighted_attack(... w_hardcoded)` instead.
- **Scenes without scheme-backed arguments** — the source resolves `ArgumentId → actors` via `actors_by_argument()`, which only knows about scheme instances seeded via `add_scheme_instance` / `add_scheme_instance_for_affordance`.

## Related

- [Migration v0.4 → v0.5](/guides/migration-v0.4-to-v0.5) — if you're coming from the old `encounter-argumentation` location.
- [Encounter integration concept](/concepts/encounter-integration) — the bigger picture.
- [`SocietasRelationshipSource` API docs](https://docs.rs/societas-encounter).
