> [日本語](./30_render_loop.ja.md) | **English**

# Rendering Loop & Batching

This chapter describes the "rendering contract" for efficiently obtaining the results computed by `libssruntime` (the Brain) and drawing them in your game engine.

## 1. Fast data retrieval via SoA buffers

In a modern Player implementation, the standard approach is not to fetch results part by part, but to **fetch them all at once in SoA (Structure of Arrays) form**.

Using APIs such as `ss_runtime_get_world_matrices(ctx, &ptr, &len)`, you can reference the computed data of every part on the current frame directly through a pointer.

| Data you can obtain | API name | Purpose |
| :--- | :--- | :--- |
| **World matrices** | `ss_runtime_get_world_matrices` | Each part's final position / rotation / scale (4x4 matrix) |
| **Local vertices** | `ss_runtime_get_local_vertices` | Vertex coordinates with pivot flipping and so on already applied (5 vertices x 2 axes) |
| **Texture hashes** | `ss_runtime_get_cell_texture_hashes` | The identifier of the texture on the **cell** side of the part (not what you bind — see §2) |
| **UV / cell meta** | `ss_runtime_get_local_uvs`, `cell_meta` | Animated UV coordinates and cell sizes |

### World matrices can be reflections — do not decompose them naively

A world matrix is a general affine basis, not a position + rotation + scale triple. Its determinant is negative whenever the pose has been mirrored an odd number of times. Two independent sources contribute: a negative scale authored on the part or inherited from an ancestor, and any mirroring the Player folds into its own root transform (a horizontal-flip feature, for example).

**They compose, so they cancel in pairs.** Mirroring the Player does not make the determinant negative — it *toggles* it. Parts authored mirrored are ordinary in real data (the `Ringo` test asset has eight such parts out of sixty-nine), and those are precisely the ones that come back positive the moment the Player flips. Never infer the sign from the flip settings, or from the absence of them; read it off the matrix you are actually holding.

A reflection is not a rotation. Engines that express orientation as a quaternion or as Euler angles cannot represent one, so their "extract the rotation from this matrix" helpers are **undefined** on a negative-determinant matrix — they return meaningless values rather than merely inaccurate ones. Handing the matrix straight to a node transform is safe only where the engine's transform type stores a full affine, meaning one that carries a signed scale and ideally skew.

This matters as soon as the Player exposes part poses to game code — attaching a weapon or a hitbox to a part, say. In that case:

* **Translation is always exact**, whatever the determinant.
* To recover an orientation, make the basis proper first (negate one column) and carry the reflection separately, typically as a negative scale on that axis.
* Which column you negate is a convention, and it decides where the leftover shows up. Put the sign on Y, and a horizontal mirror leaves an extra 180 degrees in the extracted rotation while a vertical one leaves none. Any consumer that takes the rotation but drops the scale inherits that residue, and never mirrors the attached object itself.

That last point is a property of the decomposition, not a defect to fix at this layer, so state the chosen convention and its consequence in the Player's own documentation.

### UV animation and flipping are already applied to the data you obtain

UV animation (moving, rotating, and scaling the texture cut-out region) and part / cell flipping (`flip_h` / `flip_v` / `img_flip_h` / `img_flip_v`) are **all computed on the Brain side and already reflected in `local_vertices` / `local_uvs`**. The Player only needs to pass the SoA buffers above straight through as vertex and texture coordinates; no extra logic to flip coordinates or UVs is required.

## 2. The batching contract

To minimize draw calls, the Brain presents its drawing plan in units of **`DrawBatch`**. The Player runs its loop in a hybrid fashion: "understand the structure through FlatBuffers, pull the data through SoA".

```cpp
// 1. Obtain FrameData (FlatBuffers)
uint8_t* fd_ptr; size_t fd_len;
ss_runtime_get_frame_data(ctx, frame_no, &fd_ptr, &fd_len);
auto frame_data = ss::runtime::GetFrameData(fd_ptr);

// 2. Reference the structural information
auto batches = frame_data->draw_batches();
auto draw_order = frame_data->draw_order();

// 3. Reference the SoA buffers (pointers)
const float *world_matrices, *local_vertices;
size_t m_len, v_len;
ss_runtime_get_world_matrices(ctx, &world_matrices, &m_len);
ss_runtime_get_local_vertices(ctx, &local_vertices, &v_len);

// 4. Drawing loop
for (auto batch : *batches) {
    ApplyMaterial(batch->texture_hash(), batch->blend_type());

    for (int k = 0; k < batch->count(); k++) {
        int p_idx = draw_order->Get(batch->start_rank() + k);

        // Reference the target part's data directly from the SoA buffers
        const float* m = &world_matrices[p_idx * 16];
        const float* v = &local_vertices[p_idx * 10];
        EmitGeometry(m, v);
    }
    FlushDrawCall();
}
```

### `DrawBatch.texture_hash` alone decides what you bind

`texture_hash` is one of the following, so the Player needs a texture map that can resolve **both**:

- **A hash of the cellmap name** (normally) — `ssab->cellmaps()[].name_hash`
- **A hash of a texture file name** — for a part under an authored texture change (`TCHG`). Reverse the hash back to a name through `ssab->external_textures()[].name_hash`.

Whether an override applies is folded into the batch key, so **this loop needs no changes**. Two things to watch, though:

- `ss_runtime_get_cell_texture_hashes` (§1) returns the hash of the **cellmap the cell belongs to** and does not reflect a texture change. Do not use it to decide what to bind; always use `DrawBatch.texture_hash`.
- A texture change or a cell override means **the split points, and the batch count, vary from frame to frame**. Avoid optimizations that assume a fixed batch count.

See [Managing External Resources and Textures](../40_components/70_external_resources.md) for details.

> [!IMPORTANT]
> **This `draw_order` loop is for drawing (geometry emission) only.** Do **not** drive the
> **`step` of Instance / Effect parts (advancing the clock of child animations and particles) from this
> `draw_order` loop.** The Brain excludes `Hide`-ed parts from `draw_order` / `DrawBatch`
> (see the notes in [Implementing Mask](../40_components/40_mask.md)), so "`step` only what appears in
> `draw_order`" freezes the clock of independently-played children and effects while they are hidden.
> **Enumerate every Instance / Effect part from the pack's metadata and `step` them all every frame**,
> and use this `draw_order` (and `step`'s return value `visible`) only when drawing the result. For details
> see [Instance Parts](../40_components/10_instance.md) /
> [Effect Parts](../40_components/20_effect.md).

### Notes on RenderMode

`libssruntime` has the concept of a `RenderMode` matching your engine's rendering philosophy.
The mode you specify with `ss_runtime_set_render_mode` at initialization changes the contents of the `FrameData` (FlatBuffers) you obtain.

- **`RenderMode::Direct` (recommended)**: optimized for the approach of fetching data in bulk through the C-API (SoA buffers) and drawing it directly, as Godot's `RenderingServer` does. In this mode, serializing redundant data (`vertices`, `cells`, `deforms`) into FlatBuffers is skipped, which greatly reduces CPU load and allocations. The sample code above assumes this mode.
- **`RenderMode::SceneGraph`**: for the approach of creating a game-engine node (`Node2D`, `GameObject`, and so on) per part, building a hierarchy (scene graph), and leaving drawing to the engine. In this mode the detailed data such as `vertices` inside the FlatBuffers is fully emitted, so you can read it out and update the nodes' properties.

(If your Player implementation gets `null` when trying to obtain vertex or cell information from `FrameData`, check whether you have set `RenderMode_SceneGraph` with `ss_runtime_set_render_mode`.)

### ⚠️ FlatBuffers per-element accessors are "expensive": read fixed-size struct vectors raw

Attributes not covered by the SoA getters (§1) - for example **`parts` (part state flags, alpha, and so on)** and **`parts_color` (vertex colors)** - are delivered as FlatBuffers vectors inside `FrameData`. **Reading them one element at a time through per-element accessors** such as `frame_data->parts_color()->Get(i)->lt()->rgba()->r()` **is far more expensive than you might expect** (accessor object construction + per-field bounds checks + buffer indirection; the nested `parts_color` is especially heavy). **On high-level hosts such as C# / JS-TS / Python, this accessor cost can become the dominant bottleneck in scenes with many characters.**

> [!WARNING]
> **Measured (Unity / C#, 100 characters):** reading `parts` + `parts_color` through per-element accessors every frame accounted for **about 7 ms/frame** (~72% of the frame-data ingestion cost). Switching to the "raw read" below reduced it to **about 0.27 ms** (~26x).

**Countermeasure - read fixed-size struct vectors directly from the raw pointer:**
`parts` / `parts_color` / `draw_batches` are FlatBuffers **`struct`s (fixed layout, contiguous)**, so you can take the offset of the start of the vector and **read them directly as a raw array of structures with a matching layout** (completely avoiding the per-element accessors).

- **C#**: define a `[StructLayout(LayoutKind.Explicit)]` mirror that is byte-compatible with the generated struct, and read it by casting to `(MirrorStruct*)(fb_ptr + vectorOffset)`.
- **JS/TS (WASM)**: place a `DataView` / typed-array view at the vector's start offset and read fields directly by offset.
- **Python (ctypes and so on)**: decode the vector's byte range in bulk with `struct.iter_unpack` (fixed-size struct vectors) / `struct.unpack_from` (numeric vectors), avoiding per-element accessors. Pure-Python generated accessors are especially expensive (attribute access plus bounds checks add up), so raw-reading `parts` / `parts_color` / `draw_batches` pays off particularly well in scenes with many characters.
- **C++**: FlatBuffers structs are packed and contiguous, so you can treat the start of the vector as an array (accessor overhead is small to begin with, but direct reads are still favorable on the hot path).

> [!CAUTION]
> **A common trap - a "zero-copy ByteBuffer" does not solve this:** pointing the FlatBuffers `ByteBuffer` at native memory ("zero copy", C#'s `ENABLE_SPAN_T` and so on) **does not help (and can be slower)**. The cost is not memcpy but **the per-element accessors themselves**. What you must eliminate is the accessor, not the copy.

**Maintenance note:** a raw-read mirror must match the FlatBuffers struct layout exactly (alignment / padding / endianness = little endian). To prevent silent drift when the `.fbs` is regenerated, we recommend adding a self-check at startup that **compares "reads through the generated accessors" against "reads through the mirror" and asserts that every field matches** (a size check alone misses reorderings of the same size, so a roundtrip comparison is the reliable option).

## 3. Techniques for optimizing rendering performance

In recent game engines and graphics libraries, performance improves dramatically when you drive the engine's low-level drawing API (batch rendering features and the like) directly, instead of creating large numbers of display nodes or sprite objects per part.

1. **Fetch and hold pointers in bulk (the DrawFrame pattern)**:
   Fetch pointers such as `ss_runtime_get_world_matrices` in bulk at the top of each frame's drawing loop and keep them in a temporary structure or context, which cuts unnecessary function calls and allocations inside the loop.

   **Concrete examples of wrapping pointers (zero-copy references) per language environment**:
   When calling over FFI from a high-level language such as C# or JS (WASM), you can completely avoid a per-frame memory copy by using the language feature that peeks directly into memory (borrowing), rather than allocating a new managed array from the returned pointer and copying the data.

   * **C#**: wrap it with a raw pointer inside an `unsafe` block, or with `ReadOnlySpan<float>` and similar.
   * **JS/TS (WASM)**: place a `Float32Array` view (a window) over `wasmMemory.buffer`.

   ```typescript
   // Example: the pattern that avoids allocation and copying in a JS/TS environment
   const ptr = context.get_world_matrices_buf();
   const count = context.get_parts_count();

   // No array copy: reference (borrow) the WASM memory space directly
   const worldMatrices = new Float32Array(wasmMemory.buffer, ptr, count * 16);
   ```

   > [!WARNING]
   > **The lifecycle trap in multi-threaded renderers (avoiding use-after-free)**
   > In modern engines - Unity's Job System (Burst), Godot's multi-threaded RenderingServer, Unreal Engine's RenderThread - it is common for "animation updates on the main thread" and "drawing / geometry construction on a worker thread" to run asynchronously.
   > If you design your Player to hand SoA buffers (raw pointers obtained from the C-API) to the rendering thread with zero copies, then a situation where **"the rendering thread is reading the pointer while the main thread runs the Player's disposal path and calls `ss_runtime_destroy`"** causes an access to freed memory (segfault / access violation) and a guaranteed crash.
   > To avoid this, implement strict lifecycle management: at Player (component) disposal or destructor time (`Destroy` / `Dispose`), **always wait for running drawing jobs and thread work to complete (synchronizing with Unity's `JobHandle.Complete()` or similar) before freeing native memory with `ss_runtime_destroy`**.

2. **Pool drawing resources**:
   Do not create and destroy the low-level drawing resources each batch needs (vertex buffers, index buffers, intermediate objects for draw commands) every frame; pool them in arrays and reuse them.
3. **Zero-allocation design (avoiding GC spikes)**:
   Allocating new managed arrays (`new`) inside the per-frame drawing loop, or delegating work to high-level UI components (Unity's uGUI Canvas Rebuild, for example), causes serious performance degradation and GC spikes. We strongly recommend a zero-allocation design that reuses pre-allocated native memory (`NativeArray` and so on) and updates vertex buffers directly (`SetVertexBufferData` and so on).
4. **Aggregate vertex data**:
   Pack each part's computed vertices into the buffer format your target engine accepts directly, and submit them to the GPU in bulk with a draw command.
5. **Control the draw order**:
   Use Z-index or your engine's own draw-order control to reflect the order between batches (`draw_order`) correctly.

## 4. Implementing masks

The Player hooks the mask batches to start and update the engine's stencil buffer or dedicated masking feature. This section only lays out what the FrameData gives you; [Mask implementation](../40_components/40_mask.md) is the reference for the semantics, the rules, and the approaches.

**Parts that emit a `DrawBatchKind::Mask`** (both with `count = 1` and `vertex_count` / `index_count` at `0`):

- **A mask part (`PartTypeMask`)**
- **A drawn part turned into a mask by `draw_as_mask` (Shape / Nines)** — the mask batch and its own color batch are **both emitted at the same rank**. A pure mask draws no color, so the Player has to skip the color one ([Mask implementation §3-3](../40_components/40_mask.md#3-3-common-notes))

> [!IMPORTANT]
> **Clipping (`mask_write`) emits no `DrawBatchKind::Mask`.** Clipping only ever appears as an ordinary drawing batch, so a Player that hooks on this batch kind alone **misses clipping entirely**. Settle the writers on the Player side by reading the PartData ([Mask implementation §1-2](../40_components/40_mask.md#1-2-part-roles-and-mask_influence)).

**`mask_indices`**: the FrameData's list of pure-mask part indices. **Do not use it as the writer list** — it excludes clipping and does not drop mask parts with `Hide == true`, so it does not agree with `draw_order`.

---
[Back to Table of Contents](../README.md)
