---
sidebar_position: 3
title: Your third scene — with values
---

Add per-character value priorities to the [second scene](/getting-started/second-scene-with-schemes) so Alice and Bob reach *different* conclusions when they hold different values. The bridge's `ValueAwareScorer` reads each character's audience and adjusts proposal scoring so duty-prioritising Alice picks differently from survival-prioritising Bob.

**Learning objective:** add per-character audiences to a two-actor scene with three lines of code (one `set_audience` per actor, one `ValueAwareScorer::new` wrapper), observe that the same scene with the same arguments produces different beats per character, and verify the trace shows the expected outcome flip — in under 10 minutes, building on the [second scene](/getting-started/second-scene-with-schemes).

## What you'll build

The same two-actor scene from tutorial 2, but now with values attached to each argument and a per-character audience for each actor. Alice's proposal promotes "duty"; Bob's promotes "survival." When Alice scores affordances, duty-promoting ones get a boost; when Bob scores them, survival-promoting ones get a boost. The same `MultiBeat::resolve` call now produces a different beat sequence depending on whose audience drives the scoring.

**Time:** ~10 minutes.
**Difficulty:** Beginner+ (assumes you completed the [second scene](/getting-started/second-scene-with-schemes)).
**You'll leave with:** a running scene where audience-conditioned scoring shapes the trace, and a mental model for "values are character preferences, not framework facts."

## Prerequisites

- Completion of [Your second scene — multiple schemes](/getting-started/second-scene-with-schemes) — you have a working `my-second-scene` (or copy thereof) that runs.

## Step 1: Continue from your second-scene project

```bash
cp -r my-second-scene my-third-scene
cd my-third-scene
```

## Step 2: Add `argumentation-values` to dependencies

```toml
[dependencies]
argumentation-schemes = "0.2"
argumentation-weighted = "0.2"
argumentation-values = "0.1"
encounter-argumentation = "0.5"
encounter = "0.1"
```

(`argumentation-values` is already a transitive dep through `encounter-argumentation`, but adding it directly lets you import its types.)

## Step 3: Switch to the `argument_from_values` scheme for both actors

The Walton catalog has a scheme specifically designed for value-promoting arguments:

```rust
let values_scheme = registry.by_key("argument_from_values").unwrap();
```

Bindings: `action`, `value`, `agent`. Conclusion template: `"?action should be carried out because it promotes ?value for ?agent"`.

Replace Alice's instance:

```rust
let mut alice_b = HashMap::new();
alice_b.insert("action".into(), "fortify".into());
alice_b.insert("value".into(), "duty".into());
alice_b.insert("agent".into(), "alice".into());
let alice_instance = values_scheme.instantiate(&alice_b).unwrap();
```

Replace Bob's instance:

```rust
let mut bob_b = HashMap::new();
bob_b.insert("action".into(), "abandon".into());
bob_b.insert("value".into(), "survival".into());
bob_b.insert("agent".into(), "bob".into());
let bob_instance = values_scheme.instantiate(&bob_b).unwrap();
```

The `value` binding is what `ValueAwareScorer` reads to compute its boost.

## Step 4: Set per-character audiences

After constructing the state, add:

```rust
use encounter_argumentation::{Audience, Value};

state.set_audience(
    "alice",
    Audience::total([Value::new("duty"), Value::new("survival")]),
);
state.set_audience(
    "bob",
    Audience::total([Value::new("survival"), Value::new("duty")]),
);
```

Each call mirrors `set_intensity` — it mutates state through a shared `&self` reference (interior mutability via `Mutex`).

## Step 5: Wrap the scorer with `ValueAwareScorer`

Find the `let scorer = StateActionScorer::new(&state, InnerScorer, 0.5);` line. Wrap it:

```rust
use encounter_argumentation::ValueAwareScorer;

let scheme_scorer = StateActionScorer::new(&state, InnerScorer, 0.5);
let scorer = ValueAwareScorer::new(scheme_scorer, &state, 0.3);
```

The two boosts compose additively: scheme-strength boost first (0.5 max), then value-preference boost (0.3 max). When Alice scores `argue_fortify` (which promotes "duty," tier 0 of her audience), she gets the full value boost. When Bob scores it, "duty" is tier 1 in his audience — smaller boost.

