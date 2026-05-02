---
sidebar_position: 1
title: Reading order
---

The field spans 30 years of papers, many of them technically dense. Here's one path through them that we've found useful.

## 1-hour overview

If you only read one paper:

- [**Baroni, Caminada, Giacomin (2011)**](/academic/bibliography#baroni2011) — *An introduction to argumentation semantics.* A widely-cited survey of the field.

## Full curriculum

Read in order:

1. [**Dung (1995)**](/academic/bibliography#dung1995) — *On the acceptability of arguments…* — the foundational paper. Focus on §2–§4; skim §5 onward.
2. [**Modgil & Prakken (2014)**](/academic/bibliography#modgil2014) — *ASPIC+ tutorial.* Bridges the gap from abstract to structured. Read end-to-end.
3. [**Walton, Reed, Macagno (2008)**](/academic/bibliography#walton2008) — *Argumentation Schemes.* Read Chapter 1 + Chapter 9 (Expert Opinion); skim the catalogue for a taste.
4. [**Cayrol & Lagasquie-Schiex (2005)**](/academic/bibliography#cayrol2005) — *On the acceptability of arguments in bipolar frameworks.* Short (~15 pages).
5. [**Dunne, Hunter, McBurney, Parsons, Wooldridge (2011)**](/academic/bibliography#dunne2011) — *Weighted argument systems.* §1–§3 for definitions; skim complexity results unless you care.
6. [**Bench-Capon (2003)**](/academic/bibliography#benchcapon2003) — *Persuasion in practical argument.* Hal & Carla + values.
7. [**Kaci & van der Torre (2008)**](/academic/bibliography#kaci2008) — *Preference-based argumentation: Arguments supporting multiple values.* The multi-value extension to Bench-Capon. Read §2 (defeat rule) carefully — it's the spec our `argumentation-values` crate implements.
8. [**Dunne & Bench-Capon (2004)**](/academic/bibliography#dunne2004) — *Complexity in VAF.* Read §3 for the NP/co-NP results that motivate the `ENUMERATION_LIMIT` cap on `subjectively_accepted` / `objectively_accepted` queries.

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

## If you want to teach this to someone

Use Walton (2006) — *Fundamentals of Critical Argumentation* — as the textbook. It has the short dialogues that make the formalism click for non-specialists. Then bring in the Modgil-Prakken tutorial for the formal mechanics.

## If you want to build with this

Read the Modgil-Prakken tutorial for ASPIC+ basics, then this library's [guides](/guides/installation). The `encounter-argumentation` bridge is the primary entry point for scene engines; the `argumentation-values` crate is the entry point for value-based reasoning. Pick one — the [Choose your path](/getting-started/choose-your-path) page routes you by goal.
