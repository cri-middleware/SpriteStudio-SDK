> [日本語](./40_mask.ja.md) | **English**

# Masks

How to implement SpriteStudio's "mask" and "clipping". Both are built on the same mechanism, the stencil, but **they are different features**. The Brain provides the draw order and the mask DrawBatches. The Player decides which parts each mask affects, and writes to the stencil (or the coverage bitmap).

- §1–§2 are the decision rules, independent of the engine.
- §3 is what to read from FrameData.
- §4 is an implementation with the GPU stencil, and §5 is an implementation for engines that cannot use a stencil. Choose one of them.

---

## 1. Concepts and part roles

### 1-1. Terminology

| Term / field | Read from | Meaning |
|---|---|---|
| **Mask part** `PartTypeMask` | PartData (static) | A part type dedicated to masking. Always a pure mask (draws no color) |
| **Mask** (feature) | — | Leaves the parts with **lower** draw priority than itself **outside** the shape |
| **Clipping** (feature) | — | Confines the parts with **higher** draw priority than itself **inside** the shape |
| **writer** | — | A part that writes to the stencil (a pure mask or clipping) |
| **target** | — | A drawn part affected by a mask or clipping |
| **pure mask** | — | A writer that draws no color and writes only its shape (mask parts, and drawn parts turned into masks) |
| `Priority` | **FrameData (per frame)** | Draw priority. Ties are decided by part ID (§2-1) |
| `mask` (float, `0..255`) | **FrameData (per frame)** | Doubles as the cutout threshold. `0` disables the writer (§2-2 / §2-5) |
| `mask_influence` | **PartData (static)** | **Its meaning changes with the role** (§1-3). Either the write op or the flag for receiving masks |
| `mask_write` | **PartData (static)** | The flag that makes a drawn part a **clipping** writer |
| `draw_as_mask` | **PartData (static)** | The flag that makes a drawn part (shape / text / 9-slice) a **pure mask**. In authoring it is a per-type setting, but in ssab it is a single field |
| `visible_inside_mask` | **PartData (static)** | Polarity. `1` = draw inside / `0` = draw outside (§2-4) |
| `Hide` | **FrameData (per frame)** | Hidden. **For writers, the Player checks it too** (§3) |

The mask fields are split across two tables. Do not mix up which one you read from.

- **PartData (the part definitions in ssab)**: `mask_influence` / `mask_write` / `visible_inside_mask` / `draw_as_mask` / the part type. **Once read, they do not change.**
- **FrameData (per frame)**: `mask` / `Priority` / `Hide` / parts color / opacity. **Read them again every frame.**

The pseudo-code below marks the source as `part.data.*` (PartData) or `part.frame.*` (FrameData).

### 1-2. Mask and clipping are different features

Their uses are similar and the mechanism is the same, but they are different features.

| | **Mask** | **Clipping** |
|---|---|---|
| Part that has it | **Mask part** | Drawn parts (normal / mesh / shape, and so on) |
| Its own color | **Not drawn** (writes only its shape) | **Drawn** (writes to the stencil while drawing itself) |
| Affects | Drawn parts with **lower draw priority** than itself | Drawn parts with **higher draw priority** than itself |
| Typical look | Leaves the **outside** of the shape | Confines to the **inside** of the shape |

The relationship is asymmetric: a mask is a feature of a dedicated part, while clipping is a feature that drawn parts carry in addition.

### 1-3. Part roles and `mask_influence`

A part's role is decided by whether it is a writer and whether it is a target. Only mask parts have the mask feature, but some drawn parts can take **the same role as a mask part** (a pure mask) through `draw_as_mask`.

| Part type | Default role | Condition to become a pure mask | Can it have clipping (`mask_write`)? |
|---|---|---|---|
| **Mask** (`PartTypeMask`) | Always a pure mask | (always) | — |
| **Normal** | Drawing | (cannot) | Yes |
| **Mesh** | Drawing | (cannot) | Yes |
| **Shape** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |
| **Text** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |
| **9-slice** | Drawing | `draw_as_mask == 1` | Yes (when `draw_as_mask == 0`) |

- **Shape**: a shape turned into a mask has no texture, so the cutout threshold (§2-5) does not apply, and **the whole shape becomes the mask**.
- **Text**: the Player draws the glyphs, so the Brain has nothing to write the mask shape with. A text part with `draw_as_mask == 1` is **excluded from the draw plan**, and works neither as a mask nor as a drawing. Whether an ordinary text part (`draw_as_mask == 0`) is clipped as a mask target depends on how the Player draws text.

In the Player, the roles are decided by these three expressions.

```
pure_mask   = PartTypeMask || draw_as_mask
is_writer   = pure_mask || mask_write          // every part that writes to the stencil
is_clipping = mask_write && !pure_mask         // also draws color (affects the higher draw priority side)
```

