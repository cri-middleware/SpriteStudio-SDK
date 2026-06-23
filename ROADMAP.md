# SpriteStudio-SDK Roadmap

Each item below states a **primary deliverable**. A few entries also list **Co-decided** sub-bullets — those are design points settled as part of the parent work so adjacent API contracts stay consistent. They are not separate tasks and should not be split into independent issues.

## Phase 1: Core Architecture & API Stabilization (Pre-Release)

*   [x] Restructure and improve Player Implementation Documentation.
*   [x] Add SSAB and FrameData schema documentation.
*   [x] Web (WASM) support for direct JS/TS usage via `wasm-pack`.
*   [x] Support for `no_std` environments (embedded/bare-metal).
*   [x] `DrawBatch` contract for cross-engine batched rendering (Brain-side batch emission with Mask batches and `mask_indices`).
*   [x] Particle quad emission and effect events surfaced via FrameData / FFI.
*   [x] Mesh part (`PartTypeMesh`) / Deform attribute (`PartAttributeDeform`) runtime evaluation.
*   [x] SSPlayer for Godot integration (serves as both C-API validation and Player migration).

## Phase 2: Ecosystem & Integration

*   [ ] **Override Layer API**: one cohesive per-part runtime override surface covering color, cell reference, and visibility. FFI/state only; no `.fbs` change. Prerequisite for migrating existing Players that depend on `PartsColorChange` / `CellChangeParts` / `HideSet`-equivalent semantics. Begin design during Phase 1's Godot integration so the C-API surface is validated on a fresh Player first.
    *   **ssruntime work (required)**: the Rust runtime holds a per-part override table in player state and applies it during `update()` evaluation — *after* keyframe decode, *before* FrameData emission — so an override wins over (or defers to) the keyframed attribute per the priority mode below. Exposes new FFI to set/clear each override kind, addressed by part index (host resolves part name → index), plus a cell-map/texture table swap entry point. Runtime state + C-API only; no schema change.
    *   **Color override**: per-part color with blend op (Mix / Add / Sub / Mul) and either one RGBA or four per-vertex corners (LU/RU/RD/LD); opacity carried in alpha. Mirrors `PartsColorChange`. Includes read-back of a part's authored color label (`ColorGetColorLabel`-equivalent).
    *   **Cell-reference override**: replace the displayed cell (cell-map index + cell index; sentinel = follow animation data) per part, plus whole cell-map table replacement for runtime texture/atlas swap. Mirrors `CellChangeParts` and the cell-map table ops. Normal/Mask parts only.
    *   **Visibility override**: force-hide / force-show a part, optionally cascading to child parts. Mirrors `HideSet`.
    *   **Co-decided**: a per-override **priority mode** shared across all three kinds, governing coexistence with keyframed attributes — overwrite-on-next-keyframe / hold-until-next-animation / permanent (the legacy `KindIgnoreAttribute` NON / NOW_ANIMATION / PERMANENT trichotomy).
    *   **Co-decided**: sRGB-vs-linear color-value semantics, applied uniformly to keyframed and overridden colors. Settled here so Override blend modes and Player rendering share one contract.
*   [ ] Migration of existing Players (Unity, pixi.js, etc.) to use `ssruntime`.
*   [ ] `ssplayer-wgpu` implementation (Reference implementation for Rust-native rendering).
*   [ ] **iOS dynamic XCFramework — `.framework`-wrapped slices**: package the dynamic (`dylib`) iOS XCFramework's slices as proper `.framework` bundles (`Info.plist` + bundle layout) instead of bare dylibs, for App Store-valid embedding. The initial XCFramework consolidation ships the static (`.a`) XCFramework plus a *bare-dylib* dynamic XCFramework whose install name is corrected to `@rpath/libssruntime.dylib`; that is enough for now because every current consumer static-links `libssruntime.a` on iOS (Unity drops the static `.xcframework` into `Plugins/iOS/`; Godot links the static `.a` into `libSSGodot`), so the dynamic variant has no consumer yet. Revisit when a real iOS dynamic consumer appears or App Store validation requires the framework form.

## Phase 3: Feature Expansion (Post-Release)

*(Phase 3 items are implemented without `.fbs` schema changes — additions go to FFI APIs and runtime state only. Any item that turns out to need schema changes is reconsidered for an earlier phase.)*

*   [x] Animation Mixing/Blending in `ssruntime` (whole-animation weighted blend).
*   [ ] Per-part animation blending (layered animation / per-bone mask) — backward-compatible extension of the whole-animation blend API. Optional per-source per-part weights or mask covering common use cases (upper-body aim while lower-body runs, damage reactions, additive overlays). Also considers an Additive blend mode (delta-from-base) alongside the existing interpolative mix.
*   [ ] Instance Lifecycle API — one cohesive surface covering both:
    *   Animation Instancing: shared evaluation context for driving many instances of the same animation, consumable by Players (e.g., crowd-style rendering).
    *   Dynamic instance swap: runtime replacement of the animation mounted on an Instance part (equivalent to legacy `AnimationChangeInstance`). Must remain compatible with the shared-context semantics above.
    *   **Co-decided**: a shared resource registry that holds several SSABs at once (resolve an animation by its pack hash across all loaded SSABs), used by Dynamic instance swap, External Instance resolution, and Sequence (`.ssqb`) items that reference another SSAB — one mechanism, not three. The host pre-registers the needed SSABs up front rather than the runtime calling back during `update()`, so all asset loading stays host-side and the Wasm/C# bindings need no FFI callbacks.
*   [ ] State machine implementation — runtime-side animation state graph (states + transitions) built on the blend/crossfade primitives. Covers:
    *   Game-driven transitions: states switched at runtime by Player-supplied triggers/conditions.
    *   Sequence (`.ssqb`) playback: SpriteStudio-authored playlists run as a linear, auto-advancing configuration of the state graph (each item = a state; `repeat_count` = times to play before advancing; `SequenceType` Last/Keep/Top = end-of-list behavior). Reuses `set_loop` / `crossfade`, reproduces SS-editor semantics, and needs no `.fbs` change. Items referencing another SSAB (`ref_anime_pack`) resolve via the shared registry above and switch with no crossfade (blending is single-SSAB only). Built into the state graph, not a separate playback system.
*   [ ] Particle generation cap (runtime LOD) — Brain-side upper bound on particle generation.

## Items not on the roadmap by design

*   Mask rendering / per-part mask evaluation (a **Player responsibility**, not an SDK deliverable). The Brain only emits the cross-engine data — `DrawBatchKind::Mask` batches, `mask_indices`, and the `mask_influence` flag in the DrawBatch plan, plus the static PartData mask flags (`mask_write` / `visible_inside_mask` / `draw_as_mask`) and the per-frame `mask` value — all of which shipped with the Phase 1 DrawBatch contract. Stencil/coverage rasterisation and clipping live in each Player (GPU stencil, or the coverage-bitmap hybrid where stencil is unavailable); see the Player guide [09_mask.ja.md](./libs/ssruntime/docs/09_mask.ja.md). Already implemented in the Unity and Godot players.
*   Per-part AABB in FrameData (Player can derive from vertex positions; revisit only on concrete Player-side demand).
*   Flip override and vertex correction (expressible as Player post-processing of FrameData — scale negation / UV swap; deliberately kept out of the Override Layer API. Covers legacy `FlipSetParts`).
