# SpriteStudio7-SDK Roadmap

Each item below states a **primary deliverable**. A few entries also list **Co-decided** sub-bullets — those are design points settled as part of the parent work so adjacent API contracts stay consistent. They are not separate tasks and should not be split into independent issues.

## Phase 1: Core Architecture & API Stabilization (Pre-Release)

*   [x] Restructure and improve Player Implementation Documentation.
*   [x] Add SSAB and FrameData schema documentation.
*   [x] Web (WASM) support for direct JS/TS usage via `wasm-pack`.
*   [x] Support for `no_std` environments (embedded/bare-metal).
*   [x] `DrawBatch` contract for cross-engine batched rendering (Brain-side batch emission with Mask batches and `mask_indices`).
*   [x] Particle quad emission and effect events surfaced via FrameData / FFI.
*   [x] Mesh part (`PartTypeMesh`) / Deform attribute (`PartAttributeDeform`) runtime evaluation.
*   [ ] SSPlayer for Godot integration (serves as both C-API validation and Player migration).

## Phase 2: Ecosystem & Integration

*   [ ] **Mask Support**: Implementation of per-part mask evaluation and DrawBatch emission.
*   [ ] **Override Layer API**: per-part runtime overrides for color, cell reference, and visibility. FFI/state only; no `.fbs` change. Prerequisite for migrating existing Players that depend on `PartsColorChange` / `CellChangeParts` / `HideSet`-equivalent semantics. Begin design during Phase 1's Godot integration so the C-API surface is validated on a fresh Player first.
    *   **Co-decided**: sRGB-vs-linear color-value semantics, applied uniformly to keyframed and overridden colors. Settled here so Override blend modes and Player rendering share one contract.
*   [ ] Migration of existing Players (Unity, pixi.js, etc.) to use `ssruntime`.
*   [ ] `ssplayer-wgpu` implementation (Reference implementation for Rust-native rendering).

## Phase 3: Feature Expansion (Post-Release)

*(Phase 3 items are implemented without `.fbs` schema changes — additions go to FFI APIs and runtime state only. Any item that turns out to need schema changes is reconsidered for an earlier phase.)*

*   [ ] Animation Mixing/Blending in `ssruntime` (whole-animation weighted blend).
*   [ ] Instance Lifecycle API — one cohesive surface covering both:
    *   Animation Instancing: shared evaluation context for driving many instances of the same animation, consumable by Players (e.g., crowd-style rendering).
    *   Dynamic instance swap: runtime replacement of the animation mounted on an Instance part (equivalent to legacy `AnimationChangeInstance`). Must remain compatible with the shared-context semantics above.
*   [ ] State machine implementation.
*   [ ] Particle generation cap (runtime LOD) — Brain-side upper bound on particle generation.

## Items not on the roadmap by design

*   Per-part AABB in FrameData (Player can derive from vertex positions; revisit only on concrete Player-side demand).
*   Flip override and vertex correction (expressible as Player post-processing of FrameData).