#### `mask_influence` changes meaning with the role

`mask_influence` is one field, but **its meaning changes with the part's role**.

| Role | Meaning of `mask_influence` |
|---|---|
| **Mask part** (writer only) | **Write op**. `0` = increment / `1` = invert (§2-3) |
| **Drawn part** (a target that is not a writer) | **Flag for receiving masks**. `1` = receives / `0` = does not receive (§2-2) |
| **Clipping** (both a writer and a drawn part) | **Both of the above at once**. The same value is used both as the write op and as the flag for receiving masks |

For example, clipping with `mask_influence == 1` has both of these at once:

- **Receive flag = 1**: it is also affected by other masks
- **Write op = invert**: when it writes to the stencil, it writes with invert

This double use of one field is intended by the specification. Do not split the value or drop one of its meanings in the Player.

---

## 2. Decision rules

Masks and clipping are different features, but both can be handled uniformly with the rules below.

### 2-1. Scope (which drawn parts a mask reaches)

It is decided by draw priority.

- **Mask** → affects drawn parts with **lower draw priority** than itself. It does not affect parts whose draw priority has become higher than its own
- **Clipping** → affects drawn parts with **higher draw priority** than itself. It lasts until the end of the drawing; one written inside an instance keeps going past the instance, to the end of the whole drawing (§2-7)

Draw priority is, as a rule, decided by FrameData's `Priority`. On a tie, the part with the larger ID (the index in storage order) is on top (has the higher draw priority).

In the implementation, decide by rank, the position in `draw_order` (the list of parts in ascending draw priority). With the writer's rank as `M` (mask) / `C` (clipping):

- Mask: affects parts with `rank < M`
- Clipping: affects parts with `rank > C`

### 2-2. Conditions that disable a mask

A writer works only when **both** of the following hold.

- **`mask != 0`** (if `mask == 0`, it writes nothing)
- **`Hide == false`** (if `Hide == true`, it writes nothing)

A drawn part is affected by masks **only when `mask_influence == 1`**. A drawn part with `mask_influence == 0` receives neither masks nor clipping.

**`mask_write` plays no part in this decision.** Even for clipping (a drawn part with `mask_write`), **whether its own color is clipped by other masks is decided by its own `mask_influence` alone**. Do not clip a part's own color with the mask it writes (draw first, then write). `mask_write` only decides which parts write (§1-3).

A writer that is switched off (`mask == 0` or `Hide == true`) is the same as no writer at all.

### 2-3. Stencil write op (OP)

- **Mask parts**: `mask_influence` doubles as the write op. `0` = **increment** (`stencil + 1`) / `1` = **invert** (`~stencil`)
- **Clipping**: `mask_influence` likewise doubles as the write op (the same interpretation as for mask parts). Clipping is also a drawn part, so **the same value is used both as the "receive flag" (§2-2) and as the "write op"** (§1-3)

### 2-4. Polarity (whether to draw inside or outside the mask)

Polarity is decided **not by the feature but by the target part's `visible_inside_mask`** alone.

- `visible_inside_mask == 1` → draw **inside** the mask
- `visible_inside_mask == 0` → draw **outside** the mask
- Decision: `masked == visible_inside` (draw if true, `discard` if false)

A target that draws inside (it receives masks, and its effective `visible_inside_mask` is `1`) is therefore **not drawn wherever no mask covers it — even when the animation has no mask at all**, or every writer is switched off. Do not skip the test for such a target because there is nothing to mask with; the test is what hides it.

The "mask is outside, clipping is inside" of §1-2 is **a default commonly used in authoring**; polarity itself is a setting of the target. **Even when a target is in the scope of both a mask and clipping, decide with one stencil and the target's one `visible_inside_mask`** (do not split polarity per feature).

### 2-5. Cutout threshold

**For both masks and clipping**, FrameData's `mask` (`0..255`) decides which pixels are written to the stencil. This is done in the shader.

- Threshold = `(255 - mask) / 255`. Only pixels whose texture alpha exceeds the threshold are written
- `mask == 255` → threshold 0 → **every pixel that is not fully transparent (α = 0)** is written
- `mask == 0` → threshold 1.0 → no pixel is written (consistent with §2-2)

The threshold does not apply to **shapes, which have no texture**; the whole shape is written.

### 2-6. Composing mask settings through an instance

When the target is an **instance part**, the mask reaches **its whole child animation**. The settings of the instance part (the caller) and of each part in the child animation (the callee) are then **composed**. Neither one wins.

| Setting | Composition | Meaning |
|---|---|---|
| `visible_inside_mask` | **OR** | If the instance part asks for "inside", every part of its child animation draws inside |
| `mask_influence` | **AND** | If the instance part does not receive masks, no part of its child animation receives masks |

