# SpriteStudio-SDK — AI Agent Guide

## Operating rules

*   Keep changes scoped to the requested task. Unrequested hardening is a suggestion, not a commit; a defect
    you actually hit in the code you touched is not — fix it, or say plainly why you did not.
*   Before editing, read enough surrounding context to understand callers and invariants, the ones below
    included.
*   **Anything that exists twice changes twice** — a page and its translation, a `.sh` and its `.ps1`.
    Missing one half is this family's most common defect.
*   **Never break a line between two Japanese characters**; Chrome renders the break as a space.
*   **Finish with this repository's checks**: `scripts/check.sh`, ci.yml's lint job and its workspace tests. Add
    `features=yes` when code behind a feature changed, and `docs=yes` when `libs/ssruntime/docs` did. Read
    its summary, not its exit code — a check whose prerequisite is missing skips itself.
*   **Do not restate what another file owns**; the table below says which file. A fact no file owns goes
    where its staleness will be noticed.
*   **For `clippy --fix`, `git add` your edits first and pass `--allow-staged`**, never `--allow-dirty`, so
    clippy's edits land unstaged and apart from yours.

## What this repo is

A Rust workspace that converts SpriteStudio projects (`.sspj`) into the FlatBuffers binary `.ssab` and
plays them through `libssruntime`: a stateless **Brain** emitting per-frame state and cross-engine
**DrawBatch** plans, which Players (the **Body**) render. The runtime is `no_std`-first.

## Where each answer is written

| Question | File |
|---|---|
| What the SDK is, and its crates | [README.md](./README.md) |
| Constraints, build, test, the docs site | [CONTRIBUTING.md](./CONTRIBUTING.md) |
| Releases, maintainers only | [RELEASING.md](./RELEASING.md) |
| Plans and non-goals | [ROADMAP.md](./ROADMAP.md) |
| Runtime semantics a Player must honour | the [Player Porting Guide](./libs/ssruntime/docs/README.md) |
| How the guide's pages are written and translated | [its authoring rules](./libs/ssruntime/docs/CONTRIBUTING.ja.md), in Japanese |
| What official Players owe users, and the reference Player | [MAINTAINING_PLAYERS.md](./MAINTAINING_PLAYERS.md) |
| Binary layouts | [ssab SCHEMA](./libs/ssab/SCHEMA.md), [FrameData SCHEMA](./libs/ssruntime/SCHEMA.md) |

## Invariants

*   **The DrawBatch contract is the cross-engine rendering API.** `DrawBatch`, Mask batches,
    `mask_indices`, particle quad slots and `EffectSlot` / `InstanceSlot` / `ExternalInstance` reach every
    Player, so a change breaks all of them at once. Confirm with the user first, and say so in the PR.
*   **`cargo build -p ssruntime-ffi` alone needs the allocator and panic features spelled out**: every
    release deliverable is pure `no_std`, and `ssconverter`, which supplies them through `std`, is not in
    that graph.
*   **Doctests need an allocator**: a lone `cargo test --doc -p ssruntime` fails on "no global memory
    allocator found". Run them workspace-wide, or add `--features ssruntime/std`.
*   **The Player Porting Guide keeps its layout**: pages in `libs/ssruntime/docs` beside their `.ja.md`
    translations, readable on GitHub and deep-linked by SpriteStudio-Docs. Edit the authored pages, never
    the generated `build/docs`.
*   **A `> [!NOTE]` inside a `1. ` list item needs four spaces of indent**; the three a continuation gives
    render as literal `[!NOTE]`, with no warning.
