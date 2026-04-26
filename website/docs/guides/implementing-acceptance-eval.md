---
sidebar_position: 4
title: Implement an AcceptanceEval
---

Decide whether to use `StateAcceptanceEval` as-is or wrap it, and compose a custom `AcceptanceEval<P>` when you need responder-specific logic the bridge doesn't model.

**Learning objective:** understand when `StateAcceptanceEval` alone suffices, when to wrap it, and implement a composed eval that adds personality-driven rejection on top.

## When `StateAcceptanceEval` alone is enough

- Scenes are driven by argument-theoretic acceptance only.
- Your affordances all carry a `"self"` proposer binding.
- You don't need per-responder personality biases.

In that case, just use `StateAcceptanceEval::new(&state)`.

## When to wrap

You need a custom eval when any of the following applies:
- Your proposer slot isn't `"self"` (e.g., `"speaker"`).
- You want to add a personality-based rejection probability *on top* of the bridge's decision.
- You need to intercept specific affordance names.

## Step 1: Compose with short-circuit

Run `StateAcceptanceEval` first; trust its rejection; add your own layer on top of its acceptance:

```rust
use encounter::scoring::{AcceptanceEval, ScoredAffordance};
use encounter_argumentation::StateAcceptanceEval;

struct PersonalityEval<'a> {
    bridge: StateAcceptanceEval<'a>,
    rejection_bias: f64,  // per-responder, [0.0, 1.0]
}

impl<'a, P> AcceptanceEval<P> for PersonalityEval<'a> {
    fn evaluate(&self, responder: &str, action: &ScoredAffordance<P>) -> bool {
        if !self.bridge.evaluate(responder, action) {
            return false;  // bridge rejected — trust it
        }
        // Bridge accepted; apply personality bias.
        // In production, draw from an RNG seeded per scene — e.g. via
        // `rand::Rng::random::<f64>()` from the [`rand`](https://crates.io/crates/rand)
        // crate. The simplified form below keeps the example deterministic.
        let reject_prob = self.rejection_bias;
        reject_prob < 0.5
    }
}
```

## Step 2: Wire it in

```rust
let bridge_eval = StateAcceptanceEval::new(&state);
let my_eval = PersonalityEval { bridge: bridge_eval, rejection_bias: 0.3 };

let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &my_eval);
```

## Verify

Seed a scene where the bridge would accept, but your personality eval should reject some fraction of the time. Check `result.beats[i].accepted` matches expectations.

## Related

- [Implementing ActionScorer](/guides/implementing-action-scorer) — the proposer side.
- [Encounter integration concept](/concepts/encounter-integration) — why the D4 split exists.
