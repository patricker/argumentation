---
sidebar_position: 10
title: argumentation
---

The `argumentation` crate is the semantic core of the workspace. It provides two independent layers: abstract argumentation frameworks in the Dung 1995 tradition (with grounded, complete, preferred, stable, ideal, and semi-stable extensions, plus Caminada three-valued labellings), and structured argumentation in the ASPIC+ tradition (Modgil & Prakken 2014) with strict and defeasible rules, preference-based defeat resolution via last-link and weakest-link orderings, and automatic generation of a Dung AF. All other crates in this workspace build on top of it.

**Crate:** `argumentation` ([crates.io](https://crates.io/crates/argumentation) · [rustdoc](/api/argumentation/))

## Key types

### `ArgumentationFramework<A>`
A Dung-style abstract argumentation framework generic over argument type `A`. Stores arguments and directed attack edges. Implements all canonical extension semantics as methods.
→ [Full docs](/api/argumentation/framework/struct.ArgumentationFramework.html)

### `aspic::StructuredSystem`
The top-level ASPIC+ entry point. Holds a knowledge base, rule set, and rule preference ordering, then constructs arguments, computes attacks, resolves defeats via the last-link principle, and emits a Dung AF.
→ [Full docs](/api/argumentation/aspic/struct.StructuredSystem.html)

### `aspic::BuildOutput`
The combined output of `StructuredSystem::build_framework`: the constructed arguments, computed attacks, and the resulting abstract framework. Provides helpers like `conclusions_in` and `argument_by_conclusion` to work with the results.
→ [Full docs](/api/argumentation/aspic/struct.BuildOutput.html)

### `aspic::Literal`
An atomic proposition or its negation. Use `Literal::atom("name")` for positive literals and `Literal::neg("name")` for negative ones. The reserved `__applicable_` prefix is used internally for undercut markers.
→ [Full docs](/api/argumentation/aspic/struct.Literal.html)

### `Labelling`
A Caminada three-valued labelling assigning `In`, `Out`, or `Undecided` to each argument. Produced by `ArgumentationFramework::complete_labellings`.
→ [Full docs](/api/argumentation/semantics/struct.Labelling.html)

## Key functions / methods

| Function | Returns | What it does |
|---|---|---|
| `ArgumentationFramework::grounded_extension(&self)` | `HashSet<A>` | Computes the unique grounded extension (least complete extension). Always returns exactly one set. |
| `ArgumentationFramework::preferred_extensions(&self)` | `Result<Vec<HashSet<A>>>` | Enumerates all preferred extensions via subset search. Returns `Err(TooLarge)` above `ENUMERATION_LIMIT`. |
| `ArgumentationFramework::stable_extensions(&self)` | `Result<Vec<HashSet<A>>>` | Enumerates stable extensions. A stable extension attacks every argument outside it. |
| `StructuredSystem::build_framework(&self)` | `Result<BuildOutput>` | Single-pass construction: builds arguments, computes attacks, resolves defeats via preferences, emits the AF. |
| `StructuredSystem::prefer_rule(preferred, less_preferred)` | `Result<()>` | Records a rule preference (transitive closure maintained). Rejects reflexive and cyclic preferences. |
| `BuildOutput::conclusions_in(&self, extension)` | `HashSet<&Literal>` | Maps a set of `ArgumentId`s back to their conclusion literals. |

## Errors

### `Error::ArgumentNotFound(String)`
An argument referenced in an operation was not in the framework. The payload is a `Debug` rendering of the missing argument.

### `Error::Parse(String)`
A parser (e.g. `parsers::apx`) failed to decode its input.

### `Error::Aspic(String)`
An ASPIC+ structural error: cyclic rule dependencies, illegal undercut markers, or cyclic/reflexive rule preferences.

### `Error::TooLarge { arguments, limit }`
Extension enumeration rejected because the framework exceeds `ENUMERATION_LIMIT` (22) arguments. The subset-search algorithm is O(2^n); use a smaller framework or wait for future SAT-based semantics.

## See also

- [What is argumentation?](/concepts/what-is-argumentation) — conceptual introduction to Dung frameworks
- [ASPIC+](/concepts/aspic-plus) — structured argumentation concepts
- [Semantics](/concepts/semantics) — credulous vs. skeptical acceptance, extension types
- [Tweety penguin example](/examples/tweety-penguin) — ASPIC+ walkthrough
