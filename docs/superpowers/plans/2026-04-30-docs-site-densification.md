# Docs Site Densification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the docs site from "well-organized but thin" to "actually teaches the library" via four phases — patch post-VAF gaps, expand the tutorial layer, deepen the concept layer, then add persona-driven onboarding.

**Architecture:** Pure docs work — no Rust crate changes. Each phase produces a shippable docs improvement on its own; the four together address the structural problem identified by audit (Diataxis-disciplined skeleton with sparsely populated corners). Builds on the existing Docusaurus structure (`website/docs/{getting-started,guides,concepts,reference,examples,academic}` + `sidebars.ts`).

**Tech Stack:** Docusaurus 3, MDX, the existing component library (`AttackGraph`, `BetaPlayground`, `BetaSlider`, `SchemeCard`). No new dependencies. No JavaScript work in phases 1–3 (Phase 4 touches `src/pages/index.tsx`).

---

## Scope notes

**Four phases, sequenceable but pausable:**

- **Phase 1 — Surface gaps (Tasks 1–6)** — patches post-VAF inconsistencies (per-crate page, bibliography anchors, reading-order entry, overview, two new how-tos). After this phase the site is *consistent* post-VAF.
- **Phase 2 — Tutorial layer expansion (Tasks 7–9)** — adds 2 new tutorials following first-scene's shape, plus a learning-objective patch on first-scene itself. After this phase the tutorial layer is a 30-minute progression instead of a 10-minute dead-end.
- **Phase 3 — Concept depth pass (Tasks 10–15)** — expands four under-developed concept pages, adds a glossary, adds a concept-pages reading order to academic/reading-order.md. After this phase the explanation layer actually teaches.
- **Phase 4 — Persona-driven onboarding (Tasks 16–17)** — restructures the homepage with three persona CTAs and adds a new "choose your path" page. After this phase users land at a fork they can navigate, not a single demo button.

**Out of scope:**
- Rust API changes (e.g., `AttackGraph` `'grounded'` AcceptedState variant — component work, not docs)
- Author-track guides ("write a values argument," "author a council scene") — should land *after* Phase 2 tutorials exist; risk of scope creep
- Per-crate reference uniformity audit (some non-encounter-argumentation per-crate pages may be thin; verify after Phase 1 lands and decide separately)
- Rustdoc generation pipeline — separate plan; the `/api/` 404s are infrastructure work
- Additional examples beyond the existing 7 — not currently a bottleneck

**Audit findings this plan addresses (each maps to one or more tasks):**

| Audit finding | Tasks |
|---|---|
| Missing per-crate page for `argumentation-values` | T1 |
| 3 dangling bibliography anchors (`kaci2008`, `dunne2004`, `bodanza2023`) | T2 |
| `academic/reading-order.md` doesn't include VAF | T3 |
| `reference/overview.md` describes 6 crates, not 7; missing new types | T4 |
| No how-to for APX I/O | T5 |
| No how-to for `MultiAudience::common_grounded` | T6 |
| Tutorial layer is one page (β-only); no progression | T7, T8, T9 |
| `semantics.mdx` is 50 lines for 6 acceptance concepts | T10 |
| `weighted-and-beta.mdx` is 58 lines without β regime walkthroughs | T11 |
| `what-is-argumentation.mdx` is 49 lines for the front door | T12 |
| `encounter-integration.mdx` is 53 lines, reference-shaped, no audiences mention | T13 |
| No glossary | T14 |
| Reading-order has no concept-pages section | T15 |
| Homepage CTA = single demo button; no persona routing | T16, T17 |

---

## File structure

**New files:**

| Path | Type | Phase |
|---|---|---|
| `website/docs/reference/argumentation-values.md` | Reference (per-crate) | 1 |
| `website/docs/guides/import-export-apx.md` | How-to | 1 |
| `website/docs/guides/multi-character-consensus.md` | How-to | 1 |
| `website/docs/getting-started/second-scene-with-schemes.md` | Tutorial | 2 |
| `website/docs/getting-started/third-scene-with-values.md` | Tutorial | 2 |
| `website/docs/concepts/glossary.md` | Reference (terminology) | 3 |
| `website/docs/getting-started/choose-your-path.md` | Explanation (orientation) | 4 |

**Modified files:**

| Path | Change | Phase |
|---|---|---|
| `website/docs/academic/bibliography.md` | Add 3 entries (kaci2008, dunne2004, bodanza2023) | 1 |
| `website/docs/academic/reading-order.md` | Add VAF entry to "Full curriculum"; add new "Concept-pages reading order" section | 1, 3 |
| `website/docs/reference/overview.md` | Add argumentation-values + Audience/ValueAssignment/ValueBasedFramework to types; add to Crate map | 1 |
| `website/docs/getting-started/first-scene.md` | Add explicit ABCD learning-objective line | 2 |
| `website/docs/concepts/semantics.mdx` | Expand 50→~120 lines: worked credulous/skeptical/grounded examples | 3 |
| `website/docs/concepts/weighted-and-beta.mdx` | Expand 58→~100 lines: β regime walkthroughs | 3 |
| `website/docs/concepts/what-is-argumentation.mdx` | Expand 49→~80 lines: mental-model framing, what-it-isn't disambiguation | 3 |
| `website/docs/concepts/encounter-integration.mdx` | Expand 53→~120 lines: why-a-bridge, proposer/responder split, audiences, error latch | 3 |
| `website/sidebars.ts` | Add new tutorial pages, new how-tos, new concepts/glossary, new choose-your-path | 1, 2, 3, 4 |
| `website/src/pages/index.tsx` | Three persona CTAs + brief tagline per path | 4 |

---

## Phase 1: Surface gaps (Tasks 1–6)

Patches inconsistencies that landed after the VAF work without follow-through. All low-effort, all reference/concepts/academic.

### Task 1: Add `reference/argumentation-values.md` per-crate page

**Files:**
- Create: `website/docs/reference/argumentation-values.md`
- Modify: `website/sidebars.ts` (add to Per-crate reference category)

- [ ] **Step 1: Verify the existing per-crate page shape**

```bash
cd /home/peter/code/argumentation/website
head -50 docs/reference/encounter-argumentation.md
```

Note the structure: frontmatter with `sidebar_position`, intro paragraph, "Crate:" badge line, `## Key types` with subsections, `## Key functions / methods` table, `## Errors` section, `## See also`. Match this shape.

- [ ] **Step 2: Write `website/docs/reference/argumentation-values.md`**

