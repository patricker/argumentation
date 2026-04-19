# encounter-argumentation

Bridge crate connecting encounter's social-interaction engine with the
`argumentation` stack.

## What's in the box

### v0.1.x (preserved)
- `resolve_argument` — pairwise ASPIC+ resolution between proposer and
  responder scheme instances.
- `ArgumentAcceptanceEval` — `AcceptanceEval` impl that uses
  argumentation to decide encounter action acceptance.
- `SchemeActionScorer` — wraps an existing `ActionScorer` and boosts
  scores for scheme-backed affordances.
- `ArgumentKnowledge` / `StaticKnowledge` — per-character argumentation
  capabilities.
- `critical_question_beats`, `cq_to_beat` — CQ → encounter Beat mapping.
- `scheme_value_argument` — value-based scheme construction helper.

### v0.2.0 additions — the full-stack state API

`EncounterArgumentationState` composes all four argumentation crates
(schemes, bipolar, weighted, weighted-bipolar) under one encounter-
friendly surface. Use it when you want coalition structure, weighted
attack strength, or a scene-intensity budget — anything beyond pairwise
ASPIC+ resolution.

```rust
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter_argumentation::{ArgumentId, EncounterArgumentationState};

let registry = default_catalog();
let expert = registry.by_key("argument_from_expert_opinion").unwrap();
let instance = expert.instantiate(&[
    ("expert".into(), "alice".into()),
    ("domain".into(), "military".into()),
    ("claim".into(), "fortify_east".into()),
].into_iter().collect()).unwrap();

let mut state = EncounterArgumentationState::new(registry)
    .at_intensity(Budget::new(0.4).unwrap());
let alice_arg = state.add_scheme_instance("alice", instance);
state.add_weighted_attack(&ArgumentId::new("bob_counter"), &alice_arg, 0.3).unwrap();

assert!(state.is_credulously_accepted(&alice_arg).unwrap());

for coalition in state.coalitions().unwrap() {
    println!("coalition size {} members {:?}", coalition.members.len(), coalition.members);
}
```

### Relationship modulation (Phase-A stub)

`RelationshipWeightSource` provides a default mapping from relationship
dimensions (trust, fear, respect, attraction, friendship) to attack
weights. Phase A ships a placeholder `RelationshipSnapshot` type; Phase
C will replace it with a societas adapter.

## Architecture

- Bridge depends on sibling crates via path: `encounter`, `argumentation`,
  `argumentation-schemes`, `argumentation-bipolar`, `argumentation-weighted`,
  `argumentation-weighted-bipolar`.
- `EncounterArgumentationState` internally owns a
  `WeightedBipolarFramework<ArgumentId>`. `ArgumentId` is a newtype
  over the literal's string rendering, so scheme instances with
  identical conclusions converge on a single argument node.
- The existing `resolve_argument` path is unchanged; it still compiles
  scheme instances into an ASPIC+ `StructuredSystem` and runs Dung
  preferred on the result.
