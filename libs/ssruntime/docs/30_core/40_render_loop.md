> [日本語](./40_render_loop.ja.md) | **English**

# Render Loop

This page covers how to draw a frame computed with `ss_runtime_get_frame_data`. The Brain decides the draw order and the groups of parts that can be drawn together (DrawBatch), and passes them to the Player. The Player reads the drawing plan from the FlatBuffers FrameData, and reads numeric values such as vertices directly from the SoA buffers through pointers.

Each FrameData field is defined in the [FrameData schema](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/SCHEMA.md).

## 1. SoA buffers

Calling `ss_runtime_get_frame_data` writes the values for every part of that frame into the SoA buffers (§3 lists the ones `SceneGraph` mode leaves unwritten). The Player receives a pointer and a length, and reads directly by part index. A buffer stays valid until the next call to `ss_runtime_get_frame_data`.

| Buffer | Function | Per part | Contents |
| :--- | :--- | :--- | :--- |
| World matrices | `ss_runtime_get_world_matrices` | 16 floats | A 4x4 matrix combining the parent-child transforms |
| Local vertices | `ss_runtime_get_local_vertices` | 10 floats | `(x, y)` of 5 vertices: top-left, top-right, bottom-left, bottom-right and center. The origin, flips, vertex deformation and coordinate-system settings are already applied. Ready to use by just multiplying by the world matrix |
| Local UVs | `ss_runtime_get_local_uvs` | 10 floats | The UVs for the 5 vertices above. They are **pixel coordinates on the cellmap texture**, so normalize them by dividing by the texture size |
| Cell info | `ss_runtime_get_cell_meta` | 6 floats | `[origin x, origin y, width, height, left, top]`. Left and top are the position on the cellmap texture |

The values of a part with no cell are 0. Mesh, shape and 9-slice parts pass their vertices through dedicated buffers ([Mesh](../40_parts/20_mesh.md), [9-Slice, Shape, and Text](../40_parts/30_special_parts.md)).

UV animation (moving, rotating and scaling the cut-out region) and part and cell flips (`flip_h` / `flip_v` / `img_flip_h` / `img_flip_v`) are already reflected in the local vertices and local UVs. The Player does not need to flip coordinates or UVs.

### 1-1. A world matrix can be a mirror image

A world matrix is a general affine transform, and cannot always be decomposed into the three parts "position, rotation, scale". Its determinant is negative when the pose has been mirrored an **odd number of times**. There are two sources of mirroring: **a negative scale on the part itself or on an ancestor**, and **a flip the Player adds to the root transform** (a horizontal-flip feature, for example).

**The two multiply, and an even number of them cancel out.** A flip by the Player does not necessarily make the determinant negative. A flip only **toggles** the sign. In real data, parts that are already mirrored are not rare (8 of the 69 parts in the `Ringo` test data). Those parts **turn positive again** when the Player flips. Do not infer the sign from the flip settings. Read it from the matrix you hold.

A mirror image is not a rotation. Engines that represent orientation as quaternions or Euler angles cannot represent a mirror image, so their "extract the rotation from a matrix" functions return **meaningless values** when the determinant is negative. You may pass the matrix straight to an engine node's transform only if the engine's transform can hold an affine transform that includes a signed scale. Note that this matrix **contains no shear (skew)**. SpriteStudio applies shear to the local vertices as a per-part vertex deformation, and it is not passed on to children.

This matters when the Player exposes part poses to game code (to attach a weapon or a hitbox to a part, for example). In that case, do the following.

* **Translation is always exact, whatever the determinant.**
* To extract a pose, first make the orientation of the basis positive (negate the sign of one column), and hold the mirror image separately (usually as a negative scale on that axis).
* **Which column you negate is a convention, and it decides where the error shows up.** With the convention that puts the sign on Y, a horizontal mirror leaves an extra 180 degrees in the extracted rotation, and a vertical mirror does not. A consumer that takes only the rotation and drops the scale receives this difference as is, and the attached object itself is not mirrored.

