---
sidebar_position: 10
title: Glossary
---

Quick definitions for the terms used across the site. Cross-linked from concept pages so you can look up "what does grounded mean here?" without reading a whole page.

## Argumentation primitives

### Argument
A node in an argumentation framework. The library treats arguments as opaque IDs by default (Dung's abstract framework). Schemes and ASPIC+ give arguments internal structure (premises + conclusion).

### Attack
A directed edge in an argumentation framework — `(attacker, target)` — meaning the attacker's content undermines the target. May be classical (binary) or weighted (with a strength).

### Support
The dual of attack — `(supporter, supported)` — meaning the supporter's content reinforces the target. Lives in the `argumentation-bipolar` crate.

### Framework
A pair `(A, R)` of arguments and attacks (Dung). Bipolar adds supports; weighted adds attack weights; valued adds value promotions.

### Scheme
A named pattern of human argument (e.g., expert opinion, cause-to-effect). Schemes carry premise slots, a conclusion template, and critical questions. The `argumentation-schemes` crate ships ~60 schemes from Walton, Reed & Macagno (2008).

### Critical question
A standardised challenge to a scheme. E.g., for argument-from-expert-opinion: "Is the expert reliable?" "Is the field within their expertise?" Each scheme carries its own list. The bridge translates critical questions into attacks.

## Acceptance semantics

### Conflict-free
A set of arguments where no member attacks another member. The minimal property of any extension.

### Admissible
A conflict-free set that defends each of its members against external attacks (for every attacker `b` of a member `a`, some member of the set attacks `b`).

### Grounded extension
The unique smallest admissible set — equivalently, the least fixed point of the characteristic function. Always exists. Polynomial to compute. The "skeptical answer" — what survives if you refuse to take sides on any disputed point.

### Preferred extension
A maximal admissible set — admissible, and you can't add more without violating admissibility. Multiple preferred extensions can co-exist for the same framework. Always exists.

### Complete extension
An admissible set that contains every argument it defends. Sits between admissible and preferred. Both grounded and preferred extensions are complete.

### Stable extension
A conflict-free set that attacks every argument outside it. Stricter than preferred. Doesn't always exist.

### Credulous acceptance
An argument is in *some* preferred extension. The "could a reasonable observer accept this?" question. Used by the encounter bridge for proposer-side scoring.

### Skeptical acceptance
An argument is in *every* preferred extension. The "what survives every reading?" question. Stricter than credulous; always a subset.

### Defeat
In a value-based framework, an attack that has not been filtered out by the audience. The defeat graph is the audience-conditioned subgraph of the original attack graph.

## Weighted argumentation

### Weight
A number `w ∈ [0, 1]` attached to an attack edge. Higher = stronger attack.

### Budget (β)
A threshold `β ∈ [0, 1]` on attack weight. Attacks with `w > β` bind; attacks with `w ≤ β` drop. Single-knob "scene intensity" dial.

### β-residual framework
The Dung framework you get by removing all attacks with `w ≤ β`. Acceptance semantics run on the residual.

### Binding / dropping
An attack *binds* when `w > β` (acts as a Dung attack). An attack *drops* when `w ≤ β` (treated as absent).

## Value-based argumentation

### Value
Something an argument can promote (e.g., `Value::new("life")`). The `argumentation-values` crate uses string-typed values; consumers can adopt any taxonomy.

### Value assignment
A map from arguments to the set of values each promotes. Multi-value supported per Kaci & van der Torre (2008).

### Audience
A strict partial order over values, represented as ranked tiers. Each character can have their own audience; consensus is computed via `MultiAudience`.

### Pareto-defeating
The multi-value defeat rule: A defeats B iff for every value B promotes, some value A promotes is not strictly less-preferred under the audience. Reduces to Bench-Capon (2003) when each argument promotes one value.

### Subjective acceptance
An argument is accepted by *some* total ordering of the value set. NP-complete in general; capped at 6 values.

### Objective acceptance
An argument is accepted by *every* total ordering of the value set. co-NP-complete in general; capped at 6 values.

## Encounter bridge

### Proposer
The actor whose turn it is to propose an affordance in a scene beat. Gets scored via `ActionScorer`.

### Responder
The actor evaluating the proposer's affordance for acceptance. Gets queried via `AcceptanceEval`.

### Affordance
A scene-engine concept (from `encounter`): a candidate action the proposer could take. The bridge treats each affordance as backed by a scheme instance via `ArgumentKnowledge`.

### Affordance key
The canonical `(actor, affordance_name, bindings)` triple used for forward-index lookup in the state. See `AffordanceKey`.

### Scheme instance
A scheme with concrete bindings (e.g., expert-opinion with `expert=alice, domain=military, claim=fortify_east`). The result of `Scheme::instantiate(&bindings)`.

### Scheme strength
A property of a scheme (`Strong`/`Moderate`/`Weak`). The `SchemeActionScorer` boost is proportional to strength × per-character `preference_weight`.

### Preference weight
A per-character, per-scheme-instance scalar in `[0, 1]` indicating how strongly that character holds the scheme position. Read by `SchemeActionScorer`.

### Argument knowledge
A trait that supplies per-character argument positions (which scheme they invoke for which action, with what bindings, with what preference weight). `StaticKnowledge` is the default impl for fixtures.

### Error latch
The bridge's design for handling internal failures: append errors to a `Mutex<Vec<Error>>` on the state, return permissive defaults, drain via `state.drain_errors()` after resolution.

## Library / workspace

### Crate
A Rust library/binary unit. The argumentation workspace has 7 crates: `argumentation`, `argumentation-bipolar`, `argumentation-weighted`, `argumentation-weighted-bipolar`, `argumentation-schemes`, `argumentation-values`, `encounter-argumentation`.

### Catalogue
A `CatalogRegistry` of `SchemeSpec`s. The default catalogue (`default_catalog()`) ships ~60 Walton schemes.

### `ENUMERATION_LIMIT`
A hard cap on framework size for subset-enumeration semantics. The core `argumentation` crate uses 22 (preferred / stable). The `argumentation-values` crate uses 6 for subjective/objective acceptance.

## See also

- [Acceptance semantics](/concepts/semantics) — extended treatment with worked examples.
- [Weighted attacks and β](/concepts/weighted-and-beta) — extended treatment of the weighted layer.
- [Value-based argumentation](/concepts/value-based-argumentation) — the VAF treatment.
- [Encounter bridge](/concepts/encounter-integration) — the proposer/responder model.
- [Reading order](/academic/reading-order) — for the underlying papers.
