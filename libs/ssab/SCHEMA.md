[**English**](./SCHEMA.md) | [**日本語**](./SCHEMA.ja.md)

# SSAB Schema Documentation

This document describes the SSAB (SpriteStudio Anime Binary) and SSQB (SpriteStudio Sequence Binary) schemas by comparing them with the original XML-based ss*e formats (ssae, ssce, ssee, ssse, ssqe).

## Overview

| Item | ss*e (XML) | ssab / ssqb (Binary) |
|------|-----------|---------------------|
| Format | XML (quick-xml + serde) / BMFont text | FlatBuffers |
| String Lookup | Name-based string comparison | Hash-based (uint32) lookup |
| Access Pattern | Deserialization to owned structs | Zero-copy read |
| `no_std` Support | No | Yes |

### Conversion Unit

One `.ssab` file is generated **per `.ssae` (animation pack)**. The `.ssab` contains the ssae data plus the ss*e data it references (ssce, ssee, ssse, fnt). The `.sspj` project file itself is not included in ssab — it is only used during conversion to resolve resource references.

```
.sspj (project) — used only for resource resolution
 │
 ├─ .ssae (animation pack A) ──────► .ssab (SsAnimeBinary for A)
 │    contains:
 │    ├─ animation & part data from .ssae
 │    ├─ .ssce (cellmaps used by A)
 │    ├─ .ssee (effects used by A)
 │    ├─ .ssse (sounds used by A)
 │    └─ .fnt  (fonts used by A)
 │
 ├─ .ssae (animation pack B) ──────► .ssab (SsAnimeBinary for B)
 │    contains:
 │    ├─ animation & part data from .ssae
 │    └─ .ssce (cellmaps used by B)
 │
 └─ .ssqe (sequence pack)   ──────► .ssqb (SsSequenceBinary)
```

Each `.ssab` is self-contained: it embeds only the ss*e resources that the specific animation pack references. Resources not used by the pack are excluded.

---

## SSAB Root: `SsAnimeBinary`

**FlatBuffers identifier:** `"SSAB"`

| ssab field | Type | Source | Notes |
|-----------|------|--------|-------|
| `version` | uint | — | Binary format version (0x00000001) |
| `name` | string | `SsAnimePack.name` (ssae) | Animation pack name |
| `name_hash` | uint32 | — | Hash of the name for fast lookup |
| `parts` | [PartData] | `SsAnimePack.model.part_list` (ssae) | Flat array of all parts in this pack |
| `animations` | [AnimationData] | `SsAnimePack.anime_list` (ssae) | All animations in this pack |
| `cellmaps` | [CellMap] | `SsCellMap` (ssce) | Only cellmaps referenced by this pack (including those used by effects) |
| `effects` | [Effect] | `SsEffectFile` (ssee) | Only effects referenced by Effect-type parts |
| `sound_lists` | [SoundList] | `SsAudioPack` (ssse) | Included only if the pack contains Audio-type parts |
| `font_bitmaps` | [FontBitmap] | `SsCharMap` (fnt) | Only bitmap fonts used by Text-type parts with `text_bitmap == true` |
| `external_textures` | [ExternalTexture] | — | External texture references; sorted by `name_hash`. Union of the project's `<ExternalTextures>` list and every distinct `TextureName` named by a `TCHG` key in this pack (including cellmap images such as `common.png`). This is the reverse-lookup table a Player needs to turn a `TCHG` hash — and hence `DrawBatch.texture_hash` — back into a loadable name, so it must cover both sources. |
| `external_instances` | [ExternalInstance] | — | Cross-pack instance references; sorted by `(anime_pack_name_hash, anime_name_hash)`. The table carries no FlatBuffers `(key)`, so a reader scans the vector rather than calling `LookupByKey`. Each entry carries `anime_pack_name_hash` (matches `SsAnimeBinary.name_hash` of the target pack), `anime_pack_name` (used to locate the sibling `<pack>.ssab` file), `anime_name_hash`, and `anime_name` (diagnostic). |
| `embedded_assets` | [EmbeddedAsset] | — | Optionally embedded binary assets (images, sounds, etc.) |
| `keyframes` | Keyframes | `SsAnimePack.anime_list` (ssae) | Every keyframe of every animation in this pack, one column per field. See Keyframe Data |

---

## Part Data

### `PartData` (ssab) vs `SsPart` (ssae)

