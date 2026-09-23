> [日本語](./10_parts_color.ja.md) | **English**

# Parts Color

Parts color gives a part's vertices colors and blends them with the texture color to tint the part or draw gradients. The Brain passes the colors and the blend type in FrameData, and the Player computes the blend in a shader.

## 1. Tell the two blends apart

Two different things are called a "blend". Confusing them leads to a wrong implementation, so tell them apart first.

| | **Parts color blend** (this page) | **Blend with the render target** |
|---|---|---|
| What is mixed | The texture color and the parts color | The drawn result and the color of the render target (the screen) |
| Where it is computed | **Inside the shader** (§6) | **In the GPU's blend stage** |
| Where the value comes from | **`blend_type`** in FrameData's `parts_color` | **`DrawBatch.blend_type`** (`PartData.blend_type` in the ssab) |
| Valid kinds | **4 kinds: Mix / Mul / Add / Sub** | All 12 kinds |

This page covers the parts color blend.

## 2. Blend types

| Value | Type | Computation |
| :--- | :--- | :--- |
| `0` | Mix (normal) | Linearly interpolates between the texture color and the parts color by `rate` |
| `1` | Mul (multiply) | Multiplies the texture color by the parts color. `rate` sets the strength |
| `2` | Add (additive) | Adds the parts color times `rate` to the texture color |
| `3` | Sub (subtractive) | Subtracts the parts color times `rate` from the texture color |

The type of `blend_type` is `BlendType`, which defines 12 kinds, but only the 4 above are valid for the parts color blend. Treat a value of `4` or greater as `Mix` (`0`).

## 3. Reading the parts color

No C-API returns the parts color. Read it from `parts_color` in FrameData.

- The part's `PartState.part_color` is the index into `parts_color`. `-1` means the part has no parts color (§5).
- A parts color has a blend type, `blend_type`, and a color for each of the four corners (`lt` / `rt` / `lb` / `rb`). Each corner has a color, `rgba`, and a blend strength, `rate`.
- **`rgba.a` already has the part's opacity (including the opacity inherited from its parents) multiplied in.** Do not multiply it by `PartState.alpha` again in the Player.

## 4. Rules per part type

How the parts color is used depends on the part type. The Player applies these rules (so that the ssab data is not degraded and the number of shader variants does not grow).

| Part type | Blend type | Color | `rate` |
|---|---|---|---|
| Normal | As `blend_type` says | Per corner | Per corner |
| Mesh | As `blend_type` says | The same value on every vertex (the `lt` value) | The same value on every vertex |
| Shape | No blend (equivalent to Mix with `rate = 1.0`) | Per corner | Always `1.0` |

### 4-1. Normal parts

```cpp
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    set_corner_color(LT, pc->lt()->rgba(), pc->lt()->rate());
    set_corner_color(RT, pc->rt()->rgba(), pc->rt()->rate());
    set_corner_color(LB, pc->lb()->rgba(), pc->lb()->rate());
    set_corner_color(RB, pc->rb()->rgba(), pc->rb()->rate());
    set_blend(pc->blend_type() <= 3 ? pc->blend_type() : 0);
}
```

`set_corner_color` / `set_blend` are fictitious functions that stand for writing values into the vertices you draw.

**Center vertex**: If you draw a normal part that has vertex deformation or a parts color with 5 vertices, the four corners and the center (a triangle fan), give the center vertex the average of the four corners for its color and `rate`. For its position and UV, use the 5th value of the local vertices and the local UVs. Drawing with the four corners alone changes how the deformation and the tint look.

### 4-2. Mesh

A mesh's parts color has the same value on every vertex. Give the `lt` color and `rate` to every vertex. Copying the same value to every vertex is redundant, so you may instead pass it as a single value (a uniform) in a mesh shader.

### 4-3. Shape

A shape does not blend the parts color. It uses `rgba` as the vertex color as is. With the shader in §6, blend it with a white texture as Mix with `rate = 1.0`, and the result is `rgba`. The converter has already made adjustments such as the one to `rgba.a` when `blend_type == Mix`. The Player does not need to correct the alpha.

