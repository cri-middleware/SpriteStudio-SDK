> [日本語](./50_parts_color.ja.md) | **English**

# Parts Color

An implementation guide to the color compositing used to customize a part's appearance.

---

## 1. Mechanism and implementation

Colors are composited per vertex, producing tints and gradients.

### 1-0. Distinguish the "two blends" first (important)

What this chapter calls "blending" comes in **two different kinds on different axes**. Confusing them leads to a wrong implementation, so let us separate them up front.

| | **PartColor blend** (the subject of this chapter) | **Framebuffer compositing** |
|---|---|---|
| What is mixed | The texel color and the parts color (vertex color) | The drawing result and the destination (the screen) |
| Where it is computed | **Inside the shader** (the formula in [§1-4](#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation)) | **The GPU's blending stage** (the draw material) |
| Source field | `parts_color`'s **`blend_type`** (`0..3`) | The part's **`alpha_blend_type`** |
| Valid kinds | **Only the 4: Mix / Mul / Add / Sub** | All 12 |

From [§1-1](#1-1-partcolor-blend-modes-blend-mode) to [§1-4](#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation) we deal with the **PartColor blend**. Framebuffer compositing (`alpha_blend_type`) is a draw-material setting handled separately on the [Rendering Loop](../30_core/30_render_loop.md) side - do not confuse the two.

### 1-1. PartColor blend modes (Blend Mode)

This is the scheme that composites the parts color (vertex color) with the texel color **inside the shader**. There are four valid values.

- **Mix (normal)**: linearly interpolates the texel and the part color by `rate`
- **Add**: adds the part color multiplied by `rate` to the texel
- **Sub**: subtracts the part color multiplied by `rate` from the texel
- **Mul**: multiplies the texel by the part color (with strength adjusted by `rate`)

For the concrete compositing formulas, see the reference shader in [§1-4](#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation).

Note:
Because `parts_color`'s `blend_type` field is the `BlendType` enum, a total of 12 values are defined including the four above (Mulalpha / Screen / Exclusion / Invert / Mul2 / Div2 / Screen2 / Overlay2), but **only the four above are valid as PartColor compositing modes**. If a value of `4` or greater arrives, defensively clamp it to `Mix` (`0`).

### 1-2. Obtaining the colors
There is no API such as `ss_runtime_get_parts_color`. Obtain the values directly from the `parts_color` table of `FrameData`.

There are also special rules per drawn part type.
Applying these rules is the Player's responsibility.

The "PartColor blend" column in the table below refers to the **PartColor blend** of [§1-0](#1-0-distinguish-the-two-blends-first-important) (the texel formula, four kinds, shader side); it is on a different axis from `alpha_blend_type` (framebuffer compositing).

| Part type | PartColor blend | Applied color | Handling of rate |
|---|---|---|---|
| Normal | Yes (uses `blend_type`) | Individually at the four corner vertices | Individually at the four corner vertices |
| Mesh | Yes (uses `blend_type`) | Same value for every vertex (whole-only) | Same value for every vertex |
| Shape | Forced to Mix (bypasses the formula) | Individually at the four corner vertices | Always forced to 1.0 |

The reason applying these rules is left to the Player is twofold: to degrade the data in the ssab as little as possible, and to keep shaders from proliferating into per-case variants.

#### 1-2-1. Example for "Normal" parts
```cpp
// Example of applying parts color to a Normal part (reference implementation)
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);

    // The hierarchical alpha is already composited into the a channel.
    // The Player does not need to multiply state->alpha() again.
    ConvertColor(pc->lt()->rgba(), pc->lt()->rate()); // top-left
    ConvertColor(pc->rt()->rgba(), pc->rt()->rate()); // top-right
    ConvertColor(pc->lb()->rgba(), pc->lb()->rate()); // bottom-left
    ConvertColor(pc->rb()->rgba(), pc->rb()->rate()); // bottom-right

    // The color compositing mode (Mix, Mul, Add, Sub)
    ApplyColorBlend(pc->blend_type());
}
```
Note: `ConvertColor` and `ApplyColorBlend` are fictitious pseudo-functions standing for the processing that sets values into your drawing vertex data.

> **About the center vertex (the 5th vertex)**: a Player may draw an ordinary sprite that has vertex deformation or parts color enabled with **five vertices - the four corners plus the center (a triangle fan)**. In that case give the center vertex **the average of the four corners for its color and `rate`** (interpolating position and UV to the center as well). Emitting only the four corners changes the appearance of deformed or tinted sprites.

#### 1-2-2. Example for "Mesh" parts
```cpp
// Example of applying parts color to a Mesh part (reference implementation)
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    auto cl = pc->lt()->rgba();      // meshes are whole-only, so feed lt's value to every vertex
    float rt = pc->lt()->rate();

    // Process every vertex of the mesh
    for (int i = 0; i < vert_count; i++) {
        // The hierarchical alpha is already composited into the a channel.
        // The Player does not need to multiply state->alpha() again.
        ConvertColorMesh(cl, rt);   // the same color & rate (top-left) for every vertex

        // The color compositing mode (Mix, Mul, Add, Sub)
        ApplyColorBlendMesh(pc->blend_type());
    }
}
```
Note: `ConvertColorMesh` and `ApplyColorBlendMesh` are fictitious pseudo-functions standing for the processing that sets values into your drawing vertex data.

> **Meshes are whole-only**: a mesh's parts color has the same value on every vertex (not individual corners). An implementation that copies the same color and `rate` to every vertex works correctly, but since that is redundant you can also optimize it by making it a `uniform` in a mesh-specific shader and passing a single value.

#### 1-2-3. Example for "Shape" parts
```cpp
// Example of applying parts color to a Shape part (reference implementation)
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);

    // The hierarchical alpha is already composited into the a channel.
    // The Player does not need to multiply state->alpha() again.
    ConvertColorShape(pc->lt()->rgba(), 1.0f); // top-left (rate is forced to 1.0)
    ConvertColorShape(pc->rt()->rgba(), 1.0f); // top-right (same)
    ConvertColorShape(pc->lb()->rgba(), 1.0f); // bottom-left (same)
    ConvertColorShape(pc->rb()->rgba(), 1.0f); // bottom-right (same)

    // The color compositing mode (forced to Mix)
    ApplyColorBlendShape(0);	// fixed to Mix
}
```
Note: `ConvertColorShape` and `ApplyColorBlendShape` are fictitious pseudo-functions standing for the processing that sets values into your drawing vertex data.

> **Shapes bypass the compositing formula**: shapes do not go through the PartColor compositing formula ([§1-4](#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation)); the vertex color is used directly as `pc.rgba`. In the shader, the simplest approach is to pass `rate = 1.0` with `Mix` fixed, so that `ss_partcolor_blend(white, pc.rgb, …)` converges to `pc.rgb`. Note that the SS6/SS7 reconciliation such as normalizing `rgba.a` when `blend_type == Mix` is **already applied on the converter side**, so no additional alpha correction is needed on the Player side.

#### 1-2-4. Defaults when there is no parts color (`color_idx < 0`)

Parts without a parts color are still drawn through the same general shader formula ([§1-4](#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation)). We recommend treating the defaults as **vertex color = white (with the hierarchical alpha in α) and `rate = 0`**.
With `rate = 0`, the result of the compositing formula is the **texel passing straight through** (it returns `pixel_rgb` as-is regardless of `blend_idx`), so you can unify the drawing path instead of branching on whether a parts color exists.

### 1-3. Reference: the data contained in a parts color

A parts color contains the following.
- `blend_type`: how each parts-color color is composited with the texel color
- `rgba` (per vertex color): the color to composite. `a` is the parts color's alpha material (multiplied with the part's opacity)
- `rate` (per vertex color): the ratio (compositing strength) when compositing with the texel color

Of these, `rgba`'s `a` is automatically multiplied with the part's opacity inside the Brain's processing (as shown in the implementation examples), so a basic Player implementation need not pay it special attention.

### 1-4. Reference: applying parts color in a shader (table-driven reference implementation)

It is generally sensible to compute parts-color compositing in the (vertex and fragment) shaders.
Below is a reference implementation of the four compositing formulas (Mix / Mul / Add / Sub) **without branching**.

> **On notation**: the reference implementation below is in the **Godot Shading Language**. Map it to other engines as follows.

| Godot identifier | Typical equivalent |
|---|---|
| `CUSTOM0` | An arbitrary 4-component custom attribute on the vertex, `(rate, blend_idx, pma_flag, _)` (an additional vertex stream / `TEXCOORDn` in Unity, an arbitrary semantic in D3D) |
| `COLOR` (vertex stage) | The vertex color attribute (= the PartColor rgba) |
| `TEXTURE` / `UV` | The sampler plus UV coordinates |
| `vertex()` / `fragment()` | The vertex / fragment shader entry points |
| `varying` | The interpolated VS→FS output (GLSL `out`/`in`, HLSL interpolants) |


Outline:
In the vertex shader, build a 4-element `varg` vector (a coefficient table) from `blend_type`; in the fragment shader, pass it to a single general formula and evaluate it.
The fragment shader composites with the texel color using the formula below, without caring about the kind of compositing:
"composited color = ( the pixel's RGB * varg.x ) + mix( pure white RGB, the pixel's RGB, varg.z) * the parts color's RGB * varg.y;"

#### 1-4-1. Vertex shader example (defining the coefficient table constants)
```glsl
const vec4 _PartColorCoef[4] = vec4[4](
    vec4(-1.0,  1.0, 0.0, 0.0),  // [0] Mix
    vec4(-1.0,  1.0, 1.0, 0.0),  // [1] Mul (.z=1 turns the pixel color into the coefficient instead of white)
    vec4( 0.0,  1.0, 0.0, 0.0),  // [2] Add
    vec4( 0.0, -1.0, 0.0, 0.0)   // [3] Sub
);

varying vec4 partcolor_varg;
varying vec4 partcolor_color;
```

`(c.x, c.y, c.z)` are, in order, "the pixel-side weight adjustment", "the PartColor-side weight coefficient", and "the Mul mode flag" (`c.w` is currently unused).

#### 1-4-2. Vertex shader example (assembling `varg`)
The Player packs `(rate, blend_idx, pma_flag, _)` into `CUSTOM0` per vertex (for the meaning of `rate` see [§1-3](#1-3-reference-the-data-contained-in-a-parts-color)).

```glsl
void vertex() {
    int idx = clamp(int(CUSTOM0.y + 0.5), 0, 3);  // guard blend_idx into 0..3
    vec4 c = _PartColorCoef[idx];
    float rate = CUSTOM0.x;                        // colorA
    partcolor_varg = vec4(
        1.0 + c.x * rate,                          // .x: pixel-side weight
        c.y * rate,                                // .y: PartColor-side weight
        c.z,                                       // .z: Mul mode flag
        CUSTOM0.z                                  // .w: pass the pma flag through
    );
    partcolor_color = COLOR;                       // vertex color (= the PartColor rgba)
}
```

#### 1-4-3. Fragment shader example (evaluated with a single general formula)
```glsl
vec3 ss_partcolor_blend(vec3 pixel_rgb, vec3 color_rgb, vec4 varg) {
    return pixel_rgb * varg.x + mix(vec3(1.0), pixel_rgb, varg.z) * color_rgb * varg.y;
}

void fragment() {
    vec4 p = ss_input_texture(TEXTURE, UV);
    vec3 rgb = ss_partcolor_blend(p.rgb, partcolor_color.rgb, partcolor_varg);
    float a  = p.a * partcolor_color.a;            // texture α × vertex α (the hierarchical α is already composited into partcolor_color.a)
    COLOR    = ss_output_color(vec4(rgb, a), partcolor_varg.w);
}
```

> **About `ss_input_texture()` / `ss_output_color()`**: these are hook functions provided as the **single place where color conversion happens** at the input and output stages. The design concentrates texture-input and final-output color processing (un-PMA, PMA, linear/HDR conversion, and so on) in one place; in a minimal implementation that does not deal with PMA, they can simply be a plain `texture(tex, uv)` and a function that returns the color it was given (the PMA flag in `partcolor_varg.w` is passed through).

---

## 2. Implementation checklist

- [ ] **The two blends are not confused** ([§1-0](#1-0-distinguish-the-two-blends-first-important)): the PartColor blend (`blend_type`, the shader formula, four kinds) and `alpha_blend_type` (framebuffer, 12 kinds) are treated as different axes.
- [ ] **Obtaining the index** ([§1-2](#1-2-obtaining-the-colors)): read `part_color()` (from FrameData, every frame); `< 0` means no PartColor.
- [ ] **Defaults when there is no PartColor** ([§1-2-4](#1-2-4-defaults-when-there-is-no-parts-color-color_idx--0)): give vertex color = white (with the hierarchical α) and `rate = 0`, unifying the drawing path instead of branching (with `rate=0` the formula converges to the texel passing straight through).
- [ ] **Per-part-type rules** ([§1-2](#1-2-obtaining-the-colors)): Normal = individual corners / Mesh = the same value on every vertex (whole-only) / Shape = `rate` forced to 1.0 and the formula bypassed.
- [ ] **Clamping `blend_type`** ([§1-1](#1-1-partcolor-blend-modes-blend-mode)): anything outside `0..3` becomes `Mix` (`0`).
- [ ] **Alpha is not applied twice** ([§1-3](#1-3-reference-the-data-contained-in-a-parts-color)): the hierarchical α is **already composited** into `rgba.a`. Do not multiply `state->alpha()` again on the Player side.
- [ ] **The center vertex** ([§1-2-1](#1-2-1-example-for-normal-parts)): when drawing with five vertices (four corners plus the center), the center's color, `rate`, position, and UV were given the average (interpolation) of the four corners.

---
[Back to Table of Contents](../README.md)