| ssab field | Type | ssae field | Notes |
|-----------|------|-----------|-------|
| `name` | string | `name` | Part name |
| `parent_index` | short | `parent_index` | Parent part index |
| `bounds_type` | BoundsType | `bounds_type` | Bounding calculation mode |
| `blend_type` | BlendType | `alpha_blend_type` | Renamed field |
| `visible_inside_mask` | bool | `visible_inside_mask` | The polarity of a masked drawing part: `1` = draw inside the mask / `0` = draw outside → [Mask §2-4](../ssruntime/docs/40_parts/40_mask.md#2-4-polarity-whether-to-draw-inside-or-outside-the-mask) |
| `mask_write` | bool | `write_mask` | Renamed field. Turns a drawn part into a **clipping** writer. Clipping emits no `DrawBatchKind::Mask`, so the writer list has to be settled from this flag → [Mask §1-3](../ssruntime/docs/40_parts/40_mask.md#1-3-part-roles-and-mask_influence) |
| `mask_influence` | bool | `mask_influence` | **A dual definition whose meaning changes with the role**: on a mask writer it is the OP (`0` = increment / `1` = invert); on a masked drawing part it is the target flag (`1` = affected / `0` = opted out). Clipping is both a writer and a drawing part, so the same value serves both meanings at once. Do not drop either one → [Mask §1-3](../ssruntime/docs/40_parts/40_mask.md#1-3-part-roles-and-mask_influence) |
| `draw_as_mask` | bool | `shape_mask` / `text_mask` / `nines_mask` | Unified pure-mask flag for drawing parts (Shape/Text/Nines). Mutually-exclusive per-type tags in the source .ssae. `PartTypeMask` is always a pure mask regardless of this flag |
| `output_influence` | bool (default `true`) | `output_influence` | Carried through from the authoring data unchanged; `libssruntime` does not read it |
| `mesh_binding` | PartMeshBinding | (computed) | Per-vertex bone influences for `PartTypeMesh`. See [Mesh Binding](#mesh-binding) below. Optional; absent for non-mesh parts. |
| `part_type` | PartType (union) | `part_type` + type-specific fields | See below |

### Part Type Union

In ssae, the part type is a simple enum (`SsPartType`) and type-specific data (e.g., shape, text, nines) is stored as flat fields on `SsPart`. In ssab, this is modeled as a FlatBuffers **union** (`PartType`) with dedicated tables per type:

| ssab Union Variant | ssae Equivalent | Data |
|-------------------|----------------|------|
| `PartTypeNull` | `SsPartType::Null` | None |
| `PartTypeNormal` | `SsPartType::Normal` | None |
| `PartTypeShape` | `SsPartType::Shape` | `shape_type` |
| `PartTypeText` | `SsPartType::Text` | `text`, `text_bitmap`, `text_family`, etc. |
| `PartTypeNines` | `SsPartType::Nines` | `nines_margin_l/r/t/b`, `nines_fill_mode` |
| `PartTypeInstance` | `SsPartType::Instance` | `ref_anime_pack` + `ref_anime_pack_hash`, `ref_anime_name` + `ref_anime_name_hash` (ssae: `ref_anime_pack` + `ref_anime` as strings) |
| `PartTypeArmature` | `SsPartType::Armature` | None |
| `PartTypeEffect` | `SsPartType::Effect` | `ref_effect_name_hash` (ssae: `ref_effect_name` as string) |
| `PartTypeMesh` | `SsPartType::Mesh` | None |
| `PartTypeMovenode` | `SsPartType::Movenode` | None |
| `PartTypeConstraint` | `SsPartType::Constraint` | None |
| `PartTypeMask` | `SsPartType::Mask` | None |
| `PartTypeJoint` | `SsPartType::Joint` | None |
| `PartTypeBonepoint` | `SsPartType::Bonepoint` | None |
| `PartTypeTransformConstraint` | `SsPartType::TransformConstraint` | None |
| `PartTypeCamera` | `SsPartType::Camera` | None |
| `PartTypeAudio` | `SsPartType::Audio` | None |

### Pack/Model-level Fields Not Carried Over

The following ssae pack and model-level fields are not included in ssab:

- `SsAnimePack.version` (ssab has its own `version` field)
- `SsAnimePack.settings` (pack-level animation settings; per-animation `settings` are converted)
- `SsAnimePack.cellmap_names` (replaced by embedded cellmap data in ssab)
- `SsModel.bone_list` (bone name-to-index map, editor/converter use only)
- `SsModel.ref_setup_anime_pack`, `SsModel.ref_setup_animation` (computed at load time)

### Part-level Fields Not Carried Over

The following ssae `SsPart` fields are editor-only and not included in ssab:

- `array_index`, `show`, `locked`, `color_label` (editor UI state)
- `inherit_type`, `inherit_rates` (resolved at conversion time)
- `bone_length`, `bone_position`, `bone_rotation` (bone editing data)
- `weight_position`, `weight_impact`, `mesh_weight_type`, `mesh_weight_strong` (mesh weight editing)
- `ik_depth`, `ik_rotation_arrow` (IK solver parameters)

### Mesh Binding

For `PartTypeMesh`, the converter resolves the ssae mesh-weight description into per-vertex bone influences and stores them on `PartData.mesh_binding`. Other part types omit the field. The influences are **parallel arrays indexed by a prefix-offset table**, not nested tables: one vector allocation per part rather than one per vertex.

| Field | Type | Notes |
|-------|------|-------|
| `vertex_influence_offset` | [uint32] | Prefix offsets — one per mesh vertex **plus a trailing total**, so it holds `vertex_count + 1` entries. Vertex `i`'s influences are the half-open range `[offset[i], offset[i + 1])` in the four arrays below, and the outer index matches the mesh vertex order. |
| `influence_bone` | [ushort] | Part index of the influencing bone — already remapped from the ssae bone-list index to the corresponding part index, so the runtime can index `parts` directly. |
| `influence_weight` | [float] | Influence weight, normalised to `0.0`-`1.0` (ssae authors it as a percentage). |
| `influence_coord_x` / `influence_coord_y` | [float] | Bind-pose offset of the vertex relative to the influencing bone, split across two arrays. |

---

## Cell Map

### `CellMap` (ssab) vs `SsCellMap` (ssce)

| ssab field | Type | ssce field | Notes |
|-----------|------|-----------|-------|
| `name_hash` | uint32 (key) | — | Hash for fast lookup |
| `name` | string | `name` | Cell map name |
| `image_path` | string | `image_path` | Path to source image |
| `image_path_hash` | uint32 | — | Hash of image path |
| `size_original` | Vec2 | `pixel_size` | Image pixel dimensions |
| `wrap_mode` | TexWrapMode | `wrap_mode` | Texture wrap mode |
| `filter_mode` | TexFilterMode | `filter_mode` | Texture filter mode |
| `cells` | [Cell] | `cells` | Cell definitions |

### `Cell` (ssab) vs `SsCell` (ssce)

| ssab field | Type | ssce field | Notes |
|-----------|------|-----------|-------|
| `name_hash` | uint32 (key) | — | Hash for fast lookup |
| `name` | string | `name` | Cell name |
| `region` | Region | `pos` + `size` | Stored verbatim: `x,y` = `pos`, `w,h` = `size` |
| `pivot` | Vec2 | `pivot` | Cell pivot point |
| `rotated` | bool | `rotated` | Whether cell is rotated in atlas |
| `table_coord_x` | [float] | `mesh_point_list[*].x` | SoA mesh vertex X coordinates |
| `table_coord_y` | [float] | `mesh_point_list[*].y` | SoA mesh vertex Y coordinates |
| `table_index_vertex` | [ushort] | `mesh_tri_list` | Mesh triangle indices (flattened from SsPoint3) |

### Fields Not Carried Over

- `version`, `override_tex_settings` (ssce metadata)
- `is_mesh`, `inner_point`, `outer_point` (mesh editing data)
- `div_type`, `div_w`, `div_h` (mesh division parameters)

---

## Animation Data

### `AnimationData` (ssab) vs `SsAnimation` (ssae)

| ssab field | Type | ssae field | Notes |
|-----------|------|-----------|-------|
| `name` | string (required) | `name` | Animation name. `(required)` enforces presence at verify time, but no `(key)` because the array preserves sspj source order so "first animation" semantics stay deterministic — sorting by key for `LookupByKey` would break that. |
| `name_hash` | uint32 | — | fnv1a(name). Use linear scan for hash-based lookup. |
| `start_frame` | short | `settings.start_frame` | Start frame |
| `end_frame` | short | `settings.end_frame` | End frame |
| `total_frame` | short | `settings.frame_count` | Total frame count |
| `fps` | short | `settings.fps` | Frames per second |
| `canvas_size` | Vec2 (required) | `settings.canvas_size` | Authored canvas dimensions, in pixels. |
| `pivot` | Vec2 (required) | `settings.pivot` | Normalized position of the runtime origin within the canvas. Defaults to `0, 0` (canvas center). |
| `parts_initial_attributes` | [InitialPartAttributes] | Setup animation data | See Initial Attributes section |
| `parts_animation_data` | [PartAnimationData] | `part_animes` | See Animation Data Layout section |
| `events` | [EventsPerFrame] | Event attributes per part | See Events section |
| `initial_events` | [InitialEvents] | Setup animation events | Initial event state |
| `labels` | [Label] | `labels` | Animation labels |

### Fields Not Carried Over

- `override_settings` (editor setting)
- `is_setup` (the setup animation data is merged into `parts_initial_attributes`)

---

## Animation Data Layout: Columnar vs Row-based

This is the most significant structural difference between the two formats.

### ssae: Row-based (per-part, per-attribute)

In ssae, animation data is organized hierarchically by part, then by attribute:

```
SsAnimation
  └─ part_animes: [SsPartAnime]       // one per part
       ├─ part_name: "arm"
       └─ attributes: [SsAttributeKind]  // list of attribute types
            ├─ PosX { keys: [{ time: 0, value: 10.0 }, { time: 30, value: 50.0 }] }
            ├─ RotZ { keys: [{ time: 0, value: 0.0 }, { time: 15, value: 90.0 }] }
            └─ Alpha { keys: [{ time: 0, value: 1.0 }] }
```

Each attribute is identified by a tag string (e.g., `"POSX"`, `"ROTZ"`, `"ALPH"`).

### ssab: a run of keys per attribute, the keys in columns

In ssab, each part's `PartAnimationData` names, per attribute, the run of keys that animates it, and the keys of every animation in the pack sit in one set of columns, `SsAnimeBinary.keyframes`:

```
SsAnimeBinary
  ├─ animations: [AnimationData]
  │    └─ parts_animation_data: [PartAnimationData]   // one per part
  │         ├─ position_x_keys: 3                     // run 3
  │         ├─ rotation_z_keys: 4                     // run 4
  │         └─ (every other field absent: no keys)
  └─ keyframes: Keyframes
       ├─ runs:          [ …, 10, 12, 14, … ]         // run 3 is keys 10 and 11
       ├─ frame:         [ …, 0, 30, 0, 15, … ]
       ├─ interpolation: [ …, Linear, None, Linear, None, … ]
       ├─ curve:         [ …, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, … ]
       └─ value:         [ …, 10.0, 50.0, 0.0, 90.0, … ]   // a float's bits
```

A `PartAnimationData` field is the index of a run, or its default `0xFFFFFFFF` (`ssab::NO_KEYS`) when the animation does not key that attribute on that part. Run `r` is keys `runs[r]` up to, but not including, `runs[r + 1]`. Runs with the same keys are stored once, so the setup pose every animation is seeded with costs one run however many animations name it.

### Attribute Tag Mapping

The value column says how a key's `Keyframes.value` is read for that attribute.

| ssae Tag | ssab Field | Value |
|---------|-----------|-----------|
| `CELL` | `cell_keys` | index into `cells` (PartAttributeCell) |
| `POSX` | `position_x_keys` | `float` |
| `POSY` | `position_y_keys` | `float` |
| `POSZ` | `position_z_keys` | `float` |
| `ROTX` | `rotation_x_keys` | `float` |
| `ROTY` | `rotation_y_keys` | `float` |
| `ROTZ` | `rotation_z_keys` | `float` |
| `SCLX` | `scale_x_keys` | `float` |
| `SCLY` | `scale_y_keys` | `float` |
| `LSCX` | `local_scale_x_keys` | `float` |
| `LSCY` | `local_scale_y_keys` | `float` |
| `ALPH` | `alpha_keys` | `float` |
| `LALP` | `local_alpha_keys` | `float` |
| `PRIO` | `priority_keys` | `int` |
| `FLPH` | `flip_h_keys` | `bool` (0 or 1) |
| `FLPV` | `flip_v_keys` | `bool` (0 or 1) |
| `HIDE` | `hide_keys` | `bool` (0 or 1) |
| `PCOL` | `part_color_keys` | index into `part_colors` (PartAttributePartColor) |
| `SHDR` | `shader_keys` | index into `shaders` (PartAttributeShader) |
| `VERT` | `vertex_keys` | index into `vertices` (PartAttributeVertex) |
| `PVTX` | `pivot_x_keys` | `float` |
| `PVTY` | `pivot_y_keys` | `float` |
| `ANCX` | `anchor_x_keys` | `float` |
| `ANCY` | `anchor_y_keys` | `float` |
| `SIZX` | `size_x_keys` | `float` |
| `SIZY` | `size_y_keys` | `float` |
| `IFLH` | `img_flip_h_keys` | `bool` (0 or 1) |
| `IFLV` | `img_flip_v_keys` | `bool` (0 or 1) |
| `UVTX` | `uv_translation_x_keys` | `float` |
| `UVTY` | `uv_translation_y_keys` | `float` |
| `UVRZ` | `uv_rotation_z_keys` | `float` |
| `UVSX` | `uv_scale_x_keys` | `float` |
| `UVSY` | `uv_scale_y_keys` | `float` |
| `BNDR` | `bounding_radius_keys` | `float` |
| `MASK` | `mask_keys` | `float` |
| `VERT` (skew) | `skew_keys` | index into `skews` (Vec2). *Note: ssae's `VERT` tag contains both vertex offsets and skew data in a single `SsAttributeVertexKeyValue`. In ssab, these are split into separate `vertex_keys` and `skew_keys`.* |
| `DEFM` | `deform_keys` | index into `deforms` (PartAttributeDeform) |
| `TCHG` | `texture_keys` | `uint32`. *Value is `fnv1a(TextureName)`, resolvable via `external_textures`. An empty `TextureName` (authored "no override") is normalised to `0` rather than hashed, so it stays distinguishable from a real texture and matches the `texture` default; step-evaluated, never interpolated.* |

### Event Attributes (special handling)

Event-type attributes are **not** stored in `PartAnimationData`. Instead, they are stored in `EventsPerFrame`, organized by frame:

| ssae Tag | ssab Event Table | Notes |
|---------|-----------------|-------|
| `USER` | `EventUserData` | User-defined data |
| `SIGN` | `EventSignal` | Signal commands |
| `ADIO` | `EventAudio` | Audio triggers |
| `IPRM` | `EventInstance` | Instance control |
| `EFCT` | `EventEffect` | Effect control |

```
ssab event structure:
AnimationData.events: [EventsPerFrame]
  └─ frame_index: 5     (key)
       ├─ users:     [{ part_index: 2, value: ... }]
       ├─ signals:   [{ part_index: 0, value: ... }]
       ├─ audios:    []
       ├─ instances: [{ part_index: 3, value: ... }]
       └─ effects:   []
```

---

## Keyframe Data

### Columns

| `Keyframes` column | ssae field | Notes |
|-----------|-----------|-------|
| `frame` (ushort) | `time` (i32) | Frame index |
| `interpolation` | `ip_type` | Interpolation type enum |
| `curve` (ushort) | `curve`, `easing_rate` | An index; see below |
| `value` (uint) | `value` | The value's bits, or an index; see Attribute Tag Mapping |

In ssae, keyframes can also carry `curve` data (`SsCurve`) for Bezier/Hermite interpolation with `start_time`, `start_value`, `end_time`, `end_value`, and `sync_start_end`. ssab stores each distinct set of the four control values once, in `Keyframes.curves` (`KeyCurve`), and each distinct easing rate once, in `Keyframes.easing_rates`. A key's `curve` indexes `curves` when it is Hermite or Bezier, `easing_rates` when it is one of the six eases that take a rate (`EaseIn`, `EaseOut`, `EaseInOut`, `EaseElasticIn`, `EaseElasticOut`, `EaseElasticInOut`), and is `0xFFFF` (`ssab::NO_CURVE`) for every other key. `sync_start_end` is an authoring-side convenience and is not carried. Compound values — cells, part colors, shaders, vertices, skews and deforms — are likewise stored once each, in the table the attribute tag mapping names, and the key's `value` is the index.

### Cell Attribute

| ssab `PartAttributeCell` | ssae `SsAttributeCellKeyValue` | Notes |
|-------------------------|-------------------------------|-------|
| `cellmap_name_hash` (uint32) | `map_id` (i32) | fnv1a hash of the cellmap name (without `.ssce`). Resolved from `sspj.cellmap_names[map_id]` at convert time so the runtime can look up the cellmap by hash even when only a subset is bundled. |
| `name_hash` (uint32) | `name` (String) | Hash instead of string |

### Audio Attribute

| ssab `PartAttributeAudio` | ssae `SsAttributeAudioKeyValue` | Notes |
|--------------------------|--------------------------------|-------|
| `sound_list_name_hash` (uint32) | `sound_list_id` (i32) | Hash generated from SoundList name |
| `sound_name_hash` (uint32) | `sound_name` (String) | Hash generated from alias name |
| `sound_name` (string) | — | Diagnostic alias name (optional) |
| `loop_num` (int) | `loop_num` (i32) | Loop count |

---

## Initial Part Attributes

The `InitialPartAttributes` table stores the default/setup state for each part. This corresponds to the "Setup" animation in ssae (an animation with `is_setup: true`).

### `update_flag` (PartAttributesUpdateFlags)

A 64-bit bitmask indicating which attributes have been set to non-default values. This allows the runtime to skip unchanged attributes during initialization.

```
Bit  0: AttributeCell
Bit  1: AttributePosX
Bit  2: AttributePosY
...
Bit 36: AttributeDeform
Bit 37: AttributeTexture
```

38 flags are defined in total; the bit assignment order matches the declaration order in `PartAttributesUpdateFlags`.

### Default Values

| Field | Default |
|-------|---------|
| `position_x/y/z` | 0.0 |
| `rotation_x/y/z` | 0.0 |
| `scale_x/y` | 1.0 |
| `local_scale_x/y` | 1.0 |
| `alpha` | 1.0 |
| `local_alpha` | 1.0 |
| `priority` | 0 |
| `flip_h/v` | false |
| `hide` | false |
| `pivot_x/y` | 0.0 |
| `anchor_x/y` | 0.0 |
| `size_x/y` | 1.0 |
| `img_flip_h/v` | false |
| `uv_translation_x/y` | 0.0 |
| `uv_rotation_z` | 0.0 |
| `uv_scale_x/y` | 1.0 |
| `bounding_radius` | 0.0 |
| `mask` | 0.0 |
| `skew_x/y` | 0.0 |
| `texture` | 0 |

---

## Effect Data

### `Effect` (ssab) vs `SsEffectFile` (ssee)

| ssab field | Type | ssee field | Notes |
|-----------|------|-----------|-------|
| `name_hash` | uint32 (key) | — | Hash for fast lookup |
| `name` | string | `name` | Effect name |
| `fps` | short | `effect_data.fps` | Frame rate |
| `is_lock_rand_seed` | bool | `effect_data.is_lock_rand_seed` | Lock random seed |
| `lock_rand_seed` | short | `effect_data.lock_rand_seed` | Seed value |
| `layout_scale` | Vec2 | `effect_data.layout_scale_x/y` | ssee uses separate int fields (percentage); ssab uses Vec2 |
| `effect_node` | [EffectNode] | `effect_data.node_list` | Effect node tree |

### `EffectNode` (ssab) vs `SsEffectNode` (ssee)

| ssab field | ssee field | Notes |
|-----------|-----------|-------|
| `index` | `array_index` | Node index |
| `parent_index` | `parent_index` | Parent node index |
| `effect_node_type` | `node_type` | Root / Emitter / Particle (`EffectNodeType` enum) |
| `cell_name_hash` | `behavior.cell_name` | fnv1a hash of the cell name |
| `cell_map_name_hash` | `behavior.cell_map_name` | fnv1a hash of the cellmap name (without `.ssce`) |
| `blend_type` | `behavior.blend_type` | `RenderBlendType` enum — restricted to `Mix` / `Add` (not the full `BlendType` set used by parts) |
| `behavior` | `behavior.list` | Particle behaviors (as union array, wrapped by `EffectNodeBehaviorWrapper`) |

### Effect Node Behaviors

ssee uses a tagged enum (`SsEffectFunctionType`) with named variants. ssab uses a FlatBuffers union (`EffectNodeBehavior`):

| ssee Tag Name | ssab Union Variant | Shared Table |
|--------------|-------------------|-------------|
| `"Basic"` | `EffectParticleElementBasic` | Dedicated table |
| `"OverWriteSeed"` | `EffectParticleElementRndSeedChange` | IntValueEntry |
| `"Delay"` | `EffectParticleElementDelay` | IntValueEntry |
| `"Gravity"` | `EffectParticleElementGravity` | Vec2ValueEntry |
| `"init_position"` | `EffectParticleElementPosition` | MinMax2ValueEntry |
| `"init_rotation"` | `EffectParticleElementRotation` | Dedicated table |
| `"trans_rotation"` | `EffectParticleElementRotationTrans` | Dedicated table |
| `"trans_speed"` | `EffectParticleElementTransSpeed` | Vec2ValueEntry |
| `"add_tangentiala"` | `EffectParticleElementTangentialAcceleration` | Vec2ValueEntry |
| `"init_vertexcolor"` | `EffectParticleElementInitColor` | Dedicated table |
| `"trans_vertexcolor"` | `EffectParticleElementTransColor` | Dedicated table |
| `"trans_colorfade"` | `EffectParticleElementAlphaFade` | Vec2ValueEntry |
| `"init_size"` | `EffectParticleElementSize` | MinMax2AndVec2ValueTable |
| `"trans_size"` | `EffectParticleElementTransSize` | MinMax2AndVec2ValueTable |
| `"add_pointgravity"` | `EffectParticlePointGravity` | Dedicated table |
| `"TurnToDirection"` | `EffectParticleTurnToDirectionEnabled` | FloatValueEntry |
| `"InfiniteEmit"` | `EffectParticleInfiniteEmitEnabled` | IntValueEntry |

### Fields Not Carried Over

- `SsEffectNode.name` (node name, editor use only)
- `SsEffectNode.visible` (editor visibility flag)
- `SsEffectModel.bgcolor` (editor background color)

### VarianceValue Flattening

In ssee, many effect parameters use `VarianceValue<T>` with `value` and `subvalue` (min/max range). In ssab, these are flattened:

```
ssee: speed: VarianceValue<f32> { value: 5.0, subvalue: 10.0 }
ssab: speed_min_value: 5.0, speed_max_value: 10.0
```

---

## Sound Data

### `SoundList` (ssab) vs `SsAudioPack` (ssse)

| ssab field | Type | ssse field | Notes |
|-----------|------|-----------|-------|
| `name_hash` | uint32 (key) | — | Hash for fast lookup |
| `name` | string | `name` | Sound list name |
| `table_data` | [SoundFile] | `table.sounds` | Sound file entries |

### `SoundFile` (ssab) vs `SsSoundFile` (ssse)

| ssab field | ssse field | Notes |
|-----------|-----------|-------|
| `name_hash` | — | Hash for fast lookup (generated from Alias) |
| `name` | `alias` | Sound alias name |
| `file_path` | `filename` | File path |
| `file_path_hash` | — | Hash generated from file path |
| `time_total` | `msec` | Duration in milliseconds |


---

## Bitmap Font Data

### `FontBitmap` (ssab) vs `SsCharMap` (fnt)

| ssab field | fnt field | Notes |
|-----------|----------|-------|
| `name_hash` | — | Hash for fast lookup |
| `name` | — | Font name |
| `name_face` | `info.face` | Font face name |
| `size` | — | Font size |
| `padding_l/r/u/d` | — | Padding values |
| `base` | `common.base` | Baseline |
| `height_line` | `common.line_height` | Line height |
| `scale` | `common.scale_w/h` | Texture scale |
| `spacing` | `info.spacing` | Glyph spacing (x, y) |
| `size_texture_original` | — | Original texture size |
| `name_texture` | `pages[*].file` | Texture file names |
| `name_texture_hash` | — | Hash array of texture file names |
| `table_data` | `chars` | Glyph data |

### `FontGlyphBitmap` (ssab) vs `SsCharMapChar` (fnt)

| ssab field | fnt field | Notes |
|-----------|----------|-------|
| `id` | `id` | Character ID (a UTF-16 code unit) |
| `atlas` | `x` / `y` / `width` / `height` | The glyph's region in the page image |
| `offset` | `offset` | Glyph rendering offset |
| `width_advance` | `xadvance` | Horizontal advance width |
| `text` | — | Character text (derived from id in fnt) |
| `page` | `page` | Texture page index |
| `channel` | — | Channel index |

---

## SSQB: Sequence Binary

**FlatBuffers identifier:** `"SSQB"`

### `SsSequenceBinary` (ssqb) vs `SsSequencePack` (ssqe)

| ssqb field | ssqe field | Notes |
|-----------|-----------|-------|
| `version` | `version` | Format version |
| `sequence_list` | `sequence_list` | Sequence definitions |

### `Sequence` (ssqb) vs `SsSequence` (ssqe)

| ssqb field | ssqe field | Notes |
|-----------|-----------|-------|
| `name` | `name` | Sequence name |
| `name_hash` | — | Hash for fast lookup |
| `index` | `index` | Sequence index |
| `type` | `sequence_type` | Last / Keep / Top |
| `list` | `list` | Sequence items |

### `SequenceItem` (ssqb) vs `SsSequenceItem` (ssqe)

| ssqb field | ssqe field | Notes |
|-----------|-----------|-------|
| `ref_anime_pack` | `ref_anime_pack` | Referenced animation pack name |
| `ref_anime_pack_hash` | — | Hash for fast lookup |
| `ref_anime_name` | `ref_anime` | Referenced animation name |
| `ref_anime_name_hash` | — | Hash for fast lookup |
| `repeat_count` | `repeat_count` | Number of repeats |

The sequence format is largely identical between ssqe and ssqb, with minimal transformation.

---

## Enum Mappings

All enums have a 1:1 mapping between ss*e and ssab. Key enums:

### BlendType
`Mix`, `Mul`, `Add`, `Sub`, `Mulalpha`, `Screen`, `Exclusion`, `Invert`, `Mul2`, `Div2`, `Screen2`, `Overlay2`

### InterpolationType
`None`, `Linear`, `Hermite`, `Bezier`, `Acceleration`, `Deceleration`, `EaseIn`, `EaseOut`, `EaseInOut`, `EaseExponentialIn/Out/InOut`, `EaseSineIn/Out/InOut`, `EaseElasticIn/Out/InOut`, `EaseBounceIn/Out/InOut`, `EaseBackIn/Out/InOut`

### BoundsType
`None`, `Quad`, `Aabb`, `Circle`, `CircleSmin`, `CircleSmax`

### TexWrapMode
`Clamp`, `Repeat`, `Mirror`

### TexFilterMode
`Nearest`, `Linear`

---

## Primitive Types

| ssab Type | Definition | ss*e Equivalent |
|----------|-----------|----------------|
| `Vec2` | `{ v1: float, v2: float }` | `SsPoint2 { x, y }` |
| `Bounds` | `{ left, top, right, bottom: float }` | `SsIRect { x1, y1, x2, y2 }` — the user-data rectangle, authored as four edges |
| `Region` | `{ x, y, w, h: float }` | a `pos` + `size` pair (`.ssce`), or `x`/`y`/`width`/`height` (`.fnt`) |
| `MinMax2` | `{ x_min, y_min, x_max, y_max: float }` | a `value` / `subvalue` pair per axis (effect randomization range) |
| `U8Color` | `{ a, r, g, b: ubyte }` | `SsU8Color { a, r, g, b }` |
| `KeyCurve` | `{ start_time, start_value, end_time, end_value: float }` (struct) | `SsCurve` without `sync_start_end` |

---

## Key Design Decisions in ssab

1. **Per-ssae conversion**: Each ssab corresponds to a single ssae animation pack, bundling only the ss*e resources (ssce, ssee, ssse, fnt) that the pack actually uses. The sspj project file is not represented in ssab.

2. **Hash-based lookups**: All named entities include a `name_hash: uint32` field for O(1) lookup, avoiding string comparisons at runtime.

3. **Columnar keyframes**: Every key of the pack sits in one set of columns — frame, interpolation, curve, value — and a part's attribute names a run of them, so a key search walks contiguous memory and a key costs nine bytes. Runs, curves and compound values that repeat are stored once.

4. **Event separation**: Event-type attributes (user data, signals, audio, instance, effect) are stored per-frame rather than per-part, optimizing frame-based event dispatch.

5. **Flat part hierarchy**: Parts are stored in a flat array with `parent_index` references (same as ssae), but editor-only fields are stripped.

6. **Union-based type discrimination**: FlatBuffers unions replace ssae's tag-string dispatching, providing type-safe access without runtime string matching.

7. **Zero-copy reading**: The default (runtime) mode uses FlatBuffers' zero-copy deserialization. The `converter` feature enables Object API for mutation.

8. **String to hash conversion**: References between entities (e.g., instance references, cell names, labels) use `uint32` hashes instead of strings. The original string is sometimes kept alongside the hash for debugging purposes.
