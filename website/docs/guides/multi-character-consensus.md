---
sidebar_position: 11
title: Query multi-character consensus
---

When a scene has multiple characters with different value priorities, the natural question becomes: "which proposals does the *whole council* agree on?" The `MultiAudience` API answers this — it intersects acceptance across every character's audience.

**Learning objective:** given a `ValueBasedFramework` and two audiences (e.g., for a 2-officer council), compute the unanimously-accepted argument set in 5 lines of code.

## Prerequisites

- The `argumentation-values` crate available.
- A `ValueBasedFramework` (see [Value-based argumentation](/concepts/value-based-argumentation) for setup).

## The use case

Scenes with multiple deliberators — councils, juries, cabinets — often have characters who hold different value priorities. Some proposals will land regardless of who you ask (universal consensus); others depend on whose audience you adopt. `MultiAudience::common_grounded` answers the universal-consensus question.

## Step 1: Build per-character audiences

```rust
use argumentation_values::{Audience, Value};

// Aleric values defending the realm above all.
let aleric = Audience::total([
    Value::new("duty"),
    Value::new("survival"),
    Value::new("comfort"),
]);

// Maren prioritises survival.
let maren = Audience::total([
    Value::new("survival"),
    Value::new("duty"),
    Value::new("comfort"),
]);

let council = [aleric, maren];
```

## Step 2: Construct the MultiAudience query

```rust
use argumentation_values::MultiAudience;

let multi = MultiAudience::new(&council);
```

## Step 3: Query consensus

```rust
let consensus = multi.common_grounded(&vaf)?;

if consensus.is_empty() {
    println!("The council cannot agree on anything universally.");
} else {
    println!("Universally accepted: {:?}", consensus);
}
```

`common_grounded` returns the set of arguments grounded under *every* audience in the slice — the strictest form of consensus.

## Step 4: For weaker consensus, use `common_credulous`

```rust
let weak_consensus = multi.common_credulous(&vaf)?;
```

`common_credulous` returns arguments that are credulously accepted (i.e., in *some* preferred extension) under every audience — a less restrictive answer. Use this when you want "no character could rule this out completely" rather than "every character would defend this."

## Worked example: 4-officer council

```rust
use argumentation_values::{
    Audience, MultiAudience, Value, ValueAssignment, ValueBasedFramework,
};
use argumentation::ArgumentationFramework;

// Build the siege-council framework.
let mut base = ArgumentationFramework::new();
for arg in ["fortify", "abandon", "sortie", "evacuate"] {
    base.add_argument(arg);
}
base.add_attack(&"fortify", &"abandon")?;
base.add_attack(&"abandon", &"fortify")?;
base.add_attack(&"sortie", &"abandon")?;

let mut values = ValueAssignment::new();
values.promote("fortify", Value::new("duty"));
values.promote("abandon", Value::new("survival"));
values.promote("sortie", Value::new("victory"));
values.promote("evacuate", Value::new("survival"));

let vaf = ValueBasedFramework::new(base, values);

// Each officer has their own audience.
let aleric = Audience::total([Value::new("duty"), Value::new("victory"), Value::new("survival")]);
let maren  = Audience::total([Value::new("survival"), Value::new("duty"), Value::new("victory")]);
let drust  = Audience::total([Value::new("victory"), Value::new("duty"), Value::new("survival")]);
let liss   = Audience::total([Value::new("survival"), Value::new("victory"), Value::new("duty")]);

let council = [aleric, maren, drust, liss];
let multi = MultiAudience::new(&council);

let consensus = multi.common_grounded(&vaf)?;
println!("Council consensus: {:?}", consensus);
```

## Empty audience set

`MultiAudience::new(&[])` is a valid construction — empty universal quantifier means every argument is trivially in the consensus set:

```rust
let no_audiences = MultiAudience::new(&[]);
let everything = no_audiences.common_grounded(&vaf)?;
assert_eq!(everything.len(), vaf.base().len());
```

This is rarely useful directly, but it lets the API stay total without special-casing.

## Cost

Each `common_grounded` call costs `k × O(grounded extension)` where `k` is the audience count. Each `common_credulous` call costs `k × O(preferred extensions)` — preferred is exponential in the worst case but tractable for narrative-scale frameworks (≤ 22 args, the upstream `ENUMERATION_LIMIT`).

For very large councils (10+ characters), prefer `common_grounded` over `common_credulous` — the cost gap widens with audience count.

## When NOT to use this

- **Single-character scenes.** Use `vaf.grounded_for(&audience)` directly.
- **Asymmetric votes** (e.g., the king's audience counts double). `MultiAudience` is symmetric — model weighted voting at a higher layer.
- **Probabilistic acceptance.** Not a feature. See [open areas](/concepts/open-areas) for the probabilistic AF roadmap.

## Related

- [Value-based argumentation](/concepts/value-based-argumentation) — the formalism.
- [Wiring per-character values](/guides/wiring-character-values) — for runtime use in encounter scenes.
- [The siege council](/examples/siege-council) — multi-actor flagship demo (β + climate version; audience-aware version is a future enhancement).
- [`MultiAudience` rustdoc](/api/argumentation_values/multi/struct.MultiAudience.html).
