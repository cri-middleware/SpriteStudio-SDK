# Override Layer API — Implementation Spec (cold-start)

> Status: **design / not started**. This is an SDK-internal implementation spec for the
> Phase 2 ROADMAP item **"Override Layer API"**. It is *not* Player-facing documentation
> (do not move it under `libs/ssruntime/docs/`, which is curated for Players).
>
> Scope: a per-part **runtime override** surface for **color**, **cell reference**, and
> **visibility**, exposed as new C-API/FFI on `ssruntime`. **State + FFI only — no `.fbs`
> schema change.** Flip / vertex-correction are explicitly *out* (Player post-processing of
> FrameData; see ROADMAP "Items not on the roadmap by design").

Everything below is grounded in the current code (`develop`, HEAD around `fa41e6e`). Line
numbers are anchors, not contracts — re-grep before editing.

---

## 0. TL;DR for the implementer

1. Add a per-part **override table** to `InstanceState` (`src/lib.rs:915`): a dense
   `Vec<Option<PartOverride>>` sized to `num_parts`.
2. Apply it at **one seam**: `Context::get_frame_data`, `src/lib.rs:1423` — *after* the
   source mix, *before* `compose_frame_data`. Mutate `self.instance.resolved_layer`
   (`parts_vec` + `cell_vec` + `parts_color_vec`). This makes an override win over both the
   keyframe **and** the blend result.
3. Parts are addressed by **positional `part_index` (0..num_parts)**. There is no
   name→index helper yet — add one (`name_hash → index`) built at `setup_animation`.
4. Priority mode (the legacy `KindIgnoreAttribute` trichotomy) is realized with the
   per-attribute `UpdateAttributeFlags` already on `PartState` + clearing the table on
   `setup_animation`.
5. Everything stays `no_std` (`alloc::vec::Vec` only; **no `HashMap`**).
6. Validate the C-API on the **Godot** player (calls the C header directly, no binding
   regen); use **ssplayer-wgpu** as a Rust sandbox for the state-machine logic first.

---

## 1. The pipeline and the injection seam

**Correction to the ROADMAP's mental model:** per-part decode and FrameData emission do
**not** happen in `update()`. `Context::update` (`src/lib.rs:2001`) only advances playback
controllers / collects events and returns a frame number. The decode → emit pipeline is
**`Context::get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], SSRuntimeError>`**
(`src/lib.rs:1340`):

| Stage | What happens | Where |
|---|---|---|
| **A. keyframe decode** | For each blend source, `core::framedata::extract_pose(...)` decodes per-part attributes into that source's `LayerState` (`parts_vec`, `cell_vec`, `parts_color_vec`, `shader_vec`, cursors, …). | `src/lib.rs:1373-1414` → `src/core/framedata.rs:666` |
| **mix** | Single active source → `mem::swap` fast-path; multiple → `mix_poses(...)` merges all source layers into `self.instance.resolved_layer`. | `src/lib.rs:1416-1422` → `src/lib.rs:740` |
| **⟵ SEAM** | **Apply the override table here.** After this line `resolved_layer.parts_vec` holds the final decoded pose. | **`src/lib.rs:1423`** |
| **B/C. emit** | `compose_frame_data(...)` serializes `resolved.parts_vec` (+ cell/color/shader vecs) into the FlatBuffer and builds the DrawBatch plan; then `resolve_global_state` (matrices/UVs) + `fb.finish_minimal`. | `src/lib.rs:1426-1445` → `src/core/framedata.rs:1069` |

**Why the post-mix seam (not per-source `extract_pose`):** an alternative is to intercept
`PartEvaluationResult` inside `evaluate_part` (`src/core/framedata.rs:445-566`) to reuse the
existing alpha-premultiply/index-append machinery. **Do not** — `extract_pose` runs *per
blend source, pre-mix*, so an override applied there would be blended away rather than
winning. The ROADMAP requires the override to win over the resolved attribute, so it must
run on `resolved_layer` after the mix. The fast-path `mem::swap` (`src/lib.rs:1419`) leaves
`resolved_layer` owning the primary's buffers, so mutating `resolved_layer` post-seam is
correct in **both** branches.

---

## 2. Per-part state at the seam

