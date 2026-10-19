> [日本語](./30_glossary.ja.md) | **English**

# Glossary

These are the terms used in this guide. Each concept has exactly one term. A term that is explained in detail elsewhere links to the page that explains it.

## 1. Files

| Term | Meaning |
| :--- | :--- |
| sspj | The SpriteStudio project file. It bundles the `ss*e` files below |
| ssae | An animation file. One ssae holds one or more animations |
| ssce | A cell map file |
| ssee | An effect file |
| ssse | A sound file |
| ssqe | A sequence file. It holds the settings for playing several animations in order |
| charmap | A bitmap-font definition file (`.fnt`) |
| ssab | The playback binary that ssconverter generates from one ssae. It includes the cell map, effect, sound and charmap information that the ssae uses. Its schema is FlatBuffers |
| ssqb | The sequence binary that ssconverter generates from an ssqe |

## 2. SpriteStudio concepts

| Term | Meaning |
| :--- | :--- |
| Part | An element that makes up an animation. Parts form a parent-child hierarchy and have a type (normal, null, mesh, mask, shape, 9-slice, text, instance, effect, audio and others) |
| Cell | A rectangular region cut out of a texture. Assigning a cell to a part displays that image |
| Cell map | One texture and the list of cells cut out of it |
| Attribute | A value a part holds (position, rotation, scale, opacity, user data and so on). It changes over time through keyframes |
| Draw priority | The value that decides the order in which parts are drawn. The smaller the value, the earlier (further back) the part is drawn. → [Mask §2-1](../40_parts/40_mask.md#2-1-scope-which-drawn-parts-a-mask-reaches) |
| Child animation | Another animation that an instance part references and plays. → [Instance Parts](../40_parts/50_instance.md) |
| Independent playback | A setting of instance parts and effect parts. The part advances on its own elapsed time rather than on the parent's frame. → [Instance Parts §3](../40_parts/50_instance.md#3-independent-playback) |

## 3. Brain and Player

| Term | Meaning |
| :--- | :--- |
| Brain | `libssruntime`. It computes animation state and outputs the draw plan. → [Overview §2](./10_overview.md#2-division-of-work-between-brain-and-player) |
| Player | The implementation that wraps the Brain and does loading, drawing, event notification and sound playback on the engine |
| ssconverter | The converter that reads an sspj and generates ssab. It comes as a library (`libssconverter`) and a CLI (`ssconverter-cli`) |
| Resource | Read-only data created by loading an ssab. A resource handle in the C-API, `AnimationResource` in Rust. Several Contexts can share it. → [Loading & Initialization §1](../30_core/10_initialization.md#1-resource-and-context) |
| Context | The playback state and computation buffers for one animated object. You use it by binding a resource to it |
| FrameData | The FlatBuffers buffer that the Brain outputs every frame. It contains the part states, the draw plan, parts color and more |
| PartData | The static definition of a part stored in the ssab (the ssab's `parts`). It does not change from frame to frame |
| SoA buffer | A buffer that lays out the values of all parts in a single array, such as world matrices and vertices. You read it directly through a pointer. → [Render Loop §1](../30_core/40_render_loop.md#1-soa-buffers) |
| DrawBatch | A group of parts that can be drawn in a row with the same texture and the same blend mode. The Player can draw one DrawBatch with one draw call. → [Render Loop §2](../30_core/40_render_loop.md#2-drawing-with-drawbatch) |
| `draw_order` / rank | `draw_order` is the list of part indices in the order they are drawn. A rank is a position in that list |
| Slot | The child playback state that the Player holds for each instance part and effect part. You operate it with `ss_instance_slot_*` / `ss_effect_slot_*` |
| Playhead | The current playback position (frame number). It is fractional during sub-frame playback |
| Tick | One update a Player makes in one host frame: `ss_runtime_update` advances time and `ss_runtime_get_frame_data` builds the frame. It can be skipped while nothing moves. → [Animation Mixing §1-6](../50_features/10_animation_mixing.md#1-6-skipping-ticks-when-nothing-moves) |
| Sub-frame | Interpolating and drawing the state between integer frames. → [Playback Control §6](../30_core/20_playback_control.md#6-sub-frames) |
| Frame skip | When drawing cannot keep up, advancing by skipping frames to match the elapsed time |
| Track / source | The units of animation mixing. A track is a layer on which poses are stacked; a source is one animation placed on a track. → [Animation Mixing](../50_features/10_animation_mixing.md) |
| Primary | Track 0's main animation. Discrete attributes, the default events and the playback state come from it. → [Animation Mixing §1-1](../50_features/10_animation_mixing.md#1-1-track-0-is-special) |
| Override | The feature that overrides a part's color, cell and visibility at runtime, taking priority over keyframes. → [Part Overrides](../50_features/20_part_overrides.md) |
| Rebuild request | A one-shot mark a Player that skips ticks raises after a call that changes what the next frame shows without moving the clock, withdrawn when it next builds a frame. → [Part Overrides §5-1](../50_features/20_part_overrides.md#5-1-caching-by-frame-number-blocks-overrides) |
| writer | A part that writes into the stencil (or the coverage). Pure masks (such as mask parts) and drawing parts with clipping are writers. → [Mask §1](../40_parts/40_mask.md#1-concepts-and-part-roles) |

---
[Back to contents](../README.md)
