# Override Layer API — Implementation Spec (cold-start)

> Status: **design / not started**. SDK-internal implementation spec for the Phase 2 ROADMAP
> item **"Override Layer API"**. Not Player-facing docs (do not move under
> `libs/ssruntime/docs/`, which is curated for Players).
>
> **Origin:** this feature is a port of the per-part **runtime customization API of the
> legacy SS6PlayerForUnity** (`Script_SpriteStudio6_Root`), reconciled against what the
> current SS7 SDK already provides. That reconciliation (§2) is load-bearing: it is why the
> scope is **three** per-part overrides, not the larger SS6 surface.
>
> Scope: per-part **runtime overrides** for **color**, **cell reference**, and **visibility**,
> as new C-API/FFI on `ssruntime`. **State + FFI only — no `.fbs` schema change.**

Grounded in the current code (`develop`, HEAD ~`fa41e6e`). Line numbers are anchors, not
contracts — re-grep before editing.

---

## 0. TL;DR for the implementer

1. Scope is exactly **three** per-part overrides: **Color**, **Cell**, **Visibility** (§2).
2. Override table lives on `InstanceState` (`src/lib.rs:915`): dense `Vec<Option<PartOverride>>`.
3. Apply at **one seam**: `Context::get_frame_data`, **`src/lib.rs:1423`** — after the source
   mix, before `compose_frame_data`. Mutate `self.instance.resolved_layer`. Override wins over
   both keyframe and blend.
4. **Priority mode (the SS6 `KindIgnoreAttribute` trichotomy) applies to Color + Cell ONLY.**
   Visibility is a simple force-hide flag with no priority mode (matches SS6 `HideSet`).
5. Parts addressed by **positional `part_index` (0..num_parts)**; add a `name_hash→index`
   resolver. Reserve sentinel indices for "all parts" (SS6 `-1`) if wanted.
6. `no_std`: `alloc::vec::Vec` only, **no `HashMap`**.
7. Validate the C-API on **Godot** (calls the C header directly); use **ssplayer-wgpu** as a
   Rust sandbox for the state-machine logic first.

---

## 1. The pipeline and the injection seam

**Correction to the ROADMAP's wording:** decode + emit do **not** happen in `update()`.
`Context::update` (`src/lib.rs:2001`) only advances playback / collects events. The decode →
emit pipeline is **`Context::get_frame_data(&mut self, frame_no) -> Result<&[u8], _>`**
(`src/lib.rs:1340`):

| Stage | What | Where |
|---|---|---|
| **A. keyframe decode** | per source, `extract_pose(...)` decodes per-part attrs into that source's `LayerState`. | `src/lib.rs:1373-1414` → `src/core/framedata.rs:666` |
| **mix** | single source → `mem::swap`; multiple → `mix_poses(...)` into `self.instance.resolved_layer`. | `src/lib.rs:1416-1422` → `:740` |
| **⟵ SEAM** | **apply the override table here** on `resolved_layer`. | **`src/lib.rs:1423`** |
| **B/C. emit** | `compose_frame_data(...)` serializes `resolved.parts_vec` + side vecs → FlatBuffer + DrawBatch plan. | `:1426-1445` → `src/core/framedata.rs:1069` |

**Why post-mix, not per-source `extract_pose`:** `extract_pose` runs *per source, pre-mix*, so
an override applied there would be blended away. The override must win over the resolved pose →
it must run on `resolved_layer` after the mix. The fast-path `mem::swap` (`:1419`) leaves
`resolved_layer` owning the primary's buffers, so mutating `resolved_layer` post-seam is correct
in both branches.

---

## 2. Reconciliation with SS6PlayerForUnity (why the scope is three)

Every per-part customization method of `Script_SpriteStudio6_Root`, mapped to the current SDK:

| SS6 API (file:line) | Maps to | Verdict |
|---|---|---|
| `PartsColorChange` (Overall `FunctionParts.cs:532` / Vertex `:568`) — `KindOperationBlend`, `KindIgnoreAttribute` | SDK PartColor (authored only) | ✅ **KEEP** — new per-part override; needs priority mode |
| `CellChangeParts` (`FunctionCell.cs:384`) — `KindIgnoreAttribute` | SDK Cell (authored only) | ✅ **KEEP** — new per-part override; needs priority mode |
| `HideSet` (`FunctionParts.cs:97`) — `flagInvolveChildren`, **no** `KindIgnoreAttribute` | SDK `hide` (authored only, non-cascading) | ✅ **KEEP** — new per-part override; **force flag, no priority mode**; cascade is a faithful port |
| `ColorGetColorLabel` / `FormGetColorLabel` (`:280`/`:253`) | — `color_label` **dropped from `.ssab`** (`SCHEMA.md:117`) | ❌ **DROP** — read-only; needs a schema change this item forbids |
| `TextureSetCellMap` (`FunctionMaterial.cs:704`) — **cellmap-level**, not per-part | SDK cellmap texture swap = **already Player-side** (`texture_hash` map, `70_external_resources.ja.md`) | ❌ **DROP** — already covered; SS6 has no *per-part* texture override to port |
| `FlipSetParts` (`FunctionParts.cs:336`) | — | ❌ **OUT** — deliberately Player-side already (ROADMAP "not on roadmap by design") |
| `AnimationChangeInstance` / `InstanceChange` (`FunctionMisc.cs:433/618/320`) | — | ➡️ **ELSEWHERE** — Phase 3 *Instance Lifecycle API* |
| `AdditionalColor` (`FunctionMisc.cs:61`) — whole-object color | — | ➡️ **OUT** — whole-object, not per-part; Player-side |
| `RefreshCancelTransform` (`FunctionMisc.cs:206`) | — | ❌ **N/A** — suppresses Unity `Transform` reset so the game drives a GameObject; no analogue in the Brain/Body model. SS6 has **no** `PositionSetParts`. |
| `RateOpacity` / `RateScaleLocal` / `OrderInLayer`; `ShaderSetStandard*` | — | ➡️ **OUT** — whole-object / material; Player responsibility |

**Net:** the Override Layer reduces to **Color + Cell + Visibility**, per-part. The larger SS6
surface is either already provided (texture, cellmap), deliberately Player-side (flip, shader,
whole-object color/opacity), owned by another item (instance), or not applicable to this
architecture (transform).

### SS6 addressing conveniences worth porting
Parts are `int idParts`; **`0` = Root/whole-animation**, **`-1` = all parts** for most setters
(`IDGetParts(name)` resolves names). Consider a sentinel `part_index` for "all parts" in the FFI.

---

## 3. Per-part state at the seam

`resolved_layer` is a `LayerState` (`src/lib.rs:468`); parts array is `parts_vec: Vec<PartState>`
(`:469`), indexed `0..num_parts` (`num_parts = metadata.cached_parts.len()`, `:1060`).
**Positional and stable across frames**; (re)sized only in `setup_animation_by_hash`
(`resize_for_parts`, `:1063`), matching `part_index` in `CachedPartData` (`src/core/framedata.rs:268`).

`PartState` = fixed FlatBuffers struct `PartState([u8;144])` (`src/runtime/framedata.rs:1664`):

| Attribute | Accessors | Notes |
|---|---|---|
| **Visibility** | `hide()`/`set_hide` (`framedata.rs:2947`/`2962`) | Single inline flag; gates drawing at `src/core/framedata.rs:1015`. |
| **Cell** | `cell()`/`set_cell` (`2744`/`2759`) | **Index into `resolved_layer.cell_vec`**, `-1` = none. |
| **Color** | `part_color()`/`set_part_color` (`2773`/`2788`) | **Index into `resolved_layer.parts_color_vec`**, `-1` = none. |
| Dirty mask | `set_update_flag(UpdateAttributeFlags)` (`1860`) | Per-attribute "a keyframe wrote this" mask — the key to priority mode (§6). |