## Step 6: Update the inner scorer to set the `value` binding

The `value_boost_for_affordance` function in `ValueAwareScorer` reads the `value` binding from the affordance's bindings. Update `InnerScorer` so each affordance carries its `value`:

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
            let (action, value) = match e.spec.name.as_str() {
                "argue_fortify" => ("fortify", "duty"),
                "argue_abandon" => ("abandon", "survival"),
                _ => unreachable!(),
            };
            bindings.insert("action".into(), action.into());
            bindings.insert("value".into(), value.into());
            bindings.insert("agent".into(), actor.into());
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}
```

Note the affordance names changed (we renamed `argue_against_fortify` → `argue_abandon` for clarity in the values context). Update the catalog and practice strings accordingly.

## Step 7: Update the catalog

```rust
let alice_aff = AffordanceSpec {
    name: "argue_fortify".into(),
    domain: "values".into(),
    bindings: vec!["self".into(), "action".into(), "value".into(), "agent".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let bob_aff = AffordanceSpec {
    name: "argue_abandon".into(),
    domain: "values".into(),
    bindings: vec!["self".into(), "action".into(), "value".into(), "agent".into()],
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
    affordances: vec!["argue_fortify".into(), "argue_abandon".into()],
    turn_policy: TurnPolicy::RoundRobin,
    duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
    entry_condition_source: String::new(),
};
```

## Step 8: Update affordance-key bindings

Update the `add_scheme_instance_for_affordance` calls so the affordance keys carry the `action` binding (not `claim`/`cause`):

```rust
let mut alice_af = alice_b.clone();
alice_af.insert("self".into(), "alice".into());
let alice_id = state.add_scheme_instance_for_affordance(
    "alice", "argue_fortify", &alice_af, alice_instance,
);
let mut bob_af = bob_b.clone();
bob_af.insert("self".into(), "bob".into());
let bob_id = state.add_scheme_instance_for_affordance(
    "bob", "argue_abandon", &bob_af, bob_instance,
);
```

## Step 9: Run

```bash
cargo run
```

Expected output:

```
beat 1: alice argued argue_fortify — accepted: true
beat 2: bob   argued argue_abandon — accepted: true
beat 3: alice argued argue_fortify — accepted: true
beat 4: bob   argued argue_abandon — accepted: true
```

Each character argues for the action that promotes their top-tier value. The boosts shape *which* affordance each picks; the trace shows both consistently propose their preferred action.

To see the audience flip in action, swap Alice's and Bob's audiences:

```rust
state.set_audience(
    "alice",
    Audience::total([Value::new("survival"), Value::new("duty")]),
);
state.set_audience(
    "bob",
    Audience::total([Value::new("duty"), Value::new("survival")]),
);
```

Re-run. Now Alice picks `argue_abandon` (her new top-tier value) and Bob picks `argue_fortify`. The same scene structure, same arguments, same attacks — different audiences flip the outcome.

## Why this matters

In the [first scene](/getting-started/first-scene), only β shaped acceptance. In the [second scene](/getting-started/second-scene-with-schemes), scheme choice shaped attack semantics. Here, audience shaped which arguments each character reaches for. These three dials — β, scheme, audience — compose. A scene author tunes them to produce specific dramatic effects.

The audience dial is the most expressive of the three: it lets you give each character a stable value profile that persists across many scenes. Alice always prioritises duty; Bob always prioritises survival; their disagreement plays out consistently across every scene they're in together.

## What you learned

- The `argument_from_values` Walton scheme and its `value` binding.
- How to set per-character audiences via `state.set_audience(actor, audience)`.
- How `ValueAwareScorer` composes on top of `StateActionScorer` (additive boosts).
- The audience-flip pattern: same framework, different audiences, different outcome.

## Next steps

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — the formal semantics behind audiences.
- [Wiring per-character values (how-to)](/guides/wiring-character-values) — production-level integration patterns.
- [Hal & Carla example](/examples/hal-and-carla) — the canonical legal-reasoning scene with audience-driven outcome flips.
- [Multi-character consensus (how-to)](/guides/multi-character-consensus) — for queries across audiences in council scenes.
