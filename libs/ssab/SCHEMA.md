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
| `external_textures` | [ExternalTexture] | — | External texture references |
| `external_instances` | [ExternalInstance] | — | Cross-pack instance references; sorted by `anime_name_hash` (the FlatBuffers `(key)`) so the player can `LookupByKey(ref_anime_hash)` directly. Each entry carries `anime_pack_name_hash` (matches `SsAnimeBinary.name_hash` of the target pack), `anime_pack_name` (used to locate the sibling `<pack>.ssab` file), and `anime_name` (diagnostic). |
| `embedded_assets` | [EmbeddedAsset] | — | Optionally embedded binary assets (images, sounds, etc.) |

---

## Part Data

### `PartData` (ssab) vs `SsPart` (ssae)

| ssab field | Type | ssae field | Notes |
|-----------|------|-----------|-------|
| `name` | string | `name` | Part name |
| `parent_index` | short | `parent_index` | Parent part index |
| `bounds_type` | BoundsType | `bounds_type` | Bounding calculation mode |
| `blend_type` | BlendType | `alpha_blend_type` | Renamed field |
| `part_type` | PartType (union) | `part_type` + type-specific fields | See below |

### Part Type Union

In ssae, the part type is a simple enum (`SsPartType`) and type-specific data (e.g., shape, text, nines) is stored as flat fields on `SsPart`. In ssab, this is modeled as a FlatBuffers **union** (`PartType`) with dedicated tables per type:

| ssab Union Variant | ssae Equivalent | Data |
|-------------------|----------------|------|
| `PartTypeNull` | `SsPartType::Null` | None |
| `PartTypeNormal` | `SsPartType::Normal` | None |
| `PartTypeShape` | `SsPartType::Shape` | `shape_type`, `shape_mask` |
| `PartTypeText` | `SsPartType::Text` | `text`, `text_bitmap`, `text_family`, etc. |
| `PartTypeNines` | `SsPartType::Nines` | `nines_margin_l/r/t/b`, `nines_fill_mode`, `nines_mask` |
| `PartTypeInstance` | `SsPartType::Instance` | `ref_anime_hash` (ssae: `ref_anime_pack` + `ref_anime` as strings) |
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
| `rectangle` | Rect | `pos` + `size` | ssce uses pos/size pair; ssab uses rectangle (x1,y1,x2,y2) |
| `pivot` | Vec2 | `pivot` | Cell pivot point |
| `rotated` | bool | `rotated` | Whether cell is rotated in atlas |
| `table_coordinate` | [Vec2] | `mesh_point_list` | Mesh vertex coordinates |
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
| `name` | string (key) | `name` | Animation name |
| `name_hash` | uint32 | — | Hash for fast lookup |
| `start_frame` | short | `settings.start_frame` | Start frame |
| `end_frame` | short | `settings.end_frame` | End frame |
| `total_frame` | short | `settings.frame_count` | Total frame count |
| `fps` | short | `settings.fps` | Frames per second |
| `canvas` | Rect | `settings.canvas_size` | Canvas dimensions |
| `parts_initial_attributes` | [InitialPartAttributes] | Setup animation data | See Initial Attributes section |
| `parts_animation_data` | [PartAnimationData] | `part_animes` | See Animation Data Layout section |
| `parts_mesh_data` | [PartMeshData] | `SsModel.mesh_list` | Mesh UV/index data |
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

### ssab: Columnar (per-part, arrays per attribute type)

In ssab, each part's animation data contains a fixed set of **typed arrays**, one per attribute:

```
AnimationData
  └─ parts_animation_data: [PartAnimationData]   // one per part
       ├─ position_x_array: [{ index: 0, value: 10.0 }, { index: 30, value: 50.0 }]
       ├─ rotation_z_array: [{ index: 0, value: 0.0 }, { index: 15, value: 90.0 }]
       ├─ alpha_array: [{ index: 0, value: 1.0 }]
       └─ (all other arrays are empty/null)
```

### Attribute Tag Mapping

