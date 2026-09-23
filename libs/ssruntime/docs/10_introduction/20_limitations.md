> [日本語](./20_limitations.ja.md) | **English**

# Limitations

This is the list of technical constraints of the Brain and ssab that affect Player design. Read it before you start implementing. The detailed explanation of each item, and how to deal with it, are at the link at the end of the item.

## 1. The Player implements text and sound

The Brain neither draws text nor plays sound. It passes only the data in the ssab, plus the draw order and timing.

| Feature | What the Brain passes | What the Player implements |
| --- | --- | --- |
| Text | The settings in the ssab (`PartTypeText`) and the bitmap-font glyph table (`FontBitmap` / `FontGlyphBitmap`). A `DrawBatch` with no geometry that marks the text's position in the draw order | Glyph layout, rasterization of outline fonts, and all of the drawing |
| Sound | Events that carry the playback instructions (sound name, play count, timing) | Loading and playing the audio |

Text has these further constraints:

- A text part authored as a mask (`draw_as_mask`) is left out of the draw plan. It draws nothing and does not work as a mask either.
- Kerning is not stored in the ssab.
- Bitmap-font glyphs are looked up by a 16-bit character code, so characters above U+FFFF are not supported.

→ [9-Slice, Shape and Text §3](../40_parts/30_special_parts.md#3-text) / [Audio](../50_features/50_audio.md)

## 2. WebAssembly comes in SIMD and non-SIMD builds

The standard web build uses WebAssembly SIMD, so it does not load in runtimes that do not support SIMD. For those environments (such as the NW.js bundled with RPG Maker MZ), we distribute a build that does not use SIMD (`runtime-web-nosimd`). The JavaScript interface is the same; you replace only the `.wasm`.

## 3. Independent-playback parts do not follow reverse playback or seeking

Instance parts and effect parts with independent playback (`independent=true`) advance on their own elapsed time, not on the parent's frame. Their own clock does not run backwards, and their accumulated time at an arbitrary parent frame is not uniquely determined, so they are not drawn correctly under the parent's reverse playback or seeking. They do follow forward frame skips.

If you use an animation that contains independent playback with reverse playback or seeking, tell your Player's users that those children are drawn correctly only during forward playback.

→ [Instance Parts §3](../40_parts/50_instance.md#3-independent-playback)

## 4. Seeking does not fire the events of the frames in between

When you move the playback position with `ss_runtime_set_frame_no`, the events (user data, signal, audio) between the origin and the destination are not fired.

→ [Events §3](../30_core/30_events.md#3-which-frames-are-reported)

## 5. The Brain does not validate the ssab

For processing speed and binary size, the Brain does not validate the structure of the input ssab. Passing a corrupted or crafted ssab can make it read out-of-bounds memory and crash. If your Player receives an ssab from the network or from user input, validate it before passing it to the Brain.

→ [Loading & Initialization §2](../30_core/10_initialization.md#2-validating-an-ssab-before-passing-it)

## 6. The Player keeps the bytes of a borrowed ssab alive

A resource created with `ss_resource_create_borrow` refers directly to the ssab bytes in the Player's memory. Do not free those bytes until the resource itself, and every Context and effect slot that refers to it, has been destroyed.

Resources and Contexts can be destroyed in any order. A resource is reference-counted, and calling `ss_resource_destroy` while a Context has it bound does not free it.

→ [Loading & Initialization §1](../30_core/10_initialization.md#1-resource-and-context)

## 7. ssab in embedded mode

The mode that embeds assets in the ssab has these constraints:

- Assets are stored keyed by the file paths written in the `sspj` and `ss*e` files. If the same file path appears more than once, the keys collide, so make sure no asset path in the project is duplicated.
- Only the assets the animation uses are embedded. Another ssab referenced as an external instance is not embedded, so supply that ssab file separately.

→ [External Resources and Textures §4](../50_features/30_external_resources.md#4-referencing-external-ssabs)

## 8. Animation mixing

- Only animations within the same ssab can be stacked.
- Discrete attributes such as cell, parts color and hide follow track 0.
- The phases of sources with different lengths are not synchronized.
- There is no per-part mask. The range a track affects is the range its animation has keys on.

→ [Animation Mixing §9](../50_features/10_animation_mixing.md#9-constraints)

## 9. Audio cannot play during backward playback

Many audio APIs do not support playing audio in reverse, so do not play sounds while playback is moving backward. The events themselves arrive whatever the direction.

→ [Audio §2](../50_features/50_audio.md#2-conditions-for-playing)

---
[Back to contents](../README.md)
