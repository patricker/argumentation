---
sidebar_position: 2
title: Your second scene — multiple schemes
---

Build on [your first scene](/getting-started/first-scene) by adding a counter-argument that uses a different Walton scheme. Alice still argues from expert opinion. Bob will argue from negative consequences ("fortifying causes the garrison to starve"). Each scheme has its own critical questions; we'll see how the bridge composes them.

**Learning objective:** build a two-actor scene where each actor invokes a different Walton scheme, observe how scheme strengths shape acceptance, and verify the trace shows the expected acceptance flip — in under 15 minutes, with no prior scheme-theory knowledge beyond what the [first scene](/getting-started/first-scene) covered.

## What you'll build

A 4-beat scene where Alice (expert/military) argues to fortify and Bob (negative-consequences) argues that fortifying causes a worse outcome. Critical questions from one scheme map onto attacks against the other. At β=0.5, the framework picks one over the other based on relative scheme strength.

**Time:** ~15 minutes.
**Difficulty:** Beginner+ (assumes you completed the [first scene](/getting-started/first-scene)).
**You'll leave with:** a running scene with two scheme types and a mental model for "different scheme = different attack semantics."

## Prerequisites

- Completion of [Build your first scene](/getting-started/first-scene) — you have a working `cargo new --bin my-first-scene` project that runs.
- Familiarity with `EncounterArgumentationState`, `add_scheme_instance_for_affordance`, and `MultiBeat::resolve` (all from the first scene).

## Step 1: Continue from your first-scene project

We'll build on the same project. You can either start from the completed first-scene code, or copy the directory:

```bash
cp -r my-first-scene my-second-scene
cd my-second-scene
```

If you don't have the first-scene project, follow [its setup steps](/getting-started/first-scene#step-1-create-the-project) first.

## Step 2: Identify Bob's new scheme

The default catalog ships ~60 Walton schemes. The first-scene tutorial used `argument_from_expert_opinion` for both Alice and Bob. Here we'll switch Bob to `argument_from_negative_consequences`:

```rust
let neg_scheme = registry
    .by_key("argument_from_negative_consequences")
    .expect("argument_from_negative_consequences in default catalog");
```

The negative-consequences scheme has bindings: `action`, `bad_consequence`. The conclusion is the negation of `do_?action` — i.e., asserting that the action *should not* be done because of the bad consequence. See `argumentation-schemes/src/catalog/practical.rs` for the full scheme definition.

## Step 3: Build Bob's instance with the new scheme

Replace Bob's expert-opinion instantiation from the first scene with:

```rust
let mut bob_bindings = HashMap::new();
bob_bindings.insert("action".into(), "fortify_east".into());
bob_bindings.insert("bad_consequence".into(), "garrison_starves".into());
let bob_instance = neg_scheme.instantiate(&bob_bindings).unwrap();
```

This produces an argument concluding "do not fortify_east, because garrison_starves" — Bob is asserting that Alice's proposed action causes a bad outcome.

## Step 4: Wire the attack

Now the attack is asymmetric — Bob's negative-consequences argument attacks Alice's expert-opinion argument, but not vice versa. Add only one attack edge:

```rust
state.add_weighted_attack(&bob_id, &alice_id, 0.4)?;
// Note: no add_weighted_attack(&alice_id, &bob_id, ...) here.
```

This represents the asymmetry: a negative-consequences critique can undermine an expert opinion (the expert may be right about the *call* but wrong about the *consequences*), but the reverse argument structure doesn't naturally apply.

## Step 5: Update the affordance bindings to match the new scheme

The bindings dict you pass to `add_scheme_instance_for_affordance` for Bob must match the negative-consequences scheme's slots plus `self`:

```rust
let mut bob_af = bob_bindings.clone();
bob_af.insert("self".into(), "bob".into());
let bob_id = state.add_scheme_instance_for_affordance(
    "bob",
    "argue_against_fortify",
    &bob_af,
    bob_instance,
);
```

Note `argue_against_fortify` is the affordance name (a string we pick); Bob's bindings include `action`, `bad_consequence`, and `self`.

## Step 6: Update the affordance catalog and scorer

The catalog needs to declare the bindings each affordance uses. Replace the catalog construction with:

```rust
let alice_aff = AffordanceSpec {
    name: "argue_fortify_east".into(),
    domain: "persuasion".into(),
    bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let bob_aff = AffordanceSpec {
    name: "argue_against_fortify".into(),
    domain: "persuasion".into(),
    bindings: vec!["self".into(), "action".into(), "bad_consequence".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let catalog = vec![
    CatalogEntry { spec: alice_aff, precondition: String::new() },
    CatalogEntry { spec: bob_aff, precondition: String::new() },
];
```

