> [日本語](./10_overview.ja.md) | **English**

# Overview

This guide explains how to implement a SpriteStudio Player for any game engine or platform using `libssruntime`.

## 1. Audience

- Engineers who embed SpriteStudio playback into their own engine or an existing engine (Unity, Godot and so on)
- Engineers who port to embedded environments or to WebAssembly
- AI agents assigned to this work

## 2. Division of work between Brain and Player

`libssruntime` (the Brain from here on) is a library that computes animation state. It does not load files, draw, or play sounds. The Player does those things. Gathering the computation in the Brain makes detailed rules such as interpolation and parts color give the same result in every Player, and lets the Player concentrate on bridging to the engine.

| Area | Brain | Player |
| :--- | :--- | :--- |
| Data | Expands the parts needed for playback from the ssab it is given | Loads the ssab and passes it to the Brain. Reads metadata such as part names and texture paths directly from the ssab with FlatBuffers |
| Time | Derives the frame number from the elapsed time, and manages looping and playback state | Passes the elapsed time every frame |
| Values | Interpolates between keyframes, and applies inheritance from parent to child | — |
| Transform | Outputs each part's world matrix and local vertices | Draws in the engine's coordinate system |
| Drawing | Outputs the draw plan (`DrawBatch`) | Loads textures and issues draw calls |
| Events | Lists the events passed on that frame | Converts them into the engine's callbacks or signals and notifies |
| Sound | Issues playback instructions as events | Loads audio files and plays them |

## 3. About the code examples

The code examples are written in C++ that calls the C-API (`ssruntime.h`). If you use another language, adapt them to that language's FFI. Where an example uses a function that does not exist, the example says so next to it.

The WebAssembly bindings (`wasm-bindgen`) map roughly one-to-one onto the C-API, with the `ss_runtime_` prefix removed from the names. A function that returns a pointer splits into two, `_buf` and `_len` (for example `get_world_matrices_buf` / `get_world_matrices_len`).

---
[Back to contents](../README.md)
