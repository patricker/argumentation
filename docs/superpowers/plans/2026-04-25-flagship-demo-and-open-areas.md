# Flagship Demo + Open Areas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a flagship engine-driven scene (Siege Council) for the docs site, promote Hal & Carla to a second engine-driven scene, and ship a scoping doc for "open areas" with a deeper mini-RFC for value-based argumentation (VAF).

**Architecture:** Reuses the existing `tools/scene-tracer` binary to generate JSON traces, the existing `BetaSlider` / `BetaPlayground` / `AttackGraph` components to render them, and Docusaurus pages to host. New work is additive — no refactors of shipped APIs. The siege fixture introduces a per-scene `ActionScorer` impl pattern (the existing scorer was hardcoded to east-wall) which becomes the template for future scenes.

**Tech Stack:** Rust 2024 edition (`scene-tracer` binary + `argumentation-*` + `encounter-argumentation`), TypeScript / React / Docusaurus 3 (website), no new crates, no new dependencies.

---

## Scope notes

This plan is **three phases** and each phase ships independently. You can stop after Phase 1 and have a flagship demo on the site; stop after Phase 2 and additionally have a second live scene; Phase 3 is documentation only.

**Out of scope** (separate future plans):
- Cargo `examples/*.rs` per crate (`cargo run --example east_wall`) — needs a CI story
- Generating + publishing rustdoc to fix the `/api/` 404s — release infrastructure
- Actually implementing `argumentation-values` (the VAF crate) — Phase 3 produces the *scoping doc*, not the implementation
- Publishing crates to crates.io

**Existing patterns this plan follows:**
- `tools/scene-tracer/src/main.rs:85-193` — `trace_east_wall` is the canonical fixture pattern
- `website/static/traces/east-wall-b{00,04,05,10}.json` — the canonical trace file naming
- `website/docs/examples/east-wall.mdx` — the canonical engine-driven docs page shape (intro → BetaPlayground → BetaSlider → trace explanation)
- `website/sidebars.ts` — three categories: "Worked examples (literature)", "Engine-driven scenes", "Interactive"

---

## File structure

**Files this plan creates:**

| Path | Responsibility |
|---|---|
| `tools/scene-tracer/src/scenes/siege_council.rs` | Siege council fixture (cold + warm modes), per-scene scorer |
| `tools/scene-tracer/src/scenes/hal_carla.rs` | Hal & Carla 4-argument fixture, per-scene scorer |
| `tools/scene-tracer/src/scenes/mod.rs` | Module barrel, re-exports |
| `website/static/traces/siege-cold-b{00,03,05,08}.json` | 4 cold-mode traces |
| `website/static/traces/siege-warm-b{00,03,05,08}.json` | 4 warm-mode traces |
| `website/static/traces/hal-carla-b{00,03,05,10}.json` | 4 Hal & Carla traces |
| `website/docs/examples/siege-council.mdx` | Flagship demo page |
| `website/docs/concepts/open-areas.mdx` | Overview of 5 open formalisms not yet implemented |
| `website/docs/concepts/value-based-argumentation.mdx` | VAF mini-RFC (the deeper one) |

**Files this plan modifies:**

| Path | Change |
|---|---|
| `tools/scene-tracer/src/main.rs` | Add `mod scenes;` + new match arms for 3 new scenes; existing east-wall code moves into `scenes::east_wall` for consistency |
| `tools/scene-tracer/src/scenes/east_wall.rs` | New file — extracted from main.rs (preserves existing behavior; this is a refactor done once at the start so all three scenes live in the same module shape) |
| `website/docs/examples/hal-and-carla.mdx` | Add live BetaPlayground + BetaSlider sections; keep prose intact |
| `website/sidebars.ts` | Add siege-council (top of "Engine-driven scenes"), move hal-and-carla, add 2 concepts pages |
| `website/docs/examples/_category_.json` | Update description from "(east-wall)" singular to plural list |
| `website/src/pages/index.tsx:18` | CTA link change `/examples/east-wall` → `/examples/siege-council` |
| `website/docs/concepts/_category_.json` (if exists) | Possibly reorder; check during Task 9 |

---

## Phase 1: Flagship — Siege Council

The scene: Four officers convene to decide the response to a frontier siege. Each has expertise (military, logistics, intelligence, civilian liaison) and a position. The attack graph between proposals is fixed; the *weights* on those attacks reflect the relationship climate at the table — **cold mode** is full bickering officers (full weights), **warm mode** is trusting officers (weights × 0.5). The scene runs as a multi-beat round-robin with up to 8 beats. The user explores both axes (β × relationship climate) on the page.

**Why this scene:** It uses 4 actors instead of 2, which exercises the multi-actor case the docs currently underweight. It shows two independent dials (β and weights) flipping outcomes, which is the strongest possible demonstration of "this is a real engine, not an animation." The "warm vs cold" framing motivates the [societas-modulated weights guide](../../../website/docs/guides/societas-modulated-weights.md) without requiring societas to actually be in the loop — the relationship state is baked into the trace's attack weights, which is faithful to how `SocietasRelationshipSource` works.

### Task 1: Refactor scene-tracer into a `scenes` module

Establishes the module shape that the next two scenes will follow. Pure refactor — no behavior change. Verified by running the existing east-wall command and diffing JSON output.

**Files:**
- Create: `tools/scene-tracer/src/scenes/mod.rs`
- Create: `tools/scene-tracer/src/scenes/east_wall.rs`
- Modify: `tools/scene-tracer/src/main.rs`

- [ ] **Step 1: Capture baseline output**

```bash
cd /home/peter/code/argumentation
cargo run -p scene-tracer -- east-wall 0.5 /tmp/east-wall-b05-baseline.json
```

Expected: `wrote /tmp/east-wall-b05-baseline.json`

- [ ] **Step 2: Create `tools/scene-tracer/src/trace.rs`**

Extract the shared `Trace` / `SeededArg` / `AttackEdge` / `BeatRecord` types so all three scene fixtures can use them without depending on `main`. (Created first because `east_wall.rs` in the next step `use`s these types.)

```rust
//! Shared trace types serialised by every scene fixture.

use serde::Serialize;

#[derive(Serialize)]
pub struct Trace {
    pub scene_name: String,
    pub beta: f64,
    pub participants: Vec<String>,
    pub seeded_arguments: Vec<SeededArg>,
    pub attacks: Vec<AttackEdge>,
    pub beats: Vec<BeatRecord>,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct SeededArg {
    pub actor: String,
    pub affordance_name: String,
    pub conclusion: String,
}

#[derive(Serialize)]
pub struct AttackEdge {
    pub attacker: String,
    pub target: String,
    pub weight: f64,
}

#[derive(Serialize)]
pub struct BeatRecord {
    pub actor: String,
    pub action: String,
    pub accepted: bool,
}
```

- [ ] **Step 3: Create `scenes/east_wall.rs` with extracted fixture**

```rust
//! East-wall fixture — two-actor weighted attack scene.
//!
//! Migrated from main.rs as part of the move to a per-scene module layout.

use crate::trace::{AttackEdge, BeatRecord, SeededArg, Trace};
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

struct EastWallScorer;

impl<P: Clone> ActionScorer<P> for EastWallScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available
            .iter()
            .map(|e| {
                let mut bindings = HashMap::new();
                bindings.insert("self".into(), actor.to_string());
                let claim = if e.spec.name == "argue_fortify_east" {
                    "fortify_east"
                } else {
                    "abandon_east"
                };
                bindings.insert("claim".into(), claim.into());
                bindings.insert("expert".into(), actor.to_string());
                bindings.insert("domain".into(), "military".into());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

pub fn trace(beta: f64) -> Trace {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".into(), "alice".into());
    alice_b.insert("domain".into(), "military".into());
    alice_b.insert("claim".into(), "fortify_east".into());
    let alice_instance = scheme.instantiate(&alice_b).unwrap();

    let mut bob_b = HashMap::new();
    bob_b.insert("expert".into(), "bob".into());
    bob_b.insert("domain".into(), "logistics".into());
    bob_b.insert("claim".into(), "abandon_east".into());
    let bob_instance = scheme.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".into(), "alice".into());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice",
        "argue_fortify_east",
        &alice_af,
        alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".into(), "bob".into());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob",
        "argue_abandon_east",
        &bob_af,
        bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "persuasion".into(),
        bindings: vec![
            "self".into(),
            "expert".into(),
            "domain".into(),
            "claim".into(),
        ],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let catalog = vec![
        CatalogEntry {
            spec: make_spec("argue_fortify_east"),
            precondition: String::new(),
        },
        CatalogEntry {
            spec: make_spec("argue_abandon_east"),
            precondition: String::new(),
        },
    ];
    let practice = PracticeSpec {
        name: "debate".into(),
        affordances: vec![
            "argue_fortify_east".into(),
            "argue_abandon_east".into(),
        ],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, EastWallScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".into(), "bob".into()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

    Trace {
        scene_name: "east_wall".into(),
        beta,
        participants,
        seeded_arguments: vec![
            SeededArg {
                actor: "alice".into(),
                affordance_name: "argue_fortify_east".into(),
                conclusion: "fortify_east".into(),
            },
            SeededArg {
                actor: "bob".into(),
                affordance_name: "argue_abandon_east".into(),
                conclusion: "abandon_east".into(),
            },
        ],
        attacks: vec![AttackEdge {
            attacker: "abandon_east".into(),
            target: "fortify_east".into(),
            weight: 0.4,
        }],
        beats: result
            .beats
            .iter()
            .map(|b| BeatRecord {
                actor: b.actor.clone(),
                action: b.action.clone(),
                accepted: b.accepted,
            })
            .collect(),
        errors: state.drain_errors().iter().map(|e| e.to_string()).collect(),
    }
}
```