The color of a vertex inside a shape is interpolated from the four corner colors by the vertex's position within the shape's bounding rectangle ([9-Slice, Shape, and Text §2](./30_special_parts.md#2-shape)).

## 5. Parts without a parts color

Parts without a parts color (`part_color == -1`) can be drawn with the same shader. Give them **color = white (alpha = the part's opacity) and `rate = 0`**. With `rate = 0`, the formula in §6 returns the texture color unchanged whatever the blend type, so you do not need separate drawing paths for parts with and without a parts color.

Whether a part has a parts color can change from frame to frame (for example, when you set a color override). If you do not split the drawing paths, you do not need to do anything.

## 6. Shader example

It is sensible to compute the parts color blend in a shader. This section shows how to compute the 4 blend types **without branching**.

The vertex shader builds 4 coefficients (`varg`) from `blend_type`, and the fragment shader computes the result with this single formula.

```
blended color = texture color * varg.x + mix(white, texture color, varg.z) * parts color * varg.y
```

The examples are written in the **Godot shading language**. For other engines, read them as follows.

| Godot | General equivalent |
|---|---|
| `CUSTOM0` | An arbitrary 4-component per-vertex attribute. Here it holds `(rate, blend_type, pma_flag, unused)` (an additional vertex stream or `TEXCOORDn` in Unity, an arbitrary semantic in D3D) |
| `COLOR` (vertex shader) | The vertex color (= the parts color's `rgba`) |
| `TEXTURE` / `UV` | The sampler and the UV |
| `vertex()` / `fragment()` | The entry points of the vertex shader / fragment shader |
| `varying` | A value interpolated from the vertex shader to the fragment shader (GLSL `out` / `in`, HLSL interpolators) |

### 6-1. Coefficient table

```glsl
const vec4 _PartColorCoef[4] = vec4[4](
    vec4(-1.0,  1.0, 0.0, 0.0),  // [0] Mix
    vec4(-1.0,  1.0, 1.0, 0.0),  // [1] Mul (.z = 1 multiplies by the texture color instead of white)
    vec4( 0.0,  1.0, 0.0, 0.0),  // [2] Add
    vec4( 0.0, -1.0, 0.0, 0.0)   // [3] Sub
);

varying vec4 partcolor_varg;
varying vec4 partcolor_color;
```

The coefficients `x` / `y` / `z` are, in order, "the adjustment to the texture color's weight", "the parts color's weight", and "the Mul marker". `w` is not used.

### 6-2. Vertex shader

```glsl
void vertex() {
    int idx = clamp(int(CUSTOM0.y + 0.5), 0, 3);   // keep blend_type within 0..3
    vec4 c = _PartColorCoef[idx];
    float rate = CUSTOM0.x;
    partcolor_varg = vec4(
        1.0 + c.x * rate,                            // x: weight of the texture color
        c.y * rate,                                  // y: weight of the parts color
        c.z,                                         // z: Mul marker
        CUSTOM0.z                                    // w: pass the PMA marker through
    );
    partcolor_color = COLOR;                         // rgba of the parts color
}
```

### 6-3. Fragment shader

```glsl
vec3 ss_partcolor_blend(vec3 pixel_rgb, vec3 color_rgb, vec4 varg) {
    return pixel_rgb * varg.x + mix(vec3(1.0), pixel_rgb, varg.z) * color_rgb * varg.y;
}

void fragment() {
    vec4 p = ss_input_texture(TEXTURE, UV);
    vec3 rgb = ss_partcolor_blend(p.rgb, partcolor_color.rgb, partcolor_varg);
    float a  = p.a * partcolor_color.a;              // the parts color's alpha already has the opacity multiplied in
    COLOR    = ss_output_color(vec4(rgb, a), partcolor_varg.w);
}
```

`ss_input_texture()` and `ss_output_color()` are functions that keep in one place the color conversions made when reading the texture and when outputting the color (undoing and applying PMA, converting to linear or HDR, and so on). In a minimal implementation that does not handle PMA, they can be `texture(tex, uv)` and a function that returns the color it receives unchanged.

---
[Back to contents](../README.md)