The last point is a property of how the decomposition works, not a defect to fix. Write the convention you adopted, and its consequences, in the Player's documentation.

## 2. Drawing with DrawBatch

The Brain outputs the parts in draw order (`draw_order`), and a list of DrawBatches (`draw_batches`) that cuts that order into runs that can be drawn in a row with the same texture, the same blend mode and the same kind. The Player switches the drawing state only once per DrawBatch, and can draw it with one draw call. Consecutive DrawBatches can also be drawn together in one draw call (§4-3).

```cpp
// 1. Have the Brain compute FrameData
const uint8_t* fd_ptr = nullptr; uintptr_t fd_len = 0;
if (!ss_runtime_get_frame_data(ctx, frame_no, &fd_ptr, &fd_len)) return;   // Playback Control §6-1
auto frame_data = ss::runtime::GetFrameData(fd_ptr);

// 2. Receive the SoA buffers
const float *world_matrices, *local_vertices;
uintptr_t m_len, v_len;
ss_runtime_get_world_matrices(ctx, &world_matrices, &m_len);
ss_runtime_get_local_vertices(ctx, &local_vertices, &v_len);

// 3. Draw each DrawBatch
auto draw_order = frame_data->draw_order();
for (auto batch : *frame_data->draw_batches()) {
    apply_material(batch->texture_hash(), batch->blend_type());

    for (int k = 0; k < batch->count(); k++) {
        int p = draw_order->Get(batch->start_rank() + k);
        emit_geometry(&world_matrices[p * 16], &local_vertices[p * 10]);
    }
    flush_draw_call();
}
```

`apply_material` / `emit_geometry` / `flush_draw_call` are placeholders for the engine's drawing and do not exist. This example handles only `Normal` DrawBatches. How to get the vertices for each kind is in §2-2.

The values in `draw_order` are indices into `FrameData.parts`, and the SoA buffers are indexed the same way. To look up a part definition in the ssab, use `FrameData.parts[p].part_index`.

### 2-1. The texture to bind

