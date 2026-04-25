# encounter-argumentation

## State-aware bridge (v0.3.0)

Plug an `EncounterArgumentationState` into `encounter`'s protocols via:

```rust,ignore
use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::resolution::MultiBeat;
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};

let state = EncounterArgumentationState::new(default_catalog());
state.set_intensity(Budget::new(0.4).unwrap());

// Seed one scheme instance per (actor, affordance) the scene needs.
// state.add_scheme_instance_for_affordance(actor, name, bindings, instance);

let scorer = StateActionScorer::new(&state, my_inner_scorer, 0.5);
let acceptance = StateAcceptanceEval::new(&state);

let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

// Drain any internal errors that the trait impls couldn't propagate.
for err in state.drain_errors() {
    // handle...
}
```

Scene-intensity (β) lives on the state object and can be mutated with
`state.set_intensity(...)` mid-scene through a shared reference.

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

### Relationship modulation

Societas-aware attack weights live in the **`societas-encounter`** crate
(in the `societas` workspace) under the `argumentation` feature.
`societas_encounter::SocietasRelationshipSource` implements
`argumentation_weighted::WeightSource<ArgumentId>` by reading the five
relationship dimensions from `societas-relations` and applying a
coefficient recipe.

Wiring sketch:

```rust,ignore
use argumentation_weighted::WeightSource;
use societas_core::Tick;
use societas_encounter::{SocietasRelationshipSource, names::StaticNameResolver};
use societas_memory::MemStore;
use societas_relations::RelationshipRegistry;

let store = MemStore::new();                  // any &dyn SocialStore
let registry = RelationshipRegistry::new();
let mut resolver = StaticNameResolver::new();
resolver.add("alice", /* alice's EntityId */);
resolver.add("bob",   /* bob's EntityId   */);

let source = SocietasRelationshipSource::new(
    &registry, &store, &resolver,
    state.actors_by_argument(),
    Tick(0),
);
let w = source.weight_for(&attacker_arg, &target_arg).unwrap();
state.add_weighted_attack(&attacker_arg, &target_arg, w)?;
```

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
