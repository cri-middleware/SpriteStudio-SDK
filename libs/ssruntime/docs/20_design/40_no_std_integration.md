> [日本語](./40_no_std_integration.ja.md) | **English**

# Integrating into no_std environments

The Brain runs as `#![no_std]`. It can be integrated into embedded environments without a filesystem or threads, into game consoles, and into engines that use their own memory allocator. Every native binary the SDK distributes (iOS / Android / macOS / Linux / Windows) is built as `no_std`. Read this page if you build it yourself, or if you want to use the engine's allocator.

---

## 1. Build configuration

| Feature | Default | Role |
|---|---|---|
| `ffi` | ✅ | Exports the C-API |
| `web` | — | The `wasm-bindgen` bindings. `wasm-bindgen` enables `std` |
| `libc_alloc` | — | A global allocator that uses libc's `malloc` / `free` (mobile, desktop) |
| `wasm_alloc` | — | A global allocator that uses `dlmalloc` (WebAssembly without a JavaScript dependency) |
| `emscripten_alloc` | — | A global allocator that uses Emscripten's `malloc` / `free` |
| `custom_alloc` | — | Uses `ss_custom_alloc` / `ss_custom_free` / `ss_custom_abort` provided by the host (§4) |
| `panic-handler` | — | A panic handler. Aborts on panic (`unreachable` on WebAssembly) |
| `std` | — | For development; not used in release builds |

Choose exactly one of the global-allocator features (`libc_alloc` / `wasm_alloc` / `emscripten_alloc` / `custom_alloc`). Enabling two or more is a compile error.

With `web` enabled, Rust's `std` provides the allocator and the panic handler, but the Brain's math functions still use `libm`, so the results are the same as on the other platforms (§5-1).

```bash
# Static library for mobile / console / desktop
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"

# For browsers (wasm-bindgen)
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "web"

# WebAssembly without a JavaScript dependency (WASI or embedded WebAssembly engines)
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "ffi,wasm_alloc,panic-handler"

# Embedding as a Rust crate (the host provides the allocator and the panic handler)
cargo build -p ssruntime --release
```

---

## 2. Using it from C / C++

In an environment where `std` is unavailable but the C standard library (`malloc` / `free`) is available (typical consoles and mobile), build as follows.

```bash
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"
```

This produces a static library (`.a` / `.lib`) that allocates memory with C's `malloc` and, on a panic, terminates the process with `abort()` (or stops in the debugger).

---

## 3. Using it as a Rust crate

When you use it from a `no_std` engine written in Rust, the final executable usually provides the allocator and the panic handler. Write the following in `Cargo.toml`. No feature is enabled by default, so this gives a `no_std` library with neither an allocator nor the C-API.

```toml
[dependencies]
ssruntime = "7.0.0-beta.1"
```

> [!NOTE]
> The Brain uses `Vec` and other types from the `alloc` crate. The final binary must have a `#[global_allocator]` and a `#[panic_handler]`.

---

## 4. Using the engine's allocator (`custom_alloc`)

If you want to use only the engine's memory functions (Unreal Engine's `FMalloc`, a memory pool, and so on) and not even C's `malloc`, build with `custom_alloc` and define the allocation and deallocation functions on the C / C++ side. You do not need to write any Rust code.

### 4-1. Building

What you distribute is the static library, so build with `--crate-type staticlib`.

```bash
cargo rustc -p ssruntime-ffi --release --features "custom_alloc,panic-handler" --crate-type staticlib
```

> [!IMPORTANT]
> With `custom_alloc`, `ss_custom_*` and C runtime functions such as `memcpy` are left unresolved and are resolved when you link with the engine. So a plain `cargo build -p ssruntime-ffi`, whose `crate-type` includes `cdylib`, fails with the macOS and Windows (MSVC) linkers, which do not allow unresolved symbols. Specify `--crate-type staticlib` as above (the artifact is `target/release/libssruntime.a`).

### 4-2. Defining the functions

In the C / C++ project you link against, define the following three `extern "C"` functions (`ssruntime.h` declares `ss_custom_alloc` and `ss_custom_free` with these types, so the same file can include it). C runtime functions such as `memcpy` / `memset` / `memmove` / `memcmp` / `strlen` are also needed at the final link, but the engine's CRT usually provides them (on bare metal, supply them separately).

