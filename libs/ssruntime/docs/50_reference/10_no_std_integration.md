> [日本語](./10_no_std_integration.ja.md) | **English**

# Integrating into no_std Embedded Environments

`libs/ssruntime` **fully supports `#![no_std]`** for embedded environments without a filesystem or threading, for game consoles (PS5, Nintendo Switch, and so on), and for game engines that use their own memory allocator.

**Every binary the SDK distributes officially (iOS / Android / macOS / Linux / Windows) is a `no_std` build**, which guarantees bit-identical computation results across platforms. The `std` feature exists only for developer experience during development, such as IDE completion and running tests.

---

## 1. Overview of the build configuration

| Feature | Default | Role |
|---|---|---|
| `ffi` | ✅ | Exports the C-API |
| `libc_alloc` | — | A `#[global_allocator]` based on `libc::malloc/free` |
| `wasm_alloc` | — | A `#[global_allocator]` based on `dlmalloc` for pure Wasm |
| `panic-handler` | — | Provides a `#[panic_handler]` (calls `libc::abort` or Wasm `unreachable` on panic) |
| `std` | — | Puts `flatbuffers` / `ssab` into std mode and adds `ssconverter` as a dependency for running tests |
| `web` | — | The `wasm-bindgen` bindings (this enables `wasm-bindgen`'s own `std` feature) |

> [!NOTE]
> Enabling the `web` feature links `wasm-bindgen`, which is required to run in a browser, and with it Rust's `std` provides the allocator and panic handler. However, the computation inside `ssruntime` (the math functions and so on) continues to use the `libm` crate, so determinism (bit-identical results) with other platforms is preserved.

Representative commands per environment:

```bash
# no_std staticlib for mobile / console / desktop (recommended)
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"

# no_std build for Wasm (browser / WebWorker / wasm-bindgen)
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "web"

# no_std build for pure Wasm (JS-independent; for WASI and embedded Wasm engines)
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"

# Embedding as an rlib in Rust natively (when the host provides the allocator / panic_handler)
cargo build -p ssruntime --release
```

---

## 2. Using it from C/C++ (for consoles / `no_std`)

When calling `ssruntime` over FFI from a C/C++ game engine, you can avoid the overhead of Rust's standard library (`std`) and errors from unsupported APIs.

To build for an environment where "`std` is unavailable but the C standard library (`malloc` / `free`) can be used" - typical console and mobile environments:

```bash
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"
```

This produces a static library (`.a` or `.lib`) that stays `no_std` while falling back to C's `malloc` for memory allocation, with a panic handler injected. On a panic, `libc::abort()` is called and the process terminates safely (or breaks into the debugger).

To compile `ssruntime` for an embedded pure-Wasm environment that depends on neither WASI nor JS, specify `wasm_alloc` as below. `dlmalloc` is linked internally.

```bash
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"
```

---

## 3. Delegating debug log output (log callback)

In a `no_std` environment, Rust's standard output (`println!`, `eprintln!`) is unavailable.
To receive error and warning logs from inside `ssruntime`, you must therefore register a log callback at initialization.

```cpp
#include "ssruntime.h"

// The callback that receives logs
void my_log_callback(int level, const char* message) {
    // e.g. printf("libssruntime [%d]: %s\n", level, message);
    // Forward it to your game engine's logging function (UE_LOG, Debug.Log, print_line, and so on)
}

// Register the callback at the very start of initialization
ss_runtime_set_log_callback(my_log_callback);
```

`level` is one of `SS_RUNTIME_LOG_ERROR` (1), `SS_RUNTIME_LOG_WARN` (2) or
`SS_RUNTIME_LOG_INFO` (3). Only `ERROR` is emitted today; the other two are
reserved so that a `switch` written now does not go stale.

Every C-API call that records a failure also emits one `ERROR` line, prefixed
with the entry point that failed (`ss_runtime_setup_animation: Animation not
found`). That is the same failure `ss_runtime_get_last_error` reports — the
log is for the caller that does not check a return value, not a second source of
truth. There is no cost when no callback is registered: the message is not
formatted at all.

---

## 4. Using it from Rust natively

When calling `ssruntime` from your own Rust game engine (a `no_std` application), arranging the allocator and panic handler is normally the job of the top-level executable.

Add the following to `Cargo.toml`. Nothing is on by default, so this is already the pure `no_std` library (`rlib`) with no allocator feature and no C-API exports.

```toml
[dependencies]
ssruntime = "7.0.0-beta.1"
```

> [!NOTE]
> Required functionality such as `Vec` works internally through the `alloc` crate. The final target binary must provide some `#[global_allocator]` and `#[panic_handler]`.

---

## 5. Using a fully custom allocator (Unreal Engine and similar)

If you want to "use neither C's standard `malloc` nor anything else, only the dedicated memory functions the game engine provides (Unreal Engine's `FMalloc`, a memory pool, and so on)", you can inject a custom allocator from the C/C++ side simply by enabling the `custom_alloc` feature - no wrapper crate needed.

### Implementation steps

1. When building `ssruntime`, specify `custom_alloc` instead of `libc_alloc`. Since what you ship is the **staticlib (`libssruntime.a` / `.lib`)**, build with the staticlib crate type explicitly.

```bash
cargo rustc -p ssruntime-ffi --release --features "custom_alloc,panic-handler" --crate-type staticlib
```

> [!IMPORTANT]
> With `custom_alloc`, the `ss_custom_*` symbols and C runtime functions such as `memcpy` are **left unresolved on the ssruntime side** and are resolved when finally linking against the engine. Therefore running a plain `cargo build -p ssruntime-ffi`, whose `crate-type` includes `cdylib`, fails at the `cdylib` link stage on macOS / Windows (MSVC), whose linkers do not permit unresolved symbols. Build with `--crate-type staticlib` explicitly as above (the artifact is `target/release/libssruntime.a`).

2. In the C/C++ project you link against (Unreal Engine, for example), define the following three `extern "C"` functions. In addition to these, C runtime functions such as `memcpy` / `memset` / `memmove` / `memcmp` / `strlen` are also required at final link time, but the game engine's CRT normally provides them (on bare metal, supply them separately).

```cpp
#include <stddef.h>
#include <stdlib.h>
#if defined(_MSC_VER)
#include <malloc.h> // _aligned_malloc / _aligned_free
#endif

extern "C" {

// 1. Memory allocation
//    The returned pointer must always be aligned to an `align`-byte boundary
//    (`align` is always a power of two). Plain malloc guarantees only max_align_t
//    (usually 8-16B), so an over-alignment request such as a SIMD type would be
//    undefined behavior. ssruntime does not correct alignment itself, so always
//    use an aligned allocation.
void* ss_custom_alloc(size_t size, size_t align) {
    // Example: for Unreal Engine
    // return FMemory::Malloc(size, align);
#if defined(_MSC_VER)
    return _aligned_malloc(size, align);
#else
    // posix_memalign requires align to be a multiple of sizeof(void*), so raise it
    if (align < sizeof(void*)) {
        align = sizeof(void*);
    }
    void* ptr = NULL;
    if (posix_memalign(&ptr, align, size) != 0) {
        return NULL; // return NULL on allocation failure (the Rust side detects it)
    }
    return ptr;
#endif
}

// 2. Memory deallocation
//    The same size / align used for the allocation are passed in. A pool allocator
//    that needs the size on free can use them here (malloc-family code may ignore them).
//    Note: use the free function that pairs with your allocation function. In particular,
//    MSVC requires _aligned_free; calling free on memory from _aligned_malloc is
//    undefined behavior.
void ss_custom_free(void* ptr, size_t size, size_t align) {
    (void)size;
    (void)align;
    // Example: for Unreal Engine
    // FMemory::Free(ptr);
#if defined(_MSC_VER)
    _aligned_free(ptr);
#else
    free(ptr);
#endif
}

// 3. Panic handler (abort processing)
// Note: this must be defined as noreturn
//   (returning violates the Rust side's `-> !` contract and is undefined behavior)
#if defined(_MSC_VER)
__declspec(noreturn)
#elif defined(__GNUC__) || defined(__clang__)
__attribute__((noreturn))
#endif
void ss_custom_abort() {
    // Example: log a fatal error with UE_LOG and crash
    // UE_LOG(LogTemp, Fatal, TEXT("ssruntime panic"));
    abort();
}

}
```

By defining these functions on the C/C++ side and linking the generated `libssruntime.a` (or `.lib`), you integrate seamlessly with the game engine's allocator without touching the Rust source at all.

> [!NOTE]
> `ss_custom_abort` is called only when a **panic** occurs on the Rust side. If `ss_custom_alloc` returns `NULL` (allocation failure), Rust aborts immediately through its alloc error handler and does not go through `ss_custom_abort`. If you want custom handling on allocation failure, handle it inside `ss_custom_alloc`.
>
> Note also that `custom_alloc` cannot be enabled together with `libc_alloc` / `wasm_alloc` / `emscripten_alloc`, because their `#[global_allocator]`s conflict (this is detected with `compile_error!`).

---

## 6. Behavioral differences between `std` and `no_std` builds

`ssruntime` builds from the same source for both `std` and `no_std`, but **the implementation switches in a few low-level places**. When implementing a Player, base your thinking on the `no_std` behavior, which is what ships.

### 6.1 Math functions

The implementations of `sin` / `cos` / `tan` / `atan2` / `abs` / `round` / `floor` / `fmod` / `sqrt` switch between `std` and `no_std`.

| | std | no_std |
|---|---|---|
| Implementation | `f.sin()`, `f.cos()`, … (`std::f32` methods) | `libm::sinf`, `libm::cosf`, … (the `libm` crate, a pure Rust implementation) |
| Underneath | The platform libm (macOS's Accelerate, Linux's glibc, and so on) | A Rust port of Sun's fdlibm |

The trigonometric functions may return results that **differ by a few ULP**. `sqrt` / `round` / `floor` / `abs` are usually implemented as hardware instructions and almost always match exactly between the two.

Practical impact:
- The animation looks the same to the human eye
- Bit-level comparisons such as hash comparisons may differ
- **The distributed binaries are `no_std` (unified on libm) on every platform, so results are bit-identical across platforms**

### 6.2 Panic handling

| | std | no_std + `panic-handler` |
|---|---|---|
| On panic | Prints a message and backtrace to stderr → abort | Calls `libc::abort()` immediately (no message) |

Because the workspace `Cargo.toml` sets `panic = "abort"`, unwinding never occurs in either case. In a `no_std` build **the panic reason is not left in any log**, so even if the Player catches SIGABRT, identifying the cause is difficult.

### 6.3 Global allocator

| | std | no_std + `libc_alloc` |
|---|---|---|
| Implementation | Rust's System allocator | `LibcAlloc` (provided by the `libc_alloc` feature) |
| Underneath | `libc::malloc/free` (macOS/Linux/iOS/Android) / MSVCRT (Windows) | The same |

On the major platforms **both go through libc, so they are effectively identical**.

---

## 7. Notes for Player implementers

### 7.1 Be aware that the `ssruntime` you link is `no_std` (going through libm)

Every binary the SDK distributes officially is a `no_std` build. The premises a Player should assume:

- Trigonometric results come from the `libm` crate (not the platform libm)
- No log is emitted on panic
- The global allocator calls libc malloc directly

As a result, **animation output is bit-identical across iOS / Android / Windows / macOS / Linux** (an exact match on every platform).

### 7.2 Numbers from a `std` build are "for reference"

If you build with the `std` feature for developer experience (IDE completion, running tests), the math functions go through the platform libm, so trigonometric results may differ by a few ULP from the `no_std` artifacts that ship.

- Numbers verified in a `std` build may differ from production (`no_std`) by a few ULP
- Do the final numerical verification **on real hardware with a `no_std` build**

### 7.3 Do not swallow panics in the Player layer

`ssruntime` uses `panic = "abort"` and aborts without a message even in `no_std` builds. If the Player layer traps `signal(SIGABRT)` and ignores it, **you can never trace where it died**.

- **Validate the arguments you pass on the Player side** in advance (`bind_resource` after `Context::new()`, the frame range for `setup_animation`, and so on)
- Many `ssruntime` APIs return `Result<_, SsError>`, so do not silently ignore errors - forward them to the Player's logger
- Always register `ss_runtime_set_log_callback` from section 3

### 7.4 Game logic that depends on floating-point determinism

If your design compares hashes of animation results for network synchronization / replays / lockstep:

- **Within the same SDK version, results are bit-identical on every platform** (thanks to unifying on libm)
- **Across SDK versions, a libm update may introduce differences**
- If you compare hashes, pin the SDK version as well

### 7.5 Allocator statistics drift

Even if your Player uses its own allocator (Godot / Unreal's `MemoryArena`, Unity's `Native Memory`, and so on), a `libc_alloc` build of `ssruntime` **calls libc malloc directly**, so it does not appear in the Player's allocator statistics.

Your two options are:

- Accept the drift in memory measurement, or
- Hook the Player's allocator with the wrapper pattern from section 5

### 7.6 duplicate symbol warnings when linking static libraries together

Statically linking `libssruntime.a` and `libssconverter.a` into the same final binary, as with a Godot custom module or GDExtension, makes the linker emit warnings such as:

```
ld: warning: duplicate symbol '__rust_alloc' in libssruntime.a / libssconverter.a
ld: warning: duplicate symbol 'rust_eh_personality' in ...
ld: warning: duplicate symbol 'rust_begin_unwind' in ...
```

**There is no actual harm**:
- Because of `panic = "abort"`, `rust_eh_personality` / `rust_begin_unwind` are effectively no-ops
- The allocator shims are functionally equivalent - both the std System side and the `LibcAlloc` side go through libc malloc
- `ld` adopts the first implementation it finds and the link succeeds

If the warnings bother you, you can silence them by creating a wrapper crate that depends on both `ssruntime` and `ssconverter` and bundling them into a single staticlib, but normally it is fine to accept them.

---

[Back to Table of Contents](../README.md)
