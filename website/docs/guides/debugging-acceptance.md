---
sidebar_position: 8
title: Debug "why didn't this argument get accepted?"
---

When `is_credulously_accepted` returns false (or `evaluate` rejects), it can be hard to know *why* without instrumentation. This guide walks through the diagnostic chain.

**Learning objective:** trace an unexpected acceptance verdict to its cause — missing actor, attack weight too high, β too low, missing scheme instance, or framework cycle — in under 10 minutes.

## Prerequisites

- A scene with `EncounterArgumentationState` and at least one weighted attack.
- The unexpected verdict (acceptance or rejection) in hand.

## Step 1: Inspect the framework directly

Forget the bridge for a moment — query the underlying framework:

```rust
// Is the argument even a node?
println!("argument_count = {}", state.argument_count());
println!("edge_count = {}", state.edge_count());

// Who attacks the argument?
let attackers = state.attackers_of(&target);
println!("attackers of {target}: {attackers:?}");
```

If `attackers` is empty but the argument isn't credulously accepted, the bug is upstream — re-check seeding.

## Step 2: Check actor-to-argument mapping

The bridge keys off `actors_by_argument`. If your responder doesn't appear here, `has_accepted_counter_by` returns false (the responder has nothing to counter with):

```rust
let actors = state.actors_by_argument();
for (arg_id, actor_list) in actors {
    println!("{arg_id}: {actor_list:?}");
}
```

## Step 3: Sweep β

Acceptance shifts as β crosses each attack weight. Print credulous acceptance across the [0, 1] range:

```rust
for beta_pct in 0..=10 {
    let beta = argumentation_weighted::types::Budget::new(beta_pct as f64 / 10.0).unwrap();
    state.set_intensity(beta);
    let accepted = state.is_credulously_accepted(&target).unwrap();
    println!("β={:.1}: accepted = {accepted}", beta_pct as f64 / 10.0);
}
```

If acceptance flips between β=0.3 and β=0.4, your binding attack has weight ≈0.4. The flip point IS the attack weight.

## Step 4: Check the error latch

`StateAcceptanceEval` and `StateActionScorer` can't propagate `Result`, so they latch errors. Always drain after a scene:

```rust
let errors = state.drain_errors();
for err in errors {
    eprintln!("latched: {err}");
}
```

A common entry: `Error::MissingProposerBinding` — the affordance had no `"self"` binding, so the eval defaulted to *accept* and recorded this error.

## Step 5: Check for cycles and self-attacks

Self-attacks are accepted by `add_weighted_attack` but typically indicate a bug. Mutual-attack cycles (Nixon-style) leave both arguments credulous — if you expected one to win, you're missing a tie-breaker.

```rust
// Look for self-attacks:
for atk in attackers {
    if atk == target {
        eprintln!("self-attack on {target}");
    }
}
```

## Step 6: Check the resolver (if using societas weights)

If using `SocietasRelationshipSource`, an unresolvable actor name silently falls back to baseline 0.5. Verify your resolver:

```rust
println!("alice resolves to: {:?}", resolver.resolve("alice"));
println!("bob resolves to: {:?}", resolver.resolve("bob"));
```

`None` means the name isn't registered. `StaticNameResolver::add` panics on reserved names like `"self"` — if you used a reserved name as a character name, the registration would have panicked at setup.

## A diagnostic checklist

Before opening an issue, verify:

- [ ] Argument is a node in the framework (`argument_count > 0`)
- [ ] Actor is in `actors_by_argument` mapping
- [ ] β is reasonable for the scene (not stuck at 0 or 1 by accident)
- [ ] No latched errors after a fresh `drain_errors`
- [ ] No self-attacks unless intentional
- [ ] No undocumented mutual-attack cycles
- [ ] Resolver returns `Some` for the expected actor names

If all checks pass and the verdict is still wrong, file a minimal repro against the [encounter-argumentation crate](https://github.com/patricker/argumentation).

## Related

- [Acceptance semantics](/concepts/semantics) — the credulous/skeptical distinction.
- [Tune β](/guides/tuning-beta) — picking a starting β.
- [Encounter integration](/concepts/encounter-integration) — the trait-impl architecture.