| ssae Tag | ssab Array | Value Type |
|---------|-----------|-----------|
| `CELL` | `cell_array` | PartAnimationDataCellValue |
| `POSX` | `position_x_array` | PartAnimationDataFloatValue |
| `POSY` | `position_y_array` | PartAnimationDataFloatValue |
| `POSZ` | `position_z_array` | PartAnimationDataFloatValue |
| `ROTX` | `rotation_x_array` | PartAnimationDataFloatValue |
| `ROTY` | `rotation_y_array` | PartAnimationDataFloatValue |
| `ROTZ` | `rotation_z_array` | PartAnimationDataFloatValue |
| `SCLX` | `scale_x_array` | PartAnimationDataFloatValue |
| `SCLY` | `scale_y_array` | PartAnimationDataFloatValue |
| `LSCX` | `local_scale_x_array` | PartAnimationDataFloatValue |
| `LSCY` | `local_scale_y_array` | PartAnimationDataFloatValue |
| `ALPH` | `alpha_array` | PartAnimationDataFloatValue |
| `LALP` | `local_alpha_array` | PartAnimationDataFloatValue |
| `PRIO` | `priority_array` | PartAnimationDataIntValue |
| `FLPH` | `flip_h_array` | PartAnimationDataBoolValue |
| `FLPV` | `flip_v_array` | PartAnimationDataBoolValue |
| `HIDE` | `hide_array` | PartAnimationDataBoolValue |
| `PCOL` | `part_color_array` | PartAnimationDataPartColor |
| `SHDR` | `shader_array` | PartAnimationDataShader |
| `VERT` | `vertex_array` | PartAnimationDataVertex |
| `PVTX` | `pivot_x_array` | PartAnimationDataFloatValue |
| `PVTY` | `pivot_y_array` | PartAnimationDataFloatValue |
| `ANCX` | `anchor_x_array` | PartAnimationDataFloatValue |
| `ANCY` | `anchor_y_array` | PartAnimationDataFloatValue |
| `SIZX` | `size_x_array` | PartAnimationDataFloatValue |
| `SIZY` | `size_y_array` | PartAnimationDataFloatValue |
| `IFLH` | `img_flip_h_array` | PartAnimationDataBoolValue |
| `IFLV` | `img_flip_v_array` | PartAnimationDataBoolValue |
| `UVTX` | `uv_translation_x_array` | PartAnimationDataFloatValue |
| `UVTY` | `uv_translation_y_array` | PartAnimationDataFloatValue |
| `UVRZ` | `uv_rotation_z_array` | PartAnimationDataFloatValue |
| `UVSX` | `uv_scale_x_array` | PartAnimationDataFloatValue |
| `UVSY` | `uv_scale_y_array` | PartAnimationDataFloatValue |
| `BNDR` | `bounding_radius_array` | PartAnimationDataFloatValue |
| `MASK` | `mask_array` | PartAnimationDataFloatValue |
| `VERT` (skew) | `skew_array` | PartAnimationDataVec2 | *Note: ssae's `VERT` tag contains both vertex offsets and skew data in a single `SsAttributeVertexKeyValue`. In ssab, these are split into separate `vertex_array` and `skew_array`.* |
| `DEFM` | `deform_array` | PartAnimationDataDeform |
| `TCHG` | `texture_array` | PartAnimationDataUIntValue |

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

### Interpolation

| ssab field | ssae field | Notes |
|-----------|-----------|-------|
| `index` (ushort, key) | `time` (i32) | Frame index / time |
| `interpolation` | `ip_type` | Interpolation type enum |
| `value` | `value` | The attribute value |