- [ ] **Step 4: Create `scenes/mod.rs` barrel**

The `hal_carla` and `siege_council` modules are added in Tasks 2 and 5 respectively — including them now would break the build. Start with `east_wall` only:

```rust
pub mod east_wall;
```

- [ ] **Step 5: Replace `tools/scene-tracer/src/main.rs` with a slim dispatch**

```rust
//! scene-tracer: pre-renders argumentation scenes to JSON for the website.
//!
//! Usage:
//!   cargo run -p scene-tracer -- east-wall 0.5 website/static/traces/east-wall-b05.json
//!   cargo run -p scene-tracer -- siege-cold 0.5 website/static/traces/siege-cold-b05.json
//!   cargo run -p scene-tracer -- siege-warm 0.5 website/static/traces/siege-warm-b05.json
//!   cargo run -p scene-tracer -- hal-carla 0.5 website/static/traces/hal-carla-b05.json

mod scenes;
mod trace;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: scene-tracer <scene> <beta> <out-path>");
        std::process::exit(2);
    }
    let beta: f64 = args[2].parse().expect("beta must be a float");
    let trace = match args[1].as_str() {
        "east-wall" => scenes::east_wall::trace(beta),
        // siege-cold / siege-warm added in Task 2; hal-carla added in Task 5.
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
    let json = serde_json::to_string_pretty(&trace).unwrap();
    fs::write(&args[3], json).expect("write failed");
    println!("wrote {}", args[3]);
}
```

- [ ] **Step 6: Build to verify the refactor compiles**

```bash
cd /home/peter/code/argumentation
cargo build -p scene-tracer 2>&1 | tail -20
```

Expected: `Finished ... target(s)` — no errors.

- [ ] **Step 7: Regenerate east-wall trace and diff against baseline**

```bash
cargo run -p scene-tracer -- east-wall 0.5 /tmp/east-wall-b05-after.json
diff /tmp/east-wall-b05-baseline.json /tmp/east-wall-b05-after.json
```

Expected: no diff. The refactor is byte-equivalent.

- [ ] **Step 8: Commit**

```bash
git add tools/scene-tracer/
git commit -m "refactor(scene-tracer): split scenes into per-file modules

Pure refactor in preparation for adding siege-council and hal-carla
scenes. East-wall trace output is byte-equivalent."
```

---

### Task 2: Add cold siege council fixture

**Files:**
- Create: `tools/scene-tracer/src/scenes/siege_council.rs`
- Modify: `tools/scene-tracer/src/scenes/mod.rs`
- Modify: `tools/scene-tracer/src/main.rs`

- [ ] **Step 1: Write `scenes/siege_council.rs`**

```rust
//! Siege council — four officers debate the response to a frontier siege.
//!
//! Two relationship climates: `Cold` (officers bickering, full attack weights)
//! and `Warm` (officers cooperating, weights halved). The attack graph itself
//! is identical — only the weights differ. This is faithful to how
//! `SocietasRelationshipSource` modulates attack weights via relationship
//! state in production wiring.

use crate::trace::{AttackEdge, BeatRecord, SeededArg, Trace};
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

#[derive(Clone, Copy)]
pub enum Climate {
    Cold,
    Warm,
}

impl Climate {
    fn weight_multiplier(self) -> f64 {
        match self {
            Climate::Cold => 1.0,
            Climate::Warm => 0.5,
        }
    }

    fn scene_name(self) -> &'static str {
        match self {
            Climate::Cold => "siege_council_cold",
            Climate::Warm => "siege_council_warm",
        }
    }
}

/// (actor, claim, expert-domain) for each officer.
const OFFICERS: &[(&str, &str, &str)] = &[
    ("aleric", "fortify", "military"),
    ("maren", "abandon", "logistics"),
    ("drust", "sortie", "intelligence"),
    ("liss", "evacuate_first", "civilian"),
];

/// (attacker_actor, target_actor, base_weight) — base weights are scaled by
/// climate. Edges:
///   - maren->aleric: "we have no supplies for a siege"
///   - aleric->maren: "retreat exposes the civilians we are sworn to protect"
///   - drust->maren: "the enemy is thinner than we feared; retreat is unnecessary"
///   - liss->aleric: "civilians need time to evacuate before fortification"
///   - drust->liss:  "there is no time for a full evacuation"
const ATTACKS: &[(&str, &str, f64)] = &[
    ("maren", "aleric", 0.5),
    ("aleric", "maren", 0.4),
    ("drust", "maren", 0.6),
    ("liss", "aleric", 0.3),
    ("drust", "liss", 0.5),
];

struct SiegeScorer;

impl<P: Clone> ActionScorer<P> for SiegeScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available
            .iter()
            .map(|e| {
                let mut bindings = HashMap::new();
                bindings.insert("self".into(), actor.to_string());
                bindings.insert("expert".into(), actor.to_string());
                let (claim, domain) = match e.spec.name.as_str() {
                    "argue_fortify" => ("fortify", "military"),
                    "argue_abandon" => ("abandon", "logistics"),
                    "argue_sortie" => ("sortie", "intelligence"),
                    "argue_evacuate_first" => ("evacuate_first", "civilian"),
                    other => panic!("unexpected affordance: {other}"),
                };
                bindings.insert("claim".into(), claim.into());
                bindings.insert("domain".into(), domain.into());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

pub fn trace(beta: f64, climate: Climate) -> Trace {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut ids: HashMap<&str, _> = HashMap::new();

    for (actor, claim, domain) in OFFICERS {
        let mut b = HashMap::new();
        b.insert("expert".into(), (*actor).into());
        b.insert("domain".into(), (*domain).into());
        b.insert("claim".into(), (*claim).into());
        let instance = scheme.instantiate(&b).unwrap();
        let mut af = b.clone();
        af.insert("self".into(), (*actor).into());
        let id = state.add_scheme_instance_for_affordance(
            actor,
            &format!("argue_{claim}"),
            &af,
            instance,
        );
        ids.insert(*actor, id);
    }

    let mult = climate.weight_multiplier();
    let mut attack_edges = Vec::new();
    for (atk, tgt, base_w) in ATTACKS {
        let w = (base_w * mult).min(1.0);
        state.add_weighted_attack(&ids[atk], &ids[tgt], w).unwrap();
        // Track the conclusion-level attack edge for the trace JSON.
        let atk_claim = OFFICERS.iter().find(|(a, _, _)| a == atk).unwrap().1;
        let tgt_claim = OFFICERS.iter().find(|(a, _, _)| a == tgt).unwrap().1;
        attack_edges.push(AttackEdge {
            attacker: atk_claim.into(),
            target: tgt_claim.into(),
            weight: w,
        });
    }

    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "council".into(),
        bindings: vec![
            "self".into(),
            "expert".into(),
            "domain".into(),
            "claim".into(),
        ],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let aff_names = [
        "argue_fortify",
        "argue_abandon",
        "argue_sortie",
        "argue_evacuate_first",
    ];
    let catalog: Vec<_> = aff_names
        .iter()
        .map(|n| CatalogEntry {
            spec: make_spec(n),
            precondition: String::new(),
        })
        .collect();
    let practice = PracticeSpec {
        name: "council".into(),
        affordances: aff_names.iter().map(|s| (*s).to_string()).collect(),
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 8 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, SiegeScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants: Vec<String> =
        OFFICERS.iter().map(|(a, _, _)| (*a).to_string()).collect();
    let result = MultiBeat.resolve(
        &participants,
        &practice,
        &catalog,
        &scorer,
        &acceptance,
    );

    let seeded_arguments = OFFICERS
        .iter()
        .map(|(actor, claim, _)| SeededArg {
            actor: (*actor).to_string(),
            affordance_name: format!("argue_{claim}"),
            conclusion: (*claim).to_string(),
        })
        .collect();

    Trace {
        scene_name: climate.scene_name().to_string(),
        beta,
        participants,
        seeded_arguments,
        attacks: attack_edges,
        beats: result
            .beats
            .iter()
            .map(|b| BeatRecord {
                actor: b.actor.clone(),
                action: b.action.clone(),
                accepted: b.accepted,
            })
            .collect(),
        errors: state.drain_errors().iter().map(|e| e.to_string()).collect(),
    }
}
```

