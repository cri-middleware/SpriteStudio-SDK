> [日本語](./95_special_parts.ja.md) | **English**

# Supporting Special Part Types

Implementation hints for special part kinds such as 9-slice, shape, text, and sound.

| Part type | Implementation hint |
| :--- | :--- |
| **9-slice** | Draw the `DrawBatchKind::Nines` batches from the dedicated SoA buffers described below. `local_vertices` only carries the part's plain bounding quad for these parts. |
| **Shape** | Use the dedicated SoA buffers such as `ss_runtime_get_shape_vertices` to obtain the polygon's vertex data. |
| **Text** | Glyphs are the Player's. The runtime only reserves the part's slot — a geometry-less `DrawBatchKind::Text` batch per part — so text composites by priority like anything else. See [Text](#text) for how to read the ssab and the glyph layout specification. |
| **Sound** | Watch `passed_event` inside your update loop and play the sound if `audios` events are included (see [Audio](./90_audio.md) for details). |

## 9-slice

The runtime builds the whole grid — margins, fill mode, repeats — so a Player only uploads vertices. A batch whose `kind` is `Nines` points at parts whose geometry lives in these buffers:

| Buffer | Contents |
| :--- | :--- |
| `ss_runtime_get_nines_vertices` | Interleaved `[x, y, ...]` in part-local space, with pivot, image flip and the Y-Up/Y-Down setting already applied — multiply by the part's world matrix, exactly like `local_vertices`. |
| `ss_runtime_get_nines_uvs` | Interleaved `[u, v, ...]` in cellmap texture-pixel space. Divide by the texture size to normalize, exactly like `local_uvs`. |
| `ss_runtime_get_nines_box_coords` | Interleaved `[fx, fy, ...]`, `0` at the part's left / top edge and `1` at its right / bottom. Bilinearly interpolate the four `part_color` corners with them, the same way `shape_vertex_box_coords` works. |
| `ss_runtime_get_nines_indices` | Triangle indices, **local to each part's vertex range**, wound `LT, RT, LB / LB, RT, RB`. |
| `ss_runtime_get_nines_vertex_offsets` | CSR prefix sum, length `num_parts + 1`, slicing the three vertex buffers. Multiply by 2 to index the interleaved floats. |
| `ss_runtime_get_nines_index_offsets` | CSR prefix sum, length `num_parts + 1`, slicing the index buffer. |

Part `p` therefore occupies vertices `[vertex_offsets[p], vertex_offsets[p + 1])` and indices `[index_offsets[p], index_offsets[p + 1])`; every other part has an empty range. `DrawBatch.vertex_count` / `index_count` carry the totals for the batch's parts, so buffers can be sized up front — but they are 16-bit and saturate at 65535, so a heavily tiled part should be measured from the CSR offsets instead.

The vertex count depends on the part's authored fill mode: **stretch** (`nines_fill_mode == 0`) shares a 4x4 grid — 16 vertices and 54 indices — while **tile** (`== 1`) emits one quad per repeat, so the count follows the part's size. The margins that authoring stores in `PartTypeNines` are in source-cell pixels and are clamped internally when they would overlap, so a Player never has to validate them.

> **Not applied to 9-slice parts:** the UV animation attributes (`UVTX` / `UVTY` / `UVRZ` / `UVSX` / `UVSY`) and cells rotated inside their cellmap. Both follow the authoring tool, which builds the grid from the cell rectangle alone.

## Text

Drawing text is a **Player responsibility** (see [Limitations](../10_introduction/20_limitations.md)). The runtime neither lays glyphs out nor rasterizes them: it gives the part its slot in the draw order and hands you the authored settings stored in the ssab. The rest of this section is the specification for everything from there to glyphs on screen.

### What the runtime gives you

A text part occupies **one `DrawBatch` per part**.

| `DrawBatch` field | Value | Meaning |
| :--- | :--- | :--- |
| `kind` | `DrawBatchKind::Text` | Marks the batch as a text part |
| `count` | Always `1` | Never merged with another text part |
| `vertex_count` / `index_count` | Always `0` | No geometry is carried |
| `texture_hash` | Always `0` | The texture to bind **is not determined by the batch** (resolve the font from the ssab instead) |
| `blend_type` | As usual | Use it when compositing the glyphs |

Text parts are never merged because **the runtime cannot key the font a part draws with**. Merging two text parts into one batch would hide a font change inside that batch where the Player could no longer see it. You can rely on a text batch always covering exactly one part.

#### Getting from a batch to the ssab part

```cpp
for (auto batch : *frame_data->draw_batches()) {
    if (batch->kind() != ss::runtime::DrawBatchKind::Text) continue;

    // count is always 1, but keep the loop shape identical to the other kinds
    for (int k = 0; k < batch->count(); k++) {
        int slot       = frame_data->draw_order()->Get(batch->start_rank() + k);
        int part_index = frame_data->parts()->Get(slot)->part_index();

        auto part = ssab->parts()->Get(part_index);
        auto text = part->part_type_as_PartTypeText();   // the settings described below

        const float* m = &world_matrices[slot * 16];     // this part's world matrix
        DrawGlyphs(text, m, frame_data->parts()->Get(slot));
    }
}
```

#### Prerequisites to keep in mind

- **A text part references no cell.** `PartState.cell` is `-1` and its `local_vertices` stay zeroed. You do not even get a quad, so **the Player builds all of the geometry**.
- Position, rotation, scale, alpha and priority arrive in `PartState` and the world matrix as usual. Build the glyphs in part-local coordinates and multiply by the world matrix exactly as you would for any other part.
- **The string is not animated.** The ssab holds a single static string from the part definition (`PartTypeText.text`); there is no keyframe attribute that swaps it.
- **Text authored as a mask** (`draw_as_mask`) is left out of the draw plan entirely, because the runtime has no glyphs to write coverage with (see [Mask](./40_mask.md)).

### What the ssab carries

#### `PartTypeText` — the part's settings

| Field | Type | Content |
| :--- | :--- | :--- |
| `text` | `string` | The string to draw (UTF-8). Line breaks are `\n` |
| `text_bitmap` | `bool` | `true` = bitmap font (use `text_char_map`) / `false` = outline font (use `text_family`) |
| `text_family` | `string` | Family name for an outline font (e.g. `Arial`). **The font file itself is not in the ssab** |
| `text_char_map` | `string` | Char map name for a bitmap font. It is the **same string** as `FontBitmap.name`: a project-relative path including the extension (e.g. `font/RoundedMPlus.fnt`) |
| `text_size` | `int` | Line height in px. The glyph scale is derived from it (see *Glyph scale* below) |
| `text_space` | `float` | Extra letter spacing in px. **Not scaled by `text_size`** |
| `text_smooth` | `bool` | Texture filter for the font pages. `true` = linear / `false` = nearest |
| `text_width` | `int` | Layout box width in px. `0` = auto-fit to the content |
| `text_height` | `int` | Layout box height in px. `0` = auto-fit to the content |
| `e_anchor` | `TextAnchor` | How lines are aligned inside the box |

`e_anchor` packs **horizontal in the high nibble and vertical in the low nibble**.

| | `0` | `1` | `2` |
| :--- | :--- | :--- | :--- |
| **High nibble** (`e_anchor >> 4`) | Left | Center | Right |
| **Low nibble** (`e_anchor & 0x0F`) | Top | Center | Bottom |

So `LT = 0x00`, `CC = 0x11`, `RB = 0x22`. `NO = 0xFF` means "unset" — treat it as top-left (`LT`).

#### `FontBitmap` / `FontGlyphBitmap` — the bitmap-font glyph table

`ssab->font_bitmaps()` holds **only the char maps actually referenced by a text part with `text_bitmap == true`**. In a project that uses outline fonts exclusively, the table is empty.

`FontBitmap.name_hash` is `fnv1a` of the `PartTypeText.text_char_map` string as-is, so part → font is a single hash lookup.

| `FontBitmap` | Content |
| :--- | :--- |
| `name` / `name_hash` | The char map name (= `text_char_map`) and its hash |
| `name_face` | Face name of the source font. Informational; not used for drawing |
| `base` | Baseline position in px, measured from the top of the line |
| `height_line` | Line height of the char map in px. **The denominator of the glyph scale** |
| `size_texture_original` | Original page-image size in px. **This is the value to normalize `atlas` with** — not the size of the texture you actually loaded |
| `name_texture[]` | Page image file names, **ordered by page id**, so `name_texture[glyph.page]` is the right lookup |
| `name_texture_hash[]` | Hashes of the above (the asset key in embedded mode) |
| `table_data[]` | The glyph table, sorted by ascending `id` so you can binary-search it |

| `FontGlyphBitmap` | Content |
| :--- | :--- |
| `id` | Character code (UTF-16 code unit) |
| `atlas` | Rect inside the page image. `x1,y1` = top-left, but **`x2,y2` are width and height, not the bottom-right corner** |
| `offset` | Where to start drawing the glyph relative to the pen (px, Y positive downwards) |
| `width_advance` | Advance width in px |
| `text` | `id` rendered as a character. For debugging |
| `page` | Index into `name_texture[]` |

Page image names have already been resolved by the converter from their path relative to the char map file, and arrive as the **same "project-relative file-reference string" used by every other external resource**. For the composition rules see [Loading & Initialization](../30_core/10_initialization.md) §3.0.

> [!IMPORTANT]
> **Fields you must not read, or expect anything from**
>
> - `FontBitmap`'s `size` / `padding_l` / `padding_r` / `padding_u` / `padding_d` / `scale` / `spacing`, and `FontGlyphBitmap.channel`, hold **fixed values** (`0` or `1.0`) and reflect nothing that was authored. Do not use them for layout. Letter spacing comes from `PartTypeText.text_space`, the line height from `height_line`.
> - **Kerning is not carried in the ssab.** Advance is determined by `width_advance` alone.
> - `FontGlyphBitmap.id` is 16 bits, so **characters outside the BMP (above U+FFFF) cannot be represented**. Content that needs emoji or rarer CJK ideographs has to be authored with an outline font, not a bitmap one.

### Layout specification (bitmap fonts)

Lay out in a coordinate space where **X points right and Y points down** (the direction the glyph `offset` is expressed in), then move the result into part-local coordinates at the end. Compute in **floating point**.

#### Glyph scale

```
scale = text_size / height_line
```

`text_size` states how tall one line should be in px; it is not the glyph's own size. It applies to every glyph as a ratio against the char map's `height_line`. A char map with `height_line == 0` would divide by zero — reject it at load time.

#### The offset baseline

```
base_offset = ( min(glyph.offset.x), min(glyph.offset.y) )   // component-wise minimum over every glyph in the char map
```

Subtract this minimum from each glyph's `offset` before placing it. It tightens the whole layout so that the glyph reaching furthest up and left in the char map touches the origin. It is a **constant of the char map, not of the individual string**, so compute it once at font load time and cache it.

#### Advancing through the string

Start with the pen at `pen = (0, 0)` and `line = 0`, then walk the string one character at a time:

```
if the character is '\n':
    pen = (0, pen.y + text_size)      // the line advance is text_size itself
    line += 1
    continue

if the char map has no glyph for it:
    do nothing                        // no advance, and no tofu box
    continue

pos  = pen + (glyph.offset - base_offset) * scale
size = (glyph.atlas.width, glyph.atlas.height) * scale

// Wrapping (only when text_width > 0)
if text_width > 0 and pos.x + size.x > text_width:
    pen = (0, pen.y + text_size)
    line += 1
    pos  = pen + (glyph.offset - base_offset) * scale

record this glyph at (pos, size), remembering its line number

pen.x += glyph.width_advance * scale + text_space
```

Points to keep in mind:

- **The line advance is `text_size`** (which equals `height_line * scale`).
- **`text_space` is not scaled.** Only the advance width is multiplied by `scale`; the letter spacing is added in raw px.
- **Wrapping is per character**, with no notion of word boundaries. `text_height` takes part in **neither** wrapping nor vertical clipping — it only affects the box size below.
- **Characters with no glyph are dropped entirely.** There is no fallback and no tofu box, and the pen does not advance. `\r` usually has no glyph either, so in a CRLF string the `\r` simply disappears and only the `\n` takes effect.

#### Bounding box and per-line extents

Once every glyph is placed, take the union of all glyph rects as `bound` (`left / top / right / bottom`), and the **right edge of each line, `R[line]`**.

#### Box size

```
box.w = (text_width  > 0) ? text_width  : bound.left * 2 + bound.width
box.h = (text_height > 0) ? text_height : bound.top  * 2 + bound.height
```

The auto-size case doubles the leading offset so that **the content's bounding box lands exactly at the center of the box** (`left + width + left = box.w`).

#### Applying the anchor

```
// Horizontal: per line
shift_x[line] = (box.w - R[line]) * (Left ? 0 : Center ? 0.5 : 1)

// Vertical: shared by every line
shift_y        = (box.h - bound.bottom) * (Top ? 0 : Center ? 0.5 : 1)
```

The horizontal shift looks **only at the line's right edge `R[line]`** — every line starts at `x = 0`, so its left edge can be taken as 0. The vertical shift is decided once from the bottom of the whole block and added to every line.

#### Moving into part-local coordinates

The box is placed **centered on the part's origin**, so the final glyph rects are:

```
local.x =  (pos.x + shift_x[line]) - box.w / 2
local.y = -((pos.y + shift_y)      - box.h / 2)   // flip the sign on a Y-up engine
```

Normalize UVs by dividing `atlas` by `size_texture_original` (**not** by the size of the texture you actually loaded). Bind `name_texture[glyph.page]`, with a linear or nearest filter according to `text_smooth`.

### Outline fonts (`text_bitmap == false`)

All the ssab carries is `text_family` (the family name) plus the layout settings — `text_size` / `text_space` / `text_width` / `text_height` / `e_anchor`. **The font file itself is not included.** Resolving a family name to a font is the host's job, and the result will look different on a machine where that font is not installed.

With no glyph table, the layout specification above does not apply. Hand the string and size to the host engine's text stack, and translate `e_anchor` and `text_width` / `text_height` into that stack's alignment and wrapping options.

---
[Back to Table of Contents](../README.md)
