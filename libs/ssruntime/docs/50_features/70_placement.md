> [日本語](./70_placement.ja.md) | **English**

# Placement: Fit and Align

A Player draws the animation at its authored scale into whatever target the host gives it. A host that lays out a box — a UI view, a node with a size, an editor preview — needs the animation fitted into that box instead. The Brain has no part in this. It is arithmetic over a box and a view size, so it belongs in the Player's portable core, where every face and every engine adapter inherits it and it can be tested without a renderer.

This page is what a Player that offers placement ports: the names, the defaults and the arithmetic. How it reaches the host's API — a prop, a node property, a function — is the host's.

## 1. The placement

| Field | Values | Default | Meaning |
|---|---|---|---|
| `fit` | `contain` / `cover` / `none` | `contain` | How the box is scaled to the view. `none` is no fit: scale 1, origin at the view's centre, and the caller places the animation with `offset` and `scale` |
| `align` | `center`, `top`, `bottom`, `left`, `right`, `top-left`, `top-right`, `bottom-left`, `bottom-right` | `center` | Which point of the box lands on the matching point of the view |
| `frame` | A box in player-local units | The authored canvas ([§2](#2-where-the-box-comes-from)) | The box to fit |
| `offset` | Pixels | `(0, 0)` | Added to the solved origin, after the fit |
| `scale` | A multiplier | `1` | Multiplies the fit's scale. It does not replace it |
| `flipX`, `flipY` | Booleans | `false` | Mirrors. They change which end of the box the alignment anchors to |

`fit` takes `<Image resizeMode>`'s words, because those are the ones UI hosts already have. A host with its own vocabulary — CSS `object-fit` / `object-position`, Godot's `stretch_mode` — may spell it that way on its own surface, over these three values. Rotation is not part of the placement: pass through whatever the caller already had.

`align` decomposes into two independent axes, each `-1`, `0` or `1`. **On the vertical axis, `-1` is the top of the view** — screen terms, not maths terms.

## 2. Where the box comes from

**When the caller supplies no `frame`, the box is the animation's authored canvas**: its canvas size, placed against the origin by its pivot. It costs nothing, it holds still while the animation plays, and it is the frame the artist composed in. It is the playing animation's own canvas, and one pack's animations may differ: in `Ringo`, `dead` is 1300×600 and the others 800×600. A host that wants one scale across a pack reads the canvas once, after its first `setAnimation`, and passes it as `frame` from then on.

Both come from the animation in the ssab: `canvas_size` and `pivot` on `AnimationData`. The Brain does not apply the pivot, and it is not in the runtime's animation metadata, so read both from the ssab. The pivot is normalized from the canvas centre, in editor space (Y-up). In a Y-down player-local space the box is:

```
x = -pivot.x * width - width / 2
y =  pivot.y * height - height / 2
```

`pivot (0, -0.5)` — the canvas's bottom edge, which is how a character standing on the ground is authored — gives a box entirely above the origin.

A Player may also measure the box from the geometry it draws and offer that as `frameScope`: `animation` (the box of the one playing) or `pack` (the union of every animation in the file), selected explicitly. Measuring means evaluating every frame at load. `pack` also draws the character smaller: on `Ringo`, `attack1` spans 383×527 player-local units while the pack's union spans about 1030×878, because `dead` slides the character most of a screen to the right. What `pack` buys, a scale that does not change with the animation, a host gets without measuring by passing one animation's canvas as `frame`, and a caller that wants a tighter box can supply `frame` too. **A Player that does not measure leaves `frameScope` out** rather than half-honouring it.

## 3. The arithmetic

**Scale.** `sx = viewWidth / box.width` and `sy = viewHeight / box.height`. `contain` takes `min(sx, sy)` and `cover` takes `max(sx, sy)`. The result multiplies `scale`.

**A flip is a signed scale, and the alignment follows it.** The draw maps `p → origin + s · p`, so a negative `s` swaps which end of the box is the low one. Solve each axis with the signed scale. Do not solve it unflipped and mirror afterwards.

**Each axis**, given the view's size along it, the signed scale `s`, the box's low edge `lo` and its extent `size`, and the side (`-1`, `0` or `1`):

| Side | Origin |
|---|---|
| `0` (centre) | `viewSize / 2 - s · (lo + size / 2)`. The sign of `s` does not change it, which is why it is the safe default |
| `-1` (low) | `-s · lowEdge` |
| `1` (high) | `viewSize - s · highEdge` |

`lowEdge` is `lo` when `s > 0` and `lo + size` when `s < 0`; `highEdge` is the other one.

**Then add `offset`.** It comes last, after everything else.

**A degenerate case is not an error.** A zero-sized or absent box, or a zero-sized view, leaves nothing to divide by. Skip the fit and return the unfitted placement, as `fit: none` does: the caller's `scale`, with the origin at the view's centre. A host that laid out a collapsed box gets its animation back the moment the box has a size.

## 4. Coordinates

The solved origin is in **view coordinates**, with `(0, 0)` at the view's top-left and Y pointing down. The unfitted origin is therefore the view's centre, not `(0, 0)`. If the Player's own placement call measures from somewhere else, such as the view's centre, convert on the way in. Getting that wrong looks like a correct fit displaced by half the view.

---
[Back to contents](../README.md)