The effective values are:

```
effective_influence      = instance_part.mask_influence      && self.mask_influence
effective_visible_inside = instance_part.visible_inside_mask || self.visible_inside_mask
```

- **The composition reaches one level only.** It takes the instance part's own flags, not what the instance part itself received from further up. The parts of an instance nested inside the child animation are composed with *their* instance part, and nothing above it. For example, when the instance part of a top-level animation has `mask_influence == 0`, the child animation's own parts receive no masks, but the parts of an instance inside it whose instance part has `mask_influence == 1` are still clipped by masks.
- **The composition does not depend on whether anything above masks.** Compose even when the calling animation has no mask of its own.
- **A top-level animation** that no instance calls is composed with `mask_influence = 1` / `visible_inside_mask = 0`, so each of its parts is decided by its own settings.
- **Scope** (§2-1) is decided on the order of the whole drawing, in which a child animation is drawn at its instance part's position: a mask of the calling animation reaches the child's parts where it would reach the instance part, and a mask inside the child animation applies to the child's own parts by the child's own ranks (§2-7). What the composition affects is **whether a part receives masks (§2-2) and polarity (§2-4)**.
- **`mask_write` is not composed.** It is a local fact that the part itself writes.

This was checked against SpriteStudio 7.5 with instances three levels deep, with the child animation's mask part enabled, hidden, set to `mask == 0` and removed.

In a Player that builds each child independently, the simple approach is to **pass the instance part's flags to the child just before building it**, not the values the parent itself received. Composing down the whole hierarchy (AND / OR through every level), or composing only when the calling animation masks, both disagree with SpriteStudio 7.5. So does handling such as "the child wins if it has its own setting, and inherits the parent's otherwise", which is neither OR nor AND.

### 2-7. Carry-over of clipping written inside an instance

§2-6 was the **downward** composition, where a mask reaches into an instance from outside. A mask written inside a child animation — a pure mask or clipping — first applies to **that child animation's own parts**, by the rules of §2-1 as in any animation, together with the masks that reach in from outside. There is also an **upward** carry-over in the opposite direction: **a mask written inside a child animation reaches outside it (to the parts of the parent animation drawn after that instance)**.