`resolved_layer` is a `LayerState` (`src/lib.rs:468`). The parts array is
`LayerState.parts_vec: Vec<PartState>` (`src/lib.rs:469`), indexed `0..num_parts` where
`num_parts = metadata.cached_parts.len()` (`src/lib.rs:1060`). The index space is
**positional and stable across frames**; it is (re)sized only in `setup_animation_by_hash`
via `resize_for_parts` (`src/lib.rs:1063`) and matches `part_index` in `CachedPartData`
(`src/core/framedata.rs:268`) and the ssab `parts_animation_data` order.

`PartState` is a fixed FlatBuffers struct `PartState([u8; 144])`
(`src/runtime/framedata.rs:1664`). Getters/setters relevant to overrides:

| Attribute | Accessors | Notes |
|---|---|---|
| **Visibility** | `hide() -> bool` / `set_hide` (`framedata.rs:2947`/`2962`) | Single flag; gates drawing at `src/core/framedata.rs:1015` (`!part_state.hide()`). |
| **Cell** | `cell() -> i16` / `set_cell` (`2744`/`2759`) | **Index into `resolved_layer.cell_vec`**, `-1` = none. Not a direct cell id. |
| **Color** | `part_color() -> i16` / `set_part_color` (`2773`/`2788`) | **Index into `resolved_layer.parts_color_vec`**, `-1` = none. |
| Alpha | `alpha()`/`set_alpha` (`2164`/`2179`), `local_alpha()`/`set_local_alpha` (`2208`) | Whole-part opacity; already premultiplied into color rgba.a (see §3). |
| Dirty mask | `set_update_flag(UpdateAttributeFlags)` (`1860`) | Per-attribute "a keyframe wrote this" mask — the key to priority mode (§6). |

Because `cell`/`part_color` are **indices into side vectors**, a cell or color override at
the seam must **append** a resolved entry to the side vector and repoint the index:

- Color: push a `PartAttributePartColor` onto `resolved.parts_color_vec`, then
  `part_state.set_part_color(new_index)`.
- Cell: push a resolved `PartAttributeCell` (or its `ResolvedCellMeta`) onto the cell path,
  then `part_state.set_cell(new_index)`.
- Visibility: just `part_state.set_hide(bool)` — inline, no side vector.

---

## 3. Data model of the three attributes (current)

### 3a. Color
- **Always four per-vertex corners** — there is no single-RGBA form in the format. ssab
  `PartAttributePartColor { blend_type, lt, rt, lb, rb }` (`libs/ssab/fbs/ssab.fbs:365`);
  each corner is `SsAttributePartColorKeyValueColor { rgba: U8Rect{a,r,g,b} u8, rate: f32 }`.
  FrameData mirror is the inline struct in `libs/ssruntime/fbs/framedata.fbs:40`.
  → the "one RGBA" convenience form in the ROADMAP is an **API affordance**: fill all four
  corners with the same value.
- **Blend op enum** is `BlendType` (12 variants), but PartColor uses only **`Mix/Mul/Add/Sub`
  (0..3)** — Players clamp ≥4 to Mix. (`ssab.fbs:65`, `framedata.fbs:25`.) The other 8 are the
  framebuffer `alpha_blend_type` axis — a different concept; don't conflate.
- **Alpha is pre-multiplied into `rgba.a` by the runtime** at `src/core/framedata.rs:762`
  (`apply_alpha`). A color override that sets rgba directly must replicate this (multiply in
  the resolved hierarchical alpha) so Players stay consistent — or intentionally bypass it and
  document that the override value is final.
- Legacy `PartsColorChange` ≡ the keyframed PartColor attribute
  (`PartAnimationDataPartColor`, `ssab.fbs:703`; eval `src/core/framedata.rs:551`).

### 3b. Cell reference
- **Addressed by `(cellmap_name_hash: u32, name_hash: u32)`, both FNV-1a** — not integer
  indices. ssab `PartAttributeCell` (`ssab.fbs:355`).
- Cell-map table is on the **borrowed, immutable** root binary:
  `SsAnimeBinary.cellmaps()` → `CellMap { name_hash(key), name, image_path, image_path_hash,
  size_original, cells: [Cell] }` (`ssab.fbs:194`) → `Cell { name_hash(key), name, rectangle,
  pivot, rotated, … }` (`ssab.fbs:181`). Both are `name_hash`-sorted for binary search.