- [ ] **Step 2: Wire `siege_council` into `scenes/mod.rs`**

```rust
pub mod east_wall;
pub mod siege_council;
```

- [ ] **Step 3: Wire `siege-cold` and `siege-warm` into the main dispatch**

Edit `tools/scene-tracer/src/main.rs`, replacing the match:

```rust
    let trace = match args[1].as_str() {
        "east-wall" => scenes::east_wall::trace(beta),
        "siege-cold" => scenes::siege_council::trace(beta, scenes::siege_council::Climate::Cold),
        "siege-warm" => scenes::siege_council::trace(beta, scenes::siege_council::Climate::Warm),
        // hal-carla added in Task 5.
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
```

- [ ] **Step 4: Build**

```bash
cd /home/peter/code/argumentation
cargo build -p scene-tracer 2>&1 | tail -10
```

Expected: `Finished` — no errors.

- [ ] **Step 5: Smoke-test cold mode**

```bash
cargo run -p scene-tracer -- siege-cold 0.5 /tmp/siege-cold-test.json
python3 -c "import json; t = json.load(open('/tmp/siege-cold-test.json')); print(t['scene_name'], len(t['beats']), 'beats,', len(t['attacks']), 'attacks,', len(t['errors']), 'errors')"
```

Expected: `siege_council_cold N beats, 5 attacks, 0 errors` (where N ≤ 8).

- [ ] **Step 6: Generate the four cold traces**

```bash
mkdir -p website/static/traces
cargo run -p scene-tracer -- siege-cold 0.0 website/static/traces/siege-cold-b00.json
cargo run -p scene-tracer -- siege-cold 0.3 website/static/traces/siege-cold-b03.json
cargo run -p scene-tracer -- siege-cold 0.5 website/static/traces/siege-cold-b05.json
cargo run -p scene-tracer -- siege-cold 0.8 website/static/traces/siege-cold-b08.json
```

Expected: four `wrote ...` lines.

- [ ] **Step 7: Generate the four warm traces**

```bash
cargo run -p scene-tracer -- siege-warm 0.0 website/static/traces/siege-warm-b00.json
cargo run -p scene-tracer -- siege-warm 0.3 website/static/traces/siege-warm-b03.json
cargo run -p scene-tracer -- siege-warm 0.5 website/static/traces/siege-warm-b05.json
cargo run -p scene-tracer -- siege-warm 0.8 website/static/traces/siege-warm-b08.json
```

Expected: four `wrote ...` lines.

- [ ] **Step 8: Sanity-check that cold and warm produce different outcomes at a meaningful β**

```bash
python3 -c "
import json
cold = json.load(open('website/static/traces/siege-cold-b03.json'))
warm = json.load(open('website/static/traces/siege-warm-b03.json'))
cold_accept = sum(1 for b in cold['beats'] if b['accepted'])
warm_accept = sum(1 for b in warm['beats'] if b['accepted'])
print(f'cold accepted at β=0.3: {cold_accept}/{len(cold[\"beats\"])}')
print(f'warm accepted at β=0.3: {warm_accept}/{len(warm[\"beats\"])}')
assert cold_accept != warm_accept, 'Climate mode should change outcomes'
print('OK — climate flips outcomes')
"
```

Expected: `OK — climate flips outcomes`. If the assertion fails, the base attack weights need re-tuning before proceeding to Task 3 — this is the proof-of-concept that the flagship demo's headline claim ("relationships flip outcomes") actually holds.

If the assert fails, before changing weights, also check β=0.0 (where `Cold` should reject more than `Warm`). If neither β=0.0 nor β=0.3 differentiates, increase the base weight on `drust→maren` (currently 0.6) toward 0.7 — that is the most discriminating edge.

- [ ] **Step 9: Commit**

```bash
git add tools/scene-tracer/ website/static/traces/siege-*.json
git commit -m "feat(scene-tracer): add siege-council fixture (cold + warm climates)

Four-officer council debating siege response. Identical attack graph
across both climates; weights are scaled by 1.0 (cold) or 0.5 (warm)
to model how relationship state would modulate attack strength via
SocietasRelationshipSource. Eight pre-rendered traces shipped for the
flagship demo page."
```

---

### Task 3: Build the flagship `siege-council.mdx` page

**Files:**
- Create: `website/docs/examples/siege-council.mdx`
- Modify: `website/docs/examples/_category_.json`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/examples/siege-council.mdx`**

```mdx
---
sidebar_position: 1
title: The siege council (flagship)
---

import AttackGraph from '@site/src/components/AttackGraph';
import BetaPlayground from '@site/src/components/BetaPlayground';
import BetaSlider from '@site/src/components/BetaSlider';

> Four officers convene to decide the response to a frontier siege. Aleric, the commander, wants to fortify. Maren, in charge of supply, says the keep cannot survive a siege and the garrison must abandon the position. Drust, head of intelligence, has scouted the enemy and believes a sortie attack will succeed. Liss, the civilian liaison, demands evacuation of non-combatants before any military commitment. Each presses their case; the framework decides which proposal stands at the end of the council.

The flagship engine-driven demo. Two independent dials shape the outcome — **β** (scene intensity) and **the relationship climate** between the officers. The attack graph between proposals is fixed across both climates; only the *strength* of the attacks shifts. This is faithful to how [`SocietasRelationshipSource`](/guides/societas-modulated-weights) modulates attack weights from live relationship state in a real societas-wired scene.

## The proposals and how they attack each other

<AttackGraph
  title="Siege council — argument graph"
  arguments={[
    {id: 'F', label: 'Aleric: fortify the keep'},
    {id: 'A', label: 'Maren: abandon the position'},
    {id: 'S', label: 'Drust: sortie attack'},
    {id: 'E', label: 'Liss: evacuate civilians first'},
  ]}
  attacks={[
    {from: 'A', to: 'F', weight: 0.5},
    {from: 'F', to: 'A', weight: 0.4},
    {from: 'S', to: 'A', weight: 0.6},
    {from: 'E', to: 'F', weight: 0.3},
    {from: 'S', to: 'E', weight: 0.5},
  ]}
  height={420}
  caption="Five attacks, one mutual pair (F↔A) and three asymmetric. Drust's sortie pushes against both Maren's retreat and Liss's evacuation — they cost time he claims the garrison doesn't have. Edge weights shown are the *cold-climate* values; warm-climate weights are halved."
/>

## Drag β to see the council shift

<BetaPlayground
  title="Cold climate — full attack weights"
  args={[
    {id: 'F', label: 'Aleric: fortify'},
    {id: 'A', label: 'Maren: abandon'},
    {id: 'S', label: 'Drust: sortie'},
    {id: 'E', label: 'Liss: evacuate first'},
  ]}
  attacks={[
    {from: 'A', to: 'F', weight: 0.5},
    {from: 'F', to: 'A', weight: 0.4},
    {from: 'S', to: 'A', weight: 0.6},
    {from: 'E', to: 'F', weight: 0.3},
    {from: 'S', to: 'E', weight: 0.5},
  ]}
  initialBeta={0}
/>

<BetaPlayground
  title="Warm climate — weights halved (officers trust each other)"
  args={[
    {id: 'F', label: 'Aleric: fortify'},
    {id: 'A', label: 'Maren: abandon'},
    {id: 'S', label: 'Drust: sortie'},
    {id: 'E', label: 'Liss: evacuate first'},
  ]}
  attacks={[
    {from: 'A', to: 'F', weight: 0.25},
    {from: 'F', to: 'A', weight: 0.2},
    {from: 'S', to: 'A', weight: 0.3},
    {from: 'E', to: 'F', weight: 0.15},
    {from: 'S', to: 'E', weight: 0.25},
  ]}
  initialBeta={0}
/>

The two playgrounds run the same `argumentation` Rust crate compiled to WebAssembly, on the exact same attack topology. Only the weights differ — and the credulous-acceptance pattern shifts dramatically. At low β, the cold council resolves to a single survivor; the warm council leaves multiple proposals standing because no attack is strong enough to bind. This is the relationship-modulation story made tangible.