- **Only clipping (drawn parts with `mask_write`) carries over.** A pure mask is canceled at its own rank and closes inside the child animation (§2-1 / §4-2), so it does not cross the boundary. Clipping "affects the parts after it, until the end of the animation's drawing" (§2-1); think of **that "end" as extending past the instance boundary to the end of the parent animation's drawing**.
- **Defer the close (the cancel / restore) until the parent animation has finished all its drawing.** If instances are nested, pass it further up to the next parent.
- **Scope**: a carried-over clipping affects the parent's parts after **the instance part's rank in the parent's `draw_order`** (`rank > instance_rank`) (the clipping scope of §2-1, decided at the instance part's position).
- **Write op**: the write op of a writer inside a child animation (increment / invert, §2-3) is decided by its **effective `mask_influence` (its own `mask_influence` && its instance part's)**, the same one level of composition as §2-6. If the instance part has `mask_influence == 0`, the effective value is 0, and the writer goes to **the increment (counter) side = the union** (not the invert side). So even when its `mask_influence` differs from another mask outside, the two **form a union instead of canceling each other** (consistent with the separation into two planes in §4-4). That the write op, too, takes only one level was checked against SpriteStudio 7.5 with two overlapping invert clippings three levels deep, under a `mask_influence == 0` instance at the top.
- **The target side (the parent's later parts)**: even if the parent has no writer of its own, when a visible instance leaves a clipping open, you must treat **the parent's parts after the instance as mask targets**.

Only clipping carries over, and pure masks do not, because their scopes (§2-1) differ. A pure mask is complete "before itself" and closes at its own rank. Clipping covers "after itself, until the end of the animation", and that end extends past the instance boundary to the end of the parent.

---

## 3. What to read from FrameData

**Parts that emit a mask DrawBatch (`DrawBatchKind::Mask`)** (all with `count = 1`, and `vertex_count` / `index_count` of `0`):

- **Mask parts** (`PartTypeMask`)
- **Drawn parts turned into masks by `draw_as_mask`** (shape / 9-slice). Both the mask DrawBatch and the DrawBatch of its own color are emitted **at the same rank**. A pure mask draws no color, so **do not draw the color DrawBatch.** The mask DrawBatch of this pair always has a `texture_hash` of `0`. Take the texture for the writer's cutout (§2-5) from the part's own color DrawBatch at the same rank, or from the part's cell. Looking up the mask DrawBatch's `texture_hash` finds no texture.

> [!IMPORTANT]
> **Clipping (`mask_write`) emits no mask DrawBatch.** Clipping is emitted only as an ordinary drawing DrawBatch, so a Player that looks only at `Mask` DrawBatches **misses all clipping.** Decide the writers in the Player by reading PartData (the three expressions of §1-3).

**Hidden writers**: the Brain removes from `draw_order` and the DrawBatches only **mask parts** with `Hide == true`. **A drawn part turned into a mask by `draw_as_mask` still emits its mask DrawBatch when `Hide == true`** (only its own color drawing stops; the shape is still written). Decide in the Player whether a writer works, from **both `mask != 0` and `Hide == false`** (§2-2). It only removes again what the Brain has already removed, so this check is safe in either case.

**Do not use `mask_indices` as the list of writers.** FrameData's `mask_indices` is the list of pure mask parts, but it **does not include clipping** and **does not remove mask parts with `Hide == true`**, so it does not match `draw_order`. Decide the writers by walking `draw_order` with the expressions of §1-3.

**Bit `0x01` of `DrawBatch.flags`** indicates that the DrawBatch's part has `mask_influence == 1`.

---

## 4. Implementing with the stencil

SpriteStudio's masks are designed around a stencil buffer. If your engine lets you use the stencil, using it is the simplest approach.

1. Walk `draw_order` in order
2. When a writer comes, update the stencil with its write op (increment / invert) for the pixels left after `discard` by the cutout threshold
3. When a target comes, decide whether to draw it from the stencil value and `visible_inside_mask`

> Node-based shaders such as Unity's Shader Graph usually cannot specify stencil details. You may need to move just the mask processing into hand-written shaders (HLSL and so on) or a dedicated RenderPass.

### 4-1. Stencil settings for writers and targets

Masking splits into two kinds of draws: the **writer** (writes the mask's shape to the stencil) and the **target test** (reads the written stencil and decides inside or outside).

- **writer**: writes no color (`ColorMask 0`). It always updates the stencil (`Comp = Always`), and switches the update operation `Pass` by the write op (§2-3).
    - Increment → `IncrementWrap` / `DecrementWrap` (a **counter** that counts how many overlap)
    - Invert (`mask_influence == 1`) → `Invert` (odd or even; it is its own inverse)
    - **Do not let these two share bits.** They do not commute, so mixing them makes the result depend on the drawing order, and canceling does not return it to 0 (§4-4). Split the `WriteMask` per write op.
    - `Ref` is not used (it is not a replace, so `0` is fine). For the cutout, `discard` fragments with the threshold of §2-5 (if the shape itself is the mask, as with shapes, make the threshold negative so nothing is discarded).
- **Target test**: add **only a stencil test** to ordinary color drawing. Express polarity (§2-4) with `Comp`.
    - `visible_inside_mask` (draw inside) → `NotEqual` (passes pixels where mask bits are set)
    - Draw outside → `Equal`
    - Use `Ref = 0`, and the test **does not write** the stencil (`WriteMask = 0`).

In other words, **polarity (inside or outside) belongs to the target side, not the writer**.

### 4-2. Restoring the stencil

You may want to save the stencil before drawing and restore it afterwards, but SpriteStudio's masks are designed to **cancel themselves through the symmetry of the operations**, so you need neither save and restore nor a clearing draw.

- **Pure masks**: add the shape before all parts, and **subtract the same shape at its own rank in `draw_order`**. The net change within the frame returns to **0**, and the scope "a mask affects only parts with lower draw priority than itself (earlier parts)" (§2-1) follows naturally. With invert, two `Invert`s cancel in the same way.
- **Clipping**: write at its own rank, and **after all drawing of that animation has finished**, cancel it with the inverse operation on the same shape. Because you restore it only after drawing everything it clips (the parts after it), **the look inside the animation does not change**, and the net change returns to **0** after the animation is drawn. With invert, two `Invert`s.
    - **This cancel cannot be skipped.** The stencil is cleared **only once**, at the start of the camera or frame. When you draw several SpriteStudio animations in the same frame, **it is not cleared between animations**, so leftover clipping bits corrupt the mask decisions of animations drawn later (the bit isolation of §4-3 protects against other uses such as URP, but **not against another SpriteStudio animation that uses the same bits**).

The key point is **"do not clear, save or restore; keep the state inside the animation's drawing through the symmetry of increment and decrement"** (pure masks cancel at their own rank, clipping at the end of the animation's drawing). The only cost is building the shape once more to cancel it, and masks are few, so it stays small.

To implement **the carry-over of clipping inside an instance** (§2-7) with the stencil, **defer the clipping's "cancel at the end of the animation's drawing" above past the instance boundary, to the end of the parent animation's drawing.**

- The child animation builds its drawing as usual, but **marks the draw that closes the clipping (the inverse operation) as "deferrable"** (its draw role stays a write).
- When the parent takes in the child as an instance, **it does not emit that closing draw at the instance boundary**; it rewrites it to the parent's indices, keeps it, and **emits them all at the end of the parent's own drawing.** The clipping inside the child then also clips the parent's parts drawn after the instance. If you keep the mark when emitting, **nested instances pass it up to each parent in turn.**
- **Do this whether or not the parent has masks of its own.** Even when the parent has no writer and only the child has one, the child's carry-over must flow through, so do not skip this based on whether the parent has masks. If a visible instance leaves a clipping open, test **the parent's parts after the instance as targets** (§2-7).
    - "The parts after the instance" include **the child animation of an instance drawn later** and **effect particles**. A Player that builds each child just before taking it in cannot tell yet which earlier instance leaves a clipping open, so tell every child, before building it, that coverage from outside may be live whenever the tree has masks of its own or a clipping anywhere below; otherwise a later child with no writer of its own draws untested.
- **Do not merge draws across the gap the deferred closes leave.** Once the closing draws are taken out, the child's last draw and the parent's next draw are no longer next to each other in the index buffer. A Player that joins adjacent draws of the same material into one index range has to check that the ranges are contiguous: growing the first range to cover the second also covers the closing draws' indices in between, and draws their shapes with color.
- Decide the write op and plane of a carried-over writer by its **effective `mask_influence`** (§2-7). Closing only applies the inverse of the matching write (`Invert` is its own inverse; increment ↔ decrement) to the same plane, so the plane separation of §4-4 is kept as is.

### 4-3. When the engine always uses the stencil (URP and others)

Many render pipelines, URP among them, **always use the stencil for their own purposes** (lights, decals, Rendering Layers, SSAO, material types in deferred, camera stacking, and so on). Layering SpriteStudio's masks directly on top makes them break each other.

**Confine SpriteStudio's masks to a dedicated bit range with `ReadMask` / `WriteMask`.** SSPlayerForUnity limits them to the **low 4 bits (`0x0F`)**.

- These are the **free bits** in URP's stencil allocation (bits 0–3 = UserMask / bit 4 = light / bits 5–6 = material type / bit 7 = reserved), so they can coexist with URP's features (the Built-in pipeline uses the same allocation).
- `WriteMask` stays **inside** `0x0F` (and is split further per write op; §4-4). The carry from `IncrementWrap` **does not leak** into bit 4 (URP's light) (the mask drops it).
- `ReadMask = 0x0F`: the test looks **only at its own bits**, so it can ignore what URP writes to bits 4–7.
- The 4 bits are split per write op (§4-4), so **the counter gets 3 bits = up to 7 overlapping masks**. The invert side is only odd or even, so it has no limit on the count. **Widening the bits beyond `0x0F` breaks URP's reserved bits, and they break yours.**

If the pipeline you port to uses the stencil, likewise **choose a dedicated bit range and fence it with the read / write masks.** On engines where you cannot reserve bits, or cannot use the stencil at all, switch to the coverage bitmap of §5 (which does not use the stencil).

### 4-4. Separate bit planes per write op (required)

`Invert` and `IncrementWrap` / `DecrementWrap` **each have their own inverse on their own, but they do not commute with each other.** If they use the same bits, two things happen where writers with different `mask_influence` overlap.

- **The result depends on the drawing order**: `increment → invert` gives `0 → 1 → 254` (written), and `invert → increment` gives `0 → 255 → 0` (**not written**). The same two masks give opposite results depending on the order they are drawn in.
- **Canceling does not return to 0**: `increment (1) → another invert (254) → decrement (253)`. The leftover value corrupts the mask decision of every part drawn afterwards, and the cancel of §4-2 no longer holds.

A stencil write is `new_value = (op(old_value) & WriteMask) | (old_value & ~WriteMask)`. The operation runs on the whole value, but **limiting the written bits discards carries and borrows, so each plane stays closed.** So split `WriteMask` per write op.

| Write op | Plane | Bits needed | How many can overlap |
|---|---|---|---|
| `Invert` | **Parity** | 1 bit | Odd or even. **No limit** |
| `IncrementWrap` / `DecrementWrap` | **Counter** | The remaining `n` bits | Union. Up to `2^n - 1` |

- One bit is enough for parity (`Invert` only flips that bit and does not touch the counter).
- The counter must be **contiguous from bit 0** (increment and decrement are ordinary arithmetic).
- **You do not need to change the target test.** `ReadMask` stays the whole range covering both planes, and the test stays "value ≠ 0"; this is **the union of the two planes**. It matches the specification that masks with different `mask_influence` form a union where they overlap.

SSPlayerForUnity splits the `0x0F` of §4-3 into **parity `0x08` / counter `0x07`** (masks with `mask_influence == 0` can overlap up to 7 deep).

**This rule does not depend on the implementation approach.** The coverage bitmap of §5 reproduces the same operations by replaying them, so the rule applies there as is. What you split there is not bits but **accumulator variables**, and since there is no bit budget to divide, the overlap limit above does not apply either (§5-1).

---

## 5. Implementing on engines without a stencil (coverage bitmap)

For example, Godot's 2D canvas does not let you use a stencil buffer (as of 4.7, only 3D materials have one). On such engines, you can reproduce the stencil as follows.

1. **Build a coverage bitmap**: every frame, draw all current writers into an offscreen coverage bitmap. Assign one bit to each writer, and set that bit on the pixels its shape covers.
2. **Replay at draw time**: the target's shader reads the coverage bitmap, replays the writers that are active at that target to build the stencil state, and clips inside or outside with `discard`.

The key point is **"hold no state that depends on when things are drawn; split the work into coverage (the writers' shapes) and replay (building the state on the target side)"**. Using 8 bits × 3 = 24 bits in the three RGB channels of an RGBA8 lets one coverage handle up to **24 writers**.

Instances multiply writers: an instance's writers count again each time the instance is placed, and one coverage serves the whole instance tree (§5-2), so 24 runs out quickly. Rather than growing the texture, **split the same texture into tiles** — two side by side, then 2x2 — each tile mapping the same bounding box and holding its own 24 bits. The replay reads the same texel of each tile it needs. This trades resolution for writers without using more memory. SSPlayerForGodot stops at 4 tiles (96 writers) and ignores the rest with a warning.

For **the size of the coverage bitmap**, it is practical to fit it to the bounding box of all writers and **cap the long edge (at 1024 px, for example)**. That keeps pixel-level precision on a roughly Full HD screen while staying within 4 MiB (RGBA8) per bitmap. When many animations are shown at once, you can reduce it further by matching it to the screen resolution or sharing a small pool.

Treat target parts **outside the bounding box** as if there were no writer (not masked). This avoids false masking from CLAMP at the texture edges.

### 5-1. Pseudo-code

The whole flow in engine-independent pseudo-code, for one animation with no instances. §5-2 extends it to instances.

Each writer holds its bit over an **open interval of the drawing order** (§2-1): a pure mask from the start of the animation up to itself, clipping from itself to the end. A target is affected by the writers whose interval contains its own position.

**Data structures (up to 24 writers; 8 bits × 3 in the RGB of an RGBA8)**

```
const int   MAX_WRITERS = 24;
const float SEQ_END     = 16777216.0;  // past every position; still exact as a float

struct MaskWriter {
    int   part_index;    // part index
    int   bit;           // 0..23, the bit position in the coverage
    float open;          // the writer holds its bit for positions open < p < close
    float close;
    bool  op_invert;     // write op: false = increment, true = invert
};
```

**Deciding the writers (every frame)**

```
function build_mask_writers(frame) -> [MaskWriter]:
    writers = []
    for rank, part_index in enumerate(frame.draw_order):
        part = frame.parts[part_index]
        // part.data.* = PartData (static) / part.frame.* = FrameData (per frame)
        pure_mask  = part.data.type == Mask || part.data.draw_as_mask
        is_writer  = pure_mask || part.data.mask_write
        if not is_writer:               continue
        if part.frame.mask == 0:        continue       // §2-2
        if part.frame.hide:             continue       // §2-2 / §3
        if len(writers) >= MAX_WRITERS: break

        is_clipping = part.data.mask_write && !pure_mask
        writers.append(MaskWriter {
            part_index  = part_index,
            bit         = len(writers),                 // 0, 1, 2, ...
            // §2-1 scope: clipping = after itself / mask = before itself
            open        = is_clipping ? rank    : -1,
            close       = is_clipping ? SEQ_END : rank,
            op_invert   = part.data.mask_influence,     // §2-3 (shared by pure masks and clipping)
        })
    return writers
```

**Drawing the coverage bitmap (every frame)**

Draw each writer's shape into an offscreen RGBA8 viewport with additive blending. The bits do not overlap, so adding them is a bitwise OR. Draw each writer in a color that has only its own bit set.

```
function render_coverage_bitmap(writers, frame) -> Bitmap:
    bitmap = create_offscreen_RGBA8_with_additive_blend()
    for w in writers:
        channel   = w.bit / 8                          // 0 = R, 1 = G, 2 = B
        bit_val   = (1 << (w.bit % 8)) / 255.0
        bit_color = vec4_zero_except(channel, bit_val)
        threshold = (255 - frame.parts[w.part_index].frame.mask) / 255.0

        draw_geometry(
            geometry = geometry_of(w.part_index),       // normal = sprite / mesh / shape
            shader   = MASK_WRITE_SHADER,
            uniforms = { mask_bit_color: bit_color, mask_threshold: threshold }
        )
    return bitmap
```

**Bit-writing shader (GLSL)**

```glsl
uniform vec4  mask_bit_color;   // the bit, placed in the matching RGB channel
uniform float mask_threshold;   // (255 - mask) / 255

void fragment() {
    // §2-5 cutout: write only pixels whose alpha exceeds the threshold
    if (texture(TEXTURE, UV).a <= mask_threshold) discard;
    // additive blending works as src.rgb * src.a, so set alpha to 1
    COLOR = vec4(mask_bit_color.rgb, 1.0);
}
```

**Replay at draw time (the target's shader, GLSL)**

```glsl
uniform sampler2D coverage;
uniform int   mask_count;
uniform vec4  mask_meta[MAX_WRITERS]; // (open, close, bit, op_invert). Matches MaskWriter
uniform float this_rank;              // the target's position in the drawing order
uniform float visible_inside;         // §2-4 polarity (1 = inside / 0 = outside)

bool mask_passes(vec2 cov_uv) {
    // no writer at all: nothing is masked, so polarity alone decides
    if (mask_count == 0) return visible_inside <= 0.5;
    // outside the bounding box is the same as no writer (avoids false masking from edge CLAMP)
    if (cov_uv.x < 0.0 || cov_uv.x > 1.0 || cov_uv.y < 0.0 || cov_uv.y > 1.0)
        return visible_inside <= 0.5;

    vec4 cov = texture(coverage, cov_uv);   // the sampler must be nearest
    // Plane separation of §4-4. These are variables, not a real stencil,
    // so there is no bit budget to divide; two accumulator variables are enough.
    int parity = 0;   // invert (mask_influence == 1): written where an odd number overlap
    int count  = 0;   // increment (mask_influence == 0): written where at least one overlaps
    for (int i = 0; i < mask_count; i++) {
        vec4 m = mask_meta[i];
        // §2-1 scope: the writer holds its bit over the open interval (open, close)
        bool active = this_rank > m.x && this_rank < m.y;
        if (!active) continue;
        // extract this writer's bit from the coverage
        int bit  = int(m.z + 0.5);
        int byte = int(cov[bit / 8] * 255.0 + 0.5);
        if (((byte >> (bit % 8)) & 1) == 0) continue;
        // §2-3 write op (§4-4: do not mix these two into one variable)
        if (m.w > 0.5) parity ^= 1;                     // invert
        else           count  += 1;                     // increment
    }
    bool masked = (parity != 0) || (count != 0);        // union of the two planes
    return masked == (visible_inside > 0.5);            // §2-4
}

void fragment() {
    // ... ordinary color computation ...
    if (!mask_passes(coverage_uv)) discard;
    COLOR = ...;
}
```

- **Use nearest for the coverage sampler.** With linear, neighboring bits blend and cannot be read correctly.
- **Outside the bounding box, and with no writer at all, decide by `visible_inside` alone** (nothing is masked there). A target that draws inside must still run the test in an animation with no mask, or it draws where SpriteStudio hides it (§2-4).
- Compare the target's `this_rank` with each writer's interval `mask_meta[i].xy` (§2-1).
- **Do not mix invert and increment into the same variable** (§4-4). If you apply both to one variable, the result **depends on the order of the writers** where writers with different `mask_influence` overlap (`increment → invert` is written, `invert → increment` is not).
- However, **the replay side has no bit budget.** These are shader variables, not a real stencil, so there is no limit like the "3 counter bits = 7 masks" of §4-3; overlap is possible up to the number of writers (24).

### 5-2. Coverage UV and passing to instances

**Coverage UV**: compute the replay shader's `coverage_uv` in the vertex shader with `M`, **an affine transform that maps the target's Player-local coordinates to the coverage UV `[0, 1]`** (a 2x2 basis and offset that normalize by the bounding box). The coverage bitmap spans the bounding box of the animation that owns the writers, so `M` is based on that animation's Player-local coordinates.

**Instances**: a mask of the calling animation reaches into an instance's child animation, a mask written inside the child animation applies to the child's own parts, and its clipping carries out to the calling animation (§2-6 / §2-7), at any depth. With the coverage bitmap, all of this comes down to **numbering the drawing order of the whole instance tree**, with **one coverage bitmap owned by the top-level animation**.

1. **Number the tree.** Walk the top-level animation's `draw_order` and give each part the next number. When the part is a visible instance part, give it its number, then number its child animation's `draw_order` the same way (recursively) before moving on. The child animation is drawn at its instance part's position, so this order is the order of the whole drawing. Call the number `seq`.
2. **Bake every writer of the tree into the one coverage.** Map a child animation's writer shapes into the top-level animation's local coordinates with the chain of instance placement matrices (`slot_xf`, child-local → parent-local), and include them in the bounding box. Each writer holds its bit over an open interval of `seq` (§2-1):
    - **Pure mask**: from **the start of its own animation** (its instance part's `seq`, or `-1` at the top level) **to itself**. It never reaches outside its animation.
    - **Clipping**: from **itself to the end of the whole tree** (`SEQ_END`). A clipping inside an instance keeps clipping the calling animation's parts drawn after it (§2-7), at any depth.
    - Write op: the writer's effective `mask_influence` (§2-7).
3. **Replay with `seq`.** `mask_meta` is `(open, close, bit, op_invert)` as in §5-1, and a target passes its own `seq` as `this_rank`. The shader of §5-1 works unchanged.
4. **Child parts' UV and settings.** Compose the UV transform down the tree: `M_child = M_parent · slot_xf`, so each child vertex maps to the top-level coverage. Each child part decides whether it receives masks, and its polarity, from the composition of §2-6 (its instance part's flags only).

- **No separate carry-over step is needed.** The intervals already say how far each writer reaches: a pure mask closes inside its animation, clipping reaches the end of the tree. With the stencil, the close is deferred and canceled (§4-2); the coverage bitmap has no restore step, because the replay builds the state on the target side.
- **Do this even when the top-level animation has no writer of its own**, as long as some animation in the tree has one.
- **The writers of the whole tree share the bit budget**, and an instance's writers count again each time the instance is placed. Use tiles (§5) when 24 is not enough.
- A child animation's parts are decided before the top-level animation has numbered the tree, so a child cannot yet tell which writers reach which of its parts. Test all of its parts that receive masks; the top-level animation, which knows every interval, can skip the parts no writer covers (unless they draw inside).

### 5-3. Coverage bitmap size classes and the one-frame delay

This section covers a problem, and its fixes, that appears with masks that keep scaling up and down, when you size the coverage bitmap to the bounding box (§5) and **reuse offscreen render targets from a pool**. The basic approach works without this section, but when a mask zooms hard and continuously, the mask can appear to drop out for a single frame.

**Size classes (powers of two)**

Using the bounding box's pixel size directly as the render target size means recreating the target every frame, which is wasteful. **Round each axis up to a power-of-two size class** (… / 256 / 512 / 1024), and borrow a target of that class from the pool. Class `C` covers `C/2 < px ≤ C`, so you switch to another class only when crossing a boundary, and in the usual case of smooth scaling the target is reused.

The reading side's UV comes from `M` of §5-2 (normalization by the bounding box), so it **does not depend on the size class** (it always maps the same bounding box in Player-local coordinates to `[0, 1]`). Whichever class's coverage you bind, the target is clipped correctly.

**The one-frame delay, and the dropout on the frame where the class changes**

In many engines, an offscreen render target can be read only **one frame late** (coverage drawn in one frame can be read in the next; for example, Godot's Viewport texture). Normally this only delays the mask by one frame, which is not a problem.

The problem is **the frame** where the size class changes. When you borrow a target of the new size, it **has no valid coverage yet** (nothing drawn yet, or leftovers from another use). Because of the one-frame delay, it cannot be read in that frame, so as is **the mask does not work for that one frame**, and parts that should be clipped are drawn unclipped. With continuous zooming, it flickers every time a boundary is crossed.

**Fix 1: read the old target on the frame where the class changes**

When the class changes, do not return the old target right away; **keep it for one frame**, and in that frame **read the coverage in the old target (drawn in the previous frame).** The UV does not depend on the class, so the target maps correctly even with the old target (at the previous resolution), and this is only **the same one-frame delay as a normal frame** (at the previous resolution for that one frame). The mask does not drop out. Draw that frame's coverage into the new target, and switch to it from the next frame.

**Fix 2: add slack on the shrinking side only**

If the bounding box size wobbles around a class boundary, the class flips every frame (512 ↔ 1024 …) and switches happen constantly. To prevent this, **add slack to the shrink decision only.** Grow immediately (never lower the resolution), and when shrinking, **keep the current class until the size is well below the boundary.** Delaying growth leaves the coverage short of resolution and roughens the edges (a visible loss), while delaying shrinking only costs a little more VRAM (an invisible cost).

There is no single right width for the slack. SSPlayerForGodot uses `0.15` (it keeps the current class until the size is 15% below the boundary). Raise it to reduce switches further; lower it if the VRAM of small masks lingering in large targets bothers you.

Fix 1 covers "no dropout even when switching", and fix 2 covers "fewer needless switches"; they complement each other.

**A held frame**

If the size class for the next frame is decided from this frame's bounding box, an animation that is not advancing — paused, stopped, or shown in an editor — never draws that next frame, and keeps the coverage drawn with the starting size class. With a small starting class, and more so when tiles (§5) split it further, masks come out blocky. When the class picked differs from the one just drawn with, draw the held frame once more.

---
[Back to contents](../README.md)
