> [日本語](./30_special_parts.ja.md) | **English**

# 9-Slice, Shape, and Text

These three parts take their vertices differently from ordinary sprites. For 9-slice and shape parts, the Brain passes the vertices in dedicated SoA buffers. For text, the Brain decides only the draw order, and the Player lays out and draws the characters.

| Part | `DrawBatch.kind` | Vertices |
| :--- | :--- | :--- |
| 9-slice | `Nines` | The SoA buffers for 9-slice (§1). `local_vertices` holds only the bounding rectangle |
| Shape | `Shape` | The SoA buffers for shapes (§2). There is no texture |
| Text | `Text` | None. The Player builds them from the ssab (§3) |

## 1. 9-slice

The Brain has already generated the grid, including the margins, the fill mode, and the number of repeats, so the Player only uploads the vertices.

| Function | Contents |
| :--- | :--- |
| `ss_runtime_get_nines_vertices` | `[x, y, ...]` in the part's local coordinates. The origin, image flip, and coordinate-system settings are already applied, so, as with the local vertices, you only multiply them by the world matrix |
| `ss_runtime_get_nines_uvs` | `[u, v, ...]` in pixel coordinates of the cellmap texture. As with the local UVs, divide them by the texture size to normalize them |
| `ss_runtime_get_nines_box_coords` | `[fx, fy, ...]`. `0` at the part's left and top edges, `1` at its right and bottom edges. Use them to interpolate the four corners of the parts color |
| `ss_runtime_get_nines_indices` | Triangle indices. The values are **0-based within the part**, in the order `LT, RT, LB / LB, RT, RB` |
| `ss_runtime_get_nines_vertex_offsets` | Prefix sum of length `num_parts + 1`. Slices the three vertex buffers above per part. Multiply by 2 to get the float position |
| `ss_runtime_get_nines_index_offsets` | Prefix sum of length `num_parts + 1`. Slices the index buffer per part |

Part `p` occupies vertices `[vertex_offsets[p], vertex_offsets[p + 1])` and indices `[index_offsets[p], index_offsets[p + 1])`. A part that is not a 9-slice has an empty range.

`DrawBatch.vertex_count` / `index_count` hold the totals for the DrawBatch's parts, so you can allocate buffers up front. However, they are 16-bit and saturate at 65535. For a part with a large number of tiles, take the actual counts from the prefix sums.

The number of vertices depends on the fill mode. **Stretch** (`nines_fill_mode == 0`) shares a 4x4 grid and is fixed at 16 vertices and 54 indices. **Tile** (`== 1`) emits a rectangle for each repeat, so the count grows and shrinks with the part's size. When the margins overlap, the Brain fits them, so the Player does not need to check them.

UV animation (`UVTX` / `UVTY` / `UVRZ` / `UVSX` / `UVSY`) and cells placed rotated inside the cellmap are not applied to 9-slice parts. Both match the behavior of the authoring tool.

## 2. Shape

A shape is a part that draws a figure such as a triangle, a rectangle, or a star. It has no texture, and `DrawBatch.texture_hash` is `0`.

| Function | Per part | Contents |
| :--- | :--- | :--- |
| `ss_runtime_get_shape_vertices` | 24 floats (12 vertices × `(x, y)`) | Vertices in the part's local coordinates. The origin and coordinate-system settings are already applied, so you only multiply them by the world matrix |
| `ss_runtime_get_shape_vertex_counts` | 1 value | The number of valid vertices for that part. The remaining slots are 0 |
| `ss_runtime_get_shape_vertex_box_coords` | 24 floats | Each vertex's position `(fx, fy)` within the bounding rectangle. The top-left is `(0, 0)` and the bottom-right is `(1, 1)` |

