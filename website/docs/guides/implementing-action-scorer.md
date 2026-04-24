---
sidebar_position: 3
title: Implement an ActionScorer
---

Wire your own `ActionScorer<P>` inner implementation through `StateActionScorer` so base scores flow correctly and the bridge can apply argument-credulity boosts on top.

**Learning objective:** build a non-trivial `ActionScorer<P>` impl, wrap it with `StateActionScorer`, and verify the boost applies.

## Prerequisites

- Library installed.
- You know what `ActionScorer<P>` is conceptually ([encounter integration](/concepts/encounter-integration)).

## Step 1: Define your inner scorer

The `ActionScorer<P>` trait has one method:

```rust
fn score_actions(
    &self,
    actor: &str,
    available: &[CatalogEntry<P>],
    participants: &[String],
) -> Vec<ScoredAffordance<P>>
```

Return one `ScoredAffordance` per available affordance with:
- `entry` — cloned from `available`.
- `score` — your utility / heuristic value (typically [0.0, 1.0]).
- `bindings` — a HashMap the bridge will use for `AffordanceKey` lookup. **Must contain `"self"` → `actor`** if you want `StateAcceptanceEval` to route through your affordance.

## Step 2: Example — drive-alignment scorer

```rust
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter::affordance::CatalogEntry;
use std::collections::HashMap;

struct DriveScorer {
    agreeableness: f64,  // how much this actor prefers accepting
}

impl ActionScorer<String> for DriveScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<String>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<String>> {
        available.iter().map(|e| {
            let base = if e.spec.name.starts_with("agree") {
                0.5 + 0.5 * self.agreeableness
            } else {
                0.5 - 0.3 * self.agreeableness
            };
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.to_string());
            // ... plus whatever slot bindings your schemes need
            ScoredAffordance { entry: e.clone(), score: base, bindings }
        }).collect()
    }
}
```

## Step 3: Wrap with `StateActionScorer`

```rust
let inner = DriveScorer { agreeableness: 0.8 };
let scorer = encounter_argumentation::StateActionScorer::new(&state, inner, 0.5);
```

The third argument is the **boost**: additive score added to any affordance whose argument is credulously accepted at the current β. Typical values 0.3–1.0.

## Step 4: Verify the boost

Seed an argument, run `score_actions` directly, and assert the score moved:

```rust
let scored = scorer.score_actions("alice", &catalog, &participants);
assert!(scored.iter().any(|sa| sa.score > 1.0), "boost should fire for at least one affordance");
```

## Related

- [Implementing AcceptanceEval](/guides/implementing-acceptance-eval) — the responder side.
- [Tuning β](/guides/tuning-beta) — which scenes get which β.