- Runtime resolves cells once at load: `resolve_cell_array_meta` → `ResolvedCellMeta`
  (`src/core/framedata.rs:55`, `find_cellmap`/`find_cell` binary search;
  `ResolvedCellMeta.texture_hash = cellmap.name_hash()`, `:104`). Per-frame pick:
  `evaluate_cell` (`src/core/framedata.rs:449`).
- **Constraint: Normal + Mask parts only** (texture-cell rendering). Shape uses geometry;
  Mesh resolves its cell at frame 0 and freezes it (`src/core/framedata.rs:155`). Enforce this
  in the override setter (reject / no-op for other PartTypes).
- Legacy `CellChangeParts` ≡ keyframed cell (`PartAnimationDataCellValue`, `ssab.fbs:664`).

### 3c. Visibility
- Per-part **`hide: bool`** emitted inline (`framedata.fbs:168`, flag `AttributeHide`).
  There is no separate `visible`. Eval `evaluate_bool` (`src/core/framedata.rs:546`).
- **Default is hidden**: a part with no Hide key renders hidden
  (`part_state.set_hide(true)`, `src/core/framedata.rs:441`). Parts opt into visibility with a
  `Hide=false` key.
- Legacy `HideSet` ≡ this `hide` bool.

### 3d. Hierarchy (for the optional cascade)
- **Parent index only, no child lists**: `PartData.parent_index` (`ssab.fbs:644`) →
  `CachedPartData.parent_index` (`src/core/framedata.rs:269`). Parts stored parent-before-child
  (`parent_idx < i` guard).
- **Important:** SS7 `hide` does **not** cascade today — a hidden parent does *not* hide its
  children (`src/core/framedata.rs:756`); the hierarchy is transform-only and only **alpha**
  inherits down. So a "cascade to children" visibility override is **new behavior**, not a
  mirror of the engine. To implement it, invert `parent_index` into child lists yourself
  (build once at `setup_animation`).

### 3e. Color label read-back
- **Not available.** `color_label` is parsed (`libs/sspj/src/ssae.rs:46`) but explicitly
  **dropped as editor-only UI state** (`libs/ssab/SCHEMA.md:117`); it is not on `PartData`.
  → The `ColorGetColorLabel`-equivalent read-back in the ROADMAP **requires either a schema
  addition (out of scope here) or dropping this sub-feature.** Flag as an open decision (§11).

---

## 4. Design

### 4a. State: the override table
Add to `InstanceState` (`src/lib.rs:915`, ctor `:936`):

```rust
/// Per-part runtime overrides, dense by part_index (None = no override).
/// Sized to num_parts at setup_animation; index space matches resolved_layer.parts_vec.
part_overrides: Vec<Option<PartOverride>>,

#[derive(Clone, Default)]
struct PartOverride {
    color:      Option<ColorOverride>,
    cell:       Option<CellOverride>,
    visibility: Option<VisibilityOverride>,
    // Each kind carries its own PriorityMode (§6); a part can hold all three at once.
}
```

Dense `Vec<Option<_>>` is chosen because the seam is on the per-frame hot path and
`num_parts` is small/bounded — O(1) lookup, zero per-frame allocation. (Sparse
`Vec<(u16, PartOverride)>` or `alloc::collections::BTreeMap<u16, _>` are alternatives; avoid
them unless part counts get large. `HashMap` is banned — `no_std`.)

### 4b. Applying at the seam (`src/lib.rs:1423`)
```rust
// after mix, before compose_frame_data
if self.instance.has_any_override() {
    apply_part_overrides(
        &mut self.instance.resolved_layer,   // parts_vec + cell_vec + parts_color_vec
        &self.instance.part_overrides,
        &self.metadata,                       // part types, hierarchy for cascade
    );
}
```
`apply_part_overrides` walks parts with an override, and per kind either replaces the inline
flag (`hide`) or appends a resolved entry to the side vector and repoints the index
(`cell`, `part_color`) — see §2.

### 4c. Part addressing / name resolution
No name→index helper exists (`CachedPartData` caches `part_index`/`parent_index` but **not**
the name/hash; `PartAnimationData` has no name field; `PartData.name` is a plain unhashed
string). Two options, provide both at the FFI:
- **By index** (primary): `part_index: u32` — matches every existing per-part FFI
  (e.g. `ss_runtime_get_active_event_instance(context, slot_index: u32)`, `src/ffi/mod.rs:1459`).
