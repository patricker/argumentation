---
sidebar_position: 1
title: Install the library
---

Install the `argumentation` core and the `encounter-argumentation` bridge into a new or existing Rust project, and verify the build.

**Learning objective:** install the crates into any Rust project and run `cargo check` successfully in under 2 minutes.

## Prerequisites

- Rust 1.80+ (`rustc --version` should show ≥ 1.80).
- A Cargo project (or run `cargo new --bin my-project` first).

## Step 1: Add dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
argumentation = "0.2"
```

For weighted bipolar frameworks (attacks + supports + weights):

```toml
argumentation-weighted-bipolar = "0.2"
```

For the encounter bridge:

```toml
encounter-argumentation = "0.5"
encounter = "0.1"
```

For societas-modulated attack weights (optional):

```toml
societas-encounter = { version = "0.1", features = ["argumentation"] }
```

All crates are dual-licensed MIT / Apache-2.0. The argumentation core has no `[features]` you need to enable; `societas-encounter` requires the `argumentation` feature for `SocietasRelationshipSource`.

## Step 2: Verify

```bash
cargo check
```

Expected: `Compiling ... Finished dev [unoptimized + debuginfo] target(s) in X.XXs` with no errors.

```bash
cargo test
```

Expected: your own tests compile and pass (no tests yet is fine for a fresh project).

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| `error: failed to select a version for the requirement 'argumentation ...'` | `rustc` too old | Upgrade to Rust 1.80+ via `rustup update stable` |
| `could not find Cargo.toml` | Not inside a Cargo project | Run `cargo new --bin my-project` first |
| `linker 'cc' not found` (Linux) | Missing build tools | `apt install build-essential` or equivalent |

## Related

- [Build your first scene](/getting-started/first-scene) — the quickstart tutorial.
- [Reference overview](/reference/overview) — what types/methods you'll meet first.