`cell`/`part_color` are **indices into side vectors**, so a cell/color override **appends** a
resolved entry and repoints the index; visibility just sets the inline flag.

### Current data model of the three attributes (concise)
- **Color** — always four corners `PartAttributePartColor { blend_type, lt, rt, lb, rb }`
  (`libs/ssab/fbs/ssab.fbs:365`; FrameData mirror `framedata.fbs:40`). Only `Mix/Mul/Add/Sub`
  (0..3) valid; alpha is **pre-multiplied into rgba.a** by the runtime (`src/core/framedata.rs:762`).
  Corner order in the format is `lt,rt,lb,rb` (**map SS6 `LU,RU,RD,LD` → `lt,rt,rb,lb`**).
- **Cell** — `(cellmap_name_hash, cell_name_hash)` FNV-1a (`ssab.fbs:355`); resolved once at load
  to `ResolvedCellMeta` (`src/core/framedata.rs:55`). Per-frame pick `evaluate_cell` (`:449`).
  **Normal + Mask parts only.**
- **Visibility** — `hide: bool` (`framedata.fbs:168`), default **hidden**
  (`src/core/framedata.rs:441`); parts opt into visibility via a `Hide=false` key. No separate
  `visible`.

---

## 4. Design

### 4a. State: the override table
Add to `InstanceState` (`src/lib.rs:915`, ctor `:936`):
```rust
/// Per-part runtime overrides, dense by part_index (None = no override). Sized to num_parts.
part_overrides: Vec<Option<PartOverride>>,

#[derive(Clone, Default)]
struct PartOverride {
    color:      Option<ColorOverride>,       // + PriorityMode
    cell:       Option<CellOverride>,        // + PriorityMode
    visibility: Option<VisibilityOverride>,  // force-hide (+cascade), NO PriorityMode
}
```
Dense `Vec<Option<_>>`: O(1) at the hot-path seam, zero per-frame alloc, matches the positional
model, resizes with `resize_for_parts`. (`no_std`; `HashMap` banned.)

### 4b. Apply at the seam (`src/lib.rs:1423`)
```rust
if self.instance.has_any_override() {
    apply_part_overrides(
        &mut self.instance.resolved_layer,   // parts_vec + cell_vec + parts_color_vec
        &mut self.instance.part_overrides,   // &mut: NON priority mode self-clears (§6)
        &self.metadata,                      // part types + hierarchy (cascade)
    );
}
```

### 4c. Part addressing
No name→index helper exists (`CachedPartData` caches `part_index`/`parent_index` but not the
name; `PartData.name` is a plain unhashed string). Provide both at the FFI:
- **By index** `part_index: u32` (primary) — matches existing per-part FFI
  (`ss_runtime_get_active_event_instance(context, slot_index: u32)`, `src/ffi/mod.rs:1459`).
- **By name/hash** — build `name_hash→index` at `setup_animation` (hash `.name()` with
  `fnv1a_hash_str`, `src/lib.rs:449`); expose `ss_runtime_find_part_index(ctx, name) -> i32`.
- Optional **"all parts" sentinel** (SS6 `-1`).

---

## 5. The three overrides in detail

### 5a. Color (mirrors `PartsColorChange`)
- Blend op `0=Mix 1=Mul 2=Add 3=Sub` (reject ≥4). Two forms: single RGBA (fill all 4 corners) or
  4-corner. **Normal parts only** (SS6 restricts to NORMAL).
- At the seam: build a `PartAttributePartColor`, **replicate the alpha pre-multiply**
  (`src/core/framedata.rs:762`) so overridden and keyframed colors share one contract, push onto
  `resolved.parts_color_vec`, `set_part_color(new_index)`.
- Priority mode: **yes** (§6).

### 5b. Cell (mirrors `CellChangeParts`)
- Address by `(cellmap_name_hash, cell_name_hash)` FNV-1a (host resolves from names/indices;
  matches the format). Sentinel `(0,0)` = follow animation. **Normal + Mask parts only** — guard
  and no-op others (SS6 does the same).