- **By name/hash** (ergonomic): build a `name_hash -> index` table at `setup_animation` by
  iterating `ssab.parts()` and hashing `.name()` with the existing `fnv1a_hash_str`
  (`src/lib.rs:449`); expose `ss_runtime_find_part_index(context, name) -> i32`.

---

## 5. FFI surface (proposed)

Match house style in `src/ffi/mod.rs`: opaque `context: *mut c_void` → `get_context`
(`src/ffi/mod.rs:64`); return `bool` (or sentinel) and stash `ffi_context.last_error =
e.as_code()`. All setters address a part by `u32` index; a sentinel clears/"follows anim".

```c
// ---- resolution helper ----
int32_t ss_runtime_find_part_index(void* ctx, const char* part_name);  // -1 if not found

// ---- color override (mirrors PartsColorChange) ----
//   blend_op: 0=Mix 1=Mul 2=Add 3=Sub. Single-RGBA form fills all 4 corners.
bool ss_runtime_set_part_color_override(void* ctx, uint32_t part, uint8_t blend_op,
                                        uint32_t rgba, uint8_t priority_mode);
//   four-corner form: rgba_ltrtlbrb points to 4 packed RGBA (LT,RT,LB,RB)
bool ss_runtime_set_part_color_override_corners(void* ctx, uint32_t part, uint8_t blend_op,
                                                const uint32_t* rgba_ltrtlbrb, uint8_t priority_mode);
bool ss_runtime_clear_part_color_override(void* ctx, uint32_t part);

// ---- cell override (mirrors CellChangeParts); Normal/Mask parts only ----
//   cellmap_name_hash / cell_name_hash are FNV-1a; sentinel (0,0) = follow animation data.
bool ss_runtime_set_part_cell_override(void* ctx, uint32_t part,
                                       uint32_t cellmap_name_hash, uint32_t cell_name_hash,
                                       uint8_t priority_mode);
bool ss_runtime_clear_part_cell_override(void* ctx, uint32_t part);

// ---- visibility override (mirrors HideSet) ----
//   force: 0=show 1=hide. cascade!=0 also applies to descendant parts.
bool ss_runtime_set_part_visibility_override(void* ctx, uint32_t part, uint8_t force,
                                             uint8_t cascade, uint8_t priority_mode);
bool ss_runtime_clear_part_visibility_override(void* ctx, uint32_t part);

// ---- cell-map / texture table swap (whole-table; see §7) ----
bool ss_runtime_set_cellmap_texture_override(void* ctx, uint32_t cellmap_name_hash,
                                             uint32_t new_texture_hash);
bool ss_runtime_clear_cellmap_texture_override(void* ctx, uint32_t cellmap_name_hash);

// ---- bulk ----
bool ss_runtime_clear_all_part_overrides(void* ctx);
```
Also mirror the setters/clearers in the WASM bindings (`src/web/mod.rs`) if Web parity is
wanted — but that is a later step (see §9 validation ordering). cbindgen regenerates
`ssruntime.h` from these `#[no_mangle]` exports (`generate.sh`).

---

## 6. Priority mode (the `KindIgnoreAttribute` trichotomy)

There is **no** override-precedence mechanism today — this spec introduces it. Map the legacy
trichotomy to a `PriorityMode` enum carried per override kind:

| Legacy | Enum | Semantics | Implementation |
|---|---|---|---|
| `NON` | `OverwriteOnNextKeyframe` | Override applies **only until the next keyframe** writes that attribute. | At the seam, check `part_state` `UpdateAttributeFlags` (the "a keyframe wrote this attribute this frame" mask, `src/core/framedata.rs:458` etc.). If the attribute's flag is set this frame → drop the override (clear its slot); else apply it. |
| `NOW_ANIMATION` | `HoldUntilNextAnimation` | Override holds until the animation changes. | Apply every frame; **clear the table in `setup_animation_by_hash`** (`src/lib.rs:1044`). |
| `PERMANENT` | `Permanent` | Override holds across animation changes too. | Apply every frame; **survive** `setup_animation` (but re-validate against the new `num_parts`; drop entries with `part_index >= num_parts`). |

Note the part-index space changes on `setup_animation` (§2), so `HoldUntilNextAnimation`
clearing is also the safe default; only `Permanent` needs the re-validation path.

---