In ssae, keyframes can also carry `curve` data (`SsCurve`) for Bezier/Hermite interpolation with `start_time`, `start_value`, `end_time`, `end_value`, and `sync_start_end`. In ssab, curve data is not stored per-keyframe; the interpolation type enum alone determines the easing behavior.

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
Bit 0:  AttributeCell
Bit 1:  AttributePosX
Bit 2:  AttributePosY
...
Bit 36: AttributeTexture
```

### Default Values

| Field | Default |
|-------|---------|
| `position_x/y/z` | 0.0 |
| `rotation_x/y/z` | 0.0 |
| `scale_x/y` | 1.0 |
| `local_scale_x/y` | 1.0 |
| `alpha` | 1.0 |
| `local_alpha` | 1.0 |
| `priority` | 0.0 |
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
| `effect_node_type` | `node_type` | Root / Emitter / Particle |
| `cell_index` | `behavior.cell_name` + `behavior.cell_map_name` | Resolved to index |
| `blend_type` | `behavior.blend_type` | Render blend mode |
| `behavior` | `behavior.list` | Particle behaviors (as union array) |

### Effect Node Behaviors

ssee uses a tagged enum (`SsEffectFunctionType`) with named variants. ssab uses a FlatBuffers union (`EffectNodeBehavior`):

| ssee Tag Name | ssab Union Variant | Shared Table |
|--------------|-------------------|-------------|
| `"Basic"` | `EffectParticleElementBasic` | Dedicated table |
| `"OverWriteSeed"` | `EffectParticleElementRndSeedChange` | IntValueEntry |
| `"Delay"` | `EffectParticleElementDelay` | IntValueEntry |
| `"Gravity"` | `EffectParticleElementGravity` | Vec2ValueEntry |
| `"init_position"` | `EffectParticleElementPosition` | RectValueEntry |
| `"init_rotation"` | `EffectParticleElementRotation` | Dedicated table |
| `"trans_rotation"` | `EffectParticleElementRotationTrans` | Dedicated table |
| `"trans_speed"` | `EffectParticleElementTransSpeed` | Vec2ValueEntry |
| `"add_tangentiala"` | `EffectParticleElementTangentialAcceleration` | Vec2ValueEntry |
| `"init_vertexcolor"` | `EffectParticleElementInitColor` | Dedicated table |
| `"trans_vertexcolor"` | `EffectParticleElementTransColor` | Dedicated table |
| `"trans_colorfade"` | `EffectParticleElementAlphaFade` | Vec2ValueEntry |
| `"init_size"` | `EffectParticleElementSize` | RectAndVec2ValueTable |
| `"trans_size"` | `EffectParticleElementTransSize` | RectAndVec2ValueTable |
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
| `size_texture_original` | — | Original texture size |
| `name_texture` | `pages[*].file` | Texture file names |
| `name_texture_hash` | — | Hash array of texture file names |
| `table_data` | `chars` | Glyph data |

### `FontGlyphBitmap` (ssab) vs `SsCharMapChar` (fnt)

| ssab field | fnt field | Notes |
|-----------|----------|-------|
| `id` | `id` | Character ID (Unicode code point) |
| `atlas` | `src_rect` | Source rectangle in atlas |
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
| `ref_anime` | `ref_anime` | Referenced animation name |
| `ref_anime_hash` | — | Hash for fast lookup |
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
| `Rect` | `{ x1, y1, x2, y2: float }` | `SsIRect { x1, y1, x2, y2 }` or pos+size pair |
| `U8Rect` | `{ a, r, g, b: ubyte }` | `SsU8Color { a, r, g, b }` |

---

## Key Design Decisions in ssab

1. **Per-ssae conversion**: Each ssab corresponds to a single ssae animation pack, bundling only the ss*e resources (ssce, ssee, ssse, fnt) that the pack actually uses. The sspj project file is not represented in ssab.

2. **Hash-based lookups**: All named entities include a `name_hash: uint32` field for O(1) lookup, avoiding string comparisons at runtime.

3. **Columnar animation data**: Attribute values are stored in typed arrays rather than tagged enums, enabling efficient iteration over a single attribute across frames.

4. **Event separation**: Event-type attributes (user data, signals, audio, instance, effect) are stored per-frame rather than per-part, optimizing frame-based event dispatch.

5. **Flat part hierarchy**: Parts are stored in a flat array with `parent_index` references (same as ssae), but editor-only fields are stripped.

6. **Union-based type discrimination**: FlatBuffers unions replace ssae's tag-string dispatching, providing type-safe access without runtime string matching.

7. **Zero-copy reading**: The default (runtime) mode uses FlatBuffers' zero-copy deserialization. The `converter` feature enables Object API for mutation.

8. **String to hash conversion**: References between entities (e.g., instance references, cell names, labels) use `uint32` hashes instead of strings. The original string is sometimes kept alongside the hash for debugging purposes.
