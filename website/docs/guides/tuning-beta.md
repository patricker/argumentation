---
sidebar_position: 5
title: Tune β for your scene
---

β (scene intensity) is the single-knob tension dial. This guide helps you pick one.

**Learning objective:** pick β by scene register and escalate mid-scene when the dramatic situation demands it.

## Prerequisites

- Library installed.
- You understand β conceptually ([weighted frameworks & β](/concepts/weighted-and-beta)).

## Step 1: Pick a starting β by register

| Scene type | Suggested β | What it feels like |
|---|---|---|
| Courtroom cross-examination | 0.0 – 0.2 | Every counter bites; high-stakes, adversarial |
| Formal meeting | 0.3 – 0.5 | Professional, some flexibility |
| Family dinner | 0.4 – 0.7 | Warm, counters often waved off |
| Boardroom cordiality | 0.7 – 0.9 | Diplomatic, few hard rejections |
| Celebration / reception | 0.9 – 1.0 | Everyone's agreeing, dispute is a faux pas |

## Step 2: Set at scene start

```rust
use argumentation_weighted::types::Budget;

state.set_intensity(Budget::new(0.35).unwrap());  // formal meeting
```

## Step 3: Escalate mid-scene

If the scene turns tense (say, after a specific beat), raise β *down*:

```rust
// After a confrontational beat, raise tension (lower β)
state.set_intensity(Budget::new(0.1).unwrap());
```

`set_intensity` takes `&self`, so you can mutate intensity without holding `&mut state` — useful when the state is borrowed by the bridge scorer/eval.

## Step 4: De-escalate

After a reconciliation beat:

```rust
state.set_intensity(Budget::new(0.6).unwrap());
```

## When NOT to use β

If the scene's whole point is that a specific attack *must* bind (a dramatic reveal, a strict rejection), don't set β above that attack's weight. The structural force is the point.

## Verify

Run the scene twice with β at 0.0 and at 0.8 — the number of rejected beats should decrease notably at the higher β.

## Related

- [Weighted and β concept](/concepts/weighted-and-beta) — the mechanics.
- [Thermostat demo](/examples/thermostat) — β-modulated scene in action.