## 7. Cell-map / texture-table swap (the tricky one)

The cell-map table is in the **immutable borrowed ssab**; there is no runtime-mutable cellmap
state. Today, texture/atlas swap is done **entirely Player-side** by keying a texture map on
`DrawBatch.texture_hash` / `cellmap_name_hash` — the sanctioned mechanism documented in
`libs/ssruntime/docs/40_components/70_external_resources.ja.md`.

`ResolvedCellMeta.texture_hash = cellmap.name_hash()` (`src/core/framedata.rs:104`) flows into
`DrawBatch.texture_hash` (`framedata.fbs:187`, batched at `src/core/framedata.rs:1008,1043`).
So a **runtime** swap entry point has two levels:

1. **Texture reroute (easy, same UV layout):** a small runtime remap
   `cellmap_name_hash -> new_texture_hash`, applied when `texture_hash` is written into the
   DrawBatch. This covers "same atlas layout, different texture" — the common case, and it
   overlaps with what Players already do. Recommend shipping this first
   (`ss_runtime_set_cellmap_texture_override`).
2. **Full cell-map replacement (hard, different layout):** if the replacement atlas has a
   different rectangle/UV/pivot layout, you must override the **resolved cell rect/UV**, i.e.
   supply an alternate `ResolvedCellMeta`. This is heavier and can likely be **deferred** —
   confirm with the Player owners whether atlas-layout-changing swaps are actually needed at
   runtime, or whether re-binding a different ssab covers it.

---

## 8. sRGB vs linear — decision

The runtime does **all color math in 8-bit sRGB integer space**; there is no
linear/gamma/PMA conversion anywhere (deferred to Player shader hooks
`ss_input_texture()` / `ss_output_color()`, per `50_parts_color.ja.md`). **Decision for this
layer:** override color values are interpreted in the **same 8-bit sRGB space as keyframed
PartColor**, and undergo the **same** alpha-premultiply (§3a). This keeps overridden and
keyframed colors on one contract and requires no new color pipeline. Document it explicitly so
Players don't double-convert.

---

## 9. Validation plan — which Player, and how

**Primary: Godot (`../SSPlayerForGodot`).** It is the only mature player that calls the actual
`ssruntime` C-API directly (GDExtension C++, `#include "ssruntime.h"`, static-links
`libssruntime.a`), so new FFI is callable after a **cbindgen header regen on the SDK side with
no player-side binding codegen** — the lowest-friction way to genuinely exercise the FFI
boundary the ROADMAP wants validated. It already has all three rendering hooks an override
drives (per-part color materials, `is_part_hidden`/`_part_hidden`, a working
`setCellMapOverrideTexture`), an interactive editor playback panel for live set/clear, and an
explicit "Tier 2 override" plan naming `set_part_color_override` / `set_part_cell_override`.
Matches the ROADMAP's stated "validate on a fresh Player first" intent.

**Rust sandbox (fastest inner loop): ssplayer-wgpu** (`feat/wgpu_proto`). Native Rust — links
`ssruntime` as a crate and calls it **directly, bypassing the C-API** — with a winit viewer
(play/pause/keyboard) and color parity + cellmap substrate already present. Use it to shake out
the override-table / priority-mode **runtime logic** before/while wiring the C-API. **Caveat:**
because it bypasses the C-API it cannot sign off FFI ergonomics, and it lives on an unmerged
branch — so it is a companion, not the primary target.

**Second migration: Unity** — proves the C-API generalizes to a P/Invoke consumer; has the
most detailed SS6 override port plan and existing PartColor/hide hooks. Cost: csbindgen regen.
Web (full WASM + wasm-bindgen rebuild per FFI change) and RenPy (hand-maintained ctypes +
divergent pure-Python FrameData reimpl) are heaviest — do them last.

**Suggested order:** ssplayer-wgpu (logic) → **Godot (C-API sign-off)** → Unity → Web/RenPy.

### Test approach
- **Rust unit/integration** in `libs/ssruntime` (host `std` tests OK): per-kind override
  applied at the seam produces the expected `PartState.hide` / repointed `cell` / `part_color`
  index in the emitted FrameData; priority-mode matrix (keyframe-present vs absent × 3 modes);
  clear-on-`setup_animation` for `HoldUntilNextAnimation` vs survival for `Permanent`; cascade
  child set. Reuse existing `tests/overall` fixtures.