- The vertex and box-coordinate buffers are indexed by `part_index * 24`. When no part of the pack is a shape, both are empty; the vertex counts are still one per part, all `0`.
- The vertices are ordered as a **triangle fan**. The first vertex is the center, and the last vertex returns to the position of the second vertex to close the figure.
- For the parts color, bilinearly interpolate the four corner colors by `(fx, fy)` to get each vertex's color ([Parts Color §4-3](./10_parts_color.md#4-3-shape)).
- For a shape authored as a mask, the mask threshold has no effect, and the whole figure becomes the mask ([Mask §2-5](./40_mask.md#2-5-cutout-threshold)).

## 3. Text

The Player draws text ([Limitations §1](../10_introduction/20_limitations.md#1-the-player-implements-text-and-sound)). The Brain neither lays out nor draws the characters. It gives the part its place in the draw order and passes the authored settings in the ssab. This section describes the specification for getting from there to laid-out characters.

### 3-1. What the Brain passes

A text part becomes **one DrawBatch per part**.

| DrawBatch field | Value | Meaning |
| :--- | :--- | :--- |
| `kind` | `Text` | Marks a text part |
| `count` | Always `1` | Never merged with other text parts |
| `vertex_count` / `index_count` | Always `0` | No vertices are included |
| `texture_hash` | Always `0` | The texture is not determined by the DrawBatch (look the font up in the ssab) |
| `blend_type` | As usual | Used for the blend with the render target when drawing the characters |

Text parts are not merged because **the Brain cannot tell which font a part draws with**. If two text parts were merged into one DrawBatch, the Player could not tell that the font changes partway through. You may assume that the DrawBatch covers one part.

```cpp
for (auto batch : *frame_data->draw_batches()) {
    if (batch->kind() != ss::runtime::DrawBatchKind::Text) continue;

    int p          = frame_data->draw_order()->Get(batch->start_rank());
    int part_index = frame_data->parts()->Get(p)->part_index();
    auto text      = ssab->parts()->Get(part_index)->part_type_as_PartTypeText();   // the settings in §3-2

    draw_glyphs(text, &world_matrices[p * 16], frame_data->parts()->Get(p));
}
```

`draw_glyphs` is a fictitious function that lays out and draws the characters with the layout in §3-3.

- **A text part does not reference a cell.** `PartState.cell` is `-1`, and the local vertices are 0. Not even a rectangle arrives, so **the Player builds all the vertices.**
- Position, rotation, scale, opacity, and draw priority are in `PartState` and the world matrix as usual. Build the characters in the part's local coordinates and multiply them by the world matrix.
- **The string is not animated.** The ssab holds only the one string in the part definition (`PartTypeText.text`), and no keyframe changes the string.
- **Text authored as a mask** (`draw_as_mask`) is excluded from the draw plan.

### 3-2. What the ssab contains

#### `PartTypeText`: the part's settings

| Field | Type | Contents |
| :--- | :--- | :--- |
| `text` | `string` | The string to display (UTF-8). Line breaks are `\n` |
| `text_bitmap` | `bool` | `true` = bitmap font (uses `text_char_map`) / `false` = outline font (uses `text_family`) |
| `text_family` | `string` | Family name of the outline font (e.g. `Arial`). **The font file is not in the ssab** |
| `text_char_map` | `string` | Charmap name of the bitmap font. The **same string** as `FontBitmap.name`, including the path relative to the project and the extension (e.g. `font/RoundedMPlus.fnt`) |
| `text_size` | `int` | Height of one line (px). The character scale is derived from it (§3-3) |
| `text_space` | `float` | Amount added between characters (px). **Not scaled by `text_size`** |
| `text_smooth` | `bool` | Filter for the font texture. `true` = linear / `false` = nearest |
| `text_width` | `int` | Width of the layout box (px). `0` fits it to the content |
| `text_height` | `int` | Height of the layout box (px). `0` fits it to the content |
| `e_anchor` | `TextAnchor` | Alignment within the box |

In `e_anchor`, **the upper 4 bits are horizontal and the lower 4 bits are vertical**.

| | `0` | `1` | `2` |
| :--- | :--- | :--- | :--- |
| **Upper 4 bits** (`e_anchor >> 4`) | Left | Center | Right |
| **Lower 4 bits** (`e_anchor & 0x0F`) | Top | Center | Bottom |

So `LT = 0x00`, `CC = 0x11`, and `RB = 0x22`. `NO = 0xFF` means unspecified. Treat it as top-left (`LT`).

#### `FontBitmap` / `FontGlyphBitmap`: the bitmap font glyph table

`ssab->font_bitmaps()` holds **only the charmaps that text parts with `text_bitmap == true` actually reference**. In a project with only outline-font parts, `font_bitmaps()` is absent (`nullptr` in C++).

`FontBitmap.name_hash` is the `fnv1a` hash of the `PartTypeText.text_char_map` string as is, so a single hash finds a part's font.

| `FontBitmap` | Contents |
| :--- | :--- |
| `name` / `name_hash` | The charmap name (= `text_char_map`) and its hash |
| `name_face` | Face name of the original font. For display; not used for drawing |
| `base` | Position of the baseline (px, from the top of the line) |
| `height_line` | Height of one line of the charmap (px). **The denominator of the scale** |
| `size_texture_original` | Original size of the page images (px). **Normalize `atlas` with this value** (not with the size of the loaded texture) |
| `name_texture[]` | File names of the page images. They are ordered **by page number**, so you can look one up with `name_texture[glyph.page]` |
| `name_texture_hash[]` | Hashes of the above (the asset keys in embedded mode) |
| `table_data[]` | The glyph table. It is sorted by ascending `id`, so you can binary-search it |

| `FontGlyphBitmap` | Contents |
| :--- | :--- |
| `id` | Character code (a UTF-16 code unit) |
| `atlas` | The glyph's region within the page image (`x`, `y`, `w`, `h`) |
| `offset` | Offset from the pen position to where the glyph starts (px, Y positive downward) |
| `width_advance` | Advance width (px) |
| `text` | `id` as a character. For debug display |
| `page` | Index into `name_texture[]` |

`offset` and `size_texture_original` are the ssab's `Vec2`, whose components are named `v1` / `v2`, not `x` / `y` (`v1` is X or the width, `v2` is Y or the height). `glyph.offset.x` / `glyph.offset.y` in the formulas below mean `offset.v1` / `offset.v2`.

The converter has already resolved the page image names from their paths relative to the charmap file. They are stored as **paths relative to the project**, like other external resources ([Loading and Initialization §3-1](../30_core/10_initialization.md#3-1-file-reference-strings-in-the-ssab)).

> [!IMPORTANT]
> **Fields you must not use**
>
> - `FontBitmap`'s `size` / `padding_l` / `padding_r` / `padding_u` / `padding_d` / `scale` / `spacing` and `FontGlyphBitmap.channel` hold **fixed values (`0` or `1.0`)** and do not reflect what was authored. Do not use them for layout. Take the spacing between characters from `PartTypeText.text_space` and the line height from `height_line`.
> - **Kerning is not in the ssab.** The advance is determined by `width_advance` alone.
> - `FontGlyphBitmap.id` is 16 bits, so **characters above U+FFFF are not supported.** To use emoji or some kanji, use an outline font.

### 3-3. Layout (bitmap fonts)

Compute with **X pointing right and Y pointing down** (the direction of the glyph `offset`), then move the result into the part's local coordinates at the end. Compute in **floating point**.

#### Scale

```
scale = text_size / height_line
```

`text_size` is "how many px tall one line should be", not the actual size of a glyph. Its ratio to the charmap's `height_line` applies to every glyph. A charmap with `height_line == 0` would divide by 0, so exclude it at load time.

#### Offset base

```
base_offset = ( min(glyph.offset.x), min(glyph.offset.y) )   // component-wise minimum over all glyphs in the charmap
```

Subtract this minimum from each glyph's `offset` when placing it. This packs the whole layout so that the glyph reaching furthest up and left in the charmap touches the origin. It is a **constant per charmap, not per string**, so compute it once when you load the font and cache it.

#### Advancing through the characters

Start with the pen at `pen = (0, 0)` and the line number at `line = 0`, and process the string one character at a time from the start, as follows.

```
if the character is '\n':
    pen = (0, pen.y + text_size)      // the line advance is text_size
    line += 1
    continue

if the charmap has no glyph for it:
    do nothing                        // no advance, and no substitute glyph
    continue

pos  = pen + (glyph.offset - base_offset) * scale
size = (glyph.atlas.w, glyph.atlas.h) * scale

// Wrapping (only when text_width > 0)
if text_width > 0 and pos.x + size.x > text_width:
    pen = (0, pen.y + text_size)
    line += 1
    pos  = pen + (glyph.offset - base_offset) * scale

place this glyph at (pos, size) and record its line number line

pen.x += glyph.width_advance * scale + text_space
```

- **The line advance is `text_size`** (the same value as `height_line * scale`).
- **`text_space` is not scaled.** Only the advance width is scaled. The space between characters is added in px as is.
- **Wrapping is per character** and ignores word boundaries. `text_height` affects **neither** wrapping nor vertical clipping (it only affects the box size).
- **Characters not in the charmap are ignored completely.** No substitute glyph is drawn, and the pen does not advance. `\r` usually has no glyph either, so in a CRLF string the `\r` drops out and only the `\n` takes effect.

#### Bounding rectangle and right edge of each line

After placing every glyph, compute `bound` (`left / top / right / bottom`, with `width = right - left` and `height = bottom - top`), which encloses all the glyph rectangles, and **the right edge of each line, `R[line]`**.

#### Box size

```
box.w = (text_width  > 0) ? text_width  : bound.left * 2 + bound.width
box.h = (text_height > 0) ? text_height : bound.top  * 2 + bound.height
```

The automatic size adds twice the top-left offset so that **the content's bounding rectangle sits exactly at the center of the box** (`left + width + left = box.w`).

#### Alignment

```
// Horizontal: per line
shift_x[line] = (box.w - R[line]) * (Left ? 0 : Center ? 0.5 : 1)

// Vertical: shared by all lines
shift_y        = (box.h - bound.bottom) * (Top ? 0 : Center ? 0.5 : 1)
```

Horizontally, only **the line's right edge `R[line]`** matters (every line starts at `x = 0`, so its left edge can be taken as 0). Vertically, the shift is decided once from the bottom of the whole block, and the same amount is added to every line.

#### Into the part's local coordinates

The box `box` is placed **centered on the part's origin**. The final glyph rectangle is as follows.

```
local.x =  (pos.x + shift_x[line]) - box.w / 2
local.y = -((pos.y + shift_y)      - box.h / 2)   // Y-Up (the default). With the coordinate system set to Y-Down, drop the leading -
```

Normalize the UVs by dividing `atlas` by `size_texture_original` (**not** by the size of the loaded texture). Bind the texture `name_texture[glyph.page]` with a linear or nearest filter according to `text_smooth`.

### 3-4. Outline fonts (`text_bitmap == false`)

The ssab holds only `text_family` (the family name) and the layout settings `text_size` / `text_space` / `text_width` / `text_height` / `e_anchor`. **The font file is not in the ssab.** Finding a font from the family name is the host's job, and the result looks different in an environment that does not have the same font.

There is no glyph table, so the steps in §3-3 do not apply. Pass the string and the size to the host engine's text feature, and map `e_anchor` and `text_width` / `text_height` to that feature's alignment and wrapping settings.

---
[Back to contents](../README.md)