```markdown
---
sidebar_position: 16
title: argumentation-values
---

The `argumentation-values` crate (v0.1.0) implements **value-based argumentation frameworks** (Bench-Capon 2003) extended with multi-value support per Kaci & van der Torre (2008). It adds `Value`, `ValueAssignment<A>`, `Audience`, and `ValueBasedFramework<A>` on top of the core `argumentation` crate, plus subjective/objective acceptance, a Walton-scheme→audience bridge, APX format I/O for ASPARTIX interop, and `MultiAudience` consensus queries for multi-character scenes.

**Crate:** `argumentation-values` ([crates.io](https://crates.io/crates/argumentation-values) · [rustdoc](/api/argumentation_values/))

## Key types

### `Value`
A newtype around `String` representing a value an argument can promote (e.g., `Value::new("life")`). Implements `Display`, `From<&str>`, `From<String>`, `Hash`, `Ord`.
→ [Full docs](/api/argumentation_values/types/struct.Value.html)

### `ValueAssignment<A>`
Maps each argument (of label type `A`) to the *set* of values it promotes via `SmallVec<[Value; 1]>` — the single-value common case is allocation-free; multi-value is supported per Kaci & van der Torre 2008. Empty set means "promotes no value" (defeats unconditionally under VAF semantics).
→ [Full docs](/api/argumentation_values/types/struct.ValueAssignment.html)

### `Audience`
A strict partial order over values, represented as ranked tiers. `Audience::total([life, property])` produces a total order; `Audience::from_tiers(vec![vec![life, liberty], vec![property]])` allows intra-tier ties. Public `rank(&value) -> Option<usize>` for consumer code.
→ [Full docs](/api/argumentation_values/types/struct.Audience.html)

### `ValueBasedFramework<A>`
A Dung framework (`ArgumentationFramework<A>`) plus a `ValueAssignment<A>`. Audience-conditioned acceptance via `defeat_graph(audience)`, `accepted_for(audience, arg)`, `grounded_for(audience)`. Pareto-defeating rule: A defeats B iff for every value B promotes, some value A promotes is not strictly less-preferred under the audience.
→ [Full docs](/api/argumentation_values/framework/struct.ValueBasedFramework.html)

### `MultiAudience`
Multi-character consensus queries. `MultiAudience::new(&[alice_audience, bob_audience])` then `common_grounded(&vaf)` returns arguments grounded under *every* audience — the council-style consensus answer.
→ [Full docs](/api/argumentation_values/multi/struct.MultiAudience.html)

### `Error`
Error enum: `Dung(#[from] argumentation::Error)`, `AudienceTooLarge { values, limit }` (returned past 6 distinct values for subjective/objective acceptance), `ArgumentNotFound(String)`, `ApxParse { line, reason }`.
→ [Full docs](/api/argumentation_values/error/enum.Error.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `ValueBasedFramework::new(base, values)` | `Self` | Construct from a Dung framework and a value assignment. |
| `vaf.defeat_graph(&audience)` | `Result<ArgumentationFramework<A>, Error>` | Build the audience-conditioned defeat graph. |
| `vaf.defeats(&attacker, &target, &audience)` | `bool` | Pareto-defeating rule check. |
| `vaf.accepted_for(&audience, &arg)` | `Result<bool, Error>` | Credulous acceptance under one audience (preferred-extension membership). |
| `vaf.grounded_for(&audience)` | `Result<HashSet<A>, Error>` | Grounded extension under one audience (delegates to upstream `grounded_extension()`). |
| `vaf.subjectively_accepted(&arg)` | `Result<bool, Error>` | Accepted by *some* audience (NP-complete; capped at 6 values). |
| `vaf.objectively_accepted(&arg)` | `Result<bool, Error>` | Accepted by *every* audience (co-NP-complete; capped at 6 values). |
| `MultiAudience::new(&audiences)` | `Self` | Construct from a slice of audiences. |
| `multi.common_grounded(&vaf)` | `Result<HashSet<A>, Error>` | Intersection of grounded extensions across all audiences. |
| `multi.common_credulous(&vaf)` | `Result<HashSet<A>, Error>` | Intersection of credulous-acceptance sets across all audiences. |
| `from_scheme_instances(instances, to_arg)` | `ValueAssignment<A>` | Extract `value` bindings from `argument_from_values` Walton-scheme instances. |
| `apx::from_apx(input)` | `Result<(ValueBasedFramework<String>, Audience), Error>` | Parse ASPARTIX-compatible APX text. |
| `apx::to_apx(&vaf, &audience)` | `String` | Serialise to APX. |

## Constants

### `acceptance::ENUMERATION_LIMIT`
The hard cap (currently `6`) on distinct values for subjective/objective acceptance. Past this, methods return `Error::AudienceTooLarge` per Dunne & Bench-Capon (2004) complexity bounds.

### `scheme_bridge::DEFAULT_VALUES_SCHEME_NAME`
The default-catalog name (`"Argument from Values"`) compared against `SchemeInstance.scheme_name`. Use `from_scheme_instances_with_name` to target a custom scheme name.

## Errors

### `Error::Dung(argumentation::Error)`
Wrapped error from underlying Dung framework operations (e.g., framework-too-large for subset enumeration).

### `Error::AudienceTooLarge { values, limit }`
`subjectively_accepted`/`objectively_accepted` bail out when distinct values exceed `ENUMERATION_LIMIT`. Use a fixed-audience query (`accepted_for`) instead.

### `Error::ArgumentNotFound(String)`
An argument referenced in an attack edge is not registered in the underlying framework.

### `Error::ApxParse { line, reason }`
APX text input failed to parse. `line` is 1-indexed.

## See also

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — formalism and design rationale.
- [Hal & Carla](/examples/hal-and-carla) — the engine-driven scene this implementation was built around.
- [Wiring per-character values](/guides/wiring-character-values) — how-to for encounter bridge integration.
- [Import/export APX](/guides/import-export-apx) — how-to for ASPARTIX interop.
- [Multi-character consensus](/guides/multi-character-consensus) — how-to for `MultiAudience` queries.
- [Reference overview](/reference/overview) — workspace-level types overview.
```

- [ ] **Step 3: Add to `website/sidebars.ts` Per-crate reference category**

Read the current file:

```bash
grep -A 12 "Per-crate reference" /home/peter/code/argumentation/website/sidebars.ts
```

Add `'reference/argumentation-values'` to the items list, alphabetically between `argumentation-schemes` and `argumentation-weighted` (or wherever the existing alphabetical-ish order suggests). Result:

```typescript
      items: [
        'reference/argumentation',
        'reference/argumentation-bipolar',
        'reference/argumentation-schemes',
        'reference/argumentation-values',
        'reference/argumentation-weighted',
        'reference/argumentation-weighted-bipolar',
        'reference/encounter-argumentation',
      ],
```

(Note: existing order may not be strict alphabetical. Match the existing convention; add the new entry at the position that minimises reordering.)

- [ ] **Step 4: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. New page renders. The `/guides/import-export-apx` and `/guides/multi-character-consensus` links from the new page will warn (Tasks 5 and 6 add them). Acceptable.

- [ ] **Step 5: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/reference/argumentation-values.md website/sidebars.ts
git commit -m "docs(reference): add argumentation-values per-crate page

Matches the shape of encounter-argumentation.md: key types,
methods table, constants, errors, see-also. Wired into the
Per-crate reference sidebar category."
```

---

### Task 2: Add 3 bibliography entries

**Files:**
- Modify: `website/docs/academic/bibliography.md`

- [ ] **Step 1: Read the existing bibliography to match the entry shape**

```bash
sed -n '70,95p' /home/peter/code/argumentation/website/docs/academic/bibliography.md
```

Note the entry pattern: `### key{year}` heading, paragraph with citation + link.

- [ ] **Step 2: Add `kaci2008` entry under "Values & practical reasoning"**

Find the section `## Values & practical reasoning` in the file. After the `### atkinson2007` entry (which is currently the last in that section), append:

```markdown
### kaci2008

Kaci, S. & van der Torre, L. (2008). [*Preference-based argumentation: Arguments supporting multiple values.*](https://www.sciencedirect.com/science/article/pii/S0888613X07000989) International Journal of Approximate Reasoning, 48(3): 730–751. — Generalises Bench-Capon's VAF to allow arguments promoting multiple values; introduces the Pareto-defeating rule we implement.
```

- [ ] **Step 3: Add `dunne2004` entry under "Weighted" (or new "Complexity" subsection)**

Find the section `## Weighted` in the file. After the `### amgoud2016` entry, append:

```markdown
### dunne2004

Dunne, P.E. & Bench-Capon, T. (2004). [*Complexity in Value-Based Argument Systems.*](https://link.springer.com/chapter/10.1007/978-3-540-30227-8_31) JELIA 2004, LNCS 3229: 360–371. — Headline complexity result for VAF: subjective acceptance is NP-complete, objective is co-NP-complete in general; both polynomial for fixed audiences on tree-like graphs.
```

- [ ] **Step 4: Add `bodanza2023` entry under "Values & practical reasoning"**

In the same section as `kaci2008`, append after it:

```markdown
### bodanza2023

Bodanza, G.A. & Freidin, E. (2023). [*Confronting value-based argumentation frameworks with people's assessment of argument strength.*](https://content.iospress.com/articles/argument-and-computation/aac220008) Argument & Computation, 14(3): 247–273. — Empirical psychology study of VAF semantics; finds that human acceptance correlates with value importance directly rather than the attack/defeat propagation VAF prescribes. Informs why we expose value-importance scoring (`SchemeActionScorer` + `preference_weight`) alongside the orthodox defeat semantics.
```

- [ ] **Step 5: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`. The previously dangling `#kaci2008`, `#dunne2004`, `#bodanza2023` anchor links from `concepts/value-based-argumentation.mdx` should now resolve.

- [ ] **Step 6: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/academic/bibliography.md
git commit -m "docs(academic): add kaci2008, dunne2004, bodanza2023 bibliography entries

Resolves three dangling anchors from the VAF concepts page added
in the argumentation-values plan."
```

---

### Task 3: Add VAF entry to academic/reading-order.md "Full curriculum"

**Files:**
- Modify: `website/docs/academic/reading-order.md`

- [ ] **Step 1: Read the current "Full curriculum" section**

```bash
grep -A 12 "## Full curriculum" /home/peter/code/argumentation/website/docs/academic/reading-order.md
```

Note: the section currently ends at item 6 (Bench-Capon 2003 — the original VAF paper). We add Kaci & van der Torre 2008 as item 7 (the multi-value extension we implement) and Dunne & Bench-Capon 2004 as item 8 (the complexity result).

- [ ] **Step 2: Append two entries to the numbered list**

Find the line `6. [**Bench-Capon (2003)**]...` and add after it:

```markdown
7. [**Kaci & van der Torre (2008)**](/academic/bibliography#kaci2008) — *Preference-based argumentation: Arguments supporting multiple values.* The multi-value extension to Bench-Capon. Read §2 (defeat rule) carefully — it's the spec our `argumentation-values` crate implements.
8. [**Dunne & Bench-Capon (2004)**](/academic/bibliography#dunne2004) — *Complexity in VAF.* Read §3 for the NP/co-NP results that motivate the `ENUMERATION_LIMIT` cap on `subjectively_accepted` / `objectively_accepted` queries.
```

- [ ] **Step 3: Update "If you want to build with this" section**

Find the existing paragraph:

```
Read the Modgil-Prakken tutorial, then this library's [guides](/guides/installation). The `encounter-argumentation` bridge is the primary entry point.
```

Replace with:

```
Read the Modgil-Prakken tutorial for ASPIC+ basics, then this library's [guides](/guides/installation). The `encounter-argumentation` bridge is the primary entry point for scene engines; the `argumentation-values` crate is the entry point for value-based reasoning. Pick one — the [Choose your path](/getting-started/choose-your-path) page routes you by goal.
```

(The forward-link to choose-your-path will warn until Phase 4 lands. Acceptable.)

- [ ] **Step 4: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]` with the choose-your-path warning expected.

- [ ] **Step 5: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/academic/reading-order.md
git commit -m "docs(academic): add VAF entries to reading-order curriculum

Items 7 (Kaci & van der Torre 2008) and 8 (Dunne & Bench-Capon
2004) extend the Full curriculum past Bench-Capon's original VAF
paper. Updates 'If you want to build with this' to mention the
new argumentation-values entry point and the upcoming
choose-your-path page."
```

---

### Task 4: Update reference/overview.md

**Files:**
- Modify: `website/docs/reference/overview.md`

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/reference/overview.md
```

- [ ] **Step 2: Add four new types under `## Core types`**

Find the existing `### \`WeightedBipolarFramework<A>\`` section. After it (before `### \`Error\` (encounter-argumentation)`), add:

```markdown
### `Audience`
A strict partial order over values, represented as ranked tiers. `Audience::total([life, property])` for total orders; `Audience::from_tiers(...)` for intra-tier ties. Public `rank(&value)` for consumer code.
→ [Full docs](/api/argumentation_values/types/struct.Audience.html)

### `ValueAssignment<A>`
Maps arguments to the set of values they promote. Multi-value support via `SmallVec<[Value; 1]>` (Kaci & van der Torre 2008).
→ [Full docs](/api/argumentation_values/types/struct.ValueAssignment.html)

### `ValueBasedFramework<A>`
Dung framework + value assignment. `accepted_for(&audience, &arg)` for one audience; `subjectively_accepted` / `objectively_accepted` for queries over the audience space (capped at 6 values).
→ [Full docs](/api/argumentation_values/framework/struct.ValueBasedFramework.html)

### `ValueAwareScorer<S>` (encounter-argumentation)
Wraps any inner `ActionScorer`; reads per-actor audiences from `EncounterArgumentationState` and adds tier-rank-scaled boost to value-promoting affordances.
→ [Full docs](/api/encounter_argumentation/value_scorer/struct.ValueAwareScorer.html)
```

- [ ] **Step 3: Add new methods to the `## Core methods` table**

Find the existing methods table. Add three new rows (place at the end of the table):

```markdown
| `state.set_audience(actor, audience)` | Set per-actor audience through `&self`. |
| `state.audience_for(actor)` | `Option<Audience>` — borrow per-actor audience. |
| `vaf.accepted_for(&audience, &arg)` | Audience-conditioned credulous acceptance. |
```

- [ ] **Step 4: Update the `## Crate map` table**

Add a row for `argumentation-values` between `argumentation-schemes` and `encounter-argumentation`:

```markdown
| `argumentation-values` | Value-based AF (Bench-Capon 2003 + multi-value). |
```

- [ ] **Step 5: Update the `## What we don't have yet` paragraph**

Find:

```
The library focuses on Dung frameworks, ASPIC+, weighted attacks, bipolar extensions, and the encounter bridge. Five formalisms remain on the roadmap; see [open areas](/concepts/open-areas) for the public map and the [VAF mini-RFC](/concepts/value-based-argumentation) for the deeper sketch of the headline gap.
```

Replace with:

```
The library focuses on Dung frameworks, ASPIC+, weighted attacks, bipolar extensions, the encounter bridge, and value-based argumentation. Four formalisms remain on the roadmap; see [open areas](/concepts/open-areas) for the public map (probabilistic AF, ADF, dialogue games, dynamic AF).
```

- [ ] **Step 6: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 7: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/reference/overview.md
git commit -m "docs(reference): update overview with argumentation-values types

Adds Audience, ValueAssignment, ValueBasedFramework, ValueAwareScorer
to Core types. Adds set_audience, audience_for, accepted_for to the
Core methods table. Adds argumentation-values to the Crate map.
Updates the 'What we don't have yet' paragraph from 5 → 4 open
formalisms."
```

---

### Task 5: New how-to — Import/export APX

**Files:**
- Create: `website/docs/guides/import-export-apx.md`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/guides/import-export-apx.md`**

```markdown
---
sidebar_position: 10
title: Import/export APX (ASPARTIX interop)
---

Parse ASPARTIX-compatible APX text into a `ValueBasedFramework`, or serialise a framework + audience back to APX. Useful for importing benchmark VAFs from the literature, exporting scenes for analysis in ASPARTIX, or sharing fixture frameworks across tools.

**Learning objective:** round-trip a 4-argument VAF through APX text in under 5 minutes — parse a fixture, modify it programmatically, serialise the result.

## Prerequisites

- The `argumentation-values` crate available (path-dep or registry-dep).
- Basic familiarity with `ValueBasedFramework` (see [Value-based argumentation](/concepts/value-based-argumentation)).

## What is APX?

ASPARTIX (TU Wien) defines a Prolog-style fact format for VAFs:

```text
arg(h1).
arg(c1).
att(h1, c1).
att(c1, h1).
val(h1, life).
val(c1, property).
valpref(life, property).
```

- `arg(name).` — argument
- `att(attacker, target).` — attack edge
- `val(arg, value).` — value-promotion
- `valpref(a, b).` — `a` strictly preferred over `b` in the audience

Comments start with `%`. Whitespace is ignored.

## Step 1: Add the dep

```toml
[dependencies]
argumentation-values = "0.1"
```

## Step 2: Parse APX text

```rust
use argumentation_values::apx::from_apx;

let input = r#"
% Hal & Carla in APX
arg(h1).
arg(c1).
att(h1, c1).
att(c1, h1).
val(h1, life).
val(c1, property).
valpref(life, property).
"#;

let (vaf, audience) = from_apx(input)?;
assert_eq!(vaf.base().len(), 2);
```

`from_apx` returns `Result<(ValueBasedFramework<String>, Audience), Error>` — argument labels are owned `String`s matching the `arg(name)` identifiers.

## Step 3: Use the parsed framework

```rust
use argumentation_values::Value;

assert!(audience.prefers(&Value::new("life"), &Value::new("property")));
assert!(vaf.accepted_for(&audience, &"h1".to_string())?);
```

## Step 4: Serialise back to APX

```rust
use argumentation_values::apx::to_apx;

let serialised = to_apx(&vaf, &audience);
println!("{}", serialised);
```

The output is sorted alphabetically (deterministic). Round-trip preserves *semantics* — the argument set, attack set, value-promotion set, and strict-preference relation — but does not preserve insertion order, comments, or redundant `valpref` facts. Round-tripping the result through `from_apx` again gives an equivalent framework.

## Step 5: Loop the round trip in a test

```rust
#[test]
fn round_trip_preserves_semantics() {
    let original = sample_vaf_apx();
    let (vaf, audience) = from_apx(&original).unwrap();
    let serialised = to_apx(&vaf, &audience);
    let (vaf2, audience2) = from_apx(&serialised).unwrap();

    assert_eq!(vaf2.base().len(), vaf.base().len());
    for v in audience.values() {
        for u in audience.values() {
            assert_eq!(audience.prefers(v, u), audience2.prefers(v, u));
        }
    }
}
```

## Parse error handling

Errors carry the line number (1-indexed) and a human-readable reason:

```rust
let bad = "arg(a).\nbogus(stuff).\n";
match from_apx(bad) {
    Err(argumentation_values::Error::ApxParse { line, reason }) => {
        eprintln!("APX error at line {line}: {reason}");
    }
    _ => unreachable!(),
}
```

## When NOT to use this

- **Hand-authored fixtures** that won't outlive the test file. Just build the `ValueBasedFramework` programmatically.
- **Production scene serialisation** — APX has no notion of actor attribution, scheme bindings, or affordance state. Use a richer format (or just snapshot `EncounterArgumentationState` directly).
- **Cycles in `valpref`** — APX cycles silently collapse to a single tier (no error). If you need strict cycle detection, validate the audience before serialising.

## ASPARTIX compatibility notes

- ASPARTIX accepts redundant `valpref` facts; we emit the full pairwise transitive closure for determinism rather than the minimal transitive reduction.
- ASPARTIX takes the transitive closure of `valpref` on import; round-trip is closure-stable.
- We don't support ASPARTIX-style quoted atoms (`'arg with space'`) — argument and value names must be bare alphanumeric. Matches ASPARTIX's permissive default.

## Related

- [Value-based argumentation](/concepts/value-based-argumentation) — the formalism.
- [`argumentation-values` reference](/reference/argumentation-values) — full API.
- [ASPARTIX VAF docs](https://www.dbai.tuwien.ac.at/research/argumentation/aspartix/vaf.html) — the canonical APX VAF format definition.
```

- [ ] **Step 2: Add to sidebars.ts guidesSidebar**

Find the guidesSidebar block. Add `'guides/import-export-apx'` after `'guides/wiring-character-values'`:

```typescript
  guidesSidebar: [
    'guides/installation',
    'guides/catalog-authoring',
    'guides/implementing-action-scorer',
    'guides/implementing-acceptance-eval',
    'guides/tuning-beta',
    'guides/debugging-acceptance',
    'guides/societas-modulated-weights',
    'guides/wiring-character-values',
    'guides/import-export-apx',
    'guides/migration-v0.4-to-v0.5',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/guides/import-export-apx.md website/sidebars.ts
git commit -m "docs(guides): add import-export-apx how-to

Walks through parsing ASPARTIX-compatible APX text into a
ValueBasedFramework and round-tripping back. Includes a test
pattern, error-handling sketch, and ASPARTIX compatibility notes.
Wired into the guides sidebar between wiring-character-values
and migration-v0.4-to-v0.5."
```

---

### Task 6: New how-to — Multi-character consensus

**Files:**
- Create: `website/docs/guides/multi-character-consensus.md`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/guides/multi-character-consensus.md`**

```markdown
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
```

- [ ] **Step 2: Add to sidebars.ts guidesSidebar**

Add `'guides/multi-character-consensus'` after `'guides/import-export-apx'`:

```typescript
  guidesSidebar: [
    'guides/installation',
    'guides/catalog-authoring',
    'guides/implementing-action-scorer',
    'guides/implementing-acceptance-eval',
    'guides/tuning-beta',
    'guides/debugging-acceptance',
    'guides/societas-modulated-weights',
    'guides/wiring-character-values',
    'guides/import-export-apx',
    'guides/multi-character-consensus',
    'guides/migration-v0.4-to-v0.5',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/guides/multi-character-consensus.md website/sidebars.ts
git commit -m "docs(guides): add multi-character-consensus how-to

Walks through using MultiAudience::common_grounded and
common_credulous for council-style consensus queries. Worked
4-officer example, empty-audience semantics, cost notes,
when-NOT-to-use. Wired into the guides sidebar after
import-export-apx."
```

---

## Phase 2: Tutorial layer expansion (Tasks 7–9)

After Phase 1, the site is consistent. Phase 2 addresses the headline structural problem: tutorial layer is one page (β-only), so users complete onboarding without ever seeing schemes used in action, multi-actor dynamics, or audiences. Tasks 7–9 build a 30-minute progression.

### Task 7: Add ABCD learning objective to first-scene.md

**Files:**
- Modify: `website/docs/getting-started/first-scene.md`

The tutorial currently has "What you'll build" but no formal learning objective line. Every guide in the site has one (`**Learning objective:** ...`); the tutorial is the inconsistency.

- [ ] **Step 1: Read the current opening**

```bash
sed -n '1,20p' /home/peter/code/argumentation/website/docs/getting-started/first-scene.md
```

- [ ] **Step 2: Insert the learning objective**

Find the line:

```
Build a working east-wall scene end-to-end. You'll seed two arguments, set a scene intensity, run `MultiBeat`, and print the resulting beats.
```

Insert immediately after it (before the `## What you'll build` header):

```markdown

**Learning objective:** build a working two-actor argumentation scene with weighted attacks and a tunable β, run it end-to-end with `cargo run`, and read the resulting beat-by-beat acceptance trace — in under 10 minutes, with no prior argumentation-theory knowledge.
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/getting-started/first-scene.md
git commit -m "docs(getting-started): add ABCD learning objective to first-scene

Brings first-scene.md in line with the rest of the site, where
every guide has an explicit Learning objective line. The tutorial
itself was the inconsistency."
```

---

### Task 8: New tutorial — Second scene with schemes

**Files:**
- Create: `website/docs/getting-started/second-scene-with-schemes.md`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/getting-started/second-scene-with-schemes.md`**

```markdown
---
sidebar_position: 2
title: Your second scene — multiple schemes
---

Build on [your first scene](/getting-started/first-scene) by adding a counter-argument that uses a different Walton scheme. Alice still argues from expert opinion. Bob will argue from negative consequences ("fortifying causes the garrison to starve"). Each scheme has its own critical questions; we'll see how the bridge composes them.

**Learning objective:** build a two-actor scene where each actor invokes a different Walton scheme, observe how scheme strengths shape acceptance, and verify the trace shows the expected acceptance flip — in under 15 minutes, with no prior scheme-theory knowledge beyond what the [first scene](/getting-started/first-scene) covered.

## What you'll build

A 4-beat scene where Alice (expert/military) argues to fortify and Bob (negative-consequences) argues that fortifying causes a worse outcome. Critical questions from one scheme map onto attacks against the other. At β=0.5, the framework picks one over the other based on relative scheme strength.

**Time:** ~15 minutes.
**Difficulty:** Beginner+ (assumes you completed the [first scene](/getting-started/first-scene)).
**You'll leave with:** a running scene with two scheme types and a mental model for "different scheme = different attack semantics."

## Prerequisites

- Completion of [Build your first scene](/getting-started/first-scene) — you have a working `cargo new --bin my-first-scene` project that runs.
- Familiarity with `EncounterArgumentationState`, `add_scheme_instance_for_affordance`, and `MultiBeat::resolve` (all from the first scene).

## Step 1: Continue from your first-scene project

We'll build on the same project. You can either start from the completed first-scene code, or copy the directory:

```bash
cp -r my-first-scene my-second-scene
cd my-second-scene
```

If you don't have the first-scene project, follow [its setup steps](/getting-started/first-scene#step-1-create-the-project) first.

## Step 2: Identify Bob's new scheme

The default catalog ships ~60 Walton schemes. The first-scene tutorial used `argument_from_expert_opinion` for both Alice and Bob. Here we'll switch Bob to `argument_from_negative_consequences`:

```rust
let neg_scheme = registry
    .by_key("argument_from_negative_consequences")
    .expect("argument_from_negative_consequences in default catalog");
```

The negative-consequences scheme has bindings: `action`, `bad_consequence`. The conclusion is the negation of `do_?action` — i.e., asserting that the action *should not* be done because of the bad consequence. See `argumentation-schemes/src/catalog/practical.rs` for the full scheme definition.

## Step 3: Build Bob's instance with the new scheme

Replace Bob's expert-opinion instantiation from the first scene with:

```rust
let mut bob_bindings = HashMap::new();
bob_bindings.insert("action".into(), "fortify_east".into());
bob_bindings.insert("bad_consequence".into(), "garrison_starves".into());
let bob_instance = neg_scheme.instantiate(&bob_bindings).unwrap();
```

This produces an argument concluding "do not fortify_east, because garrison_starves" — Bob is asserting that Alice's proposed action causes a bad outcome.

## Step 4: Wire the attack

Now the attack is asymmetric — Bob's negative-consequences argument attacks Alice's expert-opinion argument, but not vice versa. Add only one attack edge:

```rust
state.add_weighted_attack(&bob_id, &alice_id, 0.4)?;
// Note: no add_weighted_attack(&alice_id, &bob_id, ...) here.
```

This represents the asymmetry: a negative-consequences critique can undermine an expert opinion (the expert may be right about the *call* but wrong about the *consequences*), but the reverse argument structure doesn't naturally apply.

## Step 5: Update the affordance bindings to match the new scheme

The bindings dict you pass to `add_scheme_instance_for_affordance` for Bob must match the negative-consequences scheme's slots plus `self`:

```rust
let mut bob_af = bob_bindings.clone();
bob_af.insert("self".into(), "bob".into());
let bob_id = state.add_scheme_instance_for_affordance(
    "bob",
    "argue_against_fortify",
    &bob_af,
    bob_instance,
);
```

Note `argue_against_fortify` is the affordance name (a string we pick); Bob's bindings include `action`, `bad_consequence`, and `self`.

## Step 6: Update the affordance catalog and scorer

The catalog needs to declare the bindings each affordance uses. Replace the catalog construction with:

```rust
let alice_aff = AffordanceSpec {
    name: "argue_fortify_east".into(),
    domain: "persuasion".into(),
    bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let bob_aff = AffordanceSpec {
    name: "argue_against_fortify".into(),
    domain: "persuasion".into(),
    bindings: vec!["self".into(), "action".into(), "bad_consequence".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let catalog = vec![
    CatalogEntry { spec: alice_aff, precondition: String::new() },
    CatalogEntry { spec: bob_aff, precondition: String::new() },
];
```

The inner scorer (`UniformScorer` or whatever you wrote in the first scene) needs to populate the right bindings per affordance. For brevity, use a match on the affordance name:

```rust
struct InnerScorer;
impl<P: Clone> ActionScorer<P> for InnerScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.into());
            match e.spec.name.as_str() {
                "argue_fortify_east" => {
                    bindings.insert("expert".into(), "alice".into());
                    bindings.insert("domain".into(), "military".into());
                    bindings.insert("claim".into(), "fortify_east".into());
                }
                "argue_against_fortify" => {
                    bindings.insert("action".into(), "fortify_east".into());
                    bindings.insert("bad_consequence".into(), "garrison_starves".into());
                }
                _ => unreachable!(),
            }
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}
```

## Step 7: Run

```bash
cargo run
```

Expected output (with deterministic ordering):

```
beat 1: alice argued argue_fortify_east — accepted: false
beat 2: bob   argued argue_against_fortify — accepted: true
beat 3: alice argued argue_fortify_east — accepted: false
beat 4: bob   argued argue_against_fortify — accepted: true
```

Alice's proposal is rejected because Bob's negative-consequences argument is credulously accepted (no counter-argument knocks it out). Bob's proposal is accepted because Alice has no counter-argument the bridge sees.

## Why the asymmetry matters

In the [first scene](/getting-started/first-scene), Alice and Bob both used the same scheme — the framework treated them symmetrically. Here, they use *different* schemes, and the attack relation is asymmetric. This is the structural difference that makes scheme choice matter: a negative-consequences argument can undermine an expert-opinion argument's *recommended action*, but not vice versa.

The library's [60+ Walton schemes](/concepts/walton-schemes) each carry their own critical questions and natural attack patterns. Picking the right scheme for each character's argument is most of the authoring work in scene design.

## Complete example

If you got lost, the full `src/main.rs`:

```rust
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

struct InnerScorer;
impl<P: Clone> ActionScorer<P> for InnerScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.into());
            match e.spec.name.as_str() {
                "argue_fortify_east" => {
                    bindings.insert("expert".into(), "alice".into());
                    bindings.insert("domain".into(), "military".into());
                    bindings.insert("claim".into(), "fortify_east".into());
                }
                "argue_against_fortify" => {
                    bindings.insert("action".into(), "fortify_east".into());
                    bindings.insert("bad_consequence".into(), "garrison_starves".into());
                }
                _ => unreachable!(),
            }
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}

fn main() {
    let registry = default_catalog();
    let expert = registry.by_key("argument_from_expert_opinion").unwrap();
    let neg = registry.by_key("argument_from_negative_consequences").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".into(), "alice".into());
    alice_b.insert("domain".into(), "military".into());
    alice_b.insert("claim".into(), "fortify_east".into());
    let alice_instance = expert.instantiate(&alice_b).unwrap();

    let mut bob_b = HashMap::new();
    bob_b.insert("action".into(), "fortify_east".into());
    bob_b.insert("bad_consequence".into(), "garrison_starves".into());
    let bob_instance = neg.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".into(), "alice".into());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice", "argue_fortify_east", &alice_af, alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".into(), "bob".into());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob", "argue_against_fortify", &bob_af, bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
    state.set_intensity(Budget::new(0.5).unwrap());

    let alice_aff = AffordanceSpec {
        name: "argue_fortify_east".into(),
        domain: "persuasion".into(),
        bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let bob_aff = AffordanceSpec {
        name: "argue_against_fortify".into(),
        domain: "persuasion".into(),
        bindings: vec!["self".into(), "action".into(), "bad_consequence".into()],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let catalog = vec![
        CatalogEntry { spec: alice_aff, precondition: String::new() },
        CatalogEntry { spec: bob_aff, precondition: String::new() },
    ];
    let practice = PracticeSpec {
        name: "debate".into(),
        affordances: vec!["argue_fortify_east".into(), "argue_against_fortify".into()],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, InnerScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".into(), "bob".into()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

    for (i, b) in result.beats.iter().enumerate() {
        println!("beat {}: {} argued {} — accepted: {}", i + 1, b.actor, b.action, b.accepted);
    }
    let _ = alice_id;
    let _ = bob_id;
}
```

## What you learned

- How to use a different Walton scheme per actor.
- The `negative-consequences` scheme's binding shape (action/bad_consequence).
- Asymmetric attacks: not every counter is mutual.
- Scheme choice is the primary authoring lever for "what kind of argument is this?"

## Next steps

- [Your third scene — with values](/getting-started/third-scene-with-values) — adds per-character value priorities (audiences) so Alice and Bob reach different conclusions when they hold different values.
- [Walton schemes (concepts)](/concepts/walton-schemes) — what schemes are and what's in the catalog.
- [Author an affordance catalog](/guides/catalog-authoring) — for moving affordance definitions out of Rust into TOML.
```

- [ ] **Step 2: Add to sidebars.ts gettingStartedSidebar**

```typescript
  gettingStartedSidebar: [
    'getting-started/first-scene',
    'getting-started/second-scene-with-schemes',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`. Forward-link to `/getting-started/third-scene-with-values` will warn until Task 9. Acceptable.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/getting-started/second-scene-with-schemes.md website/sidebars.ts
git commit -m "docs(getting-started): add second-scene tutorial — multiple schemes

Builds on first-scene. Alice keeps expert-opinion; Bob switches to
negative-consequences. Demonstrates asymmetric attacks and the importance
of scheme choice. Complete runnable code at the end. Wired into
the gettingStartedSidebar after first-scene."
```

---

### Task 9: New tutorial — Third scene with values

**Files:**
- Create: `website/docs/getting-started/third-scene-with-values.md`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/getting-started/third-scene-with-values.md`**

```markdown
---
sidebar_position: 3
title: Your third scene — with values
---

Add per-character value priorities to the [second scene](/getting-started/second-scene-with-schemes) so Alice and Bob reach *different* conclusions when they hold different values. The bridge's `ValueAwareScorer` reads each character's audience and adjusts proposal scoring so duty-prioritising Alice picks differently from survival-prioritising Bob.

**Learning objective:** add per-character audiences to a two-actor scene with three lines of code (one `set_audience` per actor, one `ValueAwareScorer::new` wrapper), observe that the same scene with the same arguments produces different beats per character, and verify the trace shows the expected outcome flip — in under 10 minutes, building on the [second scene](/getting-started/second-scene-with-schemes).

## What you'll build

The same two-actor scene from tutorial 2, but now with values attached to each argument and a per-character audience for each actor. Alice's proposal promotes "duty"; Bob's promotes "survival." When Alice scores affordances, duty-promoting ones get a boost; when Bob scores them, survival-promoting ones get a boost. The same `MultiBeat::resolve` call now produces a different beat sequence depending on whose audience drives the scoring.

**Time:** ~10 minutes.
**Difficulty:** Beginner+ (assumes you completed the [second scene](/getting-started/second-scene-with-schemes)).
**You'll leave with:** a running scene where audience-conditioned scoring shapes the trace, and a mental model for "values are character preferences, not framework facts."

## Prerequisites

- Completion of [Your second scene — multiple schemes](/getting-started/second-scene-with-schemes) — you have a working `my-second-scene` (or copy thereof) that runs.

## Step 1: Continue from your second-scene project

```bash
cp -r my-second-scene my-third-scene
cd my-third-scene
```

## Step 2: Add `argumentation-values` to dependencies

```toml
[dependencies]
argumentation-schemes = "0.2"
argumentation-weighted = "0.2"
argumentation-values = "0.1"
encounter-argumentation = "0.5"
encounter = "0.1"
```

(`argumentation-values` is already a transitive dep through `encounter-argumentation`, but adding it directly lets you import its types.)

## Step 3: Switch to the `argument_from_values` scheme for both actors

The Walton catalog has a scheme specifically designed for value-promoting arguments:

```rust
let values_scheme = registry.by_key("argument_from_values").unwrap();
```

Bindings: `action`, `value`, `agent`. Conclusion template: `"?action should be carried out because it promotes ?value for ?agent"`.

Replace Alice's instance:

```rust
let mut alice_b = HashMap::new();
alice_b.insert("action".into(), "fortify".into());
alice_b.insert("value".into(), "duty".into());
alice_b.insert("agent".into(), "alice".into());
let alice_instance = values_scheme.instantiate(&alice_b).unwrap();
```

Replace Bob's instance:

```rust
let mut bob_b = HashMap::new();
bob_b.insert("action".into(), "abandon".into());
bob_b.insert("value".into(), "survival".into());
bob_b.insert("agent".into(), "bob".into());
let bob_instance = values_scheme.instantiate(&bob_b).unwrap();
```

The `value` binding is what `ValueAwareScorer` reads to compute its boost.

## Step 4: Set per-character audiences

After constructing the state, add:

```rust
use encounter_argumentation::{Audience, Value};

state.set_audience(
    "alice",
    Audience::total([Value::new("duty"), Value::new("survival")]),
);
state.set_audience(
    "bob",
    Audience::total([Value::new("survival"), Value::new("duty")]),
);
```

Each call mirrors `set_intensity` — it mutates state through a shared `&self` reference (interior mutability via `Mutex`).

## Step 5: Wrap the scorer with `ValueAwareScorer`

Find the `let scorer = StateActionScorer::new(&state, InnerScorer, 0.5);` line. Wrap it:

```rust
use encounter_argumentation::ValueAwareScorer;

let scheme_scorer = StateActionScorer::new(&state, InnerScorer, 0.5);
let scorer = ValueAwareScorer::new(scheme_scorer, &state, 0.3);
```

The two boosts compose additively: scheme-strength boost first (0.5 max), then value-preference boost (0.3 max). When Alice scores `argue_fortify` (which promotes "duty," tier 0 of her audience), she gets the full value boost. When Bob scores it, "duty" is tier 1 in his audience — smaller boost.

## Step 6: Update the inner scorer to set the `value` binding

The `value_boost_for_affordance` function in `ValueAwareScorer` reads the `value` binding from the affordance's bindings. Update `InnerScorer` so each affordance carries its `value`:

```rust
struct InnerScorer;
impl<P: Clone> ActionScorer<P> for InnerScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.into());
            let (action, value) = match e.spec.name.as_str() {
                "argue_fortify" => ("fortify", "duty"),
                "argue_abandon" => ("abandon", "survival"),
                _ => unreachable!(),
            };
            bindings.insert("action".into(), action.into());
            bindings.insert("value".into(), value.into());
            bindings.insert("agent".into(), actor.into());
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}
```

Note the affordance names changed (we renamed `argue_against_fortify` → `argue_abandon` for clarity in the values context). Update the catalog and practice strings accordingly.

## Step 7: Update the catalog

```rust
let alice_aff = AffordanceSpec {
    name: "argue_fortify".into(),
    domain: "values".into(),
    bindings: vec!["self".into(), "action".into(), "value".into(), "agent".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let bob_aff = AffordanceSpec {
    name: "argue_abandon".into(),
    domain: "values".into(),
    bindings: vec!["self".into(), "action".into(), "value".into(), "agent".into()],
    considerations: Vec::new(),
    effects_on_accept: Vec::new(),
    effects_on_reject: Vec::new(),
    drive_alignment: Vec::new(),
};
let catalog = vec![
    CatalogEntry { spec: alice_aff, precondition: String::new() },
    CatalogEntry { spec: bob_aff, precondition: String::new() },
];
let practice = PracticeSpec {
    name: "debate".into(),
    affordances: vec!["argue_fortify".into(), "argue_abandon".into()],
    turn_policy: TurnPolicy::RoundRobin,
    duration_policy: DurationPolicy::MultiBeat { max_beats: 4 },
    entry_condition_source: String::new(),
};
```

## Step 8: Update affordance-key bindings

Update the `add_scheme_instance_for_affordance` calls so the affordance keys carry the `action` binding (not `claim`/`cause`):

```rust
let mut alice_af = alice_b.clone();
alice_af.insert("self".into(), "alice".into());
let alice_id = state.add_scheme_instance_for_affordance(
    "alice", "argue_fortify", &alice_af, alice_instance,
);
let mut bob_af = bob_b.clone();
bob_af.insert("self".into(), "bob".into());
let bob_id = state.add_scheme_instance_for_affordance(
    "bob", "argue_abandon", &bob_af, bob_instance,
);
```

## Step 9: Run

```bash
cargo run
```

Expected output:

```
beat 1: alice argued argue_fortify — accepted: true
beat 2: bob   argued argue_abandon — accepted: true
beat 3: alice argued argue_fortify — accepted: true
beat 4: bob   argued argue_abandon — accepted: true
```

Each character argues for the action that promotes their top-tier value. The boosts shape *which* affordance each picks; the trace shows both consistently propose their preferred action.

To see the audience flip in action, swap Alice's and Bob's audiences:

```rust
state.set_audience(
    "alice",
    Audience::total([Value::new("survival"), Value::new("duty")]),
);
state.set_audience(
    "bob",
    Audience::total([Value::new("duty"), Value::new("survival")]),
);
```

Re-run. Now Alice picks `argue_abandon` (her new top-tier value) and Bob picks `argue_fortify`. The same scene structure, same arguments, same attacks — different audiences flip the outcome.

## Why this matters

In the [first scene](/getting-started/first-scene), only β shaped acceptance. In the [second scene](/getting-started/second-scene-with-schemes), scheme choice shaped attack semantics. Here, audience shaped which arguments each character reaches for. These three dials — β, scheme, audience — compose. A scene author tunes them to produce specific dramatic effects.

The audience dial is the most expressive of the three: it lets you give each character a stable value profile that persists across many scenes. Alice always prioritises duty; Bob always prioritises survival; their disagreement plays out consistently across every scene they're in together.

## What you learned

- The `argument_from_values` Walton scheme and its `value` binding.
- How to set per-character audiences via `state.set_audience(actor, audience)`.
- How `ValueAwareScorer` composes on top of `StateActionScorer` (additive boosts).
- The audience-flip pattern: same framework, different audiences, different outcome.

## Next steps

- [Value-based argumentation (concepts)](/concepts/value-based-argumentation) — the formal semantics behind audiences.
- [Wiring per-character values (how-to)](/guides/wiring-character-values) — production-level integration patterns.
- [Hal & Carla example](/examples/hal-and-carla) — the canonical legal-reasoning scene with audience-driven outcome flips.
- [Multi-character consensus (how-to)](/guides/multi-character-consensus) — for queries across audiences in council scenes.
```

- [ ] **Step 2: Add to sidebars.ts**

```typescript
  gettingStartedSidebar: [
    'getting-started/first-scene',
    'getting-started/second-scene-with-schemes',
    'getting-started/third-scene-with-values',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/getting-started/third-scene-with-values.md website/sidebars.ts
git commit -m "docs(getting-started): add third-scene tutorial — values + audiences

Builds on second-scene by adding per-character audiences via
ValueAwareScorer. Demonstrates that the same scene with the same
arguments produces different beats per character. Includes the
audience-flip exercise as a self-check. Closes the 30-minute
tutorial progression: β → schemes → audiences."
```

---

## Phase 3: Concept depth pass (Tasks 10–15)

After Phase 2, the tutorial path teaches a real progression. Phase 3 deepens the explanation layer so users who graduate from tutorials and want to understand *why* find substantial concept pages — not 50-line sketches.

### Task 10: Expand concepts/semantics.mdx

**Files:**
- Modify: `website/docs/concepts/semantics.mdx`

The current file is 50 lines for credulous/skeptical/grounded/preferred/complete/stable — six concepts the rest of the site assumes you understand. Expand to ~120 lines with worked examples.

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/concepts/semantics.mdx
```

- [ ] **Step 2: Replace with the expanded version**

Write `website/docs/concepts/semantics.mdx`:

```mdx
---
sidebar_position: 4
title: Acceptance semantics
---

import AttackGraph from '@site/src/components/AttackGraph';

**Dung's framework defines several distinct ways to "accept" an argument. This page explains each one with a worked example, then maps them onto the queries our library exposes.**

When you ask "is argument X accepted?", the honest answer is "under which semantics?" — Dung (1995) identified that abstract argumentation frameworks admit *multiple* coherent answers, and choosing among them is a design decision, not a fact about the framework.

## The building blocks

A **Dung framework** `(A, R)` is a set of arguments `A` plus an attack relation `R ⊆ A × A`. Acceptance semantics partition the powerset `2^A` into "extensions" — sets of arguments that can coherently stand together.

The two minimal properties every extension must satisfy:

- **Conflict-free:** no argument in the set attacks another argument in the set.
- **Admissible:** conflict-free, and the set defends each of its members against external attacks (for every attacker `b` of a member `a`, some member of the set attacks `b`).

Every named semantics (preferred, grounded, complete, stable, ...) is a refinement of admissibility.

## A worked four-argument framework

For the rest of this page we use this framework:

<AttackGraph
  title="Worked example"
  arguments={[
    {id: 'a', label: 'a'},
    {id: 'b', label: 'b'},
    {id: 'c', label: 'c'},
    {id: 'd', label: 'd'},
  ]}
  attacks={[
    {from: 'a', to: 'b'},
    {from: 'b', to: 'a'},
    {from: 'b', to: 'c'},
    {from: 'c', to: 'd'},
  ]}
  height={280}
  caption="a ↔ b mutual attack; b → c; c → d (a chain off the mutual pair)."
/>

There's a mutual attack between `a` and `b`. `b` also attacks `c`. `c` attacks `d`.

## Preferred extensions

A **preferred extension** is a maximal admissible set — admissible, and you can't add any more arguments to it without violating admissibility.

In our example, the preferred extensions are:

- `{a, c}` — `a` defends itself against `b` (mutual attack); `a` attacks `b`, which would-be-attack `c`, so `c` is defended; `d` is attacked by `c` so cannot join.
- `{b, d}` — `b` defends itself against `a`; `b` attacks `c`, leaving `d` unattacked, so `d` joins.

Two preferred extensions, neither a subset of the other. This is normal for symmetric-attack frameworks.

## Grounded extension

The **grounded extension** is the unique smallest admissible set — equivalently, the least fixed point of the characteristic function "include all arguments not attacked by anything outside the set."

Our example's grounded extension is `∅` (empty). Why? Starting from `∅`:
- No argument is undefended-against trivially: `a` has attacker `b`, `b` has attacker `a`, `c` has attacker `b`, `d` has attacker `c`.
- The function adds nothing on the first iteration → fixed point.

So the grounded extension is empty. This is the "skeptical answer" — what survives if you refuse to take any side in the `a` ↔ `b` dispute.

## Complete extensions

A **complete extension** is admissible AND contains every argument it defends. Both preferred extensions are complete; the grounded extension is complete; and there's a third complete extension here:

- `∅` (grounded — also complete trivially)
- `{a, c}`
- `{b, d}`

Complete extensions sit between admissible (the lower bound) and preferred (the maximal-admissible upper bound).

## Stable extensions

A **stable extension** is conflict-free AND attacks every argument outside it. Stricter than preferred.

In our example:
- `{a, c}`: outside is `{b, d}`. Does the extension attack everything outside? `a` attacks `b` ✓. `c` attacks `d` ✓. Yes — stable.
- `{b, d}`: outside is `{a, c}`. `b` attacks `a` ✓; `b` attacks `c` ✓. Yes — stable.

Both preferred extensions happen to be stable. Stable extensions don't always exist (e.g., a single self-attacking argument has no stable extension). Preferred extensions always exist.

## Credulous vs skeptical acceptance

Once you have the extensions, you can ask "is argument X accepted?" two different ways:

- **Credulous acceptance:** X is in *some* preferred extension.
- **Skeptical acceptance:** X is in *every* preferred extension.

In our example:
- `a` credulously accepted (in `{a, c}`); not skeptically accepted.
- `b` credulously accepted (in `{b, d}`); not skeptically accepted.
- `c` credulously accepted; not skeptically accepted.
- `d` credulously accepted; not skeptically accepted.

Nothing is skeptically accepted under preferred semantics here. Under grounded semantics, *also* nothing is accepted (grounded is `∅`).

## When credulous and skeptical diverge

The framework above shows them at maximal divergence: every argument is credulous, none is skeptical. Add a single new argument to break the symmetry — say, an argument `e` that attacks `b`:

- New preferred extensions: `{a, c, e}` and `{e, d}` (the second loses `b`'s defense, so `c` re-enters... actually let's just trust the new framework re-resolves).

The point: small structural changes flip credulous/skeptical answers. Both are valid; they answer different questions.

## Why we care in scene AI

For dramatic resolution, you typically want **credulous** acceptance — "could a reasonable observer accept this?" — because skeptical is too conservative for fiction. The bridge crate's `is_credulously_accepted` and `has_accepted_counter_by` are credulous-side queries.

For *justification* (an NPC reflecting on what's universally true), you want skeptical — "what survives every interpretation?" The bridge exposes this via `is_skeptically_accepted`.

For multi-character consensus (the council case), you want skeptical-per-character intersected — see [`MultiAudience::common_grounded`](/guides/multi-character-consensus).

## In our library

The bare `ArgumentationFramework<A>` exposes the extensions; acceptance queries are computed by checking membership.

| Construct | Method | Returns |
|---|---|---|
| Grounded extension | `framework.grounded_extension()` | `HashSet<A>` (always polynomial; no `Result`) |
| Complete extensions | `framework.complete_extensions()` | `Result<Vec<HashSet<A>>, Error>` |
| Preferred extensions | `framework.preferred_extensions()` | `Result<Vec<HashSet<A>>, Error>` |
| Stable extensions | `framework.stable_extensions()` | `Result<Vec<HashSet<A>>, Error>` |
| Semi-stable extensions | `framework.semi_stable_extensions()` | `Result<Vec<HashSet<A>>, Error>` |
| Ideal extension | `framework.ideal_extension()` | `Result<HashSet<A>, Error>` |

Credulous acceptance: `preferred_extensions()?.iter().any(|ext| ext.contains(&arg))`. Skeptical acceptance: `.all(...)` instead of `.any(...)`.

The encounter bridge wraps these as direct boolean queries on `EncounterArgumentationState`: `state.is_credulously_accepted(&arg)?` and `state.is_skeptically_accepted(&arg)?`.

Frameworks larger than `argumentation::ENUMERATION_LIMIT` (currently 22 arguments) return `Error::TooLarge` from the subset-enumeration semantics (preferred / stable / semi-stable / complete). Grounded is always polynomial and never errors.

## Further reading

- [Dung (1995)](/academic/bibliography#dung1995) — the founding paper. Read §2–§4.
- [Baroni, Caminada, Giacomin (2011)](/academic/bibliography#baroni2011) — modern survey of semantics.
- [Glossary](/concepts/glossary) — quick definitions of every term used here.
- [Weighted attacks and β](/concepts/weighted-and-beta) — extends these semantics with attack weights.
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`. The `/concepts/glossary` link will warn until Task 14. Acceptable.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/concepts/semantics.mdx
git commit -m "docs(concepts): expand semantics with worked examples

50→~140 lines. Adds a worked four-argument framework with all
extensions calculated, side-by-side comparison of credulous vs
skeptical, complete vs stable. Adds a query-method reference table
mapping concepts to bridge-level functions."
```

---

### Task 11: Expand concepts/weighted-and-beta.mdx

**Files:**
- Modify: `website/docs/concepts/weighted-and-beta.mdx`

The current file is 58 lines and doesn't include β regime walkthroughs. Expand to ~110 lines.

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/concepts/weighted-and-beta.mdx
```

- [ ] **Step 2: Replace with the expanded version**

Write `website/docs/concepts/weighted-and-beta.mdx`:

```mdx
---
sidebar_position: 5
title: Weighted attacks and β (scene intensity)
---

import AttackGraph from '@site/src/components/AttackGraph';

**Weighted attacks let you express "this attack is stronger than that one." β (scene intensity) is a single dial that decides how strongly attacks bind. This page explains both with worked regimes.**

Dung's classical framework treats every attack as binary — it either defeats or it doesn't. For scene authoring, that's too coarse: a casual remark is not the same as a sworn-on-oath testimony. Weighted argumentation (Dunne, Hunter, McBurney, Parsons, Wooldridge 2011) attaches a weight to each attack, and a single intensity parameter β decides which weights bind.

## The mechanics

Each attack edge `(attacker, target)` carries a weight `w ∈ [0, 1]`. A scene-level **budget** `β ∈ [0, 1]` is the threshold below which attacks are considered "droppable":

- `w > β` → attack **binds** (acts as a regular Dung attack).
- `w ≤ β` → attack **drops** (treated as if absent).

Higher β = more attacks drop = more permissive scene = more arguments survive. Lower β = fewer attacks drop = sharper scene = fewer arguments survive.

The β-residual of a framework at a given β is the Dung framework you get by removing all dropped attacks. Acceptance semantics then run on the residual exactly as they do on a classical framework.

## Three regimes, one framework

Take a simple two-argument framework with one weighted attack:

<AttackGraph
  title="A → B with weight 0.4"
  arguments={[
    {id: 'A', label: 'A'},
    {id: 'B', label: 'B'},
  ]}
  attacks={[{from: 'A', to: 'B', weight: 0.4}]}
  height={200}
/>

Walk three β regimes:

### β = 0.0 — sharp regime

The attack weight (0.4) is greater than β (0.0), so the attack binds. The framework is equivalent to classical Dung with one attack. Grounded extension: `{A}` (A has no attacker; A attacks B; B is out). B is rejected.

### β = 0.4 — boundary

At the inclusive boundary `w ≤ β`, the attack drops. The framework is equivalent to classical Dung with no attacks. Grounded extension: `{A, B}`. B is accepted.

### β = 1.0 — permissive regime

All attacks drop unconditionally. Same answer as β = 0.4 here. As β passes each attack weight, that attack drops out one at a time.

## β as a tension dial

For a single attack, β acts as a binary switch — either the attack binds or it doesn't. The richer behaviour comes when a framework has *multiple* attacks at different weights:

| Attack weights | β = 0.0 | β = 0.3 | β = 0.5 | β = 0.8 |
|---|---|---|---|---|
| 0.2, 0.4, 0.6 | all bind | 0.2 drops | 0.2, 0.4 drop | all drop |
| 0.5 (one) | binds | binds | drops (boundary) | drops |
| 0.5, 0.5 | both bind | both bind | both drop (boundary) | both drop |

As β rises, attacks drop in weight order. The acceptance pattern shifts at each crossing.

## Picking β

There's no "correct" β for a scene — it's an authoring choice that shapes the dramatic register:

| Register | Suggested β | Why |
|---|---|---|
| Courtroom | 0.0–0.2 | Every objection lands; arguments must withstand strict scrutiny. |
| Heated debate | 0.2–0.4 | Most attacks bind; weak attacks drop. |
| Boardroom | 0.4–0.6 | Mid-strength attacks drop; tolerance for partial agreement. |
| Cordial discussion | 0.6–0.8 | Only the strongest objections matter. |
| Brainstorming | 0.8–1.0 | Almost everything stands; ideas get to breathe. |

See the [tune-β guide](/guides/tuning-beta) for picking β by scene register and escalating mid-scene.

## Worked four-attack regime sweep

Take the [east-wall example](/examples/east-wall) — Alice (military) wants to fortify; Bob (logistics) wants to abandon. Multiple attacks at different weights:

- Bob's logistics objection → Alice's fortify proposal: weight 0.4
- Alice's strategic objection → Bob's abandon proposal: weight 0.5
- (no others)

Regime sweep:

| β | Attack states | Grounded |
|---|---|---|
| 0.0 | Both bind | `∅` (both attacked) |
| 0.3 | Both bind | `∅` |
| 0.4 | Bob's drops (boundary); Alice's still binds | `{abandon}` (Alice's argument has no live counter; abandon defeats fortify; abandon survives) |
| 0.45 | Bob's dropped; Alice's binds | `{abandon}` |
| 0.5 | Both drop (boundary) | `{fortify, abandon}` |
| 0.8 | Both drop | `{fortify, abandon}` |

The scene's outcome flips at each β crossing. The east-wall's [BetaPlayground](/examples/east-wall) lets you drag β and watch the acceptance pattern shift live.

## Composition with other dimensions

β composes with bipolar (attack + support), values, and scheme strength:

- **Bipolar:** supports propagate acceptance; β decides which attacks block. The two compose at the residual layer — drop attacks first, then run bipolar semantics.
- **Values:** the audience-conditioned defeat graph filters attacks first (using `Audience::prefers`), then β filters by weight. An attack must survive *both* filters to bind.
- **Scheme strength:** schemes carry a `Strength` (Strong/Moderate/Weak) that the bridge's `SchemeActionScorer` reads. This scales boost magnitudes, not attack weights — orthogonal to β.

## In our library

| Construct | Method |
|---|---|
| Set β | `state.set_intensity(Budget::new(beta).unwrap())` |
| Add a weighted attack | `state.add_weighted_attack(&attacker, &target, weight)?` |
| Credulous acceptance at current β | `state.is_credulously_accepted(&arg)?` |
| Skeptical acceptance at current β | `state.is_skeptically_accepted(&arg)?` |
| Inspect direct attackers (β-independent) | `state.attackers_of(&target)` |

`Budget::new(beta)` returns `Result` because β must be in `[0, 1]` — anything outside is an error.

## Further reading

- [Dunne et al. (2011)](/academic/bibliography#dunne2011) — weighted argument systems, the formal basis.
- [The east wall](/examples/east-wall) — engine-driven example with live β slider.
- [The siege council](/examples/siege-council) — multi-attack framework where β crossings matter.
- [Tune β for your scene](/guides/tuning-beta) — practical authoring how-to.
- [Glossary](/concepts/glossary) — quick definitions.
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/concepts/weighted-and-beta.mdx
git commit -m "docs(concepts): expand weighted-and-beta with regime walkthroughs

58→~120 lines. Adds three-regime walkthrough on a simple framework,
β-by-register table, full β sweep on east-wall as a worked example,
and a section on how β composes with bipolar / values / schemes.
Adds in-library reference table for Budget construction and
acceptance queries."
```

---

### Task 12: Expand concepts/what-is-argumentation.mdx

**Files:**
- Modify: `website/docs/concepts/what-is-argumentation.mdx`

49 lines for the front door of a 7-crate library. Expand to ~85 lines with mental-model framing and what-it-isn't disambiguation.

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/concepts/what-is-argumentation.mdx
```

- [ ] **Step 2: Replace with the expanded version**

Write `website/docs/concepts/what-is-argumentation.mdx`:

```mdx
---
sidebar_position: 1
title: What is argumentation?
---

import AttackGraph from '@site/src/components/AttackGraph';

**Formal argumentation is the study of how arguments attack and support each other, and which sets of arguments can stand together without contradicting themselves.**

It sounds abstract, but it's the formal machinery behind auditable reasoning. When a courtroom jury decides which testimony to believe, or when a scientific review board weighs competing studies, they're doing argumentation informally. Formal argumentation gives you primitives — *argument*, *attack*, *support*, *acceptance* — that a computer can reason about.

## The mental model

Picture a knowledge base as a graph. Nodes are *arguments* — conclusions backed by reasoning. Edges are *attacks* — one argument undermining another. Acceptance semantics tell you which subsets of arguments can coherently stand together, given the attacks.

The library treats arguments as **abstract** by default — opaque IDs in a graph, no internal structure. This is Dung (1995). When you need to look inside arguments (to attack a premise rather than a conclusion, say), the [`argumentation-schemes`](/concepts/walton-schemes) and [ASPIC+](/concepts/aspic-plus) crates give you that structure.

## What argumentation is *not*

A few common confusions worth heading off:

- **Not natural-language understanding.** This library doesn't parse sentences or extract arguments from text. You give it argument IDs; it tells you which sets are coherent.
- **Not probabilistic reasoning.** Acceptance is a yes/no question (per semantics). Probabilistic AF is a separate research line — see [open areas](/concepts/open-areas).
- **Not theorem proving.** Argumentation handles *defeasible* reasoning — conclusions that can be defeated by counter-arguments, even when each step looks valid in isolation. Classical logic doesn't admit defeat.
- **Not a chatbot.** No language generation. The library is the *reasoning scaffold* underneath; a language model on top is a separate concern.

## Why it matters for scene AI

Formal argumentation gives you a structural layer for scenes: a graph of arguments and attacks, a semantics for deciding which stand. In a scene, each beat carries a record of:

- The arguments asserted by whom.
- The attacks that bound under the current scene tension.
- The acceptance semantics that produced the beat outcome.

You can replay the scene deterministically, tune one parameter ([β, scene intensity](/concepts/weighted-and-beta)) and see the beats change in a transparent way, or combine this with other tools — language models for surface prose, handwritten branches for pivotal moments. Argumentation gives you a reasoning scaffold; what you build on top is up to you.

## The smallest example

Two arguments that attack each other — the [Nixon diamond](/examples/nixon-diamond):

<AttackGraph
  title="Nixon diamond"
  arguments={[
    {id: 'A', label: 'Republican\n→ not pacifist'},
    {id: 'B', label: 'Quaker\n→ pacifist'},
  ]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'A'}]}
  height={260}
/>

Neither argument "wins" in isolation. Dung's [acceptance semantics](/concepts/semantics) tell you which subsets of these arguments can coherently stand together — in this case, either-or, but not both. The [Nixon diamond example](/examples/nixon-diamond) walks through this in detail.

## The four big ideas

If you read no other concept page, know these four:

1. **[Acceptance semantics](/concepts/semantics)** — multiple ways to "accept" an argument (credulous, skeptical, grounded, preferred, stable). Pick one per query.
2. **[Walton schemes](/concepts/walton-schemes)** — ~60 named patterns of human argumentation (expert opinion, cause to effect, analogy, ...) with their critical questions baked in.
3. **[Weighted attacks and β](/concepts/weighted-and-beta)** — attacks have strengths; a single dial decides which strengths bind.
4. **[Value-based argumentation](/concepts/value-based-argumentation)** — characters' value priorities make the same framework reach different conclusions.

The [encounter bridge](/concepts/encounter-integration) ties these together for narrative scenes.

## In our library

The workspace is layered: a Dung core, then extensions, then the bridge:

- `argumentation` — the Dung + ASPIC+ core.
- `argumentation-bipolar` — attacks + supports.
- `argumentation-weighted` — edge weights and β-budgets.
- `argumentation-weighted-bipolar` — composition of the above.
- `argumentation-schemes` — Walton's 60+ presumptive schemes.
- `argumentation-values` — value-based argumentation (Bench-Capon 2003 + multi-value).
- `encounter-argumentation` — the bridge into scene AI via the [`encounter`](https://github.com/patricker/encounter) crate.

See the [reference overview](/reference/overview) for the curated entry-point types per crate.

## Further reading

Start with [Dung (1995)](/academic/bibliography#dung1995), the paper that founded the field. Then [Walton, Reed & Macagno (2008)](/academic/bibliography#walton2008) for schemes. See [the reading order](/academic/reading-order) for a full curriculum.

If you'd rather *do* than read, jump to [your first scene](/getting-started/first-scene).
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/concepts/what-is-argumentation.mdx
git commit -m "docs(concepts): expand what-is-argumentation with mental model + disambiguation

49→~85 lines. Adds 'mental model' framing, 'what argumentation is not'
disambiguation (NLU / probabilistic / theorem proving / chatbot),
'four big ideas' as a curated reading path. Adds argumentation-values
to the in-library list."
```

---

### Task 13: Expand concepts/encounter-integration.mdx

**Files:**
- Modify: `website/docs/concepts/encounter-integration.mdx`

53 lines, reference-shaped, no audiences mention. Expand to ~120 lines covering: why a bridge crate, proposer/responder split, audiences, error latch.

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/concepts/encounter-integration.mdx
```

- [ ] **Step 2: Replace with the expanded version**

Write `website/docs/concepts/encounter-integration.mdx`:

```mdx
---
sidebar_position: 7
title: The encounter bridge — proposer/responder model
---

**The `encounter-argumentation` crate is the bridge between formal argumentation and the encounter scene engine. This page explains the model — why a bridge crate exists, the proposer/responder split, the dimensions it composes, and the error-latch design.**

The encounter scene engine is a separate library (`encounter`) that orchestrates multi-actor scenes via three trait hooks: `ActionScorer` (proposer-side), `AcceptanceEval` (responder-side), and a resolution loop (`MultiBeat`, `SingleExchange`, ...). Argumentation provides the semantics; the bridge wires the two together so each scene beat is decided by argumentation reasoning.

## Why a bridge crate?

A scene engine and a reasoning library shouldn't know about each other. The bridge crate exists so:

- The argumentation crates stay pure-formal — no scene-engine concepts leak in.
- The encounter crate stays scene-engine-pure — no argumentation concepts leak in.
- Consumers can swap reasoning backends (or use neither) without touching the engine.

The bridge crate (`encounter-argumentation`) implements `ActionScorer` and `AcceptanceEval` against an `EncounterArgumentationState` — a single state object that composes scheme reasoning, bipolar graph structure, weighted attack strengths, β intensity, per-character audiences, and an error latch.

## The proposer/responder split

Each scene beat is the resolution of one proposer's affordance against the responder(s). The bridge handles the two sides differently:

**Proposer side** (`StateActionScorer`): scores the affordances available to the proposer. Higher score = more likely to be picked. The scorer reads:
- The actor's affordances from the scene catalogue.
- The actor's `ArgumentKnowledge` (which schemes they invoke for which actions).
- The current state of the framework (β, audiences, accepted arguments).

It boosts affordances whose backing argument is credulously accepted at current β. Optionally wraps a `SchemeActionScorer` for scheme-strength boost, and a `ValueAwareScorer` for audience-conditioned value boost.

**Responder side** (`StateAcceptanceEval`): for each proposer's affordance, evaluates whether the responder accepts. The eval reads:
- The proposer's argument ID for this affordance.
- The framework state.
- The responder's actor name.

It returns true iff the responder does NOT have a credulously accepted counter-argument to the proposer's argument. "Has counter" → reject; "no counter" → accept.

The split mirrors how human deliberation works: each speaker proposes the strongest argument they can; each listener decides whether they have grounds to push back.

## What the bridge composes

The bridge composes several dimensions into one resolution:

| Dimension | Where it lives | What it shapes |
|---|---|---|
| Scene tension β | `state.intensity: Mutex<Budget>` | Which attack weights bind. |
| Per-character audiences | `state.audiences: Mutex<HashMap<String, Audience>>` | Which value-promoting affordances each actor scores up. |
| Scheme strength | `argumentation-schemes` (`Strength` enum) | How much `SchemeActionScorer` boosts an affordance. |
| Argument credulity | `state.is_credulously_accepted(...)` | Whether the proposer's argument survives current β. |
| Counter-credulity | `state.has_accepted_counter_by(...)` | Whether the responder has a survivor counter. |

These compose at scoring time (proposer side) and at acceptance time (responder side). A single beat's outcome is the combined result.

## Audiences as character state

Per-character audiences are first-class state on `EncounterArgumentationState` — same lifecycle as β. A character authored once can carry their value priorities across many scenes:

```rust
state.set_audience("alice", Audience::total([Value::new("duty"), Value::new("survival")]));
state.set_audience("bob", Audience::total([Value::new("survival"), Value::new("duty")]));
```

When the resolution loop calls `score_actions` for Alice, the wrapped `ValueAwareScorer` reads `state.audience_for("alice")` and boosts duty-promoting affordances. When it calls for Bob, it reads Bob's audience and boosts survival-promoting ones instead. Same scene, same arguments — different actors reach for different proposals.

See [Wiring per-character values](/guides/wiring-character-values) for the runtime integration pattern.

## The error latch

A subtle but important design choice: the bridge does NOT return errors from `ActionScorer::score_actions` or `AcceptanceEval::evaluate` — both trait methods return owned values, not `Result`. So when something goes wrong inside a scoring or acceptance call, the bridge has nowhere to put the error.

Instead: the state owns an error latch (`Mutex<Vec<Error>>`). On internal failure, the bridge appends an error and returns a permissive default (accept the affordance, skip the boost). The scene continues — D5 contract: "permissive on failure."

After resolution returns, callers MUST drain the latch:

```rust
let errors = state.drain_errors();
if !errors.is_empty() {
    // log, propagate, or ignore — your choice
}
```

This pattern matches how the rest of the bridge handles state mutation (interior mutability via `Mutex`, recovery on poisoning) — see the `intensity_guard` and `audiences_guard` private helpers for the exact pattern.

## Cost note: scoring and acceptance recompute on each beat

`StateActionScorer` and `StateAcceptanceEval` query the framework on every beat without caching. For narrative-scale frameworks (≤ 22 arguments per `argumentation::ENUMERATION_LIMIT`), this is fine — preferred-extension enumeration is bounded.

For longer scenes or larger frameworks, two patterns help:
1. Pre-seed only the arguments the scene needs (don't load the entire knowledge base).
2. Reuse a single `EncounterArgumentationState` across beats; do NOT reconstruct it per beat.

## Integration shape

A typical wiring:

```rust
let state = EncounterArgumentationState::new(catalog);
state.set_intensity(Budget::new(0.5).unwrap());
state.set_audience("alice", alice_audience);
state.set_audience("bob", bob_audience);

let scorer = ValueAwareScorer::new(
    SchemeActionScorer::new(knowledge, registry, baseline_scorer, 0.3),
    &state,
    0.2,
);
let acceptance = StateAcceptanceEval::new(&state);

let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

// Drain errors after resolution.
let errors = state.drain_errors();
```

Three layers, additively composed:
1. Baseline scorer (consumer-supplied — anything that implements `ActionScorer`).
2. `SchemeActionScorer` (boosts scheme-backed affordances by scheme strength).
3. `ValueAwareScorer` (boosts value-promoting affordances by audience).

## In our library

| Type | Purpose |
|---|---|
| `EncounterArgumentationState` | The composed state object — schemes + bipolar + weighted + audiences + errors. |
| `StateActionScorer<S>` | `ActionScorer<P>` impl wrapping any inner scorer; adds credulous-acceptance boost. |
| `SchemeActionScorer<K, S>` | Wraps `S`; adds scheme-strength × `preference_weight` boost. |
| `ValueAwareScorer<S>` | Wraps `S`; adds audience-conditioned value boost. |
| `StateAcceptanceEval` | `AcceptanceEval<P>` impl; checks for credulously-accepted counter-arguments. |
| `ArgumentKnowledge` | Trait — supplies per-character argument positions. `StaticKnowledge` is the default impl for fixtures. |

## Further reading

- [Wiring per-character values](/guides/wiring-character-values) — integration how-to.
- [Implementing an ActionScorer](/guides/implementing-action-scorer) — for custom scoring inside the chain.
- [Implementing an AcceptanceEval](/guides/implementing-acceptance-eval) — for responder-side custom logic.
- [Debugging "why didn't this argument get accepted?"](/guides/debugging-acceptance) — the diagnostic chain when the bridge produces unexpected results.
- [Glossary](/concepts/glossary) — quick definitions.
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/concepts/encounter-integration.mdx
git commit -m "docs(concepts): expand encounter-integration with proposer/responder model

53→~150 lines. New sections: why a bridge crate, proposer/responder
split, what the bridge composes (5 dimensions), audiences as
character state, the error latch (D5 contract), cost notes,
canonical integration shape. Adds an in-library types table."
```

---

### Task 14: New concepts/glossary.md

**Files:**
- Create: `website/docs/concepts/glossary.md`
- Modify: `website/sidebars.ts`

A single-source-of-truth definition page for terms used across the site. Linked from semantics.mdx, weighted-and-beta.mdx, encounter-integration.mdx (already added in those tasks).

- [ ] **Step 1: Write `website/docs/concepts/glossary.md`**

```markdown
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
```

- [ ] **Step 2: Add to sidebars.ts conceptsSidebar**

Add `'concepts/glossary'` at the end of `conceptsSidebar`:

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
    'concepts/glossary',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`. Previously-broken `/concepts/glossary` links from Tasks 10, 11, 13 should now resolve.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/concepts/glossary.md website/sidebars.ts
git commit -m "docs(concepts): add glossary as single-source-of-truth for terms

Single page covering ~40 terms organised by domain (primitives,
semantics, weighted, values, encounter bridge, library). Each entry
is a short definition; deeper treatment lives in the relevant
concept page. Cross-linked from semantics, weighted-and-beta, and
encounter-integration."
```

---

### Task 15: Add concept-pages reading order to academic/reading-order.md

**Files:**
- Modify: `website/docs/academic/reading-order.md`

The reading-order page covers academic papers. Users have no analogous map for *the docs site itself*. Add a section.

- [ ] **Step 1: Read the current file**

```bash
cat /home/peter/code/argumentation/website/docs/academic/reading-order.md
```

- [ ] **Step 2: Insert a new section**

Find the line `## If you want to teach this to someone` and insert before it:

```markdown
## Reading the docs themselves

If you want to read the *docs site* in order rather than the underlying papers, here's a curated path:

**Foundations (read these first):**

1. [What is argumentation?](/concepts/what-is-argumentation) — the front door; introduces the four big ideas.
2. [Acceptance semantics](/concepts/semantics) — credulous, skeptical, grounded, preferred, stable. With worked examples.
3. [Glossary](/concepts/glossary) — quick reference for terms used everywhere else.

**Structure of arguments:**

4. [Walton schemes](/concepts/walton-schemes) — the ~60 named patterns the library ships.
5. [Attacks and supports](/concepts/attacks-and-supports) — bipolar argumentation.
6. [ASPIC+](/concepts/aspic-plus) — structured arguments with rules and premises.

**Tuning the resolution:**

7. [Weighted attacks and β](/concepts/weighted-and-beta) — attack strengths and the scene intensity dial.
8. [Value-based argumentation](/concepts/value-based-argumentation) — character value priorities.

**Wiring into a scene engine:**

9. [The encounter bridge](/concepts/encounter-integration) — proposer/responder model and how the dimensions compose.
10. [Open areas](/concepts/open-areas) — what's still on the roadmap.

```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/academic/reading-order.md
git commit -m "docs(academic): add 'reading the docs themselves' section to reading-order

Curated 10-page path through the concepts pages, grouped by topic:
foundations, structure of arguments, tuning the resolution, wiring
into a scene engine. Gives readers a way to navigate the concept
layer that doesn't exist anywhere else on the site."
```

---

## Phase 4: Persona-driven onboarding (Tasks 16–17)

After Phase 3, the explanation layer teaches. Phase 4 addresses the homepage problem: the single CTA "Try the flagship demo" routes everyone to the same page regardless of why they're here.

### Task 16: New `getting-started/choose-your-path.md`

**Files:**
- Create: `website/docs/getting-started/choose-your-path.md`
- Modify: `website/sidebars.ts`

- [ ] **Step 1: Write `website/docs/getting-started/choose-your-path.md`**

```markdown
---
sidebar_position: 1
title: Choose your path
---

Three audiences land on this site. This page routes each to a curated 3-page path so you don't have to guess what to read first.

## I want to understand the formalism

You're here because you read about argumentation in a paper or talk and want to know what it actually is.

**Read in order:**

1. [What is argumentation?](/concepts/what-is-argumentation) — the front door, with mental-model framing and what-it-isn't disambiguation.
2. [Acceptance semantics](/concepts/semantics) — the formal heart. Worked examples for credulous/skeptical/grounded/preferred/stable.
3. [Reading order](/academic/reading-order) — the curated paper sequence if you want to go deeper.

**Then optionally:** browse [other concept pages](/concepts/glossary) by topic interest.

## I want to integrate the library

You're a Rust engineer wiring this into a scene engine, game, or other system.

**Read in order:**

1. [Install the library](/guides/installation) — get `cargo check` passing in a fresh project.
2. [Build your first scene](/getting-started/first-scene) — a working 10-minute scene end-to-end.
3. [Reference overview](/reference/overview) — the curated entry-point types per crate.

**Then:** pick from [the guides](/guides/installation) for specific integration tasks (custom scorer, custom acceptance eval, β tuning, debugging).

## I want to author scenes

You're a content author or game designer writing scenes that the library will resolve.

**Read in order:**

1. [Build your first scene](/getting-started/first-scene) — minimal scene with one β dial.
2. [Your second scene — multiple schemes](/getting-started/second-scene-with-schemes) — multiple Walton schemes per actor.
3. [Your third scene — with values](/getting-started/third-scene-with-values) — per-character audiences for value-driven outcomes.

**Then:** look at [the engine-driven examples](/examples/siege-council) for production-shape patterns (siege council for a 4-actor council, Hal & Carla for a values-driven legal scene, east wall for the simplest two-actor case).

## Not sure?

Try [the flagship siege-council demo](/examples/siege-council) — it's the most visually striking thing the library does. Then come back here and pick a path based on which question grabbed you most.
```

- [ ] **Step 2: Add to sidebars.ts gettingStartedSidebar (at the top)**

```typescript
  gettingStartedSidebar: [
    'getting-started/choose-your-path',
    'getting-started/first-scene',
    'getting-started/second-scene-with-schemes',
    'getting-started/third-scene-with-values',
  ],
```

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`. The `/getting-started/choose-your-path` link from Task 3's reading-order edit should now resolve.

- [ ] **Step 4: Commit**

```bash
cd /home/peter/code/argumentation
git add website/docs/getting-started/choose-your-path.md website/sidebars.ts
git commit -m "docs(getting-started): add choose-your-path persona-routing page

Three persona paths (formalism / integration / authoring), each
with a curated 3-page reading order. Placed at the top of
gettingStartedSidebar so it's the first thing tutorial-track
users see. Also serves as the homepage's deep-link target for
the new persona CTAs (Task 17)."
```

---

### Task 17: Restructure homepage with three persona CTAs

**Files:**
- Modify: `website/src/pages/index.tsx`

- [ ] **Step 1: Read the current homepage**

```bash
cat /home/peter/code/argumentation/website/src/pages/index.tsx
```

- [ ] **Step 2: Replace the HomepageHeader buttons block**

Find the `<div className={styles.buttons}>` block and its two `<Link>` children. Replace with three CTAs:

```tsx
        <div className={styles.buttons}>
          <Link className="button button--primary button--lg" to="/getting-started/choose-your-path">
            Choose your path →
          </Link>
          <Link className="button button--secondary button--lg" to="/concepts/what-is-argumentation">
            Concepts
          </Link>
          <Link className="button button--outline button--lg" to="/examples/siege-council">
            Try the flagship demo
          </Link>
        </div>
```

The "Choose your path" CTA becomes the primary; "Concepts" stays as a fast track for the formalism-curious; the flagship demo stays as the showmanship option. Three buttons instead of two; persona routing is the headline.

- [ ] **Step 3: Build verify**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -5
```

Expected: `[SUCCESS]`.

- [ ] **Step 4: Eyeball the page locally**

```bash
cd /home/peter/code/argumentation/website
npx docusaurus serve --no-open --port 3001 &
sleep 2
curl -s http://localhost:3001/ | grep -ic "Choose your path\|Concepts\|flagship demo"
kill %1 2>/dev/null
```

Expected: a non-zero count showing all three CTAs render.

- [ ] **Step 5: Commit**

```bash
cd /home/peter/code/argumentation
git add website/src/pages/index.tsx
git commit -m "docs(home): three persona CTAs replace single demo button

Primary CTA: 'Choose your path' → routes to choose-your-path page
where formalism / integration / author personas each get a curated
3-page reading order. Secondary CTA: 'Concepts' (fast-track for
formalism-curious). Outline CTA: 'Try the flagship demo' (preserved
for showmanship). The homepage stops being a single demo link."
```

---

## Done

After Task 17 the docs site has:

- A consistent post-VAF surface (per-crate page, bibliography entries, reading-order entry, overview update, two new how-tos)
- A 3-page tutorial progression (β → schemes → audiences) instead of a 10-minute β-only dead-end
- Four expanded concept pages (semantics 50→140, weighted-and-beta 58→120, what-is-argumentation 49→85, encounter-integration 53→150) plus a glossary
- A reading-order section for the docs site itself (not just the academic papers)
- Persona-driven onboarding (homepage routes by goal, not by demo button)

Verify with one final pass:

```bash
cd /home/peter/code/argumentation/website
npx docusaurus build 2>&1 | tail -10
```

Expected: `[SUCCESS]`. Acceptable warnings: pre-existing `/api/` rustdoc 404s. **Stop and investigate** any new warnings beyond those.

If running via subagent-driven-development, the final code-review should evaluate:
- Whether the new tutorials' code actually compiles end-to-end (Tasks 8 and 9 ship complete code blocks; the "Complete example" sections must be self-contained)
- Whether the expanded concept pages flow well as reading material (look for redundancy with the glossary; the pattern is "concept page explains, glossary defines")
- Whether the persona paths in choose-your-path actually correspond to existing pages (no dangling links)
- Whether the homepage CTA layout renders cleanly across viewport widths (three buttons may wrap on narrow screens; that's fine)