- At the seam: re-resolve to a `ResolvedCellMeta` against `ssab.cellmaps()` (or a parallel
  override-resolved path), push onto the cell path, `set_cell(new_index)`.
- Priority mode: **yes** (§6).

### 5c. Visibility (mirrors `HideSet`)
- **Force-hide** (`force=1`) or **clear/revert-to-animation** (SS6 model: a force bit, *not* a
  symmetric force-show). `set_hide(true)` at the seam when active.
- **Cascade** (`cascade=1`) applies to descendants — a faithful port of `HideSet`'s
  `flagInvolveChildren`. Implement via the **existing alpha-inheritance forward pass**
  (`src/core/framedata.rs:747-753`): parts are stored parent-before-child, so a single forward
  pass propagating an "ancestor force-hidden" flag through `parent_index` suffices (no child
  lists needed). NB SS7 keyframed `hide` deliberately does **not** cascade
  (`src/core/framedata.rs:755-760`); this cascade is *new, override-only* behavior.
- Priority mode: **no** — plain force flag (SS6 `HideSet` takes no `KindIgnoreAttribute`).
- *Optional SS7-era extension (out of baseline):* a symmetric **force-visible** that overrides a
  keyframed hide. SS6 has no such thing; leave it open (§9-3).

---

## 6. Priority mode — Color + Cell only

