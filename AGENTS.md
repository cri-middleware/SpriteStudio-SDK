# SpriteStudio-SDK — AI Agent Guide

## Operating rules

*   Keep changes scoped to the requested task.
*   Do not commit, push, or open PRs unless the user explicitly asks.
*   Before editing, read enough surrounding context to understand callers and invariants; if the
    read tool truncated the file, read the remaining ranges first.
*   Follow existing code style in touched files (naming, type usage, control flow, error handling).
*   Validate with the commands in [CONTRIBUTING.md](./CONTRIBUTING.md) — `cargo fmt`, `clippy` and
    `cargo test`; for anything under `libs/ssruntime/docs`, also the both-locale build in
    [RELEASING.md](./RELEASING.md), which is the only gate a pull request does not run.
*   For `clippy --fix`, `git add` your own edits first and pass `--allow-staged`, never
    `--allow-dirty`, so clippy's edits land unstaged and review apart from yours.

## What this repo is

A Rust workspace that converts SpriteStudio projects (`.sspj` / XML) into a compact FlatBuffers
binary (`.ssab`) and plays them back through `libssruntime` — a stateless "Brain" emitting
per-frame state plus cross-engine **DrawBatch** plans, while Players (the "Body") render and
integrate. The runtime is `no_std`-first; `std` is opt-in for developer ergonomics only.

| Crate | Role |
|---|---|
| `cli/` (ssconverter-cli) | CLI wrapping `ssconverter`; batch and parallel conversion. |
| `libs/sspj` | Parses `.sspj` / `.ssae` / `.ssee`. |
| `libs/ssconverter` | Converts parsed sspj → `.ssab`. |
| `libs/ssab` | `.ssab` schema and FlatBuffers accessors. |
| `libs/ssruntime` | Playback Brain — state, interpolation, DrawBatch emission. |
| `libs/*-ffi` | C-API and WASM faces for the two above; features re-exported, `ffi` default. |

## Where each answer is written

| Question | File |
|---|---|
| What the SDK is, how to build and test it | [README.md](./README.md), [CONTRIBUTING.md](./CONTRIBUTING.md) |
| How the site is built and a release is cut | [RELEASING.md](./RELEASING.md), maintainers only |
| What to build next, and what is deliberately out of scope | [ROADMAP.md](./ROADMAP.md) |
| The constraints every contribution must hold to | [CONTRIBUTING.md](./CONTRIBUTING.md) |
| Runtime semantics a Player must honour | [Player Porting Guide](./libs/ssruntime/docs/README.ja.md) — start at [Limitations](./libs/ssruntime/docs/10_introduction/20_limitations.md) and the [Batched Rendering Contract](./libs/ssruntime/docs/30_core/30_render_loop.md) |
| What the official Players owe their users | [MAINTAINING_PLAYERS.md](./MAINTAINING_PLAYERS.md) |
| Binary layouts | [ssab SCHEMA](./libs/ssab/SCHEMA.md), [FrameData SCHEMA](./libs/ssruntime/SCHEMA.md) |
| What a crate feature does | that crate's `Cargo.toml`, commented per feature |
| Why the docs tree is generated, and what `--strict` misses | the docstring of `scripts/build-docs-tree.py` |
| Why a YAML sequence may live in only one config | the header comment of `mkdocs.base.yml` |

Do not restate those here. This file is for what an agent would otherwise break silently.

## Invariants

*   **The DrawBatch contract is the cross-engine rendering API.** `DrawBatch`, Mask batches,
    `mask_indices`, particle quad slots and `EffectSlot` / `InstanceSlot` / `ExternalInstance` are
    consumed by every Player, so a change here breaks all of them at once. Confirm with the user
    before touching it, and say so in the PR.
*   **`libssruntime` stays `#![no_std]`-compatible** — no `std::time`, `std::fs` or
    `std::collections::HashMap`; `core` and `alloc` only. Every release deliverable is pure
    no_std, so `cargo build -p ssruntime-ffi` on its own needs the allocator and panic features
    spelled out: `ssconverter`, which fulfills them transitively via `std`, is not in that graph.
*   **rayon spans two crates.** `sspj::load_from_maps`, the web entry path, is itself parallel, so
    `ssconverter`'s `parallel` feature gates rayon in `sspj` as well, each with a sequential
    fallback in its own `parallel` module. rayon cannot spawn threads on `wasm32-unknown-unknown`,
    which is why the `web` profile is the sequential one.
*   **`--all-features` is load-bearing on the clippy line.** The test modules sit behind
    `feature = "std"`, so without it they are the one body of code nothing lints.
*   **Doctests need an allocator.** A lone `cargo test --doc -p ssruntime` fails on "no global
    memory allocator found"; run them workspace-wide, or add `--features ssruntime/std`.
*   **The Player Porting Guide's layout does not change.** Pages live in `libs/ssruntime/docs`
    beside their `.ja.md` translations, readable on GitHub and deep-linked by SpriteStudio-Docs.
    Edit the authored pages, never the generated `build/docs`.
*   **A `> [!NOTE]` callout inside a `1. ` list item needs four spaces of indent, not three** —
    the three a `1. ` continuation gives you render as literal `[!NOTE]` text, with no warning.
