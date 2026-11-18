> [日本語](./80_ui_integration.ja.md) | **English**

# UI Integration: Hit Testing, UI States and Reduced Motion

A host that uses an animation as a UI component, such as a button, needs to tie three things to how the animation looks: which part the pointer is on, which state the component is in, and whether a setting to reduce motion is on. The Brain has no part in this. Hit testing reads the frame the last tick built, a state is a label section played, and reduced motion is whether playback stops, so it all happens inside the Player.

Put the UI logic in an adapter that only its users pay for: a separate package or module that depends on the host's UI components. An app that only plays animations must gain nothing. All the Player's core gains is what is computed only when called: part outlines and `hitTestPart`, and the [playback section by labels](../20_design/30_api_conventions.md#8-1-separate-selecting-from-playing).

This page is what a Player that offers UI integration keeps the same as the others: the names, the defaults and the meaning. How it reaches the host's button components — subclassing or wrapping, and the names and timing of the action callbacks — is the host's.

## 1. Hit testing

### 1-1. `hitTestPart`

| Item | Meaning |
|---|---|
| Arguments | A point, and a set of target parts (names or indices; optional) |
| Returns | The index of the part the point is on. When it is on none, whatever the Player's `findPartIndex` returns for a part it cannot find (`-1`, `None`, …) |
| Without targets | Searches every part that has an outline |
| When parts overlap | The front-most in draw order |

The point is taken in the space the Player reports part outlines in. The space of `partTransform` is recommended.

Like the other APIs that read one part, **answer from the frame the last tick built** ([Player API Conventions §9](../20_design/30_api_conventions.md#9-reading-a-part)). Do not build a frame to answer.

A FrameData's `draw_order` leaves hidden parts out (unless they draw as a mask), so the first part hit walking `draw_order` from the back is the front-most.

```cpp
// partOutline() and contains() are the Player's functions for the outlines of §1-2 (not in the C-API)
int hitTestPart(const ss::runtime::FrameData* frame, Point point, const std::set<int>* targets) {
    auto order = frame->draw_order();
    for (int rank = int(order->size()) - 1; rank >= 0; --rank) {
        int p = order->Get(rank);
        int part = frame->parts()->Get(p)->part_index();
        if (targets && !targets->count(part)) continue;
        auto outline = partOutline(frame, p);
        if (outline && contains(*outline, point)) return part;
    }
    return -1;
}
```

### 1-2. Part outlines

The outline's shape, type and coordinate space are the Player's, chosen to suit its UI toolkit. Which parts it bounds, and how, is recommended to be the same everywhere, as below. Returning the outline as four points is recommended, so that it fits turned artwork too.

| Part | Outline |
|---|---|
| A part with a cell | The four corners of its local vertices (`ss_runtime_get_local_vertices`) through the world matrix and the transform the Player adds at the root (flip, offset). Skew and vertex deform included |
| A Mesh part | The axis-aligned box around its mesh vertices (`ss_runtime_get_mesh_vertices_x` / `_y`, in world space), through the root transform. A skinned mesh part's own transform stays near the root, so its corners through the matrix do not match the artwork |
| A part with a size and no cell (a Shape) | The FrameData part's `size_x/y` around its `pivot_x/y` (left `-(0.5 + pivot_x) * size_x`; top `-(0.5 + pivot_y) * size_y` with Y down, `(0.5 - pivot_y) * size_y` with Y up, the default), through the world matrix and the root transform, as four points |
| Null, bone, Instance and Effect parts | None (their size is 0) |
| A hidden part | None. A part drawn at alpha 0 has one |

This table assumes the SoA buffers of `Direct` mode (the default). In `SceneGraph` mode neither world matrices nor mesh vertices are computed, so take them from the Player's own nodes ([Render Loop §3](../30_core/40_render_loop.md#3-rendermode)).

A corner that a Shape's vertex deform carries outside its size is not inside the outline. Reading the Shape vertex buffer would follow it; the size and pivot are what every Player already reads, so the outline is built from those alone. For a Shape without vertex deform it equals the box around the vertices drawn.

### 1-3. Where the button takes the pointer

The area an adapter's button takes the pointer in is **the whole display box unless narrowed**. Narrowed, it is the union of the target parts' outlines, and with no targets named, the union of every part that has an outline. Do not narrow keyboard and accessibility actions.

- **When the host's button takes the pointer across its whole box whatever its child answers** (Flutter's `InkWell`, for one), the child cannot narrow it. Put the narrowing outside the button, map the point into the inner view's coordinates, and pass it to `hitTestPart`. The button's padding then does not shift it.
- **On a host whose components answer their own hit test** (Flame's `containsLocalPoint`, PixiJS's `hitArea`), the component answers with the part outlines.
- **Where drawing is clipped to the box**, limit the pointer area to the box as well. What lies outside it is not on screen.

## 2. UI states

### 2-1. The state words and their precedence

There are five state words: `normal`, `hover`, `pressed`, `disabled` and `focused`. How the host's states map onto them is the Player's (uGUI's Highlighted to `hover`, Godot's `DRAW_HOVER_PRESSED` to `pressed` outside `toggle_mode` (§5), Ren'Py's insensitive to `disabled`, and so on).

When several are set at once, pick one in the order `disabled` > `pressed` > `hover` > `focused` > `normal`.

### 2-2. A state's section

By default, one animation carries a label named after each state. Labels are lower case and match exactly.

- A state's section runs from its label's frame to the frame before **the next label**, or to the end of the animation when there is none. The section loops.
- The next label is not only a state's: it is the first label of any name in the animation (`reduced_motion`, or one unrelated to the states) on a frame after the start.
- **An ssab's labels are not in frame order** (`AnimationData.labels` is ordered by name hash). Sort them by frame before looking up the next one.
- The animation whose labels are read is, by default, the one the Player has selected.

### 2-3. When a state has no label

When a state has no label, fall back in this order.

| State | Falls back to |
|---|---|
| `hover`, `focused`, `disabled` | `normal` |
| `pressed` | `hover`, then `normal` |
| `normal` | The whole animation |

### 2-4. Overrides from code

Let code replace the default for a state with an animation name, or with a section between labels.

- An override also applies to the states that fall back to it. Override `hover`, and a `pressed` with no label plays that section too.
- Report an override that names an animation or label the pack does not have once ([Player API Conventions §7](../20_design/30_api_conventions.md#7-reporting-failure)), and treat that state as having no override.

Resolving a state's section, fallbacks of §2-3 and overrides included, in one function is recommended. If the same function also returns the reduced-motion decision (§3), every branch can be tested with neither a runtime nor a display node.

### 2-5. Switching

- When the state or the selection (§2-6) changes, play the new section from its start **at once**. The section playing is dropped.
- **The adapter owns** the section, the loop count (infinite) and play and stop. The host must not drive the same Player alongside it.
- When the same state is shown again with the same selection and the same reduced motion, do nothing. The host may show the state every frame. After the pack is loaded again, show it again.

On a host with no way to learn that a pack has loaded, the adapter shows the state again on every tick, doing nothing when the state, the pack and the animation are all unchanged. The state then appears on the tick after the pack arrives.

### 2-6. The selected state

A Player whose host's button component has a selection (a toggle's on and off) is recommended to implement the selected state: Ren'Py's styles prefixed `selected_`, Flutter's `WidgetState.selected`, uGUI's `Toggle.isOn` and the like. On a host without one (PixiJS's `@pixi/ui` `ButtonContainer`, for one), it is not implemented.

- Selection is a boolean that combines with the five states of §2-1 (stem `selected`). It is not a sixth state, because a selected button can still be hovered or disabled.
- A selected state's section comes from the label named after the state with `selected_` in front (`selected_normal`, `selected_hover` and so on). Where the section ends, and the sorting, are as in §2-2.
- When there is no such label, look up the §2-3 fallbacks with `selected_` in front first, and without it after. A selected `hover` goes `selected_hover`, `selected_normal`, `hover`, `normal`, then the whole animation: the difference between on and off is kept over the response to hover.
- A selected `pressed` is the exception: look up `pressed` right after `selected_pressed`, then the rest in the same order (`selected_pressed`, `pressed`, `selected_hover`, `selected_normal`, `hover`, `normal`, then the whole animation). Being pressed lasts only while the button is held down, and tells the user the press was taken. On release the selected section comes back, so the difference between on and off is not lost.
- Let the overrides of §2-4 be given under names such as `selected_hover` as well.

## 3. Reduced motion

Take the input as a boolean (stem `reducedMotion`). Reading the OS setting (`prefers-reduced-motion`, `MediaQuery.disableAnimations`, …) is optional, because most game hosts cannot read it.

While it is true:

- Hold the first frame of the current state's section, and when the state changes, the first frame of the new one.
- If the animation has a `reduced_motion` label, `normal` (and the states that fall back to `normal`) loops that section instead, up to the frame before the next label. The other states stay held, so pressed and disabled still look it. A section taken from a selected label (§2-6) stays held too, so on still looks it.
- Do not deliver user data, sound or other events while held: hold with `stop`, not `pause`. After a seek to the section's first frame, the next `update` still reports that frame's events ([Events §3](../30_core/30_events.md#3-which-frames-are-reported)), so drop that report. While the `reduced_motion` section plays, its events are delivered as usual.

## 4. The accessibility default

- With nothing specified, **treat the animation as decoration and do not announce it**.
- Give the adapter a way to take a label, the equivalent of an image's alternative text (what goes to Flutter's `semanticLabel` or PixiJS's `accessibleTitle`). The role (button and the like) and the actions belong to the host's component.

## 5. Host differences

- **Action callbacks are not aligned.** Their names and timing follow the host's button component: a Flutter button's `onPressed` fires on release, while Flame's `ButtonComponent` has `onPressed` on press, `onReleased` and `onCancelled`. This convention fixes only the states and their sections.
- **On a host without UI focus** (a game engine, say), `focused` is something the game sets: a menu driven by gamepad or keyboard sets it on the button it has selected. The game passes `disabled` and reduced motion in the same way.
- **When the adapter is a separate package**, the core needs a way to publish an animation's label table, to resolve the sections of §2-2.
- **On a Godot button in `toggle_mode`**, `get_draw_mode()` reports a toggle that is on as `DRAW_PRESSED` (`DRAW_HOVER_PRESSED` while hovered). On such a button, do not map that to `pressed`: read the selection from `button_pressed`.

## 6. What this convention leaves open

- A crossfade when the state switches.
- Whether reduced motion is a property of the Player or lives only in the adapter.

---
[Back to contents](../README.md)