**`DrawBatch.texture_hash` alone decides the texture to bind.** The value is the hash of the cellmap name, or, for a part with an authored texture change (`TCHG`), the hash of the texture file name. How to build a map that can look up both is in [Loading and Initialization §3](./10_initialization.md#3-loading-textures).

- The cellmap-name hash in `FrameData.cells` is that of **the cellmap the cell belongs to**, and does not reflect texture changes. Do not use it for binding.
- Texture changes and cell overrides mean that **how DrawBatches are split, and how many there are, changes from frame to frame.** Do not optimize on the assumption that the number of DrawBatches is constant.

### 2-2. DrawBatch kinds

`DrawBatch.kind` decides where the vertices of the parts in that run come from.

| `kind` | Where the vertices come from | Page that explains it |
| :--- | :--- | :--- |
| `Normal` | Local vertices (§1) and world matrices | This page, [Parts Color](../40_parts/10_parts_color.md) |
| `Mesh` | The mesh SoA buffer | [Mesh](../40_parts/20_mesh.md) |
| `Shape` | The shape SoA buffer | [9-Slice, Shape, and Text §2](../40_parts/30_special_parts.md#2-shape) |
| `Nines` | The 9-slice SoA buffer | [9-Slice, Shape, and Text §1](../40_parts/30_special_parts.md#1-9-slice) |
| `Text` | No vertices. The Player builds the glyphs from the ssab | [9-Slice, Shape, and Text §3](../40_parts/30_special_parts.md#3-text) |
| `Mask` | Writes only to the stencil (or coverage). Draws no color | [Mask](../40_parts/40_mask.md) |
| `Instance` | Draws a child animation at that position | [Instance Parts](../40_parts/50_instance.md) |
| `Effect` | Draws the effect's particles at that position | [Effect Parts](../40_parts/60_effect.md) |

Bit `0x01` of `DrawBatch.flags` means that the parts in that run have `mask_influence` set to `1`. The other bits are reserved; ignore them.

### 2-3. Do not advance children's clocks with `draw_order`

`draw_order` is a list **for drawing only**. Hidden parts are not in `draw_order` or in the DrawBatches (the one exception is the mask DrawBatch of a drawing part made into a mask with `draw_as_mask`; see [Mask §3](../40_parts/40_mask.md#3-what-to-read-from-framedata)). If you advance the clocks of the children of instance parts and effect parts (the slot's `step`) while walking `draw_order`, the children do not advance while hidden, and the clock of an independent-playback child stops.

**Enumerate every instance part and effect part from the ssab, and advance all of their children's clocks every frame without exception.** Use `draw_order` and the `visible` returned by `step` only to decide whether to draw and in what order ([Instance Parts §2](../40_parts/50_instance.md#2-advancing-the-child-with-the-slot)).

## 3. RenderMode

`ss_runtime_set_render_mode` selects what FrameData carries.

| Mode | Value | Contents |
| :--- | :--- | :--- |
| **`Direct`** (default, recommended) | `0` | Numeric values such as vertices are passed only through the SoA buffers. FrameData does not carry `cells` / `vertices` / `deforms`, so there is less computation and allocation. The examples on this page assume this mode |
| **`SceneGraph`** | `1` | For Players that create an engine node per part. FrameData carries all of `cells` / `vertices` / `deforms`. World matrices are not computed, so the values of `ss_runtime_get_world_matrices` cannot be used. Mesh vertices are not computed either, and the mesh SoA buffer is empty |

`parts_color` is in FrameData in both modes. If `cells` or `vertices` in FrameData is `null`, check whether you are reading in `Direct` mode.

## 4. Performance

Using the engine's low-level drawing API directly draws much faster than creating an engine display node or sprite per part.

1. **Receive pointers once per frame.** Receive and keep the SoA buffer pointers before the drawing loop, and do not call the functions again inside the loop.
2. **Read without copying.** Even in a high-level language such as C# or JavaScript, do not build an array from the returned pointer and copy it; read the memory directly with the language's features. In C#, use an `unsafe` pointer or `ReadOnlySpan<float>`; in WebAssembly, create a `Float32Array` view on `memory.buffer`.

    ```typescript
    const worldMatrices = new Float32Array(
        memory.buffer, ctx.get_world_matrices_buf(), ctx.get_world_matrices_len());
    ```

    When the WebAssembly memory grows, views become invalid. Re-create a view right before you use it.

3. **Reuse drawing resources.** Do not create and discard vertex buffers, index buffers and intermediate objects for draw calls every frame; pool and reuse them.
4. **Do not allocate memory every frame.** `new`-ing managed arrays inside the drawing loop, or leaving the work to heavy UI machinery (such as rebuilding a Unity uGUI Canvas), lowers performance and causes GC pauses. Reuse pre-allocated native memory (`NativeArray` and so on), and update vertex buffers directly (`SetVertexBufferData` and so on).

The Brain keeps to the same rule once an animation has played through twice. The first playback grows the buffers: `ss_runtime_setup_animation` sizes what the animation's parts decide, and the rest (the FrameData, the DrawBatches, mesh deformation, effect particles) grows to the largest frame the playback meets. The FrameData buffer is then re-sized to fit that frame on the next call, which can fall at the start of the second playback. From then on, `ss_runtime_update`, `ss_runtime_get_frame_data`, `ss_effect_slot_step` and `ss_instance_slot_step` allocate no memory. Anything the instance has not done before starts this over: a [mix](../50_features/10_animation_mixing.md), a [part override](../50_features/20_part_overrides.md), an `ss_runtime_update` that passes more frames at once than any before, an effect with more particles alive than ever before. Where a scene must not allocate during play, play its animations through twice beforehand, while loading for example.

### 4-1. Multi-threaded drawing and destruction

In engines that update animation on one thread and build vertices and draw on another, such as Unity's Job System, Godot's multi-threaded RenderingServer and Unreal Engine's RenderThread, the SoA buffer pointers may be passed straight to the rendering thread.

**If you call `ss_runtime_destroy` while the rendering thread is reading a pointer, it reads freed memory and crashes.** When you destroy the Player (`Destroy` / `Dispose` / destructor), wait for running drawing jobs and threads to finish (Unity's `JobHandle.Complete()` and so on), then call `ss_runtime_destroy`.

### 4-2. Reading FrameData vectors directly

Values not in the SoA buffers, such as **`parts`** (part state) and **`parts_color`** (vertex colors), are passed as FlatBuffers vectors in FrameData. Reading them with per-element accessors, as in `frame_data->parts_color()->Get(i)->lt().rgba().r()`, is heavier than you would expect, because it builds accessor objects, checks bounds on every field and dereferences the buffer indirectly. **In high-level languages such as C#, JavaScript and Python, this can become the heaviest work in scenes with many parts.**

> [!WARNING]
> **Measured (Unity / C#, 100 characters):** reading `parts` and `parts_color` with per-element accessors every frame took **about 7ms / frame** (about 72% of the work of ingesting FrameData). Switching to the method below brought it down to **about 0.27ms** (about 26x).

`parts` / `parts_color` / `draw_batches` are FlatBuffers **`struct`s** (a fixed layout, laid out contiguously), so you can take the position of the start of the vector and **read it directly as an array of structures with the same layout.**

- **C#**: define a `[StructLayout(LayoutKind.Explicit)]` structure whose layout matches the generated struct byte for byte, cast to `(MirrorStruct*)(fb_ptr + vectorOffset)`, and read it.
- **JavaScript / TypeScript**: create a `DataView` or a typed-array view at the start of the vector, and read the fields directly by position.
- **Python (ctypes and so on)**: read the bytes from the start of the vector in bulk with `struct.iter_unpack` (vectors of structs) or `struct.unpack_from` (numeric vectors). The accessors generated for Python are especially heavy, so the gain is large.
- **C++**: FlatBuffers structs are packed contiguously, so you can treat the start of the vector as an array. The accessor cost is small to begin with, but reading directly is faster on frequently used paths.

> [!CAUTION]
> **"Zero copy", pointing the FlatBuffers `ByteBuffer` at native memory, does not solve this (it is slower instead).** What is heavy is not the copy but the per-element accessors themselves.

A structure for direct reading must match the FlatBuffers struct layout exactly (alignment, padding, little endian). So that you notice if the layout drifts when you regenerate from the `.fbs`, we recommend adding a check that **compares, at startup, the values read through the generated accessors with the values read directly, for every field**. Comparing `sizeof` alone misses fields that swapped order while the size stayed the same.

### 4-3. Drawing consecutive DrawBatches as one

DrawBatches are cut where the kind (`kind`) changes, even when the texture and the blend mode stay the same. `kind` decides only where the vertices come from (§2-2), so in a character that mixes `Normal` and `Mesh` parts, vertices that could be drawn in one call arrive split over several DrawBatches.

**When consecutive DrawBatches share the same texture and blend mode, we recommend joining their vertices into one buffer and drawing them in one draw call.**

> [!NOTE]
> **Measured (Web Player, WebGL):** over 191 animations of the SDK tests and the SpriteStudio samples, the total number of draw calls fell by about 20%. Ringo from the SDK tests went from 8 DrawBatches to one draw call.

- Join **only consecutive DrawBatches**. Drawing in a different order changes how overlapping parts blend.
- When something that changes the drawing state comes in between, draw what you have joined so far. Examples are the child animation of an instance part and parts with the shader attribute. Parts tested against a mask can be joined only while the stencil (or coverage) test settings stay the same.
- Effect particles can be joined across consecutive `Effect` DrawBatches ([Effect Parts §4](../40_parts/60_effect.md#4-batching-commands)).
- A Player that carries clipping out of an instance also has to follow [Mask §4-2](../40_parts/40_mask.md#4-2-restoring-the-stencil): do not join draws across the gap the deferred closing draws leave.

---
[Back to contents](../README.md)
