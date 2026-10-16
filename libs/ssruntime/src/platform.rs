//! The `#[global_allocator]` and `#[panic_handler]` that the `no_std` binary builds take from their features.

// Only one `#[global_allocator]` may exist, so these no_std allocator features are
// mutually exclusive; feature unification can enable two unintentionally, so emit a
// readable error instead of the compiler's cryptic "multiple global allocators".
#[cfg(all(not(feature = "std"), feature = "custom_alloc", feature = "libc_alloc"))]
compile_error!("features `custom_alloc` and `libc_alloc` are mutually exclusive: enable only one #[global_allocator]");
#[cfg(all(not(feature = "std"), feature = "custom_alloc", feature = "emscripten_alloc"))]
compile_error!("features `custom_alloc` and `emscripten_alloc` are mutually exclusive: enable only one #[global_allocator]");
#[cfg(all(not(feature = "std"), target_arch = "wasm32", feature = "custom_alloc", feature = "wasm_alloc"))]
compile_error!("features `custom_alloc` and `wasm_alloc` are mutually exclusive: enable only one #[global_allocator]");

#[cfg(all(not(feature = "std"), not(test), not(feature = "web"), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "wasm32")]
    ::core::arch::wasm32::unreachable();

    #[cfg(all(not(target_arch = "wasm32"), feature = "libc_alloc"))]
    unsafe {
        libc::abort();
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "custom_alloc"))]
    unsafe {
        unsafe extern "C" {
            fn ss_custom_abort() -> !;
        }
        ss_custom_abort();
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "libc_alloc"), not(feature = "custom_alloc")))]
    loop {}
}

#[cfg(all(not(feature = "std"), not(test), unix, feature = "panic-handler"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(all(not(feature = "std"), not(feature = "web"), target_arch = "wasm32", feature = "wasm_alloc"))]
mod wasm_allocator {
    #[global_allocator]
    static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
}

#[cfg(all(not(feature = "std"), feature = "emscripten_alloc"))]
mod emscripten_allocator {
    unsafe extern "C" {
        #[link_name = "malloc"]
        fn em_malloc(size: usize) -> *mut u8;
        #[link_name = "free"]
        fn em_free(ptr: *mut ::core::ffi::c_void);
        #[link_name = "posix_memalign"]
        fn em_posix_memalign(memptr: *mut *mut ::core::ffi::c_void, align: usize, size: usize) -> i32;
    }

    struct EmscriptenAlloc;

    #[global_allocator]
    static ALLOCATOR: EmscriptenAlloc = EmscriptenAlloc;

    // Emscripten's musl-derived libc malloc guarantees >= 8-byte alignment; larger
    // alignments go through posix_memalign. Both are released with free.
    unsafe impl alloc::alloc::GlobalAlloc for EmscriptenAlloc {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            if layout.align() <= 8 {
                return unsafe { em_malloc(layout.size()) };
            }
            let mut ptr = ::core::ptr::null_mut();
            if unsafe { em_posix_memalign(&mut ptr, layout.align(), layout.size()) } != 0 {
                return ::core::ptr::null_mut();
            }
            ptr as *mut u8
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: alloc::alloc::Layout) {
            unsafe { em_free(ptr as *mut ::core::ffi::c_void) };
        }
    }
}

/// Custom global allocator that forwards every allocation to three `extern "C"`
/// symbols the linking application must provide (e.g. a game engine allocator):
///
/// - `ss_custom_alloc(size, align)`: return `size` bytes aligned to `align`
///   (always a power of two, and at least 8) or null on failure. Unlike the
///   `libc_alloc` allocator, the runtime does **not** fix up alignment itself,
///   so the host must honor `align` (use `posix_memalign` / `_aligned_malloc` /
///   the engine's aligned allocator — plain `malloc` only guarantees
///   `max_align_t`).
/// - `ss_custom_free(ptr, size, align)`: free a block returned by
///   `ss_custom_alloc`, given the same `size` and `align`. Allocators that track
///   the size internally (`malloc`/`free`, `FMemory::Free`, ...) may ignore them.
/// - `ss_custom_abort()`: see the `#[panic_handler]` above; must not return.
#[cfg(all(not(feature = "std"), feature = "custom_alloc"))]
mod custom_allocator {
    unsafe extern "C" {
        fn ss_custom_alloc(size: usize, align: usize) -> *mut u8;
        fn ss_custom_free(ptr: *mut ::core::ffi::c_void, size: usize, align: usize);
    }