## A pre-recorded multi-beat trace at four discrete β

The snapshots below come from running the full encounter bridge with `MultiBeat` resolution. Each beat shows which officer spoke and whether their proposal was accepted by the responder.

### Cold climate

<BetaSlider
  title="Cold council across β"
  tracePaths={[
    {beta: 0.0, path: '/traces/siege-cold-b00.json'},
    {beta: 0.3, path: '/traces/siege-cold-b03.json'},
    {beta: 0.5, path: '/traces/siege-cold-b05.json'},
    {beta: 0.8, path: '/traces/siege-cold-b08.json'},
  ]}
/>

### Warm climate

<BetaSlider
  title="Warm council across β"
  tracePaths={[
    {beta: 0.0, path: '/traces/siege-warm-b00.json'},
    {beta: 0.3, path: '/traces/siege-warm-b03.json'},
    {beta: 0.5, path: '/traces/siege-warm-b05.json'},
    {beta: 0.8, path: '/traces/siege-warm-b08.json'},
  ]}
/>

## What's happening

At **β = 0** under the cold climate, every attack binds. Most proposals have a live attacker, so they aren't credulously accepted, and the council can't agree on much — the bridge rejects most affordances. The warm climate at the same β survives more proposals because the same edges, halved, are sometimes already droppable.

As **β rises**, weaker attacks become droppable in order of weight. The 0.3-weight `evacuate-attacks-fortify` edge drops at β = 0.3 (cold) or earlier (warm). The 0.6-weight `sortie-attacks-abandon` edge holds longest. By **β = 0.8**, only the strongest attacks remain — the council has high tolerance for "you both have a point" and most proposals can stand.

The **relationship climate** acts as a second dial running in the orthogonal direction. A council of officers who trust each other (warm) reaches a multi-proposal consensus much earlier than one made up of rivals (cold), even at identical scene intensity.

## Reading the outcome

Two parameters, qualitatively different roles:

- **β** is *the storyteller's dial* — turn it down for sharp scenes where every objection lands, up for cordial scenes where everyone tolerates dissent.
- **Relationship weights** are *the world's state* — set by societas-relations modifiers, simulation history, or hand-authored by the storyteller per scene.

The siege council is the smallest scene that exercises both: the static graph is a reasonable defense council; the dial settings are the tone the storyteller picks; the resolution is what the engine commits to.

## How this scene is wired

The pre-recorded traces come from `tools/scene-tracer/src/scenes/siege_council.rs`. It instantiates the four officers as `argument_from_expert_opinion` scheme instances (commander/military, logistics/supply, intelligence/scouting, civilian-liaison/duty), wires them into an `EncounterArgumentationState`, sets β with `set_intensity`, and resolves via `encounter::resolution::MultiBeat` with `StateActionScorer` and `StateAcceptanceEval`.

The cold and warm modes share an identical attack topology — the cold-mode weights are listed at the top of the `siege_council.rs` file, and warm mode multiplies all weights by 0.5. In a societas-wired production scene you would replace this static multiplier with [`SocietasRelationshipSource`](/guides/societas-modulated-weights), which derives the per-edge weight from the live trust/fear/respect/etc. dimensions between the asserting actors.

## Further reading

- [The encounter integration concept](/concepts/encounter-integration) — how the bridge composes with your scene engine.
- [β as scene intensity](/concepts/weighted-and-beta) — the mechanics of the dial.
- [Modulate attack weights with societas](/guides/societas-modulated-weights) — wire the relationship-climate variation to live state.
- [The first-scene guide](/getting-started/first-scene) — build your own version, two-actor edition.
- [The east wall](/examples/east-wall) — the simpler two-actor sibling of this scene.
```

- [ ] **Step 2: Update `_category_.json` description**

Read the current file:

```bash
cat /home/peter/code/argumentation/website/docs/examples/_category_.json
```

Replace its contents with:

```json
{
  "label": "Examples",
  "position": 3,
  "link": {
    "type": "generated-index",
    "description": "Three flavors. **Worked examples** (Nixon, Tweety, courtroom) walk through canonical literature scenarios as conceptual stories — they're educational, not always runnable. **Engine-driven scenes** (siege council, east wall, Hal & Carla) wire a real `MultiBeat` resolution against the bridge with replayable JSON traces. **Interactive** (playground) compiles the `argumentation` crate to WebAssembly so you can build frameworks live in your browser."
  }
}
```

(The Hal & Carla mention anticipates Task 5; if that task is deferred, edit accordingly. Marking this file's edit here ensures the wiring is correct *if* Task 5 lands.)

- [ ] **Step 3: Update `website/sidebars.ts` Engine-driven scenes category**

Replace the `examplesSidebar` block (currently around lines 18-39 of sidebars.ts):

```typescript
  examplesSidebar: [
    {
      type: 'category',
      label: 'Engine-driven scenes',
      items: [
        'examples/siege-council',
        'examples/east-wall',
      ],
    },
    {
      type: 'category',
      label: 'Worked examples (literature)',
      items: [
        'examples/nixon-diamond',
        'examples/tweety-penguin',
        'examples/hal-and-carla',
        'examples/courtroom',
      ],
    },
    {
      type: 'category',
      label: 'Interactive',
      items: ['examples/playground'],
    },
  ],
```

(Hal & Carla is moved out of "Worked examples" into "Engine-driven scenes" in Task 7. Until then it stays in literature.)

- [ ] **Step 4: Build the website**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -30
```

Expected: `[SUCCESS] Generated static files in "build".` Warnings about `/api/` 404s are pre-existing and acceptable. **Stop and investigate** any new warning that mentions `siege-council`, missing imports, or trace JSON 404s.

- [ ] **Step 5: Eyeball the page locally**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus serve --no-open --port 3001 &
sleep 2
curl -s http://localhost:3001/examples/siege-council | grep -c "siege council\|Siege Council\|Aleric"
kill %1 2>/dev/null
```

Expected: a non-zero count (the page is being served and contains the expected words).

- [ ] **Step 6: Commit**

```bash
git add website/docs/examples/siege-council.mdx \
        website/docs/examples/_category_.json \
        website/sidebars.ts
git commit -m "docs(examples): add flagship siege-council page

Four-officer multi-beat scene with cold/warm climate variants. Both
live (BetaPlayground) and pre-rendered (BetaSlider) interactives.
Promoted to top of Engine-driven scenes category."
```

---

### Task 4: Make siege council the homepage CTA

**Files:**
- Modify: `website/src/pages/index.tsx`

- [ ] **Step 1: Read current state**

```bash
sed -n '15,25p' /home/peter/code/argumentation/website/src/pages/index.tsx
```

Expected: shows the `<Link ... to="/examples/east-wall">Try the east-wall example</Link>` line.

- [ ] **Step 2: Replace the CTA**

Edit `website/src/pages/index.tsx`. Replace:

```tsx
          <Link className="button button--outline button--lg" to="/examples/east-wall">
            Try the east-wall example
          </Link>
```

With:

```tsx
          <Link className="button button--outline button--lg" to="/examples/siege-council">
            Try the flagship demo
          </Link>
```

- [ ] **Step 3: Rebuild and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
git add website/src/pages/index.tsx
git commit -m "docs(home): point flagship CTA to siege-council demo"
```

---

## Phase 2: Promote Hal & Carla to engine-driven