SS6 `KindIgnoreAttribute` (`Library_SpriteStudio6.cs:95`) is taken **only** by `PartsColorChange`,
`CellChangeParts`, and `AnimationChangeInstance` — the keyframe-conflicting overrides. `HideSet`
and `FlipSet` do not use it. So the priority mode applies to **Color and Cell only** (this is why
the ROADMAP's "shared across all three kinds" is corrected here).

| SS6 | Enum | Semantics | Implementation |
|---|---|---|---|
| `NON` | `OverwriteOnNextKeyframe` | One-shot: the keyframe reclaims the attribute on the next decode. | At the seam, check the attribute's `UpdateAttributeFlags` bit on `part_state` (`src/core/framedata.rs:458` etc.). If a keyframe wrote it this frame → drop the override (clear its slot); else apply. |
| `NOW_ANIMATION` | `HoldUntilNextAnimation` | Wins for the current animation; cleared on a new one. | Apply every frame; **clear in `setup_animation_by_hash`** (`src/lib.rs:1044`). |
| `PERMANENT` | `Permanent` | Persists across animation changes too. | Apply every frame; **survive** `setup_animation`, but re-validate against new `num_parts` (drop `part_index >= num_parts`). |

Part-index space changes on `setup_animation` (§3), so `HoldUntilNextAnimation`-clear is also the
safe default; only `Permanent` needs re-validation.

---

## 7. sRGB vs linear — decision

The runtime does **all color math in 8-bit sRGB integer space**; no linear/gamma/PMA conversion
anywhere (deferred to Player shader hooks `ss_input_texture()` / `ss_output_color()`,
`docs/40_components/50_parts_color.ja.md`). **Decision:** override color values are interpreted in
the **same 8-bit sRGB space** as keyframed PartColor and undergo the **same alpha pre-multiply**
(§5a). One contract, no new color pipeline. Document so Players don't double-convert.

---

## 8. FFI surface (proposed)

House style (`src/ffi/mod.rs`): opaque `context: *mut c_void` → `get_context` (`:64`); return
`bool`, stash `ffi_context.last_error = e.as_code()`. Parts by `u32` index (sentinel for "all").

```c
// resolution
int32_t ss_runtime_find_part_index(void* ctx, const char* part_name);   // -1 if not found

// color override — priority_mode: 0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent
//   blend_op: 0=Mix 1=Mul 2=Add 3=Sub. single-RGBA form fills all four corners.
bool ss_runtime_set_part_color_override(void* ctx, uint32_t part, uint8_t blend_op,
                                        uint32_t rgba, uint8_t priority_mode);
//   4-corner form: rgba_ltrtlbrb = 4 packed RGBA in lt,rt,lb,rb order (map SS6 LU,RU,RD,LD).
bool ss_runtime_set_part_color_override_corners(void* ctx, uint32_t part, uint8_t blend_op,
                                                const uint32_t* rgba_ltrtlbrb, uint8_t priority_mode);
bool ss_runtime_clear_part_color_override(void* ctx, uint32_t part);

// cell override — Normal/Mask only. (0,0) = follow animation.
bool ss_runtime_set_part_cell_override(void* ctx, uint32_t part,
                                       uint32_t cellmap_name_hash, uint32_t cell_name_hash,
                                       uint8_t priority_mode);
bool ss_runtime_clear_part_cell_override(void* ctx, uint32_t part);

// visibility override — NO priority mode. force: 0=clear/revert 1=force-hide. cascade!=0 → descendants.
bool ss_runtime_set_part_visibility_override(void* ctx, uint32_t part, uint8_t force, uint8_t cascade);
bool ss_runtime_clear_part_visibility_override(void* ctx, uint32_t part);

// bulk
bool ss_runtime_clear_all_part_overrides(void* ctx);
```
cbindgen regenerates `ssruntime.h` (`libs/ssruntime/scripts/generate.*`). WASM parity
(`src/web/mod.rs`) is a later step (§9).

> **Dropped vs the previous draft:** no `ss_runtime_set_cellmap_texture_override` / cell-map table
> swap (texture swap is Player-side already, §2); no color-label read-back (schema change); no
> `priority_mode` on visibility.

---

## 9. Lifecycle integration

| Hook | File | Action |
|---|---|---|
| `InstanceState::new` | `src/lib.rs:936` | init empty `part_overrides` |
| `setup_animation_by_hash` | `src/lib.rs:1044` | resize to new `num_parts`; **clear** `HoldUntilNextAnimation` (color/cell) + all visibility overrides; **re-validate** `Permanent`; rebuild `name_hash→index` + child-list caches |
| `reset` | `src/lib.rs:1005` | clear the whole table |
| `bind_resource` | `src/lib.rs:1026` | new ssab → clear (part identity invalid) |

---

## 10. Validation plan — which Player, and how

**Primary: Godot (`../SSPlayerForGodot`).** Only mature player that calls the real `ssruntime`
C-API directly (GDExtension C++, `#include "ssruntime.h"`, static-links `libssruntime.a`) → new
FFI is callable after a cbindgen header regen **with no player-side binding codegen** (unlike
Unity csbindgen / Web wasm-bindgen / RenPy manual ctypes). Already has all three rendering hooks
(per-part color materials, `is_part_hidden`, `setCellMapOverrideTexture`), an interactive editor
playback panel, and a named "Tier 2 override" plan (`set_part_color_override` / `set_part_cell_override`).
Matches the ROADMAP intent.

**Rust sandbox (fastest inner loop): ssplayer-wgpu** (`feat/wgpu_proto`) — native Rust, calls
`ssruntime` as a crate (bypasses the C-API), winit viewer, color parity + cellmap substrate
present. Shake out the override-table / priority-mode **logic** here first. Caveat: bypasses the
C-API (can't sign off FFI ergonomics) and is on an unmerged branch → companion, not primary.

**Second: Unity** (P/Invoke generalization; has the SS6 override port plan). Web / RenPy last
(heavier binding regen). **Order:** wgpu (logic) → **Godot (C-API sign-off)** → Unity → Web/RenPy.

### Test approach
- **Rust unit/integration** (`libs/ssruntime`, host `std` OK): each override applied at the seam
  → expected `hide` / repointed `cell` / `part_color` in emitted FrameData; priority-mode matrix
  (keyframe present vs absent × NON/NOW/PERMANENT) for color+cell; clear-on-`setup_animation` vs
  `Permanent` survival; visibility cascade over a known hierarchy. Reuse `tests/overall`.
- **Golden FrameData** diff (override off vs on).
- **Godot** interactive: GDScript set/clear on the playback panel; headless render diff if available.

---

## 11. Implementation checklist (cold-start order)

- [ ] `PartOverride` + `PriorityMode` (color/cell) types; `part_overrides` on `InstanceState`; ctor init.
- [ ] `apply_part_overrides(...)` + call at seam `src/lib.rs:1423`.
- [ ] **Visibility** first (simplest — inline flag; force-hide + clear; cascade via alpha-pass template).
- [ ] **Color** (append to `parts_color_vec`, repoint, replicate alpha pre-multiply; Normal-only guard).
- [ ] **Cell** (re-resolve `ResolvedCellMeta`, append, repoint; Normal/Mask guard).
- [ ] Priority mode (color/cell) via `UpdateAttributeFlags` + lifecycle clear/re-validate (§9).
- [ ] `name_hash→index` resolver + `ss_runtime_find_part_index`; "all parts" sentinel.
- [ ] FFI exports (§8); cbindgen regen.
- [ ] Rust tests (§10). Then Godot wiring + interactive validation.
- [ ] (later) WASM bindings; Unity migration.

---

## 12. Open decisions

1. **Force-visible extension** (§5c): baseline is SS6-faithful force-hide + clear. Add a symmetric
   force-visible that overrides a keyframed `hide`? (SS6 had none.) *Recommend: leave out of
   baseline; add later on concrete demand.*
2. **"All parts" sentinel** (SS6 `idParts == -1`) and whole-anim (`0`): port these conveniences, or
   index-only? *Recommend: support an all-parts sentinel; skip whole-anim force-hide (an object-wide
   concern the Player can do).*
3. **Cell override addressing**: FFI takes name hashes (matches the format). Also offer an
   index-based form (SS6 used cellmap/cell indices)? *Recommend: hashes only; host resolves.*

## 13. Out of scope (with reasons) — see §2 for the SS6 mapping
Texture / cell-map swap (Player-side already) · color-label read-back (needs schema change) ·
Flip (`FlipSetParts` → Player post-process, ROADMAP "not on roadmap by design") · Instance swap
(`AnimationChangeInstance` → Phase 3 Instance Lifecycle) · whole-object color/opacity/scale/order
(`AdditionalColor`/`RateOpacity`/… → Player) · per-part transform (`RefreshCancelTransform` is a
Unity GameObject artifact; no SS6 `PositionSetParts`) · shader replace (Player/material).

## 14. File reference index

| Concern | Path |
|---|---|
| `get_frame_data` + **seam** | `libs/ssruntime/src/lib.rs:1340` (seam `:1423`) |
| `InstanceState` / lifecycle | `src/lib.rs:915` (`new` `:936`, `reset` `:1005`, `bind_resource` `:1026`, `setup_animation` `:1044`) |
| `LayerState` / `parts_vec` | `src/lib.rs:468` |
| `PartState` accessors | `src/runtime/framedata.rs:1664` |
| decode / evaluate / compose / alpha / cell resolve | `src/core/framedata.rs` (`extract_pose:666`, `evaluate_part:445`, `compose_frame_data:1069`, `apply_alpha:762`, cell `:55`, cascade template `:747-760`) |
| FFI house style | `src/ffi/mod.rs` (`get_context:64`, examples `:900,:998,:1459`) |
| color / cell / hide schema | `libs/ssab/fbs/ssab.fbs` (PartColor `:365`, Cell `:355`, CellMap `:194`, parent `:644`), `libs/ssruntime/fbs/framedata.fbs` |
| SS6 origin API | `SS6PlayerForUnity/Assets/SpriteStudio6/Script/Root/Function*.cs`, `.../Library/Library_SpriteStudio6.cs` |
| ROADMAP item | `ROADMAP.md` (Phase 2, "Override Layer API") |
