---
sidebar_position: 6
title: Migrate v0.4 → v0.5
---

`encounter-argumentation v0.5.0` removes the societas-aware relationship weight source. It ships from the `societas-encounter` crate now, behind an `argumentation` feature flag. This guide walks consumers through the swap.

**Learning objective:** update existing v0.4 wiring to v0.5 in under 5 minutes, with no behavioral change at runtime.

## What changed

| v0.4.0 (in `encounter-argumentation`) | v0.5.0 (in `societas-encounter`) |
|---|---|
| `SocietasRelationshipSource<'a, R>` (generic) | `SocietasRelationshipSource<'a>` (`&dyn NameResolver`) |
| `NameResolver` trait + `HashMap<String, EntityId>` blanket impl | `NameResolver` trait + concrete `StaticNameResolver` |
| `BASELINE_WEIGHT`, `TRUST_COEF`, `FEAR_COEF`, `RESPECT_COEF`, `ATTRACTION_COEF`, `FRIENDSHIP_COEF` | Same six constants, same values |

Coefficient values are unchanged — `constants_match_phase_c_v0_4_0` test in the new home pins them.

## Step 1: Update `Cargo.toml`

Remove (if you had them as separate deps):

```toml
# old
encounter-argumentation = "0.4"
```

Replace with:

```toml
encounter-argumentation = "0.5"
societas-encounter = { version = "0.1", features = ["argumentation"] }
```

`societas-encounter` lives in the sibling [`societas`](https://github.com/patricker/societas) workspace. If you're working from path-deps in the `~/code/` layout, the path-dep form is:

```toml
societas-encounter = { path = "../societas/crates/encounter", features = ["argumentation"] }
```

## Step 2: Update imports

```rust
// old (v0.4)
use encounter_argumentation::{
    SocietasRelationshipSource, NameResolver, TRUST_COEF,
};

// new (v0.5)
use encounter_argumentation::EncounterArgumentationState;  // unchanged
use societas_encounter::{SocietasRelationshipSource, TRUST_COEF};
use societas_encounter::names::{NameResolver, StaticNameResolver};
```

Other v0.4 imports — `EncounterArgumentationState`, `StateActionScorer`, `StateAcceptanceEval`, `AffordanceKey`, `actors_by_argument()` — are unchanged. Only the societas-aware bits moved.

## Step 3: Rebuild your `NameResolver`

The blanket `HashMap<String, EntityId>` impl is gone. Switch to `StaticNameResolver`:

```rust
// v0.4: HashMap as resolver (no longer compiles)
let mut resolver: HashMap<String, EntityId> = HashMap::new();
resolver.insert("alice".into(), EntityId::from_u64(1));

// v0.5: StaticNameResolver
let mut resolver = StaticNameResolver::new();
resolver.add("alice", EntityId::from_u64(1));
```

`StaticNameResolver::add` **panics** on reserved names like `"self"`, `"target"`, `"subject"` — these collide with affordance role-binding slots and were a known footgun in the v0.4 HashMap form. Use `try_add` if you need fallible registration:

```rust
resolver.try_add("self", EntityId::from_u64(1))
    .expect("character name should not collide with binding slot");
```

## Step 4: Construct the source

The constructor signature is unchanged in shape, only the resolver type changes:

```rust
let source = SocietasRelationshipSource::new(
    &registry,                       // &RelationshipRegistry
    &store,                          // &dyn SocialStore
    &resolver,                       // &dyn NameResolver — was &R generic
    state.actors_by_argument(),      // &HashMap<ArgumentId, Vec<String>>
    Tick(0),                         // owned
);
```

The generic `R` parameter is gone — `&resolver` coerces implicitly to `&dyn NameResolver` because `NameResolver: Send + Sync`.

## Step 5: Verify

```bash
cargo build
cargo test
```

Existing tests should pass without semantic change. If a test previously seeded a HashMap-as-resolver, it'll now fail to compile — switch the construction to `StaticNameResolver` per Step 3.

## Why the move

In v0.4, `encounter-argumentation` depended on three societas crates just to host `SocietasRelationshipSource`, even though most consumers wanted the rest of the bridge (state, scorer, eval) without the societas coupling. v0.5 returns `encounter-argumentation` to its original scope — encounter ↔ argumentation only — and houses the societas adapter alongside the other societas↔encounter bridge types (`SocialActionScorer`, `PersonalityAcceptanceEval`, `NameResolver`) in `societas-encounter`.

See the [encounter integration concept](/concepts/encounter-integration) for the architectural picture.

## Related

- [Societas-modulated weights how-to](/guides/societas-modulated-weights) — wiring sketch from scratch.
- [`encounter-argumentation` CHANGELOG](https://github.com/patricker/argumentation/blob/main/crates/encounter-argumentation/CHANGELOG.md).
- [`societas-encounter` CHANGELOG](https://github.com/patricker/societas/blob/main/crates/encounter/CHANGELOG.md).