Hal & Carla is currently a conceptual page with a static AttackGraph. It's a four-argument scene with two mutual-attack pairs and one asymmetric — perfect material for an engine-driven version, *and* it sets up Phase 3's VAF discussion (Hal & Carla is Bench-Capon's canonical VAF example). Promoting it now means the conceptual prose stays, the live components add a layer, and the page becomes the bridge into the open-areas exploration.

### Task 5: Add Hal & Carla fixture

**Files:**
- Create: `tools/scene-tracer/src/scenes/hal_carla.rs`
- Modify: `tools/scene-tracer/src/scenes/mod.rs`
- Modify: `tools/scene-tracer/src/main.rs`

- [ ] **Step 1: Write `scenes/hal_carla.rs`**

```rust
//! Hal & Carla — Bench-Capon's canonical value-based example, run as an
//! abstract weighted framework. The current implementation does not yet
//! support values explicitly; this fixture treats the scene as a Dung-style
//! weighted framework so the user can see the symmetric-attack stalemate at
//! low β and the resolution as β rises. The conceptual page links from here
//! to the open-areas / VAF scoping doc that explains what changes once
//! values are wired in.
//!
//! Bench-Capon (2003): Hal, a diabetic, takes Carla's insulin to save his
//! life. Should he be punished?

use crate::trace::{AttackEdge, BeatRecord, SeededArg, Trace};
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

/// (actor, claim, expert-domain).
const PARTIES: &[(&str, &str, &str)] = &[
    ("hal", "life_over_property", "ethics"),
    ("carla", "property_rights", "ethics"),
    ("hal", "too_poor_to_compensate", "economics"),
    ("carla", "my_only_dose", "ethics"),
];

const ATTACKS: &[(usize, usize, f64)] = &[
    // (attacker_idx, target_idx, weight) — indices into PARTIES.
    (1, 0, 0.5), // C1 ↔ H1, mutual
    (0, 1, 0.5),
    (3, 2, 0.6), // C2 dampens H2 (Carla also at risk)
    (2, 1, 0.4), // H2 attacks C1 (cannot compensate)
];

struct HalCarlaScorer;

impl<P: Clone> ActionScorer<P> for HalCarlaScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available
            .iter()
            .map(|e| {
                let mut bindings = HashMap::new();
                bindings.insert("self".into(), actor.to_string());
                bindings.insert("expert".into(), actor.to_string());
                let (claim, domain) = match e.spec.name.as_str() {
                    "argue_life_over_property" => ("life_over_property", "ethics"),
                    "argue_property_rights" => ("property_rights", "ethics"),
                    "argue_too_poor_to_compensate" => ("too_poor_to_compensate", "economics"),
                    "argue_my_only_dose" => ("my_only_dose", "ethics"),
                    other => panic!("unexpected affordance: {other}"),
                };
                bindings.insert("claim".into(), claim.into());
                bindings.insert("domain".into(), domain.into());
                ScoredAffordance {
                    entry: e.clone(),
                    score: 1.0,
                    bindings,
                }
            })
            .collect()
    }
}

pub fn trace(beta: f64) -> Trace {
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut ids: Vec<_> = Vec::new();

    // Each tuple in PARTIES becomes one scheme instance. Hal speaks twice
    // (H1 and H2); Carla speaks twice (C1 and C2). Affordance names disambiguate.
    for (actor, claim, domain) in PARTIES {
        let mut b = HashMap::new();
        b.insert("expert".into(), (*actor).into());
        b.insert("domain".into(), (*domain).into());
        b.insert("claim".into(), (*claim).into());
        let instance = scheme.instantiate(&b).unwrap();
        let mut af = b.clone();
        af.insert("self".into(), (*actor).into());
        let id = state.add_scheme_instance_for_affordance(
            actor,
            &format!("argue_{claim}"),
            &af,
            instance,
        );
        ids.push(id);
    }

    let mut attack_edges = Vec::new();
    for (a, t, w) in ATTACKS {
        state.add_weighted_attack(&ids[*a], &ids[*t], *w).unwrap();
        attack_edges.push(AttackEdge {
            attacker: PARTIES[*a].1.into(),
            target: PARTIES[*t].1.into(),
            weight: *w,
        });
    }

    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "courtroom".into(),
        bindings: vec![
            "self".into(),
            "expert".into(),
            "domain".into(),
            "claim".into(),
        ],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let aff_names = [
        "argue_life_over_property",
        "argue_property_rights",
        "argue_too_poor_to_compensate",
        "argue_my_only_dose",
    ];
    let catalog: Vec<_> = aff_names
        .iter()
        .map(|n| CatalogEntry {
            spec: make_spec(n),
            precondition: String::new(),
        })
        .collect();
    let practice = PracticeSpec {
        name: "courtroom".into(),
        affordances: aff_names.iter().map(|s| (*s).to_string()).collect(),
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 6 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, HalCarlaScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    // Two unique participants — Hal and Carla — each speaks for two arguments.
    let participants: Vec<String> = vec!["hal".into(), "carla".into()];
    let result = MultiBeat.resolve(
        &participants,
        &practice,
        &catalog,
        &scorer,
        &acceptance,
    );

    let seeded_arguments = PARTIES
        .iter()
        .map(|(actor, claim, _)| SeededArg {
            actor: (*actor).to_string(),
            affordance_name: format!("argue_{claim}"),
            conclusion: (*claim).to_string(),
        })
        .collect();

    Trace {
        scene_name: "hal_carla".into(),
        beta,
        participants,
        seeded_arguments,
        attacks: attack_edges,
        beats: result
            .beats
            .iter()
            .map(|b| BeatRecord {
                actor: b.actor.clone(),
                action: b.action.clone(),
                accepted: b.accepted,
            })
            .collect(),
        errors: state.drain_errors().iter().map(|e| e.to_string()).collect(),
    }
}
```

- [ ] **Step 2: Add `hal_carla` to `scenes/mod.rs`**

```rust
pub mod east_wall;
pub mod hal_carla;
pub mod siege_council;
```

- [ ] **Step 3: Wire `hal-carla` into the main dispatch**

Edit `tools/scene-tracer/src/main.rs`, expanding the match:

```rust
    let trace = match args[1].as_str() {
        "east-wall" => scenes::east_wall::trace(beta),
        "siege-cold" => scenes::siege_council::trace(beta, scenes::siege_council::Climate::Cold),
        "siege-warm" => scenes::siege_council::trace(beta, scenes::siege_council::Climate::Warm),
        "hal-carla" => scenes::hal_carla::trace(beta),
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
```

- [ ] **Step 4: Build and smoke-test**

```bash
cd /home/peter/code/argumentation
cargo build -p scene-tracer 2>&1 | tail -5
cargo run -p scene-tracer -- hal-carla 0.5 /tmp/hal-carla-test.json
python3 -c "import json; t = json.load(open('/tmp/hal-carla-test.json')); print(t['scene_name'], len(t['beats']), 'beats,', len(t['attacks']), 'attacks,', len(t['errors']), 'errors')"
```

Expected: `hal_carla N beats, 4 attacks, 0 errors`.

- [ ] **Step 5: Generate the four traces**

```bash
cargo run -p scene-tracer -- hal-carla 0.0 website/static/traces/hal-carla-b00.json
cargo run -p scene-tracer -- hal-carla 0.3 website/static/traces/hal-carla-b03.json
cargo run -p scene-tracer -- hal-carla 0.5 website/static/traces/hal-carla-b05.json
cargo run -p scene-tracer -- hal-carla 1.0 website/static/traces/hal-carla-b10.json
```

Expected: four `wrote ...` lines.

- [ ] **Step 6: Commit**

```bash
git add tools/scene-tracer/src/scenes/hal_carla.rs \
        tools/scene-tracer/src/scenes/mod.rs \
        tools/scene-tracer/src/main.rs \
        website/static/traces/hal-carla-*.json
git commit -m "feat(scene-tracer): add hal-carla fixture (4-argument symmetric-attack scene)

Bench-Capon's canonical VAF example, run as a Dung-style weighted
framework. At low β the symmetric H1↔C1 pair stalemates; as β rises
the heavier C2→H2 attack drops out and the framework resolves. Four
pre-rendered traces shipped for the hal-and-carla page upgrade."
```

---

### Task 6: Upgrade `hal-and-carla.mdx` to engine-driven

**Files:**
- Modify: `website/docs/examples/hal-and-carla.mdx`

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/examples/hal-and-carla.mdx
```

Confirm contents match what is below.

- [ ] **Step 2: Replace the file with the engine-driven version**

Write `website/docs/examples/hal-and-carla.mdx`:

```mdx
---
sidebar_position: 3
title: Hal & Carla
---

import AttackGraph from '@site/src/components/AttackGraph';
import BetaPlayground from '@site/src/components/BetaPlayground';
import BetaSlider from '@site/src/components/BetaSlider';

> Hal, a diabetic, loses his insulin. Before collapsing, he enters Carla's house and uses some of her insulin. Carla is another diabetic, not home. Should Hal be punished?

— Trevor Bench-Capon, introducing **value-based argumentation frameworks** (2003). A widely-used worked example for reasoning about values.

## The arguments

<AttackGraph
  title="Hal & Carla — value-based attacks"
  arguments={[
    {id: 'H1', label: 'Hal: life > property'},
    {id: 'C1', label: 'Carla: property rights'},
    {id: 'H2', label: 'Hal: too poor to compensate'},
    {id: 'C2', label: 'Carla: my only dose'},
  ]}
  attacks={[
    {from: 'C1', to: 'H1'},
    {from: 'H1', to: 'C1'},
    {from: 'C2', to: 'H2'},
    {from: 'H2', to: 'C1'},
  ]}
  height={400}
  caption="H1 and C1 attack each other symmetrically. C2 neutralises Hal's compensation argument H2 by showing Carla was also endangered. H2 in turn attacks C1 — if Hal cannot pay, property-rights-as-remedy dissolves."
/>

## Drag β to see how the framework resolves

<BetaPlayground
  title="Hal & Carla — live"
  args={[
    {id: 'H1', label: 'Hal: life > property'},
    {id: 'C1', label: 'Carla: property rights'},
    {id: 'H2', label: 'Hal: too poor to compensate'},
    {id: 'C2', label: 'Carla: my only dose'},
  ]}
  attacks={[
    {from: 'C1', to: 'H1', weight: 0.5},
    {from: 'H1', to: 'C1', weight: 0.5},
    {from: 'C2', to: 'H2', weight: 0.6},
    {from: 'H2', to: 'C1', weight: 0.4},
  ]}
  initialBeta={0}
/>

At low β, the symmetric H1 ↔ C1 pair both have live attackers; neither is credulously accepted. As β rises through 0.4 the H2 → C1 attack drops, freeing C1 to be a sole survivor of the H1 ↔ C1 pair. Past β = 0.6, C2's neutralisation of H2 also drops, and H2 returns. **No β setting in this scene yields a single-survivor outcome that prefers Hal over Carla** — the abstract weighted framework cannot, on its own, encode the moral preference *life > property*. That preference is what value-based argumentation is built to express; see the [open areas](/concepts/open-areas) and [VAF scoping](/concepts/value-based-argumentation) pages.

## A pre-recorded trace at four discrete β

<BetaSlider
  title="Hal & Carla across β"
  tracePaths={[
    {beta: 0.0, path: '/traces/hal-carla-b00.json'},
    {beta: 0.3, path: '/traces/hal-carla-b03.json'},
    {beta: 0.5, path: '/traces/hal-carla-b05.json'},
    {beta: 1.0, path: '/traces/hal-carla-b10.json'},
  ]}
/>

## Why values matter

Pure Dung semantics — the engine you just played with — can't resolve this case in the way humans intuitively do. The symmetric H1 ↔ C1 attack at low β gives multiple extensions; β tuning at the scene level is too blunt to encode "life is more important than property." That's a *value preference*, not a scene-intensity setting.

Bench-Capon's solution: attach **values** to arguments. H1 promotes *life*; C1 promotes *property*; H2 promotes *fairness*; C2 promotes *life* (Carla's life, in this case). An audience is an ordering over values. Different audiences with different orderings reach different stable positions *rationally*:

- An audience that ranks life > property → H1 and C2 accepted, C1 and H2 rejected. Hal goes free.
- An audience that ranks property > life → the opposite. Hal is punished.

The framework hasn't changed; the audience has. This is the formal machinery that makes "different audiences, same arguments, different conclusions" precise rather than vague.

## In our library

Value-based argumentation is on the roadmap as an open formalism. See the [VAF scoping page](/concepts/value-based-argumentation) for the design sketch — types, semantics, integration points, and what audience-conditioned outputs would look like in a scene. Currently our [`argumentation-schemes`](/api/) supports a related mechanism: `PracticalReasoning` schemes carry a value dimension in their bindings, and `encounter-argumentation`'s `StateActionScorer` can be composed with a value-aware inner scorer to produce audience-conditioned scoring (without the full VAF semantics).

## Further reading

- [Bench-Capon (2003)](/academic/bibliography#benchcapon2003) — value-based argumentation frameworks.
- [Atkinson & Bench-Capon (2007)](/academic/bibliography#atkinson2007) — practical reasoning over VAFs.
- [Open areas](/concepts/open-areas) — formalisms not yet implemented in this library.
- [VAF scoping](/concepts/value-based-argumentation) — what an `argumentation-values` crate would look like.
```

- [ ] **Step 3: Build the website**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -20
```

Expected: `[SUCCESS]`. The `/concepts/open-areas` and `/concepts/value-based-argumentation` links will warn as broken until Phase 3 lands; that's expected and acceptable for this commit. Other warnings should be unchanged from baseline.

- [ ] **Step 4: Commit**

```bash
git add website/docs/examples/hal-and-carla.mdx
git commit -m "docs(examples): promote Hal & Carla to engine-driven

Adds live BetaPlayground + pre-rendered BetaSlider on top of the
existing conceptual prose. Sets up the bridge into the open-areas /
VAF scoping pages added in the next commits. Two cross-links to those
pages will resolve once Phase 3 lands."
```

---

### Task 7: Move Hal & Carla into the Engine-driven sidebar category

**Files:**
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Update `examplesSidebar`**

Edit `website/sidebars.ts`. Replace the Engine-driven scenes and Worked examples blocks with:

```typescript
    {
      type: 'category',
      label: 'Engine-driven scenes',
      items: [
        'examples/siege-council',
        'examples/east-wall',
        'examples/hal-and-carla',
      ],
    },
    {
      type: 'category',
      label: 'Worked examples (literature)',
      items: [
        'examples/nixon-diamond',
        'examples/tweety-penguin',
        'examples/courtroom',
      ],
    },
```

- [ ] **Step 2: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`.

- [ ] **Step 3: Commit**

```bash
git add website/sidebars.ts
git commit -m "docs(examples): move Hal & Carla into Engine-driven scenes category"
```

---

## Phase 3: Open Areas + VAF Scoping

Phase 3 is documentation only — no code changes. Two new pages: an overview of five open formalisms (value-based AF, probabilistic AF, abstract dialectical frameworks, dialogue games, dynamic AF), and a deeper VAF mini-RFC that goes one layer further on the headline open area.

These pages land in `concepts/`, not `reference/`, because they're explanations of *future* work — not API surface. They serve two audiences: existing users who want to know the library's roadmap, and academic readers who want to know how the library positions itself relative to the formalism literature.

### Task 8: Write the open-areas overview page

**Files:**
- Create: `website/docs/concepts/open-areas.mdx`

- [ ] **Step 1: Write the file**

```mdx
---
sidebar_position: 8
title: Open areas
---

**Five formalisms in the argumentation literature this library does not yet implement, with notes on what we'd build and why.**

The library focuses on Dung abstract frameworks, ASPIC+ structured arguments, weighted attacks, bipolar (attack + support) extensions, and the encounter bridge. The argumentation literature is broader. This page is a public roadmap — not a commitment to ship, but an honest map of the gap.

## 1. Value-based argumentation frameworks (VAF)

**The headline open area.** Bench-Capon (2003) extended Dung frameworks with *values* — each argument promotes a value, and an *audience* is an ordering over values. Different audiences reach different rational conclusions from the same framework. This is the formal machinery for "reasonable disagreement on principle."

**Why we want it.** Our [Hal & Carla example](/examples/hal-and-carla) demonstrates the limit of the abstract weighted framework: the moral preference *life > property* is not encodable as an attack weight or a β setting. VAF makes that preference a first-class citizen of the framework.

**See the [VAF scoping page](/concepts/value-based-argumentation)** for the deeper sketch — types, semantics, audience-conditioned outputs, and integration with the existing weighted/bipolar machinery.

**Key reference:** [Bench-Capon (2003)](/academic/bibliography#benchcapon2003).

## 2. Probabilistic argumentation frameworks

Hunter (2013) introduced two families of probabilistic semantics over Dung frameworks: the *epistemic* approach (probability *of* an argument's acceptance reflects belief in the argument) and the *constellation* approach (probability *over* sub-frameworks reflects uncertainty in which arguments and attacks exist).

**Why a consumer might want it.** Scene engines often have uncertain knowledge about which arguments an actor *actually* holds — a witness's testimony is not certain to be remembered correctly; a rumour might not be trusted. The constellation approach lets you sample sub-frameworks and reason over the distribution of outcomes.

**What we'd build.** A `argumentation-probabilistic` crate adding `ProbabilisticArgument` (an argument with a probability) and `ConstellationFramework` (samples sub-frameworks and aggregates extension membership). Composes with `argumentation-weighted` — an argument can be both probabilistic and have weighted attacks.

**Tradeoff against shipping.** Adds a Monte Carlo dimension to extension-finding. Either you sample and aggregate (slower, gives confidence intervals on acceptance) or you compute exact probabilities by enumeration (intractable past ~20 uncertain arguments). Either is a meaningful design conversation, not a bolt-on.

**Key references:** [Hunter (2013) "A probabilistic approach to modelling uncertain logical arguments"](https://www.sciencedirect.com/science/article/pii/S0888613X12001399); [Li, Oren, Norman (2011) "Probabilistic argumentation frameworks"](https://link.springer.com/chapter/10.1007/978-3-642-29184-5_1).

## 3. Abstract dialectical frameworks (ADF)

Brewka, Strass, Ellmauthaler, Wallner, Woltran (2013) generalised Dung frameworks: an argument's acceptance is not just "no live attacker" but an arbitrary *acceptance condition* over the truth values of its parents. Dung frameworks are a special case (the acceptance condition is "no parent is in"). ADFs let you say "this argument is in if at least two of its three parents are in," or "this argument is in iff its single supporter is in and no attacker is in."

**Why a consumer might want it.** Bipolar reasoning at the framework level. The current bipolar crate handles attacks and supports as separate edge kinds with separate semantics; ADF unifies them under a single per-argument condition language.

**What we'd build.** A `argumentation-adf` crate with an `AcceptanceCondition` AST (boolean formulas over parent truth values), per-argument acceptance evaluation, and the four ADF semantics (grounded, complete, preferred, stable — all generalised). Likely supersedes part of `argumentation-bipolar` once mature.

**Tradeoff against shipping.** ADF is more general but its semantics are computationally harder. Stable models are NP-hard in the general case. We'd need to scope the supported acceptance-condition language carefully.

**Key reference:** [Brewka et al. (2013) "Abstract dialectical frameworks revisited"](https://www.ijcai.org/Proceedings/13/Papers/119.pdf).

## 4. Persuasion dialogue games

Walton & Krabbe (1995) classified dialogue types (information-seeking, inquiry, persuasion, negotiation, deliberation, eristic). Persuasion dialogues are turn-based two-party games where each move is a speech act (*assert*, *challenge*, *concede*, *retract*) constrained by the dialogue's commitment store. Prakken (2006) gave a formal protocol; many subsequent papers refined the semantics.

**Why a consumer might want it.** Our `MultiBeat` resolution loop is a flat round-robin — each actor proposes, each responder accepts or rejects. Real argumentation dialogues have richer move types (challenge, concede), commitment tracking, and termination conditions tied to the protocol. A dialogue-game layer above `encounter-argumentation` would let you author scenes where the *protocol* matters as much as the arguments.

**What we'd build.** A `argumentation-dialogue` crate with `DialogueMove` enum, `CommitmentStore` per actor, and a `DialogueProtocol` trait the way `encounter` has a `Practice` trait. The existing `MultiBeat` becomes one of several protocols rather than the only one.

**Tradeoff against shipping.** Significant new surface area. Worth doing only when a consumer actually needs a non-flat protocol — speculatively building this would create code without users.

**Key references:** [Walton & Krabbe (1995) Commitment in Dialogue](https://www.amazon.com/Commitment-Dialogue-Interpersonal-Reasoning-Communication/dp/0791424596); [Prakken (2006) "Formal systems for persuasion dialogue"](https://www.cambridge.org/core/journals/knowledge-engineering-review/article/abs/formal-systems-for-persuasion-dialogue/3A38B73B89A1AC5DCED81F0697C0D43A).

## 5. Dynamic argumentation frameworks

Cayrol, de Saint-Cyr, Lagasquie-Schiex (2010) studied how extensions change as arguments and attacks are added or removed *over time*. The basic question: given a framework `F` with known extensions, and a small change `Δ`, can you compute the new extensions of `F + Δ` faster than recomputing from scratch?

**Why a consumer might want it.** Scenes evolve. Each beat can introduce a new argument or invalidate an existing one (an argument's premise gets undercut, the argument falls). Currently the bridge recomputes acceptance from scratch each beat. A dynamic-AF layer would track incremental changes and reuse prior computation.

**What we'd build.** Not a new crate — an extension to `argumentation` itself. `Framework::add_argument_incremental(...)` and `Framework::remove_argument_incremental(...)` returning a delta over previous extensions. Useful primarily as a performance optimisation; only worth doing once the bridge has profilable users hitting recompute cost.

**Tradeoff against shipping.** Pure speed-of-light optimisation. Doesn't change what the library can express. Premature without a concrete bottleneck.

**Key reference:** [Cayrol, de Saint-Cyr, Lagasquie-Schiex (2010) "Change in abstract argumentation frameworks"](https://www.sciencedirect.com/science/article/pii/S088861410800120X).

## How to vote on which area to ship next

Open an issue on the [argumentation repo](https://github.com/patricker/argumentation) titled `[open-area] <name>` and describe:
- What you're building that needs it.
- Which scenes can't be modelled without it.
- Whether you're willing to co-design or only consume.

We prioritise areas where there's a real consumer with a real scene that the current library cannot express.

## Further reading

- [Value-based argumentation framework — full mini-RFC](/concepts/value-based-argumentation) — the deeper sketch of the headline open area.
- [Reading order](/academic/reading-order) — the literature path that gets you to these formalisms.
- [Hal & Carla](/examples/hal-and-carla) — the engine-driven scene that motivates VAF.
```

- [ ] **Step 2: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. The `/concepts/value-based-argumentation` link will still warn as broken until Task 9 lands; the academic-bibliography anchor warnings are acceptable if those entries don't yet exist (we'll add them if needed in Task 9).

- [ ] **Step 3: Commit**

```bash
git add website/docs/concepts/open-areas.mdx
git commit -m "docs(concepts): add open-areas overview page

Public roadmap covering five formalisms not yet in the library: VAF,
probabilistic AF, ADF, dialogue games, dynamic AF. Each section has
references, what-we-d-build, and the tradeoff against shipping."
```

---

### Task 9: Write the VAF mini-RFC scoping page

**Files:**
- Create: `website/docs/concepts/value-based-argumentation.mdx`
- Modify: `website/sidebars.ts`
- Possibly modify: `website/docs/academic/bibliography.md` (only if it doesn't already cite Bench-Capon 2003 + Atkinson & Bench-Capon 2007)

- [ ] **Step 1: Verify bibliography entries**

```bash
grep -E "benchcapon2003|atkinson2007|Bench-Capon" /home/peter/code/argumentation/website/docs/academic/bibliography.md 2>/dev/null
```

Expected: at least one Bench-Capon entry. If neither key appears, note that you'll need to add them to bibliography.md. If they appear, proceed.

- [ ] **Step 2: Write `value-based-argumentation.mdx`**

```mdx
---
sidebar_position: 9
title: Value-based argumentation (mini-RFC)
---

**A scoping document for an `argumentation-values` crate. Not implemented yet — this page describes what we'd build.**

The headline [open area](/concepts/open-areas). Bench-Capon (2003) introduced Value-based Argumentation Frameworks (VAFs) as a Dung extension where arguments promote values, and audiences are orderings over values. Different audiences reach different rational conclusions from the same framework. This page sketches the design.

## The motivating example

Hal, a diabetic, takes Carla's insulin to save his life. Should he be punished? See the full [Hal & Carla scene](/examples/hal-and-carla) for the engine-driven version. The abstract weighted framework cannot, on its own, encode the preference "life > property" — that preference is the audience, not the framework.

## Types

```rust
/// A value an argument can promote. Strings for now; could become an
/// extensible enum-like trait in the future.
pub struct Value(pub String);

/// Maps each argument to the value it promotes. An argument might
/// promote no value (None) — those arguments behave as in pure Dung.
pub struct ValueAssignment {
    promoted: HashMap<ArgumentId, Value>,
}

/// An audience is a strict partial order over values. We represent it
/// as a Vec of value names from most to least preferred (ties allowed
/// via the second nested level).
///
/// Example: vec![vec!["life"], vec!["fairness"], vec!["property"]]
/// means life > fairness > property strictly.
///
/// Example: vec![vec!["life", "fairness"], vec!["property"]]
/// means {life, fairness} > property; life and fairness incomparable.
pub struct Audience {
    ranked: Vec<Vec<Value>>,
}

impl Audience {
    /// Strict preference: returns true iff `a` is strictly preferred to `b`.
    pub fn prefers(&self, a: &Value, b: &Value) -> bool { /* ... */ }
}

/// A VAF — Dung framework + value assignment. Extensions are computed
/// per-audience, not globally.
pub struct ValueBasedFramework {
    base: Framework,
    values: ValueAssignment,
}
```

## Semantics

A VAF defeats the symmetric-attack ambiguity by adding an *audience-specific defeat* relation:

> Argument A *defeats* argument B (with respect to audience X) iff:
>   1. A attacks B in the underlying Dung framework, and
>   2. The value B promotes is *not* strictly preferred over the value A promotes by audience X.

In Hal & Carla under audience X = `[[life], [property]]`:
- H1 (promotes life) attacks C1 (promotes property). Is C1's value (property) strictly preferred over H1's (life)? No. So H1 *defeats* C1.
- C1 (promotes property) attacks H1 (promotes life). Is H1's value (life) strictly preferred over C1's (property)? Yes. So C1 does *not* defeat H1.

The symmetric attack becomes asymmetric defeat; H1 is in the unique grounded extension, C1 is out. Hal goes free.

Standard Dung extensions (grounded, preferred, stable) are then computed over the audience-specific *defeat* graph rather than the original *attack* graph. Two new acceptance terms enter the vocabulary:

- **Subjectively acceptable** in a VAF: accepted by *some* audience.
- **Objectively acceptable**: accepted by *every* audience.

## Integration with existing crates

```
argumentation
    ↑
    │ (re-uses Framework, ArgumentId)
    │
argumentation-values         (this RFC)
    ↑
    ├─ argumentation-weighted-values  (β + values; future)
    └─ argumentation-bipolar-values   (supports + values; future)
```

The simplest version is `argumentation-values` alone — it composes a `Framework` with a `ValueAssignment` and exposes the audience-specific defeat graph. Weighted and bipolar variants follow the same pattern as today: a value-aware framework is an inner Dung framework + values + the additional structure.

The `WeightSource<ArgumentId>` trait already in `argumentation-weighted` is the natural plug-in point — values modulate weights at resolution time. An audience could be expressed as a `WeightSource` that boosts attacks in line with the preference order, collapsing the defeat semantics into the existing weighted machinery without a separate crate. Both approaches are viable; the first is more faithful to Bench-Capon, the second is cheaper to build and composes with everything.

## Integration with the encounter bridge

`StateActionScorer` already wraps an inner scorer to add credulous-acceptance boost. A VAF-aware scorer would wrap a baseline scorer to add an *audience parameter* — same affordances scored differently by characters with different value orderings.

```rust
let alice_scorer = ValueAwareScorer::new(baseline, &vaf, alice_audience);
let bob_scorer   = ValueAwareScorer::new(baseline, &vaf, bob_audience);
```

In a multi-actor scene each character can have their own audience, and the same proposal scores differently for each. This is the formal grounding for "Alice prioritises duty; Bob prioritises survival; same scene, different character preferences."

## Open design questions

These are the questions that need answers before implementation, not blockers to publishing the scoping doc:

1. **Should `Value` be a string, an extensible enum, or generic over a trait?** Current sketch uses `String` for ergonomics. Trait-based would let consumers attach domain semantics (numerical magnitude, hierarchical taxonomies). String is enough for v0; revisit if a consumer needs more.

2. **How are audiences specified at scene authoring time?** Two options: per-character (each actor carries an audience as part of their state) or per-scene (the storyteller specifies which audience the framework resolves under). Per-character is more expressive but means every acceptance evaluation is per-character. Per-scene is cheaper but less faithful to multi-perspective scenes.

3. **Subjective vs objective acceptance: which does the bridge surface?** The bridge needs *some* boolean answer. Most likely: subjective for `StateActionScorer` (the proposer wants to know "could this land with anyone?") and objective for `StateAcceptanceEval` (the responder rejects only what no audience would accept). Open to debate.

4. **Compatibility with weighted attacks.** β tunes how strictly attacks bind. Values change *which* attacks are defeats. Composing both: attack from A to B is a *binding defeat* iff its weight `w` satisfies `w > β` AND audience does not strictly prefer B's value over A's. Straightforward, but worth writing tests against the published VAF semantics.

5. **Hal & Carla as the integration test.** The scoping doc is finished when there's a unit test that, given the Hal & Carla framework and audience `[[life], [property]]`, returns the grounded extension `{H1, C2}` and rejects `{C1, H2}`. That test is the success criterion.

## What this RFC is not

- **Not a commitment to ship.** Open areas are open until a consumer needs them.
- **Not the only path to value-aware argumentation.** A consumer could build value-aware scoring as a `WeightSource` today, without waiting for the crate. The crate is the principled formalisation; the workaround is the practical one.
- **Not a replacement for ASPIC+ or weighted frameworks.** It composes with both; it does not subsume either.

## Further reading

- [Bench-Capon (2003)](/academic/bibliography#benchcapon2003) — the original VAF paper.
- [Atkinson & Bench-Capon (2007)](/academic/bibliography#atkinson2007) — practical reasoning over VAFs.
- [Open areas](/concepts/open-areas) — the broader roadmap context.
- [Hal & Carla](/examples/hal-and-carla) — the scene this RFC is built around.
```

- [ ] **Step 3: Add both new pages to the conceptsSidebar**

Edit `website/sidebars.ts`. The `conceptsSidebar` block becomes:

```typescript
  conceptsSidebar: [
    'concepts/what-is-argumentation',
    'concepts/walton-schemes',
    'concepts/attacks-and-supports',
    'concepts/semantics',
    'concepts/weighted-and-beta',
    'concepts/aspic-plus',
    'concepts/encounter-integration',
    'concepts/open-areas',
    'concepts/value-based-argumentation',
  ],
```

- [ ] **Step 4: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -20
```

Expected: `[SUCCESS]`. The previously-broken `/concepts/value-based-argumentation` link from the open-areas page (Task 8) and the Hal & Carla page (Task 6) now resolves. Acceptable warnings: pre-existing `/api/` links. **Investigate** any new warning naming open-areas, value-based-argumentation, or hal-and-carla.

- [ ] **Step 5: Commit**

```bash
git add website/docs/concepts/value-based-argumentation.mdx website/sidebars.ts
git commit -m "docs(concepts): add VAF mini-RFC scoping page

Design sketch for an argumentation-values crate: Value /
ValueAssignment / Audience types, audience-specific defeat semantics,
integration with weighted + bipolar + the encounter bridge, open
design questions, and the Hal & Carla integration test as the
success criterion. Not a commitment to ship — explicit roadmap
artifact."
```

---

### Task 10: Cross-link the open areas pages from the existing site

**Files:**
- Modify: `website/docs/concepts/walton-schemes.mdx`
- Modify: `website/docs/reference/overview.md` (if it exists; check first)

- [ ] **Step 1: Check whether walton-schemes already mentions VAF or open areas**

```bash
grep -E "value-based|VAF|open.areas|open-areas" /home/peter/code/argumentation/website/docs/concepts/walton-schemes.mdx
```

If matches: skip Step 2 — schema already cross-links.
If no matches: proceed.

- [ ] **Step 2: Add a one-liner to walton-schemes.mdx pointing at the open areas page**

Read the current file:

```bash
cat /home/peter/code/argumentation/website/docs/concepts/walton-schemes.mdx | tail -20
```

Find a "Further reading" or "See also" section near the bottom. Add to its bullet list:

```mdx
- [Open areas](/concepts/open-areas) — formalisms beyond what schemes can express on their own (values, probability, dialogue protocols).
```

If the file has no Further reading section, append at the very end:

```mdx
## Further reading

- [Open areas](/concepts/open-areas) — formalisms beyond what schemes can express on their own (values, probability, dialogue protocols).
```

- [ ] **Step 3: Check reference/overview.md**

```bash
ls /home/peter/code/argumentation/website/docs/reference/overview.md 2>/dev/null
```

If the file exists, read it and append (under whatever "what's next" or "what we don't have" section is most appropriate, or add the section if absent):

```mdx
## What we don't have yet

The library focuses on Dung frameworks, ASPIC+, weighted attacks, bipolar extensions, and the encounter bridge. Five formalisms remain on the roadmap; see [open areas](/concepts/open-areas) for the public map and [VAF mini-RFC](/concepts/value-based-argumentation) for the deeper sketch of the headline gap.
```

If `overview.md` doesn't exist, skip this and proceed.

- [ ] **Step 4: Build the website**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`.

- [ ] **Step 5: Commit**

```bash
git add website/docs/
git commit -m "docs: cross-link open-areas and VAF pages from existing concept pages"
```

---

## Done

After Task 10, the docs site has:
- A flagship engine-driven scene (siege council) that exercises 4 actors and shows two independent dials
- A second engine-driven scene (Hal & Carla) that motivates the next open area
- A public roadmap of five open formalisms
- A deeper VAF mini-RFC that is the first artifact on the path to actually building it
- Updated homepage CTA pointing at the flagship
- Three pages cross-linked into existing site structure

Verify with one final build:

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`.

If running via subagent-driven-development, the final code-review should evaluate:
- Whether the siege council attack weights produce dramatic-enough cold-vs-warm differences across the four sampled β values (re-tune in `siege_council.rs` and regenerate traces if not)
- Whether the open-areas page accurately represents what each formalism is (any factual error here will be caught by a reader who knows the literature; reviewer should treat as if academic-pedantic)
- Whether the VAF mini-RFC is concrete enough that an engineer could pick it up and build the crate (the success criterion at the bottom — Hal & Carla integration test — is the key indicator)
