# SpriteStudio-SDK — AI Agent Guide

## Operating rules

*   Keep changes scoped to the requested task.
*   Do not commit unless the user explicitly asks.
*   Before editing, read enough surrounding context to understand callers and invariants; if the read tool truncated the file, read the remaining ranges before making non-trivial changes.
*   Follow existing code style in touched files (naming, type usage, control flow, error handling).
*   Run formatting and linting after editing Rust code (see [Verification](#verification)). If you use `cargo clippy --fix`, first `git add` your own edits, then run with `--allow-staged` only (not `--allow-dirty`) so clippy's auto-fixes appear as unstaged changes and can be reviewed separately from your work.

## Architecture in one paragraph

A Rust workspace that converts SpriteStudio projects (`.sspj` / XML) into a compact FlatBuffers binary (`.ssab`) and plays them back through `libssruntime` — a stateless "Brain" that emits per-frame state plus cross-engine **DrawBatch** plans. Player implementations (the "Body") handle rendering and platform integration. The runtime is `no_std`-first; `std` is opt-in for developer ergonomics only.

See [README.md](./README.md) for the workflow diagram and [ROADMAP.md](./ROADMAP.md) for planned work. [MAINTAINING_PLAYERS.md](./MAINTAINING_PLAYERS.md) holds the working agreement for the official Players' getting-started experience — our repositories, not the porting guide's subject.

## Key components

| Crate / Tool | Role | Docs |
|---|---|---|
| `cli/` (ssconverter-cli) | CLI wrapping `ssconverter`; batch/parallel conversion. | [README](./cli/README.md) / [ja](./cli/README.ja.md) |
| `libs/sspj` | Parses `.sspj` / `.ssae` / `.ssee` (quick-xml + serde). | — |
| `libs/ssconverter` | Converts parsed sspj → `.ssab` binary. C-API. | [USAGE](./libs/ssconverter/docs/USAGE.md) / [ja](./libs/ssconverter/docs/USAGE.ja.md) |
| `libs/ssab` | `.ssab` schema and FlatBuffers (zero-copy). | [SCHEMA](./libs/ssab/SCHEMA.md) |
| `libs/ssruntime` | Playback Brain — state, interpolation, DrawBatch emission. C-API + WASM bindings. `no_std`-first. | [Player Guide (ja)](./libs/ssruntime/docs/README.ja.md), [FrameData SCHEMA](./libs/ssruntime/SCHEMA.md) / [ja](./libs/ssruntime/SCHEMA.ja.md) |

> Before implementing a Player, read [**Limitations**](./libs/ssruntime/docs/10_introduction/20_limitations.md) and [**Batched Rendering Contract**](./libs/ssruntime/docs/30_core/30_render_loop.md).

## Hard constraints

*   **`no_std` strictness:** `libssruntime` must remain `#![no_std]` compatible. Do not introduce `std` (e.g., `std::time`, `std::fs`, `std::collections::HashMap`). Use `core` / `alloc` instead.
*   **FFI safety:** When modifying C-APIs (`ssruntime`, `ssconverter`), preserve memory safety across the boundary. Do not expose Rust-specific layouts directly.
*   **Zero-copy playback:** `.ssab` deserialization is zero-copy via FlatBuffers. Do not introduce allocations in the playback hot path unless strictly necessary.
*   **DrawBatch contract:** Batch emission (`DrawBatch`, Mask batches, `mask_indices`, particle quad slots, `EffectSlot` / `InstanceSlot` / `ExternalInstance`) is the cross-engine rendering API. Changes here are breaking for all Players — confirm with the user.

## Verification

| Check | Command |
|---|---|
| Lint (check) | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Lint (auto-fix) | **After `git add` your edits**: `cargo clippy --workspace --all-targets --all-features --fix --allow-staged -- -D warnings` |
| Format check | `cargo fmt --all -- --check` |
| Format apply | `cargo fmt` |
| Tests | `cargo test --workspace --all-features` |
| Rustdoc (broken links) | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` |
| Doctests only | `cargo test --doc --workspace` |
| `no_std` standalone (mobile-style) | `cargo build -p ssruntime-ffi --features libc_alloc,panic-handler` |
| `no_std` custom allocator (console/engine) | `cargo rustc -p ssruntime-ffi --features custom_alloc,panic-handler --crate-type staticlib` |
| `std` opt-in (desktop DX) | `cargo build -p ssruntime-ffi --features std` |
| `ssconverter` web (browser WASM) | `cargo build -p ssconverter --no-default-features --features web --target wasm32-unknown-unknown` |
| Docs site (after editing `libs/ssruntime/docs`) | `scripts/prepare-docs.sh` once, then `scripts/build-docs.sh` — `.ps1` twins on Windows |

The auto-fix variant uses `--allow-staged` (not `--allow-dirty`) so clippy's edits land as unstaged changes — review them via `git diff` before staging.

The docs site build is **not** run on a pull request — `pages.yml` only fires on `release: published` and dispatch — so the local build is the only gate on a broken link, a stale heading anchor, or a page missing its `日本語 | English` switcher line. Run it before merging any change under `libs/ssruntime/docs`. Note `serve` validates nothing: `zensical serve --strict` is accepted and currently does nothing.

`scripts/build-docs.sh` generates the guide trees, then builds English and Japanese in that order, both `--strict`. It takes `key=value` options and `--help` like the other scripts here (`locale=ja`, `venv=…`). Every repository in the family spells this pair the same way; the `-docs` suffix keeps it apart from the scripts that build the code. What it does *not* build is the rustdoc the published site carries at `/api/`.

`scripts/build-pages.sh` is the whole published site: that docs build, then `cargo doc --workspace --no-deps --all-features` grafted onto it at `site/api/` — which has to come second, because the English build clears `site/`. `pages.yml`'s build job **is** this script (`prepare=no docs=yes api=yes serve=no`, every option spelled out so a publish cannot change because a default did), so CI and the local build cannot drift. `serve=yes` then serves the tree over HTTP, which is the only way to check the language selector and the nav's Rust API entry the way a reader meets them; `api=no` skips the rustdoc when only prose changed.

```bash
scripts/build-pages.sh serve=yes    # docs + rustdoc -> http://localhost:8000/
```

### How the site is built

The site runs on [Zensical](https://zensical.org), the successor to MkDocs + Material for MkDocs, like every other repository in the family. Zensical builds **one locale per configuration, from one directory**, which the guide's layout does not provide: pages live in `libs/ssruntime/docs` with each one next to its `.ja.md` translation, because that is what makes them readable on GitHub and what SpriteStudio-Docs deep-links. **That layout does not change.**

`scripts/build-docs-tree.py` bridges the two, generating `build/docs/en` and `build/docs/ja`. It replaces three MkDocs mechanisms at once, all of which Zensical ignores *silently* — it reports "No issues found" and emits a flat 47-page tree with `.ja` in half the URLs:

*   `plugins.i18n` with `docs_structure: suffix`, which paired the two files and resolved the `.ja.md` links the Japanese pages carry;
*   `docs/hooks.py`, which stripped the per-page language switcher line (now a hard error if a page lacks it, rather than a warning `strict` promoted);
*   `exclude_docs`, which kept `CONTRIBUTING.ja.md` out of the site.

`scripts/build-docs.sh` runs it before either build, every time, so the generated tree cannot go stale. Everything else follows the family pattern: `mkdocs.base.yml` holds what both locales share, `mkdocs.yml` builds English to `site/`, `mkdocs.ja.yml` builds Japanese to `site/ja/` — English first, because it clears `site/`, which contains `site/ja`.

**The three-config split has one rule: a YAML sequence must be defined in exactly one of the three files.** Zensical's `INHERIT` *concatenates* parent and child sequences where MkDocs replaces them, so a `nav` in the base as well as a locale config renders both navigations, one after the other.

Two more things worth knowing when editing the guide:

*   **`overrides/main.html` repairs the page title.** Zensical never sets `page.is_homepage`, which its own `base.html` branches on, so the landing page would otherwise title itself "SpriteStudio SDK - SpriteStudio SDK". `overrides/partials/alternate.html` likewise rebuilds the language selector so switching language keeps the reader on the same page — the i18n plugin used to do that, and nothing does once the locales are separate builds.
*   **A `> [!NOTE]` callout inside a `1. ` list item needs the item indented four spaces, not three.** Zensical parses callouts natively now (`pymdownx.quotes`), but not at a three-space indent — which is exactly what a `1. ` continuation would give you. It renders as literal `[!NOTE]` text with nothing to warn you.

Doctests need an allocator, so a lone `cargo test --doc -p ssruntime` fails on "no global memory allocator found"; run them workspace-wide (feature unification pulls in `std`) or add `--features ssruntime/std`. The full `cargo test --workspace --all-features` in CI covers them. Rustdoc's broken-link check is a *separate* CI step: `cargo test` never sees intra-doc links.

## `ssruntime` feature layout

`libs/ssruntime` defaults to a no_std-friendly set; `std` is opt-in.

| Feature | Purpose |
|---------|---------|
| `ffi` (default) | C-API exports |
| `libc_alloc` | `libc::malloc/free`-backed `#[global_allocator]` for no_std FFI builds (mobile/desktop). |
| `wasm_alloc` | `dlmalloc`-backed `#[global_allocator]` for WASM `no_std` builds. |
| `emscripten_alloc` | Emscripten libc `malloc/free`-backed `#[global_allocator]` (manual over-alignment) for Emscripten `no_std` builds. |
| `custom_alloc` | Routes `#[global_allocator]` (+ `#[panic_handler]` abort) to host-provided `ss_custom_alloc` / `ss_custom_free` / `ss_custom_abort` C symbols; for game-console/engine allocators. Mutually exclusive with the other allocator features; build as `staticlib`. |
| `panic-handler` | Provides `#[panic_handler]` for no_std FFI builds. |
| `web` | `wasm-bindgen` bindings for JS/TS interop. |
| `std` | Enables `std` on `flatbuffers` / `ssab` and pulls in `ssconverter` (tests only). Never used by release scripts. |

All release deliverables for `ssruntime` (mobile / desktop / WASM) are pure no_std. Desktop scripts build the workspace first (to emit `ssconverter` / `ssconverter-cli`), then re-build `ssruntime` with `libc_alloc,panic-handler` to overwrite its `cdylib` / `staticlib`. WASM uses `wasm_alloc,panic-handler` with `--no-default-features`. `cargo build -p ssruntime-ffi` alone needs the allocator/panic features explicit because `ssconverter` (which transitively fulfills them via `std`) is not in the graph.

## `ssconverter` feature layout

`libs/ssconverter` builds in two profiles from a shared in-memory conversion core (`convert_sspj_to_ssab_data`, which returns a `name -> bytes` map).

| Feature | Purpose |
|---------|---------|
| `ffi` (default) | C-API exports; implies `native`. |
| `native` | CLI / Tauri backend: parallel conversion + filesystem I/O + scan. Implies `parallel`, `scan`. |
| `parallel` | rayon-backed parallel conversion; propagates to `sspj/parallel`. |
| `scan` | directory traversal (`walkdir`) for batch conversion. |
| `web` | browser WASM bindings (`wasm-bindgen` + `js-sys`). Sequential, with in-memory I/O via `MemoryAssetProvider`. |

The web target is packaged by `scripts/release-web.sh` into `pkg/converter-web/`. **rayon spans two crates:** both `ssconverter` and `sspj` use it (the web entry path `sspj::load_from_maps` is itself parallel), so the `parallel` feature gates rayon in *both* — the sequential fallback lives in each crate's `parallel` module (rayon cannot spawn threads on `wasm32-unknown-unknown`). Tauri consumes the native profile (`{ default-features = false, features = ["native"] }`) for full rayon + filesystem; file picking is the host's job (e.g. `tauri-plugin-dialog`), so the converter has no zip/archive handling.

## Codegen and release scripts

*   **FlatBuffers schemas:** if you change `.fbs` files under `libs/ssab/fbs/` or `libs/ssruntime/fbs/`, regenerate the Rust bindings with the platform script in the affected library's `scripts/` directory (`generate.sh` or `generate.ps1`). Requires `flatc`.
*   **Release builds:** see `scripts/release-*.sh` / `.ps1`. Each handles cross-compilation and platform-specific packaging (e.g., `lipo` for macOS universal binaries). One per platform, and each builds only what its own operating system can.
*   **Release packaging:** `scripts/build-release.sh` turns those per-platform outputs into the archives a consumer downloads — the third-party notices, the `static/` `dynamic/` `web/` layout, the zips and `SHA256SUMS`. `release.yml`'s package job **is** this script (`in=artifacts out=release-dist licenses=yes verify=yes clean=yes`, every option spelled out so a release cannot change because a default did), so CI and a local run cannot drift.
*   **Release version:** `scripts/release-version.sh` prints `[workspace.package] version` from `Cargo.toml`. It is not what names a release — the tag is — but it is what a release *should* be tagged as, and `build-release.sh` says so when the two disagree.

### Cutting a release

A release is a tag, pushed first and built second:

```bash
git tag v7.0.0-beta.1 && git push origin v7.0.0-beta.1
# Actions → release → "Use workflow from" = v7.0.0-beta.1, upload_release = true
# → a draft Release on that tag → review the assets and notes → Publish from the UI
```

Nothing passes the tag around: `build-release.sh` reads `GITHUB_REF_NAME` (or, locally, the tag `HEAD` is checked out at) and stamps it into every archive's `VERSION`, and `release.yml` uses the same ref for `tag_name`. Dispatching from a branch is an artifact build, not a release — `upload_release=true` from anything but a `v*` tag fails the run rather than silently skipping the Release.

Bump `[workspace.package] version` to match the tag before tagging. It is the string every Player pins in its own `scripts/SDK_VERSION.txt`.

### Reproducing the archives locally

The packaging step is the one part of a release that never needed the build matrix: it is pure layout, and six Players' `scripts/download-sdk.sh` read that layout. Trying a change to it used to mean pushing it and waiting for seven platforms to rebuild. Any past run's artifacts can be replayed through it in seconds instead:

```bash
git checkout v7.0.0-beta.1                # which release
gh run download <run-id> -D artifacts     # which build's artifacts
scripts/build-release.sh                  # assemble + verify
```

Those are three steps on purpose. `build-release.sh` is the phase *after* it has been settled what is being released, so it takes no option that re-decides the tag or the source — the same reason `release.yml` keeps its checkout and its artifact download as steps of their own. Re-running is free: the input tree is copied from, never moved out of, so one download serves any number of attempts.

`verify=yes` (the default) re-opens the finished zips and checks what has actually broken releases before: a missing or empty archive, a tree that lost the root layout `download-sdk.sh` navigates, a macOS `libssruntime.framework` whose version symlinks the zip dereferenced (which invalidates its code signature), a `SHA256SUMS` that covers the wrong set. It does **not** build anything — seven platforms need seven operating systems, and a locally built "release" would only ever be a partial one.

## Workspace conventions

*   Single Cargo workspace, Rust edition 2024, MSRV 1.98.
*   Centralized versioning via `[workspace.package]` in the root `Cargo.toml`; member crates inherit.
*   Inter-crate deps use `{ workspace = true }` or `{ path = "..." }`.
*   `cbindgen` generates C/C++ headers for `ssconverter` and `ssruntime`.
*   CI enforces `cargo fmt --check` and `cargo clippy -D warnings`. **`--all-features` is load-bearing on the clippy line** — the test modules are behind `feature = "std"`, so without it they are the one body of code nothing lints.
