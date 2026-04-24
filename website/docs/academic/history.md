---
sidebar_position: 3
title: A brief history of formal argumentation
---

Formal argumentation didn't appear fully-formed. It assembled itself over ~40 years out of several parallel strands of AI research. This is a narrative overview — no numbered steps, no tables, just context to ground the rest of the library.

## 1980s — default logic and non-monotonic reasoning

The modern story begins with [Reiter (1980)](/academic/bibliography#reiter1980) on default logic and McCarthy's circumscription work. The puzzle: classical logic can't handle "birds generally fly, but penguins don't." Tweety the penguin flies or doesn't depending on which default you apply last. Reiter's default rules introduced the concept of **defeasible inference** — conclusions that hold *unless* contradicted. This is the soil argumentation grew from.

## 1990s — abstract argumentation is founded

[Dung (1995)](/academic/bibliography#dung1995) is the hinge. Dung abstracted away what an argument *is* — treating each argument as a node in a directed graph, with attack edges between them — and asked what's logically defensible. His conflict-free, admissible, preferred, stable, and grounded semantics gave the field a formal core that 30 years of subsequent work has built on.

Parallel work (Gordon, Verheij, Vreeswijk) tackled *structured* arguments — arguments as derivations from premises via rules. These two threads (abstract vs structured) stayed somewhat separate until ASPIC+ unified them.

## 2000s — structure, values, bipolarity, schemes

Four major contributions this decade:

- **ASPIC** and early structured-argumentation frameworks (Prakken's group) — formalizing how structured arguments reduce to Dung frameworks.
- **[Bench-Capon (2003)](/academic/bibliography#benchcapon2003)** — value-based argumentation, the Hal & Carla insulin example, and the idea that different audiences rationally reach different conclusions from the same framework.
- **[Cayrol & Lagasquie-Schiex (2005)](/academic/bibliography#cayrol2005)** — adding support relations alongside attacks (bipolar frameworks). Arguments reinforce each other, not just contradict.
- **[Walton, Reed, Macagno (2008)](/academic/bibliography#walton2008)** — the 60+ argument schemes catalogue. Walton had been cataloguing schemes since the early 90s; this book became the canonical reference.

## 2010s — weighted semantics and ASPIC+ matures

[Dunne, Hunter, McBurney, Parsons & Wooldridge (2011)](/academic/bibliography#dunne2011) introduced weighted argument systems — attacks carry real-valued weights, and a budget β tolerates a total attack weight being dropped. This single-knob machinery gave argumentation a continuous parameter, which is exactly what scene AI needs for tension modulation.

[Modgil & Prakken (2014)](/academic/bibliography#modgil2014) published the ASPIC+ tutorial — the friendliest entry point to structured argumentation, complete with penguin examples and undercutting semantics.

[Baroni, Caminada, Giacomin (2011)](/academic/bibliography#baroni2011) surveyed the field's semantics — complete, preferred, stable, grounded, plus ideal, eager, CF2, stage, and more. If Dung opened the question "which sets stand?", this paper shows how the field has answered it in multiple non-equivalent ways.

## 2020s — applications branch out

Argumentation has moved into:

- **Legal informatics** — evidence reasoning, case-based argumentation.
- **Deliberative systems** — multi-agent negotiation and policy deliberation.
- **AI explainability** — using argument structure as a scaffold for interpretable reasoning.
- **Scene AI** — which is what *this* library is about.

The library you're reading about treats 30 years of this literature as primitives and asks a new question — what if we use all of this to drive *scenes*, not just resolve arguments? β becomes scene intensity; schemes become action templates; acceptance semantics becomes beat outcome. The formal machinery was always ready. The missing piece was an integration layer.
