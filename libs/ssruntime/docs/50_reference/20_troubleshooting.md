> [日本語](./20_troubleshooting.ja.md) | **English**

# Troubleshooting

Common problems that occur while implementing a Player or during the build, and how to solve them. At present this covers platform-specific problems at build / link time. Runtime troubles and other categories will be added here in future.

## Link errors on Windows (MSVC)

### LNK2019 / LNK2001: unresolved external symbol

**Symptom**: linking fails with `LNK2019` / `LNK2001` (unresolved external symbol) when using `libssconverter.lib` or `libssruntime.lib` in a Windows C++ project.

**Cause**: the Windows API symbols that the Rust standard library references internally are not resolved when the final binary is linked.

**Fix**: add the following standard Windows libraries to your project's link settings.

- **Userenv.lib**
- **Bcrypt.lib**
- **Ntdll.lib**
- **Ws2_32.lib**

Examples per build environment:

- **Visual Studio**: add the four above under Project Properties → Linker → Input → Additional Dependencies
- **SCons**: `env.Append(LIBS=["Userenv", "Bcrypt", "Ntdll", "Ws2_32"])`
- **CMake**: `target_link_libraries(<target> PRIVATE Userenv Bcrypt Ntdll Ws2_32)`

### LNK2005 / LNK1169: duplicate symbols (linking `ssruntime.lib` and `ssconverter.lib` together)

**Symptom**: linking both static libraries into the same final binary (for example the Windows build of a Godot custom module / GDExtension) fails with `link.exe` reporting `LNK2005` / `LNK1169`.

```
ssruntime.windows.editor.x86_64.lib(...) : error LNK2005: ___rust_alloc is already defined in ssconverter.windows.editor.x86_64.lib(...).
godot.windows.editor.x86_64.exe : fatal error LNK1169: one or more multiply defined symbols found.
```

**Cause**: each Rust `staticlib` embeds a self-contained runtime (allocator shims, panic-related symbols), so `__rust_alloc` / `rust_begin_unwind` / `rust_eh_personality` and others are duplicated across both `.lib` files. The MSVC linker treats duplicate symbols as errors (macOS's `ld` demotes them to warnings and links successfully, which is why this surfaces only on Windows).

**Fix**: add `/FORCE:MULTIPLE` to the link flags of the final binary.

```python
# SCons example (Godot custom module / GDExtension)
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006"])
```

- `/FORCE:MULTIPLE`: demotes `LNK2005` to a warning (`LNK4006`) and continues linking
- `/IGNORE:4006`: suppresses that warning as well (optional)

#### When `/WX` is also in use (working around `LNK4088`)

If godot-cpp or something similar enables `/WX` (warnings as errors), the `LNK4088` warning ("image being generated due to /FORCE option") that `/FORCE:MULTIPLE` always produces is promoted to `LNK1218` and the build fails.

```
warning LNK4088: image being generated due to /FORCE option...
LINK : error LNK1218: warning treated as error.
```

`LNK4088` is on the list of warnings that MSVC cannot suppress with `/IGNORE`, so **`/IGNORE:4088` has no effect**. Pass `/WX:NO` to the linker instead, to cancel warnings-as-errors at link time.

```python
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006", "/WX:NO"])
```

`/WX:NO` is a **linker-only** cancellation flag; it does not affect `/WX` on the compiler side (`CCFLAGS` / `CXXFLAGS`). Warnings in your source code continue to be detected as errors.

> [!IMPORTANT]
> Once `/WX:NO` is in place, **every subsequent link-time warning passes through silently**. Add this flag last, after you have resolved all link errors (`LNK2005` and so on). If you add `/WX:NO` first, other important warnings may be buried.

#### Note: effect on behavior

Because the workspace `Cargo.toml` specifies `panic = "abort"`, `rust_begin_unwind` / `rust_eh_personality` are effectively no-ops, and the allocator shims are functionally equivalent since both call MSVCRT's `malloc` / `free`. Whichever definition is adopted, the binary behaves the same.

## Link warnings on macOS (ld)

### duplicate symbol warnings (linking `libssruntime.a` and `libssconverter.a` together)

**Symptom**: linking both static libraries into the same final binary (for example the macOS build of a Godot custom module / GDExtension) makes `ld` emit duplicate symbol warnings. **The link itself succeeds** and the resulting binary works correctly.

```
ld: warning: duplicate symbol '___rust_alloc' in:
    .../libssconverter.a[arm64](...)
    .../libssruntime.a[arm64](...)
```

**Cause**: the same as the Windows [LNK2005 / LNK1169 section](#lnk2005--lnk1169-duplicate-symbols-linking-ssruntimelib-and-ssconverterlib-together) - both staticlibs contain a duplicate copy of the Rust runtime.

**Fix**: **none needed** - `ld` adopts the first definition it finds and the link succeeds. For the effect on behavior see [Note: effect on behavior](#note-effect-on-behavior) above. If you want to suppress the warning log, filtering the build log with `grep -v "duplicate symbol"` is a realistic approach.

## Common problems during implementation and at runtime

### Native pointer lifecycle vs. parallel processing (crashes / segfaults)

**Symptom**: when you dispose a native pointer such as `SsPlayerHandle` managed from a high-level language like C# (Unity), or when the application exits, the whole engine terminates with an access violation (segfault).

**Cause**: if you hand a native pointer or data buffer to an asynchronous job on another thread (a worker), such as Unity's Burst Job System, and then release that pointer on the main thread with `ss_player_release` or similar, you get a race.

**Fix**: before disposing a native pointer (or before the next frame's update), always synchronize on the completion of any asynchronous job that references that pointer (for example by calling `JobHandle.Complete()`), and only then run `Dispose` or the release path, in a state that is guaranteed to be safe.

### Animations do not switch / playback breaks silently (hash mismatch)

**Symptom**: no error appears at runtime, but a different animation plays, or an instance (sub-animation) part's reference is not resolved and nothing is displayed.

**Cause**: this happens when you **reimplement** FNV-1a or a similar algorithm yourself in the target language (C# and so on) to generate the hash value (`uint32`) from an animation or part name, and the result differs from the output of Rust's `ss_runtime_hash_string` by even one bit.

**Fix**: **never implement string hashing yourself - always call `ss_runtime_hash_string(const char* s)`, which is provided over the FFI.**

---
[Back to Table of Contents](../README.md)
