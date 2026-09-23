> [日本語](./10_troubleshooting.ja.md) | **English**

# Troubleshooting

Problems that often occur while building a Player, and their causes.

## 1. Find by symptom

| Symptom | Cause | Where it is explained |
| :--- | :--- | :--- |
| The animation does not move | `ss_runtime_play` is not called. Setup alone does not make `update` advance it | [Playback Control §2](../30_core/20_playback_control.md#2-setup-and-play) |
| The animation does not advance past the start / a finite loop never ends | The same animation is set up every frame | [Player API Conventions §8-2](../20_design/30_api_conventions.md#8-2-do-nothing-when-the-selected-animation-is-specified-again) |
| Resuming from a pause jumps back to the start | `play` is called instead of `resume` | [Playback Control §4-1](../30_core/20_playback_control.md#4-1-what-each-operation-means) |
| The end notification arrives every frame / never arrives | The rising edge of `is_finished` is not detected | [Playback Control §7](../30_core/20_playback_control.md#7-detecting-loops-and-the-end) |
| The animation stops in an environment where one update advances less than one frame | The frame number held as state is rounded and written back | [Playback Control §6-2](../30_core/20_playback_control.md#6-2-do-not-discard-the-fraction) |
| Every part vanishes for an instant at the loop boundary | An out-of-range frame number is passed to `ss_runtime_get_frame_data` | [Playback Control §6-1](../30_core/20_playback_control.md#6-1-accepted-frame-numbers) |
| Events or sounds on frame 0 sometimes do not occur | Event processing is gated on whether the frame is drawn | [Events §3-1](../30_core/30_events.md#3-1-events-on-the-start-frame) |
| Parts with a texture change are not drawn | `external_textures()` is not loaded | [Loading and Initialization §3](../30_core/10_initialization.md#3-loading-textures) |
| The image is upside down / flipped twice | The coordinate system setting and a flip in the Player are both applied | [Player Architecture §5](../20_design/20_architecture.md#5-coordinate-system) |
| An effect or child animation lags / is frozen after being shown again | The child's clock is advanced from `draw_order` | [Instance Parts §2](../40_parts/50_instance.md#2-advancing-the-child-with-the-slot) |
| Independent-playback effects or children do not start when playback restarts | Slots are not initialized on `play` | [Instance Parts §6](../40_parts/50_instance.md#6-restarting-playback) |
| Child animations or effects do not fade out with the parent's fade | The opacity of the instance part or effect part is not applied | [Instance Parts §5](../40_parts/50_instance.md#5-drawing-the-child) / [Effect Parts §3](../40_parts/60_effect.md#3-drawing-the-effectdrawplan) |
| Clipping does not work | The writers are decided from the `Mask` DrawBatches alone | [Mask §3](../40_parts/40_mask.md#3-what-to-read-from-framedata) |
| Where masks overlap, the result changes with the drawing order | The same bit is used for add and invert | [Mask §4-4](../40_parts/40_mask.md#4-4-separate-bit-planes-per-write-op-required) |
| Masks of another animation on the same frame are wrong | The clipping is not undone | [Mask §4-2](../40_parts/40_mask.md#4-2-restoring-the-stencil) |
| Tinting comes out doubly dark | The parts color alpha is multiplied by the opacity again | [Parts Color §3](../40_parts/10_parts_color.md#3-reading-the-parts-color) |
| Overrides seem to have no effect | Frames are cached by frame number | [Part Overrides §5-1](../50_features/20_part_overrides.md#5-1-caching-by-frame-number-blocks-overrides) |
| Destroying crashes | The Context is destroyed while the render thread reads the SoA buffers | [Render Loop §4-1](../30_core/40_render_loop.md#4-1-multi-threaded-drawing-and-destruction) |
| An animation or part specified by name is not found | You compute string hashes yourself, and the value differs from the Brain's | §2 |

## 2. Name hashes do not match

**Symptom**: no error appears, but a different animation plays, or an instance part's reference is not found and nothing is drawn.

**Cause**: the hash (`uint32`) of an animation or part name is computed by **implementing FNV-1a or similar yourself** in the target language, and the result differs by even one bit from the Brain's `ss_runtime_hash_string`.

**Fix**: do not implement string hashing yourself; **always call `ss_runtime_hash_string(const char* s)`** to make hashes.

## 3. Link errors on Windows (MSVC)

### 3-1. LNK2019 / LNK2001: unresolved external symbol

**Symptom**: when you use `libssconverter.lib` or `libssruntime.lib` in a Windows C++ project, linking reports `LNK2019` / `LNK2001`.

**Cause**: the Windows API symbols the Rust standard library uses are not resolved when the final binary is linked.

**Fix**: add the following standard Windows libraries to your link settings.

- **Userenv.lib**
- **Bcrypt.lib**
- **Ntdll.lib**
- **Ws2_32.lib**

- **Visual Studio**: add the four above under Project Properties → Linker → Input → Additional Dependencies
- **SCons**: `env.Append(LIBS=["Userenv", "Bcrypt", "Ntdll", "Ws2_32"])`
- **CMake**: `target_link_libraries(<target> PRIVATE Userenv Bcrypt Ntdll Ws2_32)`

### 3-2. LNK2005 / LNK1169: duplicate symbols (linking both `ssruntime.lib` and `ssconverter.lib`)

**Symptom**: linking both static libraries into the same binary (the Windows build of a Godot module or GDExtension, for example) makes `link.exe` fail with `LNK2005` / `LNK1169`.

```
ssruntime.windows.editor.x86_64.lib(...) : error LNK2005: ___rust_alloc is already defined in ssconverter.windows.editor.x86_64.lib(...).
godot.windows.editor.x86_64.exe : fatal error LNK1169: one or more multiply defined symbols found.
```

**Cause**: each Rust static library contains the Rust runtime (the allocator and panic functions), so `__rust_alloc` / `rust_begin_unwind` / `rust_eh_personality` and others appear in both `.lib` files. The MSVC linker treats duplicates as errors (macOS's `ld` demotes them to warnings and links anyway, so this happens only on Windows).

**Fix**: add `/FORCE:MULTIPLE` to the link options of the final binary.

```python
# SCons example (Godot module / GDExtension)
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006"])
```

- `/FORCE:MULTIPLE`: demotes `LNK2005` to a warning (`LNK4006`) and continues linking
- `/IGNORE:4006`: suppresses that warning too (optional)

#### When you use `/WX` (`LNK4088`)

If godot-cpp or similar enables `/WX` (treat warnings as errors), the `LNK4088` warning ("image being generated due to /FORCE option") that `/FORCE:MULTIPLE` always produces becomes `LNK1218`, and the build fails.

```
warning LNK4088: image being generated due to /FORCE option...
LINK : error LNK1218: warning treated as error.
```

`LNK4088` is one of the warnings MSVC cannot suppress with `/IGNORE`, so **`/IGNORE:4088` has no effect.** Instead, pass `/WX:NO` to the linker so that warnings are not treated as errors at link time only.

```python
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006", "/WX:NO"])
```

`/WX:NO` affects **the linker only**, so `/WX` for the compiler (`CCFLAGS` / `CXXFLAGS`) stays as it is. Warnings in your source code are still treated as errors.

> [!IMPORTANT]
> Once you add `/WX:NO`, **every link warning passes through.** Resolve all link errors (`LNK2005` and so on) first, and add this option last.

#### Effect on behavior

The libraries are built with `panic = "abort"`, so `rust_begin_unwind` / `rust_eh_personality` are never actually used, and both allocators call MSVCRT's `malloc` / `free`, so they behave the same. Whichever definition is used, the behavior does not change.

## 4. Link warnings on macOS (ld)

### 4-1. Duplicate symbol warnings (linking both `libssruntime.a` and `libssconverter.a`)

**Symptom**: linking both static libraries into the same binary makes `ld` emit duplicate symbol warnings. **The link succeeds**, and the resulting binary works correctly.

```
ld: warning: duplicate symbol '___rust_alloc' in:
    .../libssconverter.a[arm64](...)
    .../libssruntime.a[arm64](...)
```

**Cause**: the same as §3-2.

**Fix**: `ld` completes the link with the first definition it finds, so **you do not need to do anything.** As described in §3-2, there is no effect on behavior. If the warnings bother you, filtering them out of the build log with `grep -v "duplicate symbol"` is enough.

---
[Back to contents](../README.md)