The inner scorer (`UniformScorer` or whatever you wrote in the first scene) needs to populate the right bindings per affordance. For brevity, use a match on the affordance name:

```rust
struct InnerScorer;
impl<P: Clone> ActionScorer<P> for InnerScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.into());
            match e.spec.name.as_str() {
                "argue_fortify_east" => {
                    bindings.insert("expert".into(), "alice".into());
                    bindings.insert("domain".into(), "military".into());
                    bindings.insert("claim".into(), "fortify_east".into());
                }
                "argue_against_fortify" => {
                    bindings.insert("action".into(), "fortify_east".into());
                    bindings.insert("bad_consequence".into(), "garrison_starves".into());
                }
                _ => unreachable!(),
            }
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}
```

## Step 7: Run

```bash
cargo run
```

Expected output (with deterministic ordering):

```
beat 1: alice argued argue_fortify_east — accepted: false
beat 2: bob   argued argue_against_fortify — accepted: true
beat 3: alice argued argue_fortify_east — accepted: false
beat 4: bob   argued argue_against_fortify — accepted: true
```

Alice's proposal is rejected because Bob's negative-consequences argument is credulously accepted (no counter-argument knocks it out). Bob's proposal is accepted because Alice has no counter-argument the bridge sees.

## Why the asymmetry matters

In the [first scene](/getting-started/first-scene), Alice and Bob both used the same scheme — the framework treated them symmetrically. Here, they use *different* schemes, and the attack relation is asymmetric. This is the structural difference that makes scheme choice matter: a negative-consequences argument can undermine an expert-opinion argument's *recommended action*, but not vice versa.

The library's [60+ Walton schemes](/concepts/walton-schemes) each carry their own critical questions and natural attack patterns. Picking the right scheme for each character's argument is most of the authoring work in scene design.

## Complete example

If you got lost, the full `src/main.rs`:

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};
use std::collections::HashMap;

struct InnerScorer;
impl<P: Clone> ActionScorer<P> for InnerScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.into());
            match e.spec.name.as_str() {
                "argue_fortify_east" => {
                    bindings.insert("expert".into(), "alice".into());
                    bindings.insert("domain".into(), "military".into());
                    bindings.insert("claim".into(), "fortify_east".into());
                }
                "argue_against_fortify" => {
                    bindings.insert("action".into(), "fortify_east".into());
                    bindings.insert("bad_consequence".into(), "garrison_starves".into());
                }
                _ => unreachable!(),
            }
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}

fn main() {
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let neg = registry.by_key("argument_from_negative_consequences").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".into(), "alice".into());
    alice_b.insert("domain".into(), "military".into());
    alice_b.insert("claim".into(), "fortify_east".into());
    let alice_instance = expert.instantiate(&alice_b).unwrap();

    let mut bob_b = HashMap::new();
    bob_b.insert("action".into(), "fortify_east".into());
    bob_b.insert("bad_consequence".into(), "garrison_starves".into());
    let bob_instance = neg.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".into(), "alice".into());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice", "argue_fortify_east", &alice_af, alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".into(), "bob".into());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob", "argue_against_fortify", &bob_af, bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
    state.set_intensity(Budget::new(0.5).unwrap());

    let alice_aff = AffordanceSpec {
        name: "argue_fortify_east".into(),
        domain: "persuasion".into(),
        bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let bob_aff = AffordanceSpec {
        name: "argue_against_fortify".into(),
        domain: "persuasion".into(),
        bindings: vec!["self".into(), "action".into(), "bad_consequence".into()],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let catalog = vec![
        CatalogEntry { spec: alice_aff, precondition: String::new() },
        CatalogEntry { spec: bob_aff, precondition: String::new() },
    ];
    let practice = PracticeSpec {
        name: "debate".into(),
        affordances: vec!["argue_fortify_east".into(), "argue_against_fortify".into()],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, InnerScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".into(), "bob".into()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

    for (i, b) in result.beats.iter().enumerate() {
        println!("beat {}: {} argued {} — accepted: {}", i + 1, b.actor, b.action, b.accepted);
    }
    let _ = alice_id;
    let _ = bob_id;
}
```

## What you learned

- How to use a different Walton scheme per actor.
- The `negative-consequences` scheme's binding shape (action/bad_consequence).
- Asymmetric attacks: not every counter is mutual.
- Scheme choice is the primary authoring lever for "what kind of argument is this?"

## Next steps

- [Your third scene — with values](/getting-started/third-scene-with-values) — adds per-character value priorities (audiences) so Alice and Bob reach different conclusions when they hold different values.
- [Walton schemes (concepts)](/concepts/walton-schemes) — what schemes are and what's in the catalog.
- [Author an affordance catalog](/guides/catalog-authoring) — for moving affordance definitions out of Rust into TOML.
