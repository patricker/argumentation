---
sidebar_position: 1
title: Build your first scene
---

Build a working east-wall scene end-to-end. You'll seed two arguments, set a scene intensity, run `MultiBeat`, and print the resulting beats.

**Learning objective:** build a working two-actor argumentation scene with weighted attacks and a tunable β, run it end-to-end with `cargo run`, and read the resulting beat-by-beat acceptance trace — in under 10 minutes, with no prior argumentation-theory knowledge.

## What you'll build

A small Rust program that runs a 4-beat scene between Alice and Bob over whether to fortify the east wall. Alice asserts an argument from expert opinion. Bob asserts a counter. At β=0.5, the scorer boosts Alice's claim, Bob rejects with his credulous counter, and the beats print deterministically.

**Time:** ~10 minutes.  
**Difficulty:** Beginner.  
**You'll leave with:** a running `cargo run` example and a mental model for "how to get arguments into a scene."

## Prerequisites

- Rust 1.80+ (`rustc --version`)
- A terminal
- No prior argumentation-theory knowledge — we'll explain as we go

## Step 1: Create the project

```bash
cargo new --bin my-first-scene
cd my-first-scene
```

Add to `Cargo.toml`:

```toml
[dependencies]
argumentation-schemes = "0.2"
argumentation-weighted = "0.2"
encounter-argumentation = "0.5"
encounter = "0.1"
```

## Step 2: Set up state

Replace `src/main.rs` with:

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::EncounterArgumentationState;

fn main() {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let state = EncounterArgumentationState::new(registry);
    state.set_intensity(Budget::new(0.5).unwrap());  // scene tension

    println!("State initialized at β=0.5");
}
```

Run:

```bash
cargo run
```

Expected output:

```
State initialized at β=0.5
```

## Step 3: Seed Alice's argument

Alice is a military expert arguing to fortify the east wall. Extend `main()`:

```rust
use std::collections::HashMap;
// ... (keep previous imports)

let mut alice_bindings = HashMap::new();
alice_bindings.insert("expert".into(), "alice".into());
alice_bindings.insert("domain".into(), "military".into());
alice_bindings.insert("claim".into(), "fortify_east".into());
alice_bindings.insert("self".into(), "alice".into());

let alice_instance = scheme.instantiate(&alice_bindings).unwrap();
let alice_id = state.add_scheme_instance_for_affordance(
    "alice",
    "argue_fortify_east",
    &alice_bindings,
    alice_instance,
);

println!("Seeded Alice's argument: {:?}", alice_id);
```

## Step 4: Seed Bob's counter-argument

Bob is a logistics expert. His argument attacks Alice's with weight 0.4:

```rust
let mut bob_bindings = HashMap::new();
bob_bindings.insert("expert".into(), "bob".into());
bob_bindings.insert("domain".into(), "logistics".into());
bob_bindings.insert("claim".into(), "abandon_east".into());
bob_bindings.insert("self".into(), "bob".into());

let bob_instance = scheme.instantiate(&bob_bindings).unwrap();
let bob_id = state.add_scheme_instance_for_affordance(
    "bob",
    "argue_abandon_east",
    &bob_bindings,
    bob_instance,
);

state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
```

## Step 5: Build the encounter catalog and practice

```rust
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};

let make_spec = |name: &str| AffordanceSpec {
    name: name.into(),
    domain: "persuasion".into(),
    bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let catalog = vec![
    CatalogEntry { spec: make_spec("argue_fortify_east"), precondition: String::new() },
    CatalogEntry { spec: make_spec("argue_abandon_east"), precondition: String::new() },
];
let practice = PracticeSpec {
    name: "east-wall-debate".into(),
    affordances: vec!["argue_fortify_east".into(), "argue_abandon_east".into()],
    turn_policy: TurnPolicy::RoundRobin,
    duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
    entry_condition_source: String::new(),
};
```

## Step 6: Write an inner scorer

The bridge boosts arguments, but you still need a base scorer. A uniform 1.0 scorer will do:

```rust
use encounter::scoring::{ActionScorer, ScoredAffordance};

struct UniformScorer;
impl<P: Clone> ActionScorer<P> for UniformScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut b = HashMap::new();
            b.insert("self".into(), actor.to_string());
            let claim = if e.spec.name == "argue_fortify_east" { "fortify_east" } else { "abandon_east" };
            b.insert("claim".into(), claim.into());
            b.insert("expert".into(), actor.to_string());
            b.insert("domain".into(), "military".into());
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings: b }
        }).collect()
    }
}
```

## Step 7: Resolve the scene

```rust
use encounter::resolution::MultiBeat;
use encounter_argumentation::{StateAcceptanceEval, StateActionScorer};

let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
let acceptance = StateAcceptanceEval::new(&state);
let participants = vec!["alice".into(), "bob".into()];

let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

for beat in &result.beats {
    println!(
        "{} proposed {} — {}",
        beat.actor,
        beat.action,
        if beat.accepted { "accepted" } else { "rejected" },
    );
}

for err in state.drain_errors() {
    eprintln!("latched: {}", err);
}
```

Expected output:

```
alice proposed argue_fortify_east — rejected
bob proposed argue_abandon_east — accepted
alice proposed argue_fortify_east — rejected
bob proposed argue_abandon_east — accepted
```

Alice picks her boosted argument every beat, but Bob's credulous counter rejects her. Bob's unattacked argument is accepted.

## Complete example

Your full `src/main.rs` should look like [this working source](https://github.com/patricker/argumentation/blob/main/crates/encounter-argumentation/tests/uc_multibeat_scene.rs) — the integration test that ships with the library exercises the same scene.

## What you learned

You can now:

- **Seed argument instances** per actor via `add_scheme_instance_for_affordance`.
- **Add weighted attacks** between argument ids.
- **Set a scene intensity** via `set_intensity`.
- **Run `MultiBeat`** with a bridge-backed scorer and acceptance eval.
- **Drain errors** latched by bridge impls that can't propagate `Result`.

## Next steps

- [Tune β](/guides/tuning-beta) — pick the right intensity per scene register.
- [Author catalogs declaratively](/guides/catalog-authoring) — move affordance definitions to TOML.
- [Understand the encounter bridge](/concepts/encounter-integration) — what those two trait impls actually do.
