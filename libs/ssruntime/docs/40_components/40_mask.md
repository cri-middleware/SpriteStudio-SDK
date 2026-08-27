> [日本語](./40_mask.ja.md) | **English**

# Implementing Mask

An implementation guide for SpriteStudio's "mask" and "clipping mask" features. Both are realized by the same stencil mechanism, but **they are different features**. This document treats the feature definitions as primary and the implementation patterns as subordinate to them.

---

## 1. Concepts and part roles

### 1-0. Terminology

Let us first sort out the field names and terms that are easy to confuse. See each section for details.

| Term / field | Read from | Meaning |
|---|---|---|
| **Mask (part)** `PartTypeMask` | PartData (static) | A part type dedicated to masking. Always a pure mask (draws no color) |
| **Mask (feature)** | — | Leaves the parts with **lower** draw priority than itself **outside** the shape |
| **Clipping (feature)** | — | Confines the parts with **higher** draw priority than itself **inside** the shape |
| **writer** | — | A part that writes to the stencil (a pure mask or a clipping part) |
| **target** | — | A drawn part affected by masking / clipping |
| **pure mask** | — | A writer that draws no color and becomes a hole (mask parts plus drawing parts turned into masks) |
| `Priority` | **FrameData (per frame)** | Draw priority. Ties are decided by part ID ([§2-1](#2-1-scope-which-drawn-parts-a-mask-reaches)) |
| `mask` (float, `0..255`) | **FrameData (per frame)** | Doubles as the cutout threshold. `0` disables the writer ([§2-2](#2-2-opting-out-of-the-mask-effect) / [§2-5](#2-5-cutout-threshold-which-pixels-are-written-into-the-coverage)) |
| `mask_influence` | **PartData (static)** | **Its meaning changes with the role** ([§1-2](#1-2-part-roles-and-mask_influence)). Either the OP or the mask-target flag |
| `mask_write` | **PartData (static)** | The flag that makes a drawn part a **clipping** writer |
| `draw_as_mask` | **PartData (static)** | The unified flag that turns a drawn part (Shape/Text/Nines) into a **pure mask**. In authoring it is a per-type setting, but in the ssab it is consolidated into one |
| `visible_inside_mask` | **PartData (static)** | The polarity. `1` = draw inside / `0` = draw outside ([§2-4](#2-4-polarity-whether-to-draw-inside-or-outside-the-mask)) |
| `Hide` | **FrameData (per frame)** | Hidden. **A writer has to be checked against it on the Player side** (see the note in [§2-2](#2-2-opting-out-of-the-mask-effect)) |

> **[Important] Distinguishing where values are read from**: the mask-related fields are split across two different tables. Always be conscious of which one you read from.
> - **PartData (static, per part)**: `mask_influence` / `mask_write` / `visible_inside_mask` / `draw_as_mask` / the part type. **Read once; unchanged in later frames.**
> - **FrameData (per frame, animated)**: `mask` (float) / `Priority` (float) / `Hide` / `part_color` / `alpha`. **Re-read every frame.**
>
> The pseudocode below distinguishes the origin with `part.data.*` (PartData) and `part.frame.*` (FrameData).

### 1-1. Mask and clipping are different features

Their purposes are similar and the implementation mechanism is the same, but as features they are different.

| | **Mask** | **Clipping** |
|---|---|---|
| Part type it works on | The **"mask" part** | Drawn parts (normal / mesh / shape and so on) |
| Draws its own color | **No** (pure; it becomes a hole) | **Yes** (draws itself while also writing to the stencil) |
| Applies to | Drawn parts with **lower draw priority** than itself | Drawn parts with **higher draw priority** than itself |
| Typical appearance | Leaves the **outside** of the shape | Confines to the **inside** of the shape |

The relationship is asymmetric: "mask" is a feature that a dedicated part type has on its own, while "clipping" is a feature that drawn parts carry incidentally.

### 1-2. Part roles and `mask_influence`

Each part's role is determined by "is it a writer?" and "is it a target?". Only mask parts have the "mask feature", but some drawn part types can **change into the role of a mask part** (become a pure mask) through the shared flag **`draw_as_mask`**. In authoring this is a per-type setting, but in the ssab it is consolidated into the single `PartData.draw_as_mask`, so Shape/Text/Nines in the table below share the same condition.

| Part type | Default role | Condition to become a pure mask | Can it have clipping (`mask_write`)? |
|---|---|---|---|
| **Mask** (`PartTypeMask`) | Always a pure mask | (always) | — |
| **Normal** | Drawing | (cannot become a pure mask) | Yes |
| **Mesh** | Drawing | (cannot become a pure mask) | Yes |
| **Shape** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |
| **Text** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |
| **Nines** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |

> **The shape special case**: a shape turned into a mask with `draw_as_mask == 1` has no cell (texture), so the cutout threshold described later ([§2-5](#2-5-cutout-threshold-which-pixels-are-written-into-the-coverage)) does not apply. **The whole shape becomes the mask.**

> **The text exception**: the row above describes what authoring expresses, but glyphs are the Player's, so the runtime has nothing to write coverage with. A text part with `draw_as_mask == 1` is therefore **dropped from the draw plan entirely** — it neither writes coverage nor draws. Ordinary text parts (`draw_as_mask == 0`) do get their slot in the plan, and whether they can be clipped as a mask target depends on the Player's own text drawing. See [Limitations](../10_introduction/20_limitations.md).

The Player's writer-classification logic is conceptually as follows:

```
pure_mask   = PartTypeMask || draw_as_mask
is_writer   = pure_mask || mask_write          // every part that writes to the stencil
is_clipping = mask_write && !pure_mask         // also draws color (applies to higher priorities)
```

#### The dual definition of `mask_influence` (important)

`mask_influence` is a single field, but **its meaning changes with the part's role**. Since `mask_influence` appears in several rules in [§2](#2-decision-rules), let us pin its meaning down here first. The Player must use it according to the role.

| Role | Meaning of `mask_influence` |
|---|---|
| **Mask part (writer only)** | The **OP**. `0` = increment / `1` = invert ([§2-3](#2-3-op-the-write-operation-into-the-stencil)) |
| **Drawn part (a target, not a writer)** | The **mask-target flag**. `1` = affected / `0` = opted out ([§2-2](#2-2-opting-out-of-the-mask-effect)) |
| **Clipping (both a writer and a drawn part)** | **Both of the above at once**. The **same value** of `mask_influence` is interpreted as both "the OP" and "the target flag" |

Because clipping is "both a drawn part and a writer", the value of `mask_influence` is used for both roles simultaneously. For example, a clipping part with `mask_influence == 1` has both of these at once:

- **Target flag = 1**: it is itself affected by other masks (opting in to masking)
- **OP = invert**: when it writes as a writer, it writes with the invert OP

This is a deliberate reuse of the same field by design. Do not split the value or discard one of the meanings on the Player side.

---

## 2. Decision rules

Despite the difference in features (mask / clipping), the application logic decomposes into a few simple rules that can be handled uniformly. Note that the meaning of `mask_influence` used in each rule changes with the role ([§1-2](#1-2-part-roles-and-mask_influence)).

### 2-1. Scope (which drawn parts a mask reaches)

It is determined relative to draw priority:

- **Mask** → applies to drawn parts with **lower draw priority** than itself (the mask effect disappears automatically once draw priority exceeds its own)
- **Clipping** → applies to drawn parts with **higher draw priority** than itself (it persists to the highest draw priority, so the effect never disappears automatically)

How draw priority is determined (a common SpriteStudio rule):

- In principle it is decided by **FrameData's Priority**
- On a tie, **the higher part ID (storage index) comes on top** (= higher draw priority)

In implementation terms, letting `rank` be the position in `draw_order` (the sequence of part indices sorted by ascending Priority, with ties broken by ascending ID):

- Mask: `rank < M` (the scope before writer M)
- Clipping: `rank > C` (the scope after writer C)

### 2-2. Opting out of the mask effect

- For the masking side (the writer) to work, **both** of the following conditions must hold.
    - **`mask != 0`** (`mask == 0` → writes nothing = does not operate)
    - **`Hide == false`** (`Hide == true` → writes nothing)

- For a drawn part to be masked, the following must hold.
    - A drawn part is affected by masking / clipping **only when `mask_influence == 1`**. Drawn parts with `mask_influence == 0` are unaffected by masks (opted out).

> **Note (check `Hide` on the Player side too)**: what the Brain drops from `draw_order` / `DrawBatch` is only a **mask part (`PartTypeMask`)** with `Hide == true`. **A drawn part turned into a mask by `draw_as_mask` (Shape / Nines) still gets its mask batch when `Hide == true`** — all that is suppressed is its own color drawing; the coverage is still written.
>
> So decide whether a writer is active from **both `mask != 0` and `Hide == false`** on the Player side. It only double-rejects what the Brain already dropped, so the check is safe on either path.

### 2-3. OP (the write operation into the stencil)

- **Mask parts**: `mask_influence` doubles as the OP. `0` = **increment** (`stencil + 1`) / `1` = **invert** (`~stencil`)
- **Clipping**: `mask_influence` likewise doubles as the OP (the same interpretation as for mask parts). Because clipping parts are also drawn parts, **the same `mask_influence` value is used simultaneously as both "the target flag" ([§2-2](#2-2-opting-out-of-the-mask-effect)) and "the OP"** (see [§1-2](#1-2-part-roles-and-mask_influence) for details)

### 2-4. Polarity (whether to draw inside or outside the mask)

The polarity is **not decided by the feature**; it is decided uniformly by the masked target part's `visible_inside_mask`.

- `visible_inside_mask == 1` → draw **inside** the mask
- `visible_inside_mask == 0` → draw **outside** the mask
- The decision formula: `masked == visible_inside` (draw when true, `discard` when false)

> Note: "mask = outside / clipping = inside" as written in the table of [§1-1](#1-1-mask-and-clipping-are-different-features) is a **typical authoring default**; the polarity itself is a property of the target. **Even when a target part falls into the scope of both a mask and a clipping part at once, the decision is made with one stencil plus the single target's `visible_inside_mask`** (the polarity is not split per feature).

### 2-5. Cutout threshold (which pixels are written into the coverage)

**Common** to mask and clipping, it is determined from FrameData's `mask` value (`0..255`). It is expected to be handled in the shader:

- threshold = `(255 - mask) / 255`
- `mask == 255` → threshold 0 → **every pixel except the fully transparent ones (α=0)** is written into the coverage
- `mask == 0` → threshold 1.0 → requires α > 1.0 (impossible) → **nothing is written** (consistent with [§2-2](#2-2-opting-out-of-the-mask-effect))

**Shapes, which have no texture,** are exempt from the threshold (the whole geometry forms the shape).

### 2-6. Composing mask settings down an instance hierarchy

When the masked target is an **instance part (a sub-animation reference)**, the mask reaches **the whole sub-animation**. The caller's settings (on the instance part) and the callee's (on each part inside the sub-animation) are then **composed** — neither one simply wins.

| Setting | Composition | Meaning |
|---|---|---|
| `visible_inside_mask` | **OR** (accumulates down the hierarchy) | once any level asks for "inside", everything below it draws inside |
| `mask_influence` | **AND** (chains down the hierarchy) | if the caller is not a mask target, nothing below it is masked |

Computing the effective values:

```
effective influence      = caller's effective influence && own mask_influence
effective visible_inside = caller's effective visible_inside || own visible_inside_mask
```

- **The identity elements are `influence = 1` / `visible_inside = 0`.** A top-level animation (called by no instance) starts from those, so each of its parts is evaluated exactly by its own settings.
- Once `mask_influence` reaches `0` through the AND it can never return to `1`, so "not masked" is settled for the whole subtree.
- **Scope** (which writers reach the part, [§2-1](#2-1-scope-which-drawn-parts-a-mask-reaches)) is still decided by **the instance part's own rank**. What composes is the **target decision ([§2-2](#2-2-opting-out-of-the-mask-effect)) and the polarity ([§2-4](#2-4-polarity-whether-to-draw-inside-or-outside-the-mask))**.
- **`mask_write` does not take part in the composition.** It states that the part writes a mask of its own — a local fact, not something a descendant takes on. (It still makes that part itself a target, as in [§2-2](#2-2-opting-out-of-the-mask-effect).)

> **Implementation note**: in a Player that builds children independently, the natural approach is to **hand the composed values to the child immediately before the child builds**. The child can then resolve its own parts' roles against them, and the parent never has to rewrite the child's result afterwards. Because each level passes on what it has already composed, nesting chains with nothing extra to write. A fallback along the lines of "a child that carries its own setting wins, and only the ones without inherit from the caller" is neither the OR nor the AND — it is a third behaviour, and it disagrees with SS7.5 in both directions.

> **Verification status**: the single-level (one caller) OR / AND has been checked against SS7.5. **Multi-level (nested) chaining is a straightforward generalisation of that single-level rule and is unverified.**

---

## 3. Implementation approaches

### 3-1. When a GPU stencil buffer is available (recommended)

SpriteStudio's original masking mechanism is designed around a stencil buffer. If your engine exposes the stencil, using it is the most straightforward option:

> **Note**: modern node-based shaders such as Unity's Shader Graph usually do not support fine-grained stencil output. Depending on your engine's constraints, be aware that you may need to separate out the masking passes into hand-written shaders (HLSL and so on) or a dedicated RenderPass.

1. Walk `draw_order` in order
2. When a writer arrives: update the stencil with the OP (increment / invert) for the pixels that survive the `discard` by the cutout threshold
3. When a target drawn part arrives: decide `discard` from the stencil value and `visible_inside_mask`

#### 3-1-1. The two roles of Writer and Test (assigning stencil state)

Masking splits into two kinds of draws: the **Writer** (which carves the mask's shape into the stencil) and the masked target's **Test** (which reads the carved stencil and decides inside vs outside).

- **Writer**: writes no color (`ColorMask 0`). The stencil is always updated (`Comp = Always`), and the update operation `Pass` switches by the OP ([§2-3](#2-3-op-the-write-operation-into-the-stencil)):
  - Increment mask → `IncrementWrap` / `DecrementWrap` (a **counter**; it counts how many masks overlap)
  - Invert mask (the invert setting of `mask_influence`) → `Invert` (odd/even = the winding scheme; it is its own inverse operation)
  - **These two OPs must never share bits.** They do not commute with each other, so mixing them makes coverage depend on draw order and leaves the cancellation short of zero ([§3-1-4](#3-1-4-give-each-op-its-own-bit-plane-required)). Split the `WriteMask` per OP.
  - `Ref` is unused (this is not Replace, so `0` is fine). The cutout is a fragment `discard` using the threshold from [§2-5](#2-5-cutout-threshold-which-pixels-are-written-into-the-coverage) (when the geometry itself is the mask, as with Shape, make the threshold negative to disable the discard).
- **Test (the masked target)**: adds **only a stencil test** to the ordinary color drawing. The polarity ([§2-4](#2-4-polarity-whether-to-draw-inside-or-outside-the-mask)) is expressed through `Comp`:
  - `visible_inside_mask` (draw inside) → `NotEqual` (passes on pixels where the mask bits are set)
  - Draw outside → `Equal`
  - `Ref = 0`, and the test **does not write** the stencil (`WriteMask = 0`).

In other words, **the polarity (inside/outside) lives on the Test side, not the Writer side**.

#### 3-1-2. Preserving stencil state (before and after drawing)

Naively you would want to "save the stencil before drawing masks and restore it afterwards", but SS's masking is designed to **cancel itself out through the symmetry of the operations**, so no explicit save/restore or clear pass is needed:

- **Pure masks**: increment before every body part (a pre-pass), then **decrement the same shape at its own `draw_order` rank**. This returns the net contribution within the frame to **zero** and naturally produces the scope "a mask affects only the parts with lower priority (later) than itself" ([§2-1](#2-1-scope-which-drawn-parts-a-mask-reaches)). For an invert mask, two `Invert`s cancel likewise.
- **Clipping masks**: write at its own rank, then **after finishing every draw of that object (animation)**, apply the inverse operation with the same shape to cancel it. Because the clip targets (the parts after it) have all been drawn before you revert it, **the appearance within the object is unchanged** while the net contribution returns to **zero** after the object is drawn.
    - **This cancellation cannot be omitted.** The stencil buffer is cleared at camera / frame start, but that happens **only once at the head of the frame**. When several SS objects are drawn within the same frame, **it is not cleared between objects**, so leaving clipping bits behind lets that residue contaminate the mask tests of subsequent objects. (The bit isolation in [§3-1-3](#3-1-3-workaround-when-the-engine-uses-the-stencil-routinely-urp-and-similar) protects you from other users such as URP, but **not from another SS object using the same bit range**.) Cancelling at the end prevents contamination both across frames and between objects.

The key point is **"do not clear or save/restore; close the state within the object's drawing through the symmetry of increment and decrement"** (pure masks at their own rank, clipping at the end of their object's drawing). The cost is only building the geometry once more for the cancellation, and since masks are few, it is negligible.

#### 3-1-3. Workaround when the engine uses the stencil routinely (URP and similar)

Many rendering pipelines, URP included, **use the stencil constantly for their own purposes** (lights, decals, rendering layers, SSAO, the material type in deferred, camera stacking, and so on). Layering SS's masking on top of that naively causes mutual destruction.

The workaround is to **isolate SS masking into a dedicated bit range with `ReadMask` / `WriteMask`**. The ForUnity implementation limits it to the **low 4 bits (`0x0F`)**:

- This matches the **free bits** of URP's user-reserved allocation (bits 0-3 = UserMask / bit 4 = light / bits 5-6 = material type / bit 7 = reserved), so it can coexist with URP's features (the Built-in pipeline uses the same layout).
- The `WriteMask` stays **inside** `0x0F` (and is split further per OP — see [§3-1-4](#3-1-4-give-each-op-its-own-bit-plane-required)): the carry of `IncrementWrap` **does not leak** into bit 4 (URP's light bit) - it is dropped by the mask.
- `ReadMask = 0x0F`: the test looks **only at its own bits**, so it can ignore whatever URP wrote into bits 4-7.
- Because the 4 bits have to be split per OP ([§3-1-4](#3-1-4-give-each-op-its-own-bit-plane-required)), **only 3 are left for the counter = 7 overlapping masks**; the invert side is odd/even and has no limit. Note that **widening the bit range beyond `0x0F` destroys (and is destroyed by) URP's reserved bits**.

> If the pipeline you port to uses the stencil, the standard approach is likewise to **choose a dedicated bit range and fence it off with the read/write masks**. On an engine where you cannot secure bits, or where the stencil is not exposed at all, switch to the coverage-bitmap approach (without a stencil) of [§3-2](#3-2-when-the-engine-does-not-expose-a-stencil).

#### 3-1-4. Give each OP its own bit plane (required)

`Invert` and `IncrementWrap`/`DecrementWrap` are **each their own inverse in isolation, but they do not commute with each other**. While they share bits, two things go wrong wherever writers of differing `mask_influence` overlap:

- **Coverage depends on draw order**: `increment → invert` gives `0 → 1 → 254` (covered), while `invert → increment` gives `0 → 255 → 0` (**not covered**). The same two masks flip their result depending on the order they were written in.
- **The cancellation does not return to zero**: `increment(1) → another invert(254) → decrement(253)`. What is left contaminates the mask test of every part drawn afterwards, breaking the self-cancellation of [§3-1-2](#3-1-2-preserving-stencil-state-before-and-after-drawing).

A stencil write is `new = (OP(old) & WriteMask) | (old & ~WriteMask)`. The OP itself still runs on the whole value, but **restricting the written bits throws the carry / borrow away, so each plane stays closed**. So split the `WriteMask` per OP:

| OP | Plane | Bits needed | Overlap |
|---|---|---|---|
| `Invert` | **parity** | 1 bit | odd/even. **No limit** |
| `IncrementWrap` / `DecrementWrap` | **counter** | the remaining `n` bits | union. Up to `2^n - 1` |

- One bit is enough for parity (`Invert` just flips that bit and never touches the counter).
- The counter must be **contiguous from bit 0** (increment/decrement are ordinary arithmetic).
- **The Test side (target parts) needs no change.** `ReadMask` still covers both planes and the decision is still "value ≠ 0", which is exactly a **union of the two planes** — matching the specification that masks of differing `mask_influence` compose as a union where they overlap.

The ForUnity implementation splits the `0x0F` of [§3-1-3](#3-1-3-workaround-when-the-engine-uses-the-stencil-routinely-urp-and-similar) into **parity `0x08` / counter `0x07`** (so `mask_influence == 0` masks stack up to 7 deep).

> **This rule does not depend on the implementation approach.** The coverage-bitmap approach of [§3-2](#3-2-when-the-engine-does-not-expose-a-stencil) replays the very same operations, so it applies there unchanged. What it splits is not bits but **accumulators**, and with no bit budget to divide the depth limit above does not apply either (see the replay in [§3-2-1](#3-2-1-reference-implementation-pseudocode-for-the-coverage-bitmap-approach)).

### 3-2. When the engine does not expose a stencil

For example, Godot 4.6's 2D canvas does not expose a stencil buffer. On such an engine you can reproduce the stencil with the following hybrid approach:

1. **Build a coverage bitmap**: each frame, rasterize every mask writer of that moment into an offscreen coverage bitmap. Assign one bit to each writer and set that bit on the pixels it covers.
2. **Replay at draw time**: the masked target samples the coverage bitmap inside its drawing shader, replays the active writers in `draw_order` to reconstruct the stencil state, and clips inside/outside with `discard`.

The key point is **"hold no state that depends on draw time; separate coverage (the writers' shapes) from replay (the target's reconstruction)"**. An encoding that handles up to 24 writers in a single coverage texture (8 bits × 3 = 24 bits in the RGB channels of an RGBA8) is practical.

> **On the size of the coverage bitmap**: a practical implementation scales it dynamically to the bbox of all writers, **normalizing the long edge to an upper bound (1024 px, for example)**. That secures pixel-parity precision on a Full HD-class screen while staying within 4 MiB (RGBA8) per texture. When displaying many animations at once, you can reduce it further by tying it to the screen resolution or sharing a small pool (optimization ideas).

#### 3-2-1. Reference implementation (pseudocode for the coverage-bitmap approach)

The overall flow in engine-independent pseudocode.

**Data structures (up to 24 writers; 8 bits × 3 in the RGB channels of an RGBA8)**

```
const int MAX_WRITERS = 24;

struct MaskWriter {
    int  part_index;     // the part index
    int  bit;            // 0..23, the bit position within the coverage
    int  draw_rank;      // the rank in draw_order
    bool op_invert;      // OP: false = increment, true = invert
    bool is_clipping;    // clipping (also draws color)
};
```

**Determining the writer set (every frame)**

```
function build_mask_writers(frame) -> [MaskWriter]:
    writers = []
    for rank, part_index in enumerate(frame.draw_order):
        part = frame.parts[part_index]
        // part.data.*  = PartData (static) / part.frame.* = FrameData (per frame)
        pure_mask  = part.data.type == Mask || part.data.draw_as_mask
        is_writer  = pure_mask || part.data.mask_write
        if not is_writer:               continue
        if part.frame.mask == 0:        continue       // §2-2 opt-out (a per-frame value)
        if len(writers) >= MAX_WRITERS: break

        writers.append(MaskWriter {
            part_index  = part_index,
            bit         = len(writers),                 // 0,1,2,...
            draw_rank   = rank,
            op_invert   = part.data.mask_influence,     // §2-3 OP (shared by pure masks and clipping)
            is_clipping = part.data.mask_write && !pure_mask,
        })
    return writers
```

**Building the coverage bitmap (every frame)**

Draw each writer's geometry into an offscreen RGBA8 viewport with additive blending. So that disjoint bits are OR-ed together, each writer is drawn with a color in which only its own bit is set.

```
function render_coverage_bitmap(writers, frame) -> Bitmap:
    bitmap = create_offscreen_RGBA8_with_additive_blend()
    for w in writers:
        channel   = w.bit / 8                          // 0=R, 1=G, 2=B
        bit_val   = (1 << (w.bit % 8)) / 255.0
        bit_color = vec4_zero_except(channel, bit_val)
        threshold = (255 - frame.parts[w.part_index].frame.mask) / 255.0  // mask is a float

        draw_geometry(
            geometry = geometry_of(w.part_index),       // Normal = sprite / Mesh / Shape
            shader   = MASK_WRITE_SHADER,
            uniforms = { mask_bit_color: bit_color, mask_threshold: threshold }
        )
    return bitmap
```

**The bit-writing shader (GLSL)**

```glsl
uniform vec4  mask_bit_color;   // the bit pre-encoded into the relevant RGB channel
uniform float mask_threshold;   // (255 - mask) / 255

void fragment() {
    // §2-5 cutout: only pixels whose α exceeds the threshold go into the coverage
    if (texture(TEXTURE, UV).a <= mask_threshold) discard;
    // additive blending works as src.rgb * src.a, so alpha = 1 is required
    COLOR = vec4(mask_bit_color.rgb, 1.0);
}
```

**Replay at draw time (the masked target's shader, GLSL)**

```glsl
uniform sampler2D coverage;
uniform int   mask_count;
uniform vec4  mask_meta[MAX_WRITERS]; // (draw_rank, bit, op_invert, is_clipping) - matches MaskWriter from build_mask_writers
uniform float this_rank;
uniform float visible_inside;         // §2-4 polarity (1 = inside / 0 = outside)

bool mask_passes(vec2 cov_uv) {
    if (mask_count == 0) return true;
    // outside the bbox, treat it the same as having no writer (avoiding false masking from edge CLAMP)
    if (cov_uv.x < 0.0 || cov_uv.x > 1.0 || cov_uv.y < 0.0 || cov_uv.y > 1.0)
        return visible_inside <= 0.5;

    vec4 cov = texture(coverage, cov_uv);   // the sampler must be nearest
    // The plane split of §3-1-4. These are ordinary shader variables rather than
    // a real stencil, so there is no bit budget to divide — two accumulators do.
    int parity = 0;   // invert (mask_influence == 1): covered where an odd number overlap
    int count  = 0;   // increment (mask_influence == 0): covered where any overlap
    for (int i = 0; i < mask_count; i++) {
        vec4 m = mask_meta[i];
        bool is_clipping = m.w > 0.5;
        // §2-1 scope decision (clipping = after me / mask = before me)
        bool active = is_clipping ? (this_rank > m.x) : (this_rank < m.x);
        if (!active) continue;
        // extract this writer's bit from the coverage
        int bit  = int(m.y + 0.5);
        int byte = int(cov[bit / 8] * 255.0 + 0.5);
        if (((byte >> (bit % 8)) & 1) == 0) continue;
        // §2-3 OP (§3-1-4: never mix these two into one accumulator)
        if (m.z > 0.5) parity ^= 1;                     // invert
        else           count  += 1;                     // increment
    }
    bool masked = (parity != 0) || (count != 0);        // union of the two planes
    return masked == (visible_inside > 0.5);            // §2-4 decision formula
}

void fragment() {
    // ... ordinary color compositing ...
    if (!mask_passes(coverage_uv)) discard;
    COLOR = ...;
}
```

**Implementation notes**

- **The coverage sampler must be nearest** (with LINEAR, adjacent bits blend and decode incorrectly)
- **Outside the bbox, branch on `visible_inside` alone** (treat it as equivalent to having no writer)
- Compare `mask_meta[i].x` (= the writer's own `draw_rank`) against the target's `this_rank` ([§2-1](#2-1-scope-which-drawn-parts-a-mask-reaches) scope decision)
- **Never mix invert and increment into one accumulator** ([§3-1-4](#3-1-4-give-each-op-its-own-bit-plane-required)). That rule is not specific to the GPU-stencil approach: **the replay reproduces the same operations, so it applies here unchanged**. Folding both into a single `stencil` makes coverage depend on **the order the writers happen to be listed in** wherever writers of differing `mask_influence` overlap (`increment → invert` covers, `invert → increment` does not).
- The replay side, however, has **no bit budget to worry about**. These are shader variables rather than a real stencil, so the "3 bits of counter = 7 overlapping masks" ceiling of [§3-1-3](#3-1-3-workaround-when-the-engine-uses-the-stencil-routinely-urp-and-similar) does not apply; overlap is free up to the writer count (24).

#### 3-2-2. Deriving the coverage UV and inheritance into instances (sub-animations)

**Deriving the coverage UV**: the replay shader's `coverage_uv` is computed in the vertex stage with **an affine transform `M` that maps the masked target's player-local coordinates into the coverage UV `[0,1]`** (the bbox normalization of [§3-2](#3-2-when-the-engine-does-not-expose-a-stencil) = a 2x2 basis plus an offset). Because the coverage bitmap is stretched over the bbox of "the animation that owns the mask writers", `M` is based on that animation's player-local space.

**Inheritance into instances (sub-animations)**: in SpriteStudio, when the masked target is **an instance part (a sub-animation reference)**, the mask reaches **the entire sub-animation**. The coverage and metadata are owned by the parent animation, so hand them to the child and replay as follows.

1. The parent passes **its coverage texture, `mask_meta`, and writer count** to the instance child's Player.
2. Compose the UV transform: `M_child = M_parent · slot_xf` (`slot_xf` = the instance placement matrix from child-local to parent-local). This correctly maps each of the child's vertices' player-local coordinates into **the parent's coverage UV**.
3. Hand over the **composed mask settings** as well ([§2-6](#2-6-composing-mask-settings-down-an-instance-hierarchy)). Each part inside the sub-animation decides its target status and polarity from **the effective values handed down composed with its own settings**. **Use the instance part's own rank for the scope decision** (the replay shader's `this_rank`), because the sub-animation sits at the instance part's position in the parent's draw order.

Key points:

- **The settings compose** ([§2-6](#2-6-composing-mask-settings-down-an-instance-hierarchy)): `visible_inside_mask` with OR, `mask_influence` with AND. Do not decide the sub-animation as one settings-less blob from the caller's values alone - the child parts' own settings take part in the decision too.
- **The composition chains through the hierarchy.** If each level passes on what it has already composed, nesting needs nothing extra (multi-level is unverified - see the verification status in [§2-6](#2-6-composing-mask-settings-down-an-instance-hierarchy)).
- The coverage and metadata (which writer owns which bit) are **owned solely by the parent**. The child only references the parent's coverage and never builds its own (if the child has masks of its own, those are a separate track).
- The parent's mask reaches the child only when the instance part is **a mask target (`mask_influence == 1` or `mask_write`)**; otherwise the AND leaves everything below it unmasked.

#### 3-2-3. Coverage-bitmap size classes and the one-frame delay (handling continuous scaling)

This section covers the pitfalls specific to masks that scale continuously, when implementing the "on the size of the coverage bitmap" advice from [§3-2](#3-2-when-the-engine-does-not-expose-a-stencil) (dynamic scaling to the bbox) **while reusing offscreen targets from a pool**. The basic approach works without this section, but violently zooming a mask can make it drop out for a single frame; here is the countermeasure.

**Size classes (quantizing to powers of two)**

Using the bbox's pixel size directly as the target size every frame means re-allocating the target every frame, which is inefficient. In practice, **round each axis up to a power-of-two "size class"** (… / 256 / 512 / 1024) and borrow a target of that class from the pool. You switch to a different class's target only when the footprint crosses a class boundary (class `C` covers `C/2 < px ≤ C`), so in the normal case of smooth scaling the target is reused and no reallocation occurs.

The sampling-side UV is **independent of the size class** thanks to `M` (the bbox normalization) from [§3-2-2](#3-2-2-deriving-the-coverage-uv-and-inheritance-into-instances-sub-animations) - it always maps the same player-local-space bbox to `[0,1]` - so the target is clipped correctly whichever size class's coverage you attach.

**The one-frame delay and the "one-frame dropout at a size-class transition"**

In many engines, an offscreen target can only be sampled with a **one-frame delay** (the coverage you drew this frame is readable next frame; Godot's Viewport texture, for example). Normally this merely makes the mask lag its content by one frame, which is harmless.

The problem is the **transition frame** where the size class changes. When you borrow a target of the new size, that target **has no valid coverage yet** (undrawn, or a leftover image from another use). Because of the one-frame delay it cannot be read this frame, and left as-is **masking is disabled for this frame only**, so the clip targets are drawn unclipped and the mask flickers out for an instant. With continuous zooming it flickers every time a boundary is crossed.

**Countermeasure 1: sample the old target on the transition frame (a lightweight double buffer)**

At a size-class transition, do not release the old target immediately - **keep it for one frame** and, on that transition frame, **sample the coverage in the old target (drawn on the previous frame)**. Because the UV is bbox-relative and size-class independent, the target maps correctly even with the old target (the previous resolution); this is nothing more than **the same one-frame delay as a normal frame** (at the previous resolution for that one frame). The mask does not drop out. Draw this frame's coverage into the new target and switch to it from the next frame.

**Countermeasure 2: hysteresis on the shrinking side (against boundary chatter)**

If the footprint wobbles near a class boundary, quantization flips the class every frame (512↔1024…) and transitions occur constantly. To prevent this, put **a dead band on the shrink decision**: grow immediately (never lower the resolution), and shrink only once the footprint is **comfortably below the boundary**, keeping the current class until then. The asymmetry exists because being reluctant to grow leaves the coverage under-resolved and the edges rough (visible quality loss), whereas lingering at a larger class only costs a little VRAM (an invisible cost).

> **A guideline for the dead-band width**: this ratio is a tuning knob with no single correct value. The Godot reference Player uses an initial value of `0.15` (stay in the current class until 15% below the boundary). Treat it as an adjustment range: raise it to further reduce transitions from boundary oscillation, lower it if you are bothered by the VRAM of small masks lingering in a target one class too large.

Countermeasure 1 ensures "no dropout even when a transition happens", and countermeasure 2 "reduces pointless transitions in the first place"; they complement each other.

### 3-3. Common notes

- **Pure masks draw no color**: skip the color-drawing batches of mask parts (and of drawn parts turned into pure masks). A `draw_as_mask` drawn part emits **both** its mask batch and its own color batch **at the same rank**.
- **Do not use `mask_indices` as the writer list**: `mask_indices` in the FrameData enumerates the part indices of pure masks (`PartTypeMask` / `draw_as_mask`), but it **excludes clipping (`mask_write`)** and **does not drop mask parts with `Hide == true`** — so it does not agree with `draw_order`. Settle the writers by walking `draw_order` with the three formulas of [§1-2](#1-2-part-roles-and-mask_influence).
- **The upper limit on writers**: the stencil width / the number of bits in the coverage encoding determines the writer limit. In practice it stays within a few to a few dozen per frame.
- **Handling outside the bbox in the coverage approach**: with the coverage-bitmap approach, treating target parts outside the bbox the same as "no writer" (unmasked) avoids false masking caused by CLAMP at the texture edges.

---

## 4. Implementation checklist

For self-checking against omissions. See the corresponding section for the rationale behind each item.

- [ ] **Role classification** ([§1-2](#1-2-part-roles-and-mask_influence)): classified with the three formulas `pure_mask = PartTypeMask || draw_as_mask` / `is_writer = pure_mask || mask_write` / `is_clipping = mask_write && !pure_mask`.
- [ ] **Where values are read from** (the terminology table): `mask_influence` / `mask_write` / `visible_inside_mask` / `draw_as_mask` are read from **PartData (static)**, and `mask` / `Priority` / `Hide` from **FrameData (per frame)**.
- [ ] **Disabling writers** ([§2-2](#2-2-opting-out-of-the-mask-effect)): writers with `mask == 0` **or `Hide == true`** are skipped (exclusion from `draw_order` only covers mask parts, so decide it on the Player side).
- [ ] **Opting out** ([§2-2](#2-2-opting-out-of-the-mask-effect)): a target drawn part is affected only when `mask_influence == 1`.
- [ ] **OP** ([§2-3](#2-3-op-the-write-operation-into-the-stencil)): `mask_influence` is applied as `0` = increment / `1` = invert, shared by pure masks and clipping.
- [ ] **A plane per OP** ([§3-1-4](#3-1-4-give-each-op-its-own-bit-plane-required), **both approaches**): `Invert` and `Incr`/`Decr` are not mixed into the same bits or the same accumulator. With a GPU stencil that means a separate `WriteMask` per OP (the Test side's `ReadMask` still covers both planes); with the coverage bitmap it means two accumulators in the replay, combined as a union ([§3-2](#3-2-when-the-engine-does-not-expose-a-stencil)).
- [ ] **Polarity** ([§2-4](#2-4-polarity-whether-to-draw-inside-or-outside-the-mask)): decided by **the target part's `visible_inside_mask`**, not by the feature, with `masked == visible_inside`.
- [ ] **Scope** ([§2-1](#2-1-scope-which-drawn-parts-a-mask-reaches)): masks are applied to targets with a lower rank than themselves, clipping to targets with a higher rank.
- [ ] **Cutout threshold** ([§2-5](#2-5-cutout-threshold-which-pixels-are-written-into-the-coverage)): the pixels written into the coverage are narrowed with `(255 - mask) / 255` (`mask` is a float). Shapes without a texture are exempt.
- [ ] **Preserving stencil state** ([§3-1-2](#3-1-2-preserving-stencil-state-before-and-after-drawing), the GPU stencil approach): pure masks cancel at their own rank and **clipping cancels at the end of its object's drawing**, applying the inverse operation with the same shape as the write (preventing contamination of other SS objects within the same frame).
- [ ] **Skipping color drawing for pure masks** ([§3-3](#3-3-common-notes)): the color batches of mask parts / parts turned into pure masks are not drawn.
- [ ] **Instance-hierarchy composition** ([§2-6](#2-6-composing-mask-settings-down-an-instance-hierarchy)): each part inside a sub-animation is decided from its own settings composed with the caller's - `visible_inside_mask` with OR, `mask_influence` with AND (`mask_write` stays out of it; the scope uses the instance part's own rank).

---
[Back to Table of Contents](../README.md)
