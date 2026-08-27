> [日本語](./10_introduction.ja.md) | **English**

# Introduction

This document is a set of guidelines for implementing a SpriteStudio animation playback Player for your own game engine or platform using `libssruntime`.

## Purpose of this document

`libssruntime` acts as the "brain" that specializes in animation computation (interpolation, time management, data expansion); it does **not** perform the actual rendering or any platform-specific processing. That design is what gives it its high portability.

This guide covers the basic architecture for implementing a Player, the order in which to implement things, the limitations to take into account, and a staged feature-verification roadmap based on `overall.sspj`.

## Intended audience

- Engineers who want to add SpriteStudio support to their own game engine
- Developers who want to build a more heavily customized Player for an existing engine such as Unity or Godot
- Developers porting to resource-constrained embedded environments or to WebAssembly

## About the sample code

The code snippets in this document are written primarily with **C++** in mind.
Because `libssruntime` provides a C-API, please adapt them to the FFI (Foreign Function Interface) syntax and memory-management model of your language - C#, Rust, Go, JavaScript (WebAssembly), and so on.

---

# Glossary

## Project files
- sspj
    - The SpriteStudio project file.
- ssae
    - An animation project file. It belongs to an sspj file.
    - One ssae holds one or more animations.
- ssce
    - A cell project file. It belongs to an sspj file.
- ssee
    - An effect project file. It belongs to an sspj file.
- ssse
    - A sound project file. It belongs to an sspj file.
- ssqe
    - A sequence project file. It holds the settings for playing the animations of several ssae files in order.
- charmap
    - A bitmap-font project file.

## Binary files
- ssab
    - The binary file for animation playback.
    - An ssab contains one ssae file plus the information of the ssce, ssee, ssse, and charmap files that this ssae uses.
- ssqb
    - The binary file for sequence playback.
    - It contains the information of an ssqe.

## SpriteStudio-specific concepts
- Cell
    - A region cut out of an image (texture). Assigning a cell to a part is what makes an image appear.
- Part
    - An element (node) that makes up an animation. Parts form a parent/child hierarchy and come in kinds such as "normal", "null", "instance", "mesh", and "effect".
- Attribute
    - A property held by a part (position, rotation, scale, opacity, user data, and so on). Its value changes over time.

## SDK and runtime terminology
- libssruntime / ssruntime
    - The C-API animation playback core library provided by SpriteStudio-SDK. It handles time management and the computation of animation data; it does not render.
- Player
    - The implementation / library that wraps `libssruntime` for a target environment such as a game engine, and actually loads images, performs rendering, fires events, and so on.
- FrameData
    - The computed attribute information (in FlatBuffers format) for every part that should be drawn on the current frame, obtained as the result of advancing `libssruntime`'s playback time (calling Update).
- ssconverter / ssconverter-cli
    - The conversion tool that reads an sspj project file and generates optimized binary files such as ssab.
- Frame skip
    - Playing back by skipping frame numbers ahead to the frame that should be displayed for a given elapsed time, when rendering cannot keep up with the configured FPS.
    - Other tools and environments sometimes call this "Time-Based Animation" or "variable frame rate" playback.
    - `libssruntime` provides this feature - computing the appropriate frame from a delta time - as standard.
- Sub-frame
    - A feature of `libssruntime` that interpolates and draws the state *between* integer frames.
    - Internally the frame number is managed as a floating-point value, so animation stays smooth even at slow playback speeds or on high-refresh-rate displays.

---
[Back to Table of Contents](../README.md)
