---
sidebar_position: 9
title: Wire per-character values into a scene
---

Wire per-character audiences into an encounter scene so Alice and Bob reach different conclusions when they hold different values. The `ValueAwareScorer` reads each character's audience from `EncounterArgumentationState` and adjusts proposal scoring accordingly.

**Learning objective:** add per-character value preferences to an existing encounter scene with one new dependency, two `set_audience` calls, and one `ValueAwareScorer::new` wrapper around the existing scorer chain.

## Prerequisites

- A working scene from [Build your first scene](/getting-started/first-scene).
- `encounter-argumentation` v0.5+ with `argumentation-values` available (path-dep or registry-dep).

## Step 1: Add the dep

Already a transitive dep through `encounter-argumentation`, but if you use `argumentation-values` types directly:

```toml
[dependencies]
argumentation-values = "0.1"
```

## Step 2: Set per-character audiences before resolve

```rust
use encounter_argumentation::{Audience, EncounterArgumentationState, Value};

let state = EncounterArgumentationState::new(catalog);

// Alice prioritises duty above all else.
state.set_audience(
    "alice",
    Audience::total([Value::new("duty"), Value::new("survival"), Value::new("comfort")]),
);

// Bob's audience inverts duty and survival.
state.set_audience(
    "bob",
    Audience::total([Value::new("survival"), Value::new("duty"), Value::new("comfort")]),
);
```

The audience storage uses interior mutability (mirroring how `set_intensity` works) — you can call `set_audience` through a shared `&state` reference at any point before resolve.

## Step 3: Wrap your existing scorer

If you're already using `SchemeActionScorer`, just stack `ValueAwareScorer` on top:

```rust
use encounter_argumentation::{SchemeActionScorer, ValueAwareScorer};

let scheme_scorer = SchemeActionScorer::new(
    knowledge,
    registry,
    baseline_scorer,
    0.3,  // scheme-strength boost magnitude
);
let value_scorer = ValueAwareScorer::new(
    scheme_scorer,
    &state,
    0.2,  // value-preference boost magnitude
);
```

The two boosts compose additively: scheme-strength boost first, then value-preference boost. Both skip silently when the actor has no audience configured.

## Step 4: Resolve as usual

```rust
use encounter::resolution::MultiBeat;
use encounter_argumentation::StateAcceptanceEval;

let acceptance = StateAcceptanceEval::new(&state);
let participants = vec!["alice".into(), "bob".into()];
let result = MultiBeat.resolve(&participants, &practice, &catalog, &value_scorer, &acceptance);
```

When Alice scores her affordances, those backed by schemes promoting `duty` get the largest boost (tier 0). When Bob scores the same affordances, `survival`-promoting ones get the largest. Same scene, same arguments — different proposals win.

## Verify

A worked test pattern:

```rust
#[test]
fn alice_and_bob_reach_different_outcomes() {
    let state = build_state_with_two_proposals();
    state.set_audience("alice", Audience::total([Value::new("duty")]));
    state.set_audience("bob", Audience::total([Value::new("survival")]));

    let scorer = build_value_aware_scorer(&state);
    let alice_scored = scorer.score_actions("alice", &affordances, &participants);
    let bob_scored = scorer.score_actions("bob", &affordances, &participants);

    // Alice's top pick should be the duty-promoting affordance; Bob's the
    // survival-promoting one.
    assert_ne!(alice_scored[0].entry.spec.name, bob_scored[0].entry.spec.name);
}
```

## How the value boost is computed

For each affordance, the scorer checks if its `bindings` map contains a `value` slot. If yes, and that value is ranked in the actor's audience, the boost is:

```
boost = max_boost * (tier_count - rank) / tier_count
```

Where `rank = 0` for the most preferred value (largest boost) and `rank = tier_count - 1` for the least preferred (smallest non-zero boost). Unranked values get zero boost.

## When NOT to use this

- **Scenes where all characters share an audience.** Set the audience once in scene setup and skip the per-character storage.
- **Scenes where values aren't relevant to the proposals.** If the affordances aren't backed by `argument_from_values` schemes, `ValueAwareScorer` is a no-op — skip it.
- **Scenes where the storyteller wants to *force* an outcome regardless of character values.** Use direct `add_weighted_attack` with hard-coded weights instead.

## Related

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — the formalism this scorer is built on.
- [Hal & Carla](/examples/hal-and-carla) — the canonical scene that motivates per-character audiences.
- [Modulate attack weights with societas](/guides/societas-modulated-weights) — for live relationship-driven attack weight modulation.
- [`argumentation-values` API docs](https://docs.rs/argumentation-values).