- **Golden FrameData** diff: override off vs on for a known frame.
- **Godot** interactive: GDScript sets/clears overrides on the playback panel; eyeball + a
  headless render diff if the harness supports it.

---

## 10. Lifecycle integration (don't miss these)

| Hook | File | Action |
|---|---|---|
| `InstanceState::new` | `src/lib.rs:936` | init empty `part_overrides` |
| `Context::setup_animation_by_hash` | `src/lib.rs:1044` | resize table to new `num_parts`; **clear** `HoldUntilNextAnimation` entries; **re-validate** `Permanent` entries against new `num_parts`; rebuild `name_hash->index` + child-list caches |
| `Context::reset` | `src/lib.rs:1005` | clear the whole table |
| `Context::bind_resource` | `src/lib.rs:1026` | new ssab bound → clear (part identity no longer valid) |

---

## 11. Open decisions (confirm before/while building)

1. **Color-label read-back** (`ColorGetColorLabel`): not representable without a schema change
   (§3e). **Drop it, or promote a small schema addition to Phase-2?** (Recommend: drop from
   this no-schema item; track separately if a Player actually needs it.)
2. **Full cell-map replacement** with a different atlas layout (§7 level 2): needed at runtime,
   or is texture-reroute + ssab re-bind enough? (Recommend: ship reroute; defer full replace.)
3. **Visibility cascade** is new behavior (§3d) — confirm the desired semantics (does a
   force-hidden parent hide transform-anchor children? probably yes for an *explicit* override,
   unlike keyframed `hide`).
4. **Single-RGBA vs four-corner** default in the FFI: ship both (done in §5); confirm corner
   order LT,RT,LB,RB matches Player expectations.
5. **Web/WASM parity** timing — same PR or a follow-up?

---

## 12. Implementation checklist (cold-start order)

- [ ] `PartOverride` + `PriorityMode` types; `part_overrides` field on `InstanceState`; ctor init.
- [ ] `apply_part_overrides(resolved_layer, overrides, metadata)` + call at seam `src/lib.rs:1423`.
- [ ] Visibility override first (simplest — inline flag, no side vector), incl. cascade + child-list cache.
- [ ] Color override (append to `parts_color_vec`, repoint index, replicate alpha-premultiply).
- [ ] Cell override (append resolved cell, repoint index; Normal/Mask guard).
- [ ] Priority-mode gating via `UpdateAttributeFlags` + lifecycle clear/re-validate (§10).
- [ ] `name_hash->index` resolver + `ss_runtime_find_part_index`.
- [ ] FFI exports (§5); cbindgen regen (`libs/ssruntime/scripts/generate.*`).
- [ ] Cellmap texture-reroute (`set_cellmap_texture_override`) feeding DrawBatch `texture_hash`.
- [ ] Rust tests (§9). Then Godot wiring + interactive validation.
- [ ] (later) WASM bindings; Unity migration.

---

## 13. File reference index

| Concern | Path |
|---|---|
| `get_frame_data` pipeline + **seam** | `libs/ssruntime/src/lib.rs:1340` (seam `:1423`) |
| `InstanceState` / lifecycle | `libs/ssruntime/src/lib.rs:915` (`new` `:936`, `reset` `:1005`, `bind_resource` `:1026`, `setup_animation` `:1044`) |
| `LayerState` / `parts_vec` | `libs/ssruntime/src/lib.rs:468` |
| `PartState` accessors | `libs/ssruntime/src/runtime/framedata.rs:1664` |
| decode / evaluate / compose | `libs/ssruntime/src/core/framedata.rs` (`extract_pose:666`, `evaluate_part:445`, `compose_frame_data:1069`, `apply_alpha:762`, cell resolve `:55`) |
| FFI house style | `libs/ssruntime/src/ffi/mod.rs` (`get_context:64`, examples `:900,:998,:1459`) |
| color / cell / hide schema | `libs/ssab/fbs/ssab.fbs` (PartColor `:365`, Cell `:355`, CellMap `:194`, parent `:644`), `libs/ssruntime/fbs/framedata.fbs` |
| Player-facing color/cell/texture docs | `libs/ssruntime/docs/40_components/50_parts_color.ja.md`, `.../70_external_resources.ja.md` |
| ROADMAP item | `ROADMAP.md` (Phase 2, "Override Layer API") |