    struct CustomAlloc;

    #[global_allocator]
    static ALLOCATOR: CustomAlloc = CustomAlloc;

    /// The least alignment asked of the host. FrameData is built in a byte
    /// buffer, which Rust asks for at alignment 1, and a Player reads its
    /// structs where they lie: `PartState` holds a u64. Raising every request
    /// to 8 keeps that buffer on an 8-byte boundary, whatever the host's
    /// allocator does with a smaller `align`.
    const MIN_ALIGN: usize = 8;

    unsafe impl alloc::alloc::GlobalAlloc for CustomAlloc {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            unsafe { ss_custom_alloc(layout.size(), layout.align().max(MIN_ALIGN)) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::alloc::Layout) {
            unsafe { ss_custom_free(ptr as *mut ::core::ffi::c_void, layout.size(), layout.align().max(MIN_ALIGN)) }
        }
    }
}
#[cfg(all(not(feature = "std"), feature = "libc_alloc"))]
mod allocator {
    #[cfg(unix)]
    #[link(name = "c")]
    unsafe extern "C" {}

    // `libc_alloc` delegates to the host libc so the runtime shares its heap with
    // the embedding application. It is supported only on targets that expose a
    // standard aligned allocator; elsewhere select `custom_alloc`.
    #[cfg(not(any(unix, windows)))]
    compile_error!("feature `libc_alloc` requires a unix or windows target; use `custom_alloc` elsewhere");

    struct LibcAlloc;

    #[global_allocator]
    static ALLOCATOR: LibcAlloc = LibcAlloc;

    // libc `malloc` already satisfies alignments up to the platform's scalar
    // alignment (>= 8 on every supported target); larger alignments go through the
    // platform's aligned allocator. `dealloc` mirrors the same `align` branch so
    // each block is released by the function that allocated it.
    unsafe impl alloc::alloc::GlobalAlloc for LibcAlloc {
        #[cfg(unix)]
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            if layout.align() <= 8 {
                return unsafe { libc::malloc(layout.size()) as *mut u8 };
            }
            // posix_memalign requires `align` to be a power of two and a multiple of
            // size_of::<*mut _>(); both hold because `align` is a power of two > 8.
            let mut ptr = ::core::ptr::null_mut();
            if unsafe { libc::posix_memalign(&mut ptr, layout.align(), layout.size()) } != 0 {
                return ::core::ptr::null_mut();
            }
            ptr as *mut u8
        }

        #[cfg(unix)]
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: alloc::alloc::Layout) {
            // Blocks from both malloc and posix_memalign are released with free.
            unsafe { libc::free(ptr as *mut libc::c_void) };
        }

        #[cfg(windows)]
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            if layout.align() <= 8 {
                unsafe { libc::malloc(layout.size()) as *mut u8 }
            } else {
                // libc::aligned_malloc binds the CRT `_aligned_malloc` (size, align);
                // its blocks must be released with the matching aligned_free.
                unsafe { libc::aligned_malloc(layout.size(), layout.align()) as *mut u8 }
            }
        }

        #[cfg(windows)]
        unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::alloc::Layout) {
            if layout.align() <= 8 {
                unsafe { libc::free(ptr as *mut libc::c_void) };
            } else {
                unsafe { libc::aligned_free(ptr as *mut libc::c_void) };
            }
        }
    }
}