```cpp
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#if defined(_MSC_VER)
#include <malloc.h> // _aligned_malloc / _aligned_free
#endif

extern "C" {

// 1. Memory allocation
//    The returned pointer must be aligned to an `align`-byte boundary (`align` is always a power of two,
//    and at least 8: a Player reads FrameData's structs in place, so the Brain never asks for less).
//    Plain malloc guarantees alignment only up to max_align_t (usually 8-16 bytes),
//    so a request for a larger boundary is undefined behavior. The Brain does not correct the alignment.
uint8_t* ss_custom_alloc(uintptr_t size, uintptr_t align) {
    // Example: for Unreal Engine
    // return FMemory::Malloc(size, align);
#if defined(_MSC_VER)
    return (uint8_t*)_aligned_malloc(size, align);
#else
    // align is at least 8, so already a multiple of sizeof(void*), as posix_memalign requires
    void* ptr = NULL;
    if (posix_memalign(&ptr, align, size) != 0) {
        return NULL; // return NULL on allocation failure (the Brain detects it)
    }
    return (uint8_t*)ptr;
#endif
}

// 2. Memory deallocation
//    The same size / align used for the allocation are passed in. A pool allocator that needs the size to free can use them.
//    Use the free function that pairs with the allocation function. On MSVC, calling free on memory
//    allocated with _aligned_malloc is undefined behavior.
void ss_custom_free(void* ptr, uintptr_t size, uintptr_t align) {
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

// 3. Abort on panic
//    Must not return (noreturn). Returning is undefined behavior.
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

Define these three and link `libssruntime.a` (or `.lib`), and every memory allocation in the Brain goes through the engine's allocator.

> [!NOTE]
> `ss_custom_abort` is called only when a panic occurs in the Brain. If `ss_custom_alloc` returns `NULL` (allocation failure), Rust's allocation-error handling aborts immediately without going through `ss_custom_abort`. If you want your own handling of allocation failure, do it inside `ss_custom_alloc`.

---

## 5. Differences between `std` and `no_std`

The Brain builds from the same source as both `std` and `no_std`, but the implementation of some low-level operations switches between them. **What is distributed is the `no_std` build, so base your Player on its behavior.**

### 5-1. Math functions

The implementations of `sin` / `cos` / `tan` / `atan2` / `abs` / `round` / `floor` / `fmod` / `sqrt` switch.

| | std | no_std |
|---|---|---|
| Implementation | `f32` methods (`f.sin()` and so on) | The `libm` crate (`libm::sinf` and so on; a Rust port of Sun's fdlibm) |
| Underneath | The platform libm (Accelerate on macOS, glibc on Linux, and so on) | The same Rust implementation on every platform |

The trigonometric functions may differ between the two by a few ULP. `sqrt` / `round` / `floor` / `abs` are often hardware instructions and almost always match. The results look the same, but a bitwise comparison may show differences.

Every distributed binary uses `libm` (the WebAssembly build included), so **with the same SDK version, results are bit-identical across platforms.** Across SDK versions, a `libm` update may introduce differences. In a design that compares hashes of animation results, such as replays or lockstep, pin the SDK version too.

Numbers you verify in a `std` build may differ from the distributed build by a few ULP. Do the final numerical verification with a `no_std` build.

### 5-2. Panics

| | std | no_std + `panic-handler` |
|---|---|---|
| On panic | Prints a message and a backtrace to stderr, then aborts | Calls `abort()` immediately (no message) |

Both are built with `panic = "abort"`, so unwinding never occurs. In the distributed build the reason for a panic is not logged, so catching `SIGABRT` in the Player does not tell you the cause. **Do not swallow panics in the Player layer.** You would no longer be able to trace where it stopped. Instead, validate the arguments you pass to the Brain on the Player side, and send the reason for a failed call to the log ([Loading and initialization §4](../30_core/10_initialization.md#4-logs-and-errors)).

### 5-3. Global allocator

| | std | no_std + `libc_alloc` |
|---|---|---|
| Implementation | Rust's System allocator | `LibcAlloc`, provided by `libc_alloc` |
| Underneath | libc's `malloc` / `free` (MSVCRT on Windows) | The same |

On the major platforms both go through libc, so they are the same.

A Brain built with `libc_alloc` calls libc's `malloc` directly, so even if the Player uses the engine's own allocator (Unity's Native Memory and so on), the Brain's memory does not appear in that allocator's statistics. To include it in the statistics, pass the engine's allocator with `custom_alloc` (§4).

---
[Back to contents](../README.md)
