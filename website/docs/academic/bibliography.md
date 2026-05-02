---
sidebar_position: 2
title: Bibliography
---

The complete reference list for the formalisms this library implements. Anchored by paper ID; concept and example pages link here.

## Foundational

### dung1995
**Dung, P. M. (1995).** *On the acceptability of arguments and its fundamental role in nonmonotonic reasoning, logic programming and n-person games.* Artificial Intelligence, 77(2), 321–357.
[[PDF]](https://cse-robotics.engr.tamu.edu/dshell/cs631/papers/dung95acceptability.pdf)

The paper that founded the field. Defines abstract argumentation frameworks, conflict-free / admissible / preferred / stable / grounded semantics. Uses the Nixon diamond.

### reiter1980
**Reiter, R. (1980).** *A logic for default reasoning.* Artificial Intelligence, 13(1–2), 81–132.

The default logic paper that motivated much of the field. Home of the Tweety-flies-because-bird problem.

## Walton's canon

### walton2008
**Walton, D., Reed, C., Macagno, F. (2008).** *Argumentation Schemes.* Cambridge University Press.
[[Publisher]](https://www.cambridge.org/core/books/argumentation-schemes/9AE7E4E6ABDE690565442B2BD516A8B6)

A comprehensive catalogue of ~60 presumptive argument schemes. Each scheme comes with premises, conclusion, and critical questions. Our `argumentation-schemes` crate ships a subset.

### walton2006
**Walton, D. (2006).** *Fundamentals of Critical Argumentation.* Cambridge University Press.

A teaching-focused book — short worked dialogues (parent-child, doctor-patient, courtroom). Good entry point if the 2008 book feels dense.

## Bipolar

### cayrol2005
**Cayrol, C., Lagasquie-Schiex, M.-C. (2005).** *On the acceptability of arguments in bipolar argumentation frameworks.* ECSQARU 2005.
[[Springer]](https://link.springer.com/chapter/10.1007/11518655_33)

Introduces support edges alongside attacks. Argues supports aren't reducible to attacks. The foundation for our `argumentation-bipolar` crate.

### amgoud2008
**Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., Livet, P. (2008).** *On bipolarity in argumentation frameworks.* International Journal of Intelligent Systems, 23(10), 1062–1093.
[[PDF]](https://www.irit.fr/~Leila.Amgoud/docfinal-v4.pdf)

The survey paper on bipolar. Covers multiple support semantics — worth reading if you want to understand *which* support semantics our implementation picks.

## Weighted

### dunne2011
**Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., Wooldridge, M. (2011).** *Weighted argument systems: Basic definitions, algorithms, and complexity results.* Artificial Intelligence, 175(2), 457–486.
[[PDF]](http://www.cs.ox.ac.uk/people/michael.wooldridge/pubs/aij2011a.pdf)

Introduces weighted argument systems and β-budget inconsistency tolerance. The semantic foundation of our `argumentation-weighted` crate. The β-as-scene-intensity mapping is a direct application of this paper's machinery.

### amgoud2016
**Amgoud, L., Ben-Naim, J. (2016).** *Axiomatic foundations of acceptability semantics.* KR 2016.

Graded semantics in weighted frameworks — ordering arguments by acceptance strength. Relevant if you want per-argument acceptance *scores* rather than binary accept/reject.

### dunne2004

**Dunne, P.E. & Bench-Capon, T. (2004).** [*Complexity in Value-Based Argument Systems.*](https://link.springer.com/chapter/10.1007/978-3-540-30227-8_31) JELIA 2004, LNCS 3229: 360–371.

Headline complexity result for VAF: subjective acceptance is NP-complete, objective is co-NP-complete in general; both polynomial for fixed audiences on tree-like graphs.

## ASPIC+

### prakken2010
**Prakken, H. (2010).** *An abstract framework for argumentation with structured arguments.* Argument and Computation, 1(2), 93–124.

The original ASPIC+ paper. Builds structured arguments from strict/defeasible rules and reduces to Dung frameworks.

### modgil2014
**Modgil, S., Prakken, H. (2014).** *The ASPIC+ framework for structured argumentation: a tutorial.* Argument and Computation, 5(1), 31–62.
[[Publisher]](https://journals.sagepub.com/doi/10.1080/19462166.2013.869766)

An approachable entry point. Uses the Tweety penguin example. A good tutorial for newcomers to ASPIC+.

## Values & practical reasoning

### benchcapon2003
**Bench-Capon, T. (2003).** *Persuasion in practical argument using value-based argumentation frameworks.* Journal of Logic and Computation, 13(3), 429–448.
[[arXiv]](https://arxiv.org/pdf/cs/0207059)

Introduces value-based argumentation. Home of the Hal & Carla example.

### atkinson2007
**Atkinson, K., Bench-Capon, T. (2007).** *Practical reasoning as presumptive argumentation using action based alternating transition systems.* Artificial Intelligence, 171(10–15), 855–874.
[[ScienceDirect]](https://www.sciencedirect.com/science/article/pii/S0004370207000689)

Practical reasoning (deliberation between actions) modeled as argumentation over action-transitions. Relevant to scene AI where characters deliberate.

### kaci2008

**Kaci, S. & van der Torre, L. (2008).** [*Preference-based argumentation: Arguments supporting multiple values.*](https://www.sciencedirect.com/science/article/pii/S0888613X07000989) International Journal of Approximate Reasoning, 48(3): 730–751.

Generalises Bench-Capon's VAF to allow arguments promoting multiple values; introduces the Pareto-defeating rule we implement.

### bodanza2023

**Bodanza, G.A. & Freidin, E. (2023).** [*Confronting value-based argumentation frameworks with people's assessment of argument strength.*](https://content.iospress.com/articles/argument-and-computation/aac220008) Argument & Computation, 14(3): 247–273.

Empirical psychology study of VAF semantics; finds that human acceptance correlates with value importance directly rather than the attack/defeat propagation VAF prescribes. Informs why we expose value-importance scoring (`SchemeActionScorer` + `preference_weight`) alongside the orthodox defeat semantics.

## Legal / forensic

### bex2003
**Bex, F., Prakken, H., Reed, C., Walton, D. (2003).** *Towards a formal account of reasoning about evidence: argumentation schemes and generalisations.* Artificial Intelligence and Law, 11, 125–165.

Argumentation applied to legal evidence. Source of the snoring-witness / undercutting examples.

## Semantics surveys

### baroni2011
**Baroni, P., Caminada, M., Giacomin, M. (2011).** *An introduction to argumentation semantics.* The Knowledge Engineering Review, 26(4), 365–410.

A widely-cited survey of Dung + post-Dung semantics. Covers complete, preferred, stable, grounded, plus ideal, eager, CF2, stage, and others.
