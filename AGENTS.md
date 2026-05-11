# SpriteStudio7-SDK — AI Agent Guide

## Operating rules

*   Keep changes scoped to the requested task.
*   Do not commit unless the user explicitly asks.
*   Before editing, read enough surrounding context to understand callers and invariants; if the read tool truncated the file, read the remaining ranges before making non-trivial changes.
*   Follow existing code style in touched files (naming, type usage, control flow, error handling).
*   Run formatting and linting after editing Rust code (see [Verification](#verification)). If you use `cargo clippy --fix`, first `git add` your own edits, then run with `--allow-staged` only (not `--allow-dirty`) so clippy's auto-fixes appear as unstaged changes and can be reviewed separately from your work.

## Architecture in one paragraph

A Rust workspace that converts SpriteStudio 7 projects (`.sspj` / XML) into a compact FlatBuffers binary (`.ssab`) and plays them back through `libssruntime` — a stateless "Brain" that emits per-frame state plus cross-engine **DrawBatch** plans. Player implementations (the "Body") handle rendering and platform integration. The runtime is `no_std`-first; `std` is opt-in for developer ergonomics only.

See [README.md](./README.md) for the workflow diagram and [ROADMAP.md](./ROADMAP.md) for planned work.

## Key components

| Crate / Tool | Role | Docs |
|---|---|---|
| `cli/` (ssconverter-cli) | CLI wrapping `ssconverter`; batch/parallel conversion. | [README](./cli/README.md) / [ja](./cli/README.ja.md) |
| `libs/sspj` | Parses `.sspj` / `.ssae` / `.ssee` (quick-xml + serde). | — |
| `libs/ssconverter` | Converts parsed sspj → `.ssab` binary. C-API. | [USAGE](./libs/ssconverter/docs/USAGE.md) / [ja](./libs/ssconverter/docs/USAGE.ja.md) |
| `libs/ssab` | `.ssab` schema and FlatBuffers (zero-copy). | [SCHEMA](./libs/ssab/SCHEMA.md) |
| `libs/ssinterpolate` | Platform-agnostic interpolation math (Hermite, Bezier, SIMD). | — |
| `libs/ssruntime` | Playback Brain — state, interpolation, DrawBatch emission. C-API + WASM bindings. `no_std`-first. | [Player Guide (ja)](./libs/ssruntime/docs/README.ja.md), [FrameData SCHEMA](./libs/ssruntime/SCHEMA.md) / [ja](./libs/ssruntime/SCHEMA.ja.md) |

> Before implementing a Player, read [**Limitations**](./libs/ssruntime/docs/00_limitations.ja.md) and [**Batched Rendering Contract**](./libs/ssruntime/docs/09_batched_rendering.ja.md).

## Hard constraints

*   **`no_std` strictness:** `libssruntime` must remain `#![no_std]` compatible. Do not introduce `std` (e.g., `std::time`, `std::fs`, `std::collections::HashMap`). Use `core` / `alloc` instead.
*   **FFI safety:** When modifying C-APIs (`ssruntime`, `ssconverter`), preserve memory safety across the boundary. Do not expose Rust-specific layouts directly.
*   **Zero-copy playback:** `.ssab` deserialization is zero-copy via FlatBuffers. Do not introduce allocations in the playback hot path unless strictly necessary.
*   **DrawBatch contract:** Batch emission (`DrawBatch`, Mask batches, `mask_indices`, particle quad slots, `EffectSlot` / `InstanceSlot` / `ExternalInstance`) is the cross-engine rendering API. Changes here are breaking for all Players — confirm with the user.

## Verification

| Check | Command |
|---|---|
| Lint (check) | `cargo clippy --workspace --all-targets -- -D warnings` |
| Lint (auto-fix) | **After `git add` your edits**: `cargo clippy --workspace --all-targets --fix --allow-staged -- -D warnings` |
| Format check | `cargo fmt --all -- --check` |
| Format apply | `cargo fmt` |
| Tests | `cargo test --workspace --all-features` |
| `no_std` standalone (mobile-style) | `cargo build -p ssruntime --features libc_alloc,panic-handler` |
| `std` opt-in (desktop DX) | `cargo build -p ssruntime --features std` |

The auto-fix variant uses `--allow-staged` (not `--allow-dirty`) so clippy's edits land as unstaged changes — review them via `git diff` before staging.

## `ssruntime` feature layout

`libs/ssruntime` defaults to a no_std-friendly set; `std` is opt-in.

| Feature | Purpose |
|---------|---------|
| `ffi` (default) | C-API exports |
| `libc_alloc` | `libc::malloc/free`-backed `#[global_allocator]` for no_std FFI builds (mobile/desktop). |
| `wasm_alloc` | `dlmalloc`-backed `#[global_allocator]` for WASM `no_std` builds. |
| `panic-handler` | Provides `#[panic_handler]` for no_std FFI builds. |
| `web` | `wasm-bindgen` bindings for JS/TS interop. |
| `std` | Enables `std` on `flatbuffers` / `ssab` and pulls in `ssconverter` (tests only). Never used by release scripts. |

All release deliverables for `ssruntime` (mobile / desktop / WASM) are pure no_std. Desktop scripts build the workspace first (to emit `ssconverter` / `ssconverter-cli`), then re-build `ssruntime` with `libc_alloc,panic-handler` to overwrite its `cdylib` / `staticlib`. WASM uses `wasm_alloc,panic-handler` with `--no-default-features`. `cargo build -p ssruntime` alone needs the allocator/panic features explicit because `ssconverter` (which transitively fulfills them via `std`) is not in the graph.

## Codegen and release scripts

*   **FlatBuffers schemas:** if you change `.fbs` files under `libs/ssab/fbs/` or `libs/ssruntime/fbs/`, regenerate the Rust bindings with the platform script in the affected library's `scripts/` directory (`generate.sh` or `generate.ps1`). Requires `flatc`.
*   **Release builds:** see `scripts/release-*.sh` / `.ps1`. Each handles cross-compilation and platform-specific packaging (e.g., `lipo` for macOS universal binaries).

## Workspace conventions

*   Single Cargo workspace, Rust edition 2024, MSRV 1.95.
*   Centralized versioning via `[workspace.package]` in the root `Cargo.toml`; member crates inherit.
*   Inter-crate deps use `{ workspace = true }` or `{ path = "..." }`.
*   `cbindgen` generates C/C++ headers for `ssconverter` and `ssruntime`.
*   CI enforces `cargo fmt --check` and `cargo clippy -D warnings`.
