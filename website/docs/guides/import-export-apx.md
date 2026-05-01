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
