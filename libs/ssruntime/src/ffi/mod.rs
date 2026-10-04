//! The C ABI — how every non-Rust Player talks to the runtime.
//!
//! A flat set of `extern "C"` functions mirroring the Rust API, plus the opaque handles
//! that carry state across the boundary. `cbindgen` turns this module into `ssruntime.h`
//! at build time; that header, not this page, is what a C or C++ integrator reads.
//!
//! # Handles
//!
//! Two opaque pointers, both `*mut c_void` to C:
//!
//! - **Resource** (`ss_resource_create_copy` / `_borrow` → `ss_resource_destroy`) — parsed
//!   `.ssab` data. Internally reference-counted, so it is safe to destroy a resource that
//!   contexts are still bound to; the buffer stays alive until the last one lets go.
//!   The `_borrow` variant does *not* copy, so the caller's buffer must outlive it.
//! - **Context** ([`ss_runtime_create`] → [`ss_runtime_destroy`]) — the playback state of
//!   one animation. Bind a resource with [`ss_runtime_bind_resource`]; several contexts
//!   may share one.
//!
//! Effect and instance parts get their own handles ([`ss_effect_slot_create`],
//! [`ss_instance_slot_create`]), stepped once per tick by the Player.
//!
//! # Calling convention
//!
//! Fallible calls return `bool` (or a null pointer, or [`SS_RUNTIME_INVALID_HANDLE`]) and
//! record the reason on the context. [`ss_runtime_get_last_error`] reads back the stable
//! code (compare against [`crate::error::SSRuntimeError::as_code`]), and
//! [`ss_runtime_error_message`] turns that code into the text to put in front of a user —
//! which is what a Player owes its caller.
//! **The message takes the code, not the context**, so it is a pure lookup with nothing
//! to own and nothing to outlive. Getters that return a float use `NaN` for "no such
//! thing", which is distinguishable from a legitimate `0.0`.
//!
//! The two `ss_resource_create_*` constructors are the exception: they take no context,
//! so a null return is the whole report. It needs no more than that, because there are
//! only two ways to get one — a null `ptr`, which the caller passed, and an `.ssab` whose
//! schema version this runtime does not read. (Bytes that are not an `.ssab` at all are
//! not among them; nothing here validates the input. See *Validation* below.)
//!
//! # Building a resource
//!
//! `ss_resource_create_*` then [`ss_runtime_bind_resource`] is the full form: it leaves
//! you holding the resource handle, which is what [`ss_effect_slot_create`] takes and
//! what lets one parsed pack serve several contexts. `ss_runtime_load_ssab_*` is the
//! one-liner for when you need neither — it creates, binds, and drops its own reference.
//!
//! **They are alternatives, not a sequence.** Calling both on the same bytes parses the
//! pack twice, and the parse is where every animation's metadata is built.
//!
//! Buffers handed out through `out_data` / `out_len` pairs point into runtime-owned
//! memory and stay valid only until the next call that rewrites them. Copy anything you
//! need to keep.
//!
//! # Validation
//!
//! Loading does not validate the `.ssab` — see the crate docs. Every entry point that
//! takes a data pointer trusts it.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_lazy_continuation)]

pub use crate::types::*;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::{CStr, c_char, c_float, c_int, c_uchar, c_void};
use core::sync::atomic::{AtomicPtr, Ordering};

/// Severity passed to [`SsLogCallback`]. The runtime only ever emits `ERROR`
/// today; `WARN` and `INFO` are reserved so a host can switch on the level
/// without its arms going stale when the runtime starts using them.
pub const SS_RUNTIME_LOG_ERROR: c_int = 1;
/// See [`SS_RUNTIME_LOG_ERROR`].
pub const SS_RUNTIME_LOG_WARN: c_int = 2;
/// See [`SS_RUNTIME_LOG_ERROR`].
pub const SS_RUNTIME_LOG_INFO: c_int = 3;

pub type SsLogCallback = extern "C" fn(level: core::ffi::c_int, message: *const core::ffi::c_char);
#[doc(hidden)]
pub static LOG_CALLBACK: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_log_callback(callback: SsLogCallback) {
    LOG_CALLBACK.store(callback as *mut c_void, Ordering::Relaxed);
}

#[macro_export]
macro_rules! ss_log {
    ($level:expr, $($arg:tt)*) => {{
        let cb_ptr = $crate::ffi::LOG_CALLBACK.load(::core::sync::atomic::Ordering::Relaxed);
        // Nothing is formatted while no callback is registered, so an unwatched
        // build pays one relaxed load per failure and no allocation.
        if !cb_ptr.is_null()
            && let Ok(message) = ::alloc::ffi::CString::new(::alloc::format!($($arg)*))
        {
            // Safety: `LOG_CALLBACK` only ever holds what
            // `ss_runtime_set_log_callback` stored, which is this type.
            let cb: $crate::ffi::SsLogCallback = unsafe { ::core::mem::transmute(cb_ptr) };
            cb($level, message.as_ptr());
        }
    }};
}

use crate::SsabResource;

pub struct FfiContext {
    pub ctx: crate::Context<'static>,
    // We hold the resource here to keep it alive
    pub bound_resource: Option<Arc<SsabResource>>,
    /// The reason the last call that tracks one failed, or `None` if it
    /// succeeded. Held as the error itself rather than its code so that the
    /// payload `InvalidVersion` carries is still here for a Rust caller and the
    /// wasm face, both of which format it.
    pub last_error: Option<crate::error::SSRuntimeError>,
}
impl FfiContext {
    /// Record a failure on both faces: [`ss_runtime_get_last_error`] for the
    /// caller that checks the return value, and the log callback for the one
    /// that does not. `api` names the entry point, because the error text
    /// alone ("Animation not found") does not say which call produced it.
    fn fail(&mut self, api: &str, err: crate::error::SSRuntimeError) {
        ss_log!(SS_RUNTIME_LOG_ERROR, "{}: {}", api, err);
        self.last_error = Some(err);
    }
}

impl Default for FfiContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiContext {
    pub fn new() -> Self {
        Self { ctx: crate::Context::new(), bound_resource: None, last_error: None }
    }
}

#[inline(always)]
fn get_context<'a>(ptr: *mut c_void) -> Option<&'a mut FfiContext> {
    if ptr.is_null() {
        return None;
    }
    unsafe { Some(&mut *(ptr as *mut FfiContext)) }
}

/// Parse `data` into a resource handle, keeping the reason on failure.
///
/// The reason is the point: a version mismatch and a malformed buffer both come
/// back as a null handle from the two `ss_resource_create_*` entry points, which
/// have no context to record on. The entry points that DO have one record the
/// difference, readable through [`ss_runtime_get_last_error`].
///
/// # Safety
///
/// Same as [`SsabResource::new`]: `data` must hold a valid `.ssab`. Every entry point below
/// passes that precondition on to its C caller in its "Security and Validation" section.
unsafe fn create_ffi_resource(data: crate::SsabData) -> Result<*mut c_void, crate::error::SSRuntimeError> {
    // SAFETY: forwarded from this function's own contract.
    let arc = unsafe { SsabResource::new(data) };
    if arc.resource().is_none() {
        return Err(arc.parse_error().cloned().unwrap_or(crate::error::SSRuntimeError::Unknown));
    }
    Ok(Arc::into_raw(arc) as *mut c_void)
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Creates an AnimationResource by copying the provided data.
///
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_resource_create_copy(ptr: *const c_uchar, len: usize) -> *mut c_void {
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let data = unsafe { ::core::slice::from_raw_parts(ptr, len) };
    match crate::SsabData::load_copy(data) {
        // SAFETY: the C caller guarantees a valid `.ssab` (see "Security and Validation").
        Ok(ssab_data) => unsafe { create_ffi_resource(ssab_data) }.unwrap_or(core::ptr::null_mut()),
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Creates an AnimationResource by borrowing the provided data.
///
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block of at least `len` bytes,
/// AND the memory must remain valid and unmodified until this resource is destroyed.
pub unsafe extern "C" fn ss_resource_create_borrow(ptr: *const c_uchar, len: usize) -> *mut c_void {
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    match unsafe { crate::SsabData::load_borrow(ptr, len) } {
        // SAFETY: the C caller guarantees a valid `.ssab` (see "Security and Validation").
        Ok(ssab_data) => unsafe { create_ffi_resource(ssab_data) }.unwrap_or(core::ptr::null_mut()),
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_resource_destroy(resource: *mut c_void) {
    if resource.is_null() {
        return;
    }
    unsafe {
        let _ = Arc::from_raw(resource as *mut SsabResource);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_create() -> *mut c_void {
    let c = FfiContext::new();
    Box::into_raw(Box::new(c)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_destroy(context: *mut c_void) {
    if let Some(context) = get_context(context) {
        unsafe {
            let _ = Box::from_raw(context as *mut FfiContext);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_reset(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.reset();
        ffi_context.bound_resource = None;
        ffi_context.last_error = None;
    }
}

/// Why the last call that tracks a reason failed, as the stable code from
/// [`crate::error::SSRuntimeError::as_code`]. `0` means the last such call
/// succeeded, and so does a null `context`.
///
/// Every fallible entry point that has a context to write to clears this on
/// entry, so it always describes the most recent one rather than some earlier
/// failure. The exceptions are the resource constructors
/// (`ss_resource_create_borrow` / `ss_resource_create_copy`), which take no
/// context and so have nowhere to record: a null return from those is all there
/// is.
///
/// Reading does not clear it. Read it immediately after the call that returned
/// `false`, before making another on the same context.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_last_error(context: *mut c_void) -> i32 {
    match get_context(context) {
        Some(ffi_context) => ffi_context.last_error.as_ref().map_or(0, |e| e.as_code()),
        None => 0,
    }
}

/// The wording behind a code from [`ss_runtime_get_last_error`], as
/// NUL-terminated UTF-8.
///
/// **Takes the code, not a context**, so it is a pure lookup: no lifetime to
/// respect, nothing to free, and the same answer forever. An unrecognised code
/// — including `0`, which is "no error" — gives an empty string, so the result
/// goes straight into a log call without a branch.
///
/// The text carries no numbers. `InvalidVersion` reads "Invalid SSAB version";
/// a Player that wants to say *which* versions should compare
/// [`ss_runtime_ssab_schema_version`] against the pack's own `version` field,
/// which gives it two integers rather than a sentence to parse.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_error_message(code: i32) -> *const c_char {
    // `&'static CStr` literals: no allocation, and nothing for the caller to own.
    // Keyed off the code rather than a variant because the code is what crossed
    // the ABI; `error_message_matches_every_variant` pins the two together.
    let text: &'static CStr = match code {
        0 => c"",
        1 => c"SSAB data is empty",
        2 => c"Invalid SSAB version",
        3 => c"Animation not found",
        4 => c"SSAB data is not loaded",
        5 => c"Invalid parameter",
        6 => c"Memory allocation failed",
        7 => c"Effect data is malformed",
        99 => c"Unknown error occurred",
        _ => c"",
    };
    text.as_ptr()
}

/// The `.ssab` schema version this runtime reads.
///
/// The counterpart of `ss_converter_ssab_schema_version`, and what turns an
/// `InvalidVersion` failure into something a Player can say: compare it against
/// the `version` field of the pack it just failed to load.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_ssab_schema_version() -> u32 {
    ssab::SSAB_SCHEMA_VERSION
}

/// Binds an AnimationResource to a Context.
///
/// # Safety and Lifetimes
/// - To save memory, a single Resource can be bound to multiple Contexts.
/// - **Lifetime Safety:** This implementation uses `Arc` for internal reference counting.
///   It is safe to destroy the `resource` (via `ss_resource_destroy`) even if it is still bound
///   to contexts; the memory will be kept alive until all bound contexts are also destroyed or reset.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_bind_resource(context: *mut c_void, resource: *mut c_void) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    if resource.is_null() {
        ffi_context.fail("ss_runtime_bind_resource", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    }

    // Safety: Increment refcount by cloning from the raw pointer.
    let arc = unsafe { Arc::from_raw(resource as *mut SsabResource) };
    let arc_clone = arc.clone();
    let _ = Arc::into_raw(arc); // Keep original pointer valid

    // Use a raw pointer to break the borrow on `arc_clone` so it can be moved
    // into `bound_resource` before the pointer is dereferenced. The pointer
    // stays valid because `bound_resource` owns the surrounding Arc.
    let res_ptr = arc_clone.resource_static_ptr();
    if res_ptr.is_null() {
        // The resource handle is live but carries no parsed animations — the
        // bytes behind it were not a readable SSAB.
        ffi_context.fail("ss_runtime_bind_resource", crate::error::SSRuntimeError::EmptyData);
        return false;
    }
    ffi_context.bound_resource = Some(arc_clone);
    ffi_context.ctx.bind_resource(unsafe { &*res_ptr });
    true
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_runtime_load_ssab_copy(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    if ptr.is_null() {
        ffi_context.fail("ss_runtime_load_ssab_copy", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    }

    let data = unsafe { ::core::slice::from_raw_parts(ptr, len) };
    match crate::SsabData::load_copy(data) {
        Ok(ssab_data) => {
            // The real reason, not `MemoryError`: this fails on a version
            // mismatch far more often than on an allocation, and the two want
            // opposite things from the user.
            // SAFETY: the C caller guarantees a valid `.ssab` (see "Security and Validation").
            let res_ptr = match unsafe { create_ffi_resource(ssab_data) } {
                Ok(p) => p,
                Err(e) => {
                    ffi_context.fail("ss_runtime_load_ssab_copy", e);
                    return false;
                }
            };
            let success = ss_runtime_bind_resource(context, res_ptr);
            ss_resource_destroy(res_ptr);
            success
        }
        Err(e) => {
            ffi_context.fail("ss_runtime_load_ssab_copy", e);
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_runtime_load_ssab_borrow(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    if ptr.is_null() {
        ffi_context.fail("ss_runtime_load_ssab_borrow", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    }

    match unsafe { crate::SsabData::load_borrow(ptr, len) } {
        Ok(ssab_data) => {
            // The real reason, not `MemoryError`: this fails on a version
            // mismatch far more often than on an allocation, and the two want
            // opposite things from the user.
            // SAFETY: the C caller guarantees a valid `.ssab` (see "Security and Validation").
            let res_ptr = match unsafe { create_ffi_resource(ssab_data) } {
                Ok(p) => p,
                Err(e) => {
                    ffi_context.fail("ss_runtime_load_ssab_borrow", e);
                    return false;
                }
            };
            let success = ss_runtime_bind_resource(context, res_ptr);
            ss_resource_destroy(res_ptr);
            success
        }
        Err(e) => {
            ffi_context.fail("ss_runtime_load_ssab_borrow", e);
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_ssab(context: *mut c_void, out_data: *mut *const c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            unsafe {
                if !out_data.is_null() {
                    *out_data = ssab_buf.as_ptr() as *const c_uchar;
                }
                if !out_len.is_null() {
                    *out_len = ssab_buf.len();
                }
            }
        } else {
            ffi_context.fail("ss_runtime_get_ssab", crate::error::SSRuntimeError::NotLoaded);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_buf(context: *mut c_void) -> *const c_uchar {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            return ssab_buf.as_ptr() as *const c_uchar;
        } else {
            ffi_context.fail("ss_runtime_get_ssab_buf", crate::error::SSRuntimeError::NotLoaded);
        }
    }
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_len(context: *mut c_void) -> usize {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            return ssab_buf.len();
        } else {
            ffi_context.fail("ss_runtime_get_ssab_len", crate::error::SSRuntimeError::NotLoaded);
        }
    }
    0
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid null-terminated string.
pub unsafe extern "C" fn ss_runtime_setup_animation(context: *mut c_void, name: *const c_char) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    if name.is_null() {
        ffi_context.fail("ss_runtime_setup_animation", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    }
    let c_buf = unsafe { CStr::from_ptr(name) };
    let a = c_buf.to_str().unwrap_or_default();
    match ffi_context.ctx.setup_animation(a) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_setup_animation", e);
            false
        }
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_runtime_setup_animation_by_hash(context: *mut c_void, name_hash: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.setup_animation_by_hash(name_hash) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_setup_animation_by_hash", e);
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_frame_data(context: *mut c_void, frame_no: c_float, out_data: *mut *const c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        match ffi_context.ctx.get_frame_data(frame_no) {
            Ok(ret) => unsafe {
                if !out_data.is_null() {
                    *out_data = ret.as_ptr();
                }
                if !out_len.is_null() {
                    *out_len = ret.len();
                }
            },
            Err(e) => {
                ffi_context.fail("ss_runtime_get_frame_data", e);
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_world_matrices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_world_matrices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_world_matrices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_local_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_local_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_local_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part resolved cell metadata for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout:
/// `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]` per part,
/// indexed by `part_index * 6`. `(rect_left, rect_top)` is the cell
/// rectangle origin in the cellmap's texture pixel space, letting a
/// Player build a complete source rect (origin + size) without its own
/// cellmap lookup. Parts without a current cell carry zeros.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_cell_meta(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_cell_meta_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_cell_meta_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part local-space vertex positions for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout per part:
/// `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
/// = 10 floats, indexed by `part_index * 10`. Coordinates are in part-local
/// space with `coordinate_system` (Y-Up/Y-Down) flip already applied, so a
/// Player just multiplies them by the world matrix. Includes pivot, image
/// flip, and per-frame vertex deformation. Parts without a current cell
/// carry zeros. Replaces `FrameData::vertices` for renderers that consume
/// pre-resolved SoA buffers.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_local_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_local_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_local_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part shape vertex positions for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout: 12 vertices × `(x, y)` = 24 floats
/// per part, indexed by `part_index * 24`. Vertices are in local space,
/// post-pivot, post-coord-system, so a Player just multiplies them by the
/// world matrix. Only the first
/// `ss_runtime_get_shape_vertex_counts(...)[part_index]` vertices are
/// valid; remaining slots are zero-initialized. Non-shape parts carry zero
/// count and zero-filled vertex slots. When no part of the pack is a shape,
/// the buffer is empty, as is `ss_runtime_get_shape_vertex_box_coords`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-vertex normalized bounding-box coordinates paired with
/// `ss_runtime_get_shape_vertices`. Layout: 12 vertices × `(fx, fy)` = 24
/// floats per part, indexed by `part_index * 24`. `(0, 0)` corresponds to
/// the LT corner of the shape's pre-pivot bounding box, `(1, 1)` to the RB
/// corner, regardless of pivot offset or coordinate system. Players use
/// this to interpolate the four part-color corners without re-deriving the
/// box layout.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertex_box_coords(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertex_box_coords_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertex_box_coords_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part valid vertex count for `ss_runtime_get_shape_vertices`. Indexed
/// by `part_index`. Zero for parts that are not shape parts (or shape
/// parts whose size collapsed to zero).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertex_counts(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertex_counts_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertex_counts_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Skinned mesh per-vertex X coordinates in world space, concatenated across all
/// parts. Pair with `ss_runtime_get_mesh_vertex_offsets` to slice per-part
/// ranges: part `p`'s vertices live at indices `offsets[p]..offsets[p + 1]`.
/// Non-mesh parts contribute an empty range. Indirect render mode leaves the
/// buffer empty (length 0).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertices_x(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertices_x_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertices_x_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Skinned mesh per-vertex Y coordinates. SoA companion to
/// `ss_runtime_get_mesh_vertices_x`; sliced identically via
/// `ss_runtime_get_mesh_vertex_offsets`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertices_y(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertices_y_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertices_y_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) describing where each
/// part's skinned mesh vertices live inside `ss_runtime_get_mesh_vertices_x`
/// / `_y`. Part `p` occupies indices `[offsets[p], offsets[p + 1])`; non-mesh
/// parts have `offsets[p] == offsets[p + 1]`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertex_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertex_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertex_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Mesh per-vertex UV coordinates in cellmap texture-pixel space, interleaved
/// `[u, v, ...]`, concatenated across all parts. Static — resolved once from
/// each mesh part's frame-0 Cell. Sliced per-part by
/// `ss_runtime_get_mesh_vertex_offsets`, identically to the vertex buffers.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Mesh triangle indices (part-local, 0-based), concatenated across all parts.
/// Static. Slice part `p`'s range via `ss_runtime_get_mesh_index_offsets`:
/// `[offsets[p], offsets[p + 1])`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_indices(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_indices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_indices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
/// `ss_runtime_get_mesh_indices`. Part `p` occupies `[offsets[p], offsets[p + 1])`;
/// non-mesh parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_index_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_index_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_index_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex positions for the current frame, interleaved
/// `[x, y, ...]` and concatenated across all parts, written by
/// `ss_runtime_get_frame_data`. Slice per part with
/// `ss_runtime_get_nines_vertex_offsets`; parts drawn by another
/// `DrawBatchKind` occupy an empty range.
///
/// Coordinates are part-local with pivot, image flip and the
/// `coordinate_system` (Y-Up/Y-Down) flip already applied — exactly like
/// `ss_runtime_get_local_vertices` — so a Player just multiplies them by the
/// part's world matrix. The grid is a 4x4 sheet for `nines_fill_mode == 0`
/// (stretch) and independent quads per repeat for `== 1` (tile).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex UVs in cellmap texture-pixel space, interleaved
/// `[u, v, ...]`. Shares the vertex CSR of
/// `ss_runtime_get_nines_vertex_offsets`. Divide by the texture size to
/// normalize, the same as `ss_runtime_get_local_uvs`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex box coordinates, interleaved `[fx, fy, ...]` with
/// `0` at the part's left / top edge and `1` at its right / bottom edge.
/// Shares the vertex CSR of `ss_runtime_get_nines_vertex_offsets`.
///
/// Use them to bilinearly interpolate the four `part_color` corners across the
/// grid — the same weighting `ss_runtime_get_shape_vertex_box_coords` provides
/// for Shape parts. A part flip is already folded in, so `fx` / `fy` always
/// describe the drawn box.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_box_coords(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_box_coords_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_box_coords_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice triangle indices, concatenated across all parts and sliced by
/// `ss_runtime_get_nines_index_offsets`. Values are **part-local**
/// (`0..vertex_count`), so a Player passes them alongside the part's vertex
/// slice without rebasing. Each quad is wound `LT, RT, LB / LB, RT, RB`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_indices(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_indices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_indices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) over the vertex buffers
/// `ss_runtime_get_nines_vertices` / `_uvs` / `_box_coords`. Part `p` occupies
/// vertices `[offsets[p], offsets[p + 1])` — multiply by 2 to index the
/// interleaved floats. Non-nines parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_vertex_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_vertex_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_vertex_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
/// `ss_runtime_get_nines_indices`. Part `p` occupies
/// `[offsets[p], offsets[p + 1])`; non-nines parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_index_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = None;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_index_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_index_offsets_len();
            }
        }
    }
}

// Frame Number getter functions
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no(context: *mut c_void) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_frame_no();
    }
    0.0
}

/// Returns `true` once the configured loops have completed, staying `true`
/// until the next `ss_runtime_play` / `ss_runtime_setup_animation`. Always
/// `false` under an infinite loop count, and never raised by
/// `ss_runtime_stop`.
///
/// This is a *state*, not a pulse: to notify once, take its rising edge.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_finished(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_finished();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_just_looped(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.just_looped();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_playing();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing_forward(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_playing_forward();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_pausing(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_pausing();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_skip_frames(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_skip_frames();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_direction(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) { if ffi_context.ctx.get_playback_direction() == crate::core::playback_controller::PlaybackDirection::Forward { 0 } else { 1 } } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_style(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_playback_style() as i32;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_loop_count(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_loop_count();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_remaining_loops(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_remaining_loops();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_start_frame(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_start_frame();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_end_frame(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_end_frame();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_rate(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_frame_rate();
    }
    0
}

/// The playback speed in effect, as a multiple of `ss_runtime_get_frame_rate`.
///
/// Reads back what `ss_runtime_set_speed_scale` stored, which is not
/// always what it was handed: a negative rate clamps to `0.0`.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_speed_scale(context: *mut c_void) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_speed_scale();
    }
    0.0
}

// Playback setup
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_skip_frames(context: *mut c_void, skip: bool) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_skip_frames(skip);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_loop_count(context: *mut c_void, loop_count: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_loop_count(loop_count);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_rate(context: *mut c_void, frame_rate: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_frame_rate(frame_rate);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_playback_direction(context: *mut c_void, direction: c_int, style: c_int) {
    if let Some(ffi_context) = get_context(context) {
        let d = if direction == 0 { crate::core::playback_controller::PlaybackDirection::Forward } else { crate::core::playback_controller::PlaybackDirection::Backward };
        let s = if style == 0 { crate::core::playback_controller::PlaybackStyle::Normal } else { crate::core::playback_controller::PlaybackStyle::PingPong };
        ffi_context.ctx.set_playback_direction(d, s);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_speed_scale(context: *mut c_void, speed_scale: c_float) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_speed_scale(speed_scale);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_section(context: *mut c_void, start_frame: c_int, end_frame: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_animation_section(start_frame, end_frame);
    }
}

// Playback control

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update(context: *mut c_void, delta: f32) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.update(delta);
    }
    0.0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.play();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(context: *mut c_void, frame_no: f32) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.play_with_start_frame(frame_no);
    }
}

/// Holds playback where it stands, keeping the current frame.
///
/// Idempotent: pausing an already-paused animation leaves it paused.
/// `ss_runtime_is_playing` stays true while held — a pause is a hold, not a
/// stop — so tell the two apart with `ss_runtime_is_pausing`. Lift the hold
/// with [`ss_runtime_resume`]; `ss_runtime_play` rewinds and cannot resume.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_pause(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.pause();
    }
}

/// Lifts a [`ss_runtime_pause`], carrying on from the frame it stopped at.
///
/// Idempotent, and a no-op on an animation that was never paused. It does not
/// start a stopped animation; use `ss_runtime_play` for that.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_resume(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.resume();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_stop(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.stop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_no(context: *mut c_void, frame: c_float) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_frame_no(frame);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_next_frame(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.next_frame();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_prev_frame(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.prev_frame();
    }
}

/// Run a fallible `Context` call, recording the error code on failure. Every
/// bool-returning track entry point funnels through this so the `last_error`
/// contract is stated once.
fn report<F>(ffi_context: &mut FfiContext, f: F) -> bool
where
    F: FnOnce(&mut crate::Context<'static>) -> Result<(), crate::error::SSRuntimeError>,
{
    match f(&mut ffi_context.ctx) {
        Ok(()) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_prev_frame", e);
            false
        }
    }
}

/// # Safety
/// `p` must be null or a valid null-terminated string.
unsafe fn cstr_to_string(p: *const c_char) -> Option<alloc::string::String> {
    if p.is_null() {
        return None;
    }
    let c_buf = unsafe { CStr::from_ptr(p) };
    Some(alloc::string::String::from_utf8_lossy(c_buf.to_bytes()).into_owned())
}

/// Sentinel returned by the handle-returning track functions on failure.
pub const SS_RUNTIME_INVALID_HANDLE: u32 = u32::MAX;

/// Track blend operators, mirroring [`crate::TrackBlend`].
pub const SS_RUNTIME_TRACK_BLEND_MIX: u32 = 0;
pub const SS_RUNTIME_TRACK_BLEND_ADDITIVE: u32 = 1;

// Animation mixing — the track stack. Tracks fold onto the setup pose in
// ascending order: `alpha` is a track's strength, `weight` only decides
// proportions among several sources sharing one track. Track 0 carries the
// structure (discrete attributes and events) and is what `setup_animation`
// establishes.

/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_set_track_animation(context: *mut c_void, track: u32, name: *const c_char) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.fail("ss_runtime_set_track_animation", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    };
    report(ffi_context, |ctx| ctx.set_track_animation(track, &a))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_animation_by_hash(context: *mut c_void, track: u32, hash: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_track_animation_by_hash(track, hash))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_alpha(context: *mut c_void, track: u32, alpha: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_track_alpha(track, alpha))
}

/// Returns `NaN` for a track that does not exist, so callers can tell it apart
/// from a valid `alpha` of 0.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_alpha(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_track_alpha(track)).unwrap_or(f32::NAN)
}

/// `blend` is one of `SS_RUNTIME_TRACK_BLEND_*`.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_blend(context: *mut c_void, track: u32, blend: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    let blend = match blend {
        SS_RUNTIME_TRACK_BLEND_MIX => crate::TrackBlend::Mix,
        SS_RUNTIME_TRACK_BLEND_ADDITIVE => crate::TrackBlend::Additive,
        _ => {
            ffi_context.fail("ss_runtime_set_track_blend", crate::error::SSRuntimeError::InvalidParameter);
            return false;
        }
    };
    report(ffi_context, |ctx| ctx.set_track_blend(track, blend))
}

/// Returns `SS_RUNTIME_INVALID_HANDLE` for a track that does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_blend(context: *mut c_void, track: u32) -> u32 {
    match get_context(context).and_then(|c| c.ctx.get_track_blend(track)) {
        Some(crate::TrackBlend::Mix) => SS_RUNTIME_TRACK_BLEND_MIX,
        Some(crate::TrackBlend::Additive) => SS_RUNTIME_TRACK_BLEND_ADDITIVE,
        None => SS_RUNTIME_INVALID_HANDLE,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_count(context: *mut c_void) -> u32 {
    get_context(context).map_or(0, |c| c.ctx.get_track_count())
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_track(context: *mut c_void, track: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.clear_track(track))
}

/// Fade `track` to `name` over `duration_ms` — the same time unit as
/// `ss_runtime_update`'s delta.
///
/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_crossfade(context: *mut c_void, track: u32, name: *const c_char, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.fail("ss_runtime_crossfade", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    };
    report(ffi_context, |ctx| ctx.crossfade(track, &a, duration_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_crossfade_by_hash(context: *mut c_void, track: u32, hash: u32, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.crossfade_by_hash(track, hash, duration_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_fade_track(context: *mut c_void, track: u32, target_alpha: c_float, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.fade_track(track, target_alpha, duration_ms))
}

/// Progress in `[0, 1]` of the fade on `track`; `NaN` when it is not fading.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_fade_progress(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_track_fade_progress(track)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_cancel_track_fade(context: *mut c_void, track: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.cancel_track_fade(track))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_frame(context: *mut c_void, track: u32, frame: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_track_frame(track, frame))
}

/// Returns `NaN` when the track has no animation on it.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_frame(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_track_frame(track)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_speed(context: *mut c_void, track: u32, speed: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_track_speed(track, speed))
}

/// Handle of the source carrying the track's animation, so it can be given a
/// weight once the track holds several. `SS_RUNTIME_INVALID_HANDLE` when the
/// track has no animation.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_track_main_source(context: *mut c_void, track: u32) -> u32 {
    get_context(context).and_then(|c| c.ctx.get_track_main_source(track)).unwrap_or(SS_RUNTIME_INVALID_HANDLE)
}

/// Add another weighted source to `track` (locomotion-style blending). Returns
/// `SS_RUNTIME_INVALID_HANDLE` on failure.
///
/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_add_track_source(context: *mut c_void, track: u32, name: *const c_char, weight: c_float) -> u32 {
    let Some(ffi_context) = get_context(context) else {
        return SS_RUNTIME_INVALID_HANDLE;
    };
    ffi_context.last_error = None;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.fail("ss_runtime_add_track_source", crate::error::SSRuntimeError::InvalidParameter);
        return SS_RUNTIME_INVALID_HANDLE;
    };
    match ffi_context.ctx.add_track_source(track, &a, weight) {
        Ok(h) => h,
        Err(e) => {
            ffi_context.fail("ss_runtime_add_track_source", e);
            SS_RUNTIME_INVALID_HANDLE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_add_track_source_by_hash(context: *mut c_void, track: u32, hash: u32, weight: c_float) -> u32 {
    let Some(ffi_context) = get_context(context) else {
        return SS_RUNTIME_INVALID_HANDLE;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.add_track_source_by_hash(track, hash, weight) {
        Ok(h) => h,
        Err(e) => {
            ffi_context.fail("ss_runtime_add_track_source_by_hash", e);
            SS_RUNTIME_INVALID_HANDLE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_weight(context: *mut c_void, handle: u32, weight: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_source_weight(handle, weight))
}

/// Returns `NaN` for an invalid or released handle, so `weight == 0` stays
/// distinguishable from "no such source".
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_source_weight(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_source_weight(handle)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_frame(context: *mut c_void, handle: u32, frame: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_source_frame(handle, frame))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_source_frame(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_source_frame(handle)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_event_threshold(context: *mut c_void, handle: u32, threshold: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    report(ffi_context, |ctx| ctx.set_source_event_threshold(handle, threshold))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_source_event_threshold(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.get_source_event_threshold(handle)).unwrap_or(f32::NAN)
}

// NOTE: declaration order matters — `inner` borrows from `resource`,
// so it must drop first.
struct FfiEffectSlot {
    inner: crate::EffectSlot<'static>,
    #[allow(dead_code)]
    resource: Arc<SsabResource>,
}

/// Create a new EffectSlot bound to `resource`. Returns null on failure
/// (resource not loaded, effect_name_hash not found, malformed effect data).
///
/// The slot holds its own reference to `resource`, so `ss_resource_destroy`
/// may be called while the slot is still alive, exactly as it may while a
/// Context is still bound.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_create(resource: *const c_void, effect_name_hash: u32, seed: u32) -> *mut c_void {
    if resource.is_null() {
        return core::ptr::null_mut();
    }
    // Safety: Increment refcount by cloning from the raw pointer.
    let arc = unsafe { Arc::from_raw(resource as *const SsabResource) };
    let arc_clone = arc.clone();
    let _ = Arc::into_raw(arc); // Keep original pointer valid

    // Use a raw pointer to break the borrow on `arc_clone` so it can be moved
    // into the slot before the pointer is dereferenced. The pointer stays valid
    // because the slot owns the surrounding Arc.
    let res_ptr = arc_clone.resource_static_ptr();
    if res_ptr.is_null() {
        return core::ptr::null_mut();
    }
    match crate::EffectSlot::new(unsafe { &*res_ptr }, effect_name_hash, seed) {
        Ok(inner) => Box::into_raw(Box::new(FfiEffectSlot { inner, resource: arc_clone })) as *mut c_void,
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_destroy(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(slot as *mut FfiEffectSlot);
    }
}

/// Reset the slot's playback / "last started key" memory to its just-created
/// state without destroying it. Call on an explicit head-out (`ss_runtime_play`
/// / replay) so a re-played animation re-fires its effect from scratch instead
/// of debouncing the first key as the same one the slot last started. The
/// resource binding and emitter caches are preserved.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_reset(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    let slot = unsafe { &mut *(slot as *mut FfiEffectSlot) };
    slot.inner.reset();
}

/// Advance the slot one frame. The caller resolves the active effect event
/// (via `ss_runtime_get_active_effect_event`) and feeds it here together with
/// timing inputs. `parent_looped` re-arms transition detection across the
/// parent's loop edge.
///
/// `y_flip = true` mirrors particle Y position and rotation sign, for Y-down
/// rendering of Y-up authored content (Godot, pixi.js). Pass `false` for
/// Y-up engines (Unity 2D).
///
/// `vert_stride` selects the vertex layout of the returned `EffectDrawPlan`:
/// `2` packs (x, y) pairs (Godot, pixi.js, wgpu); `3` packs (x, y, 0) triples
/// (Unity Mesh, Unreal procedural mesh). Other values fall back to `2`.
///
/// Returns `EffectDrawPlan` FlatBuffer pointer + length when visible. Buffer
/// is valid only until the next call on the same slot.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_step(slot: *mut c_void, ev: ss_effect_event_info, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8) -> ss_effect_step_result {
    if slot.is_null() {
        return ss_effect_step_result::not_visible();
    }
    let slot = unsafe { &mut *(slot as *mut FfiEffectSlot) };
    slot.inner.step(ev, parent_frame, delta_seconds, fps, parent_looped, y_flip, vert_stride)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_get_seed(slot: *mut c_void) -> u32 {
    if slot.is_null() {
        return 0;
    }
    let slot = unsafe { &*(slot as *const FfiEffectSlot) };
    slot.inner.get_seed()
}

/// Whether the slot is currently running independently of the parent's frame.
/// Reflects the most recently seen `EventEffect.independent`. Callers use
/// this to decide whether to schedule continuous tick updates regardless of
/// parent frame change.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_is_independent(slot: *mut c_void) -> bool {
    if slot.is_null() {
        return false;
    }
    let slot = unsafe { &*(slot as *const FfiEffectSlot) };
    slot.inner.is_independent()
}

/// Create a fresh InstanceSlot. Slots are stateless at construction (no
/// resource binding) — `step` operates against a child runtime context
/// supplied per call.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_create() -> *mut c_void {
    Box::into_raw(Box::new(crate::InstanceSlot::new())) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_destroy(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(slot as *mut crate::InstanceSlot);
    }
}

/// Reset the slot's "last started key" memory to its just-created state without
/// destroying it. Call on an explicit head-out (`ss_runtime_play` / replay) so a
/// re-played animation re-fires the instance child from scratch instead of
/// debouncing the first key as the same one the slot last started.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_reset(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    let slot = unsafe { &mut *(slot as *mut crate::InstanceSlot) };
    slot.reset();
}

/// Drive the instance slot for one tick. The caller resolves the active
/// EventInstance (via `ss_runtime_get_active_event_instance` against the
/// *parent* runtime context) and passes the *child* runtime context here so
/// that label resolution and frame stepping happen against the right Context.
/// Returns `transitioned == true` on EventInstance edges, having configured the
/// child for the new key without starting it: the caller then calls
/// `ss_runtime_play` on the child (an independent child never advances
/// otherwise). Draw the child at `child_frame_no` on that tick too: the slot
/// already put it where `play` snaps it, and a synced child's value follows the
/// parent, which reading the frame back after `play` would lose.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_step(slot: *mut c_void, ev: ss_event_instance_info, child_ctx: *mut c_void, parent_frame: f32, delta_seconds: f32, parent_looped: bool) -> ss_instance_step_result {
    if slot.is_null() {
        return ss_instance_step_result::not_visible();
    }
    let Some(child_ffi) = get_context(child_ctx) else {
        return ss_instance_step_result::not_visible();
    };
    let slot = unsafe { &mut *(slot as *mut crate::InstanceSlot) };
    slot.step(ev, &mut child_ffi.ctx, parent_frame, delta_seconds, parent_looped)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_is_independent(slot: *mut c_void) -> bool {
    if slot.is_null() {
        return false;
    }
    let slot = unsafe { &*(slot as *const crate::InstanceSlot) };
    slot.is_independent()
}

/// Set the render mode. `mode` is interpreted as:
///   0 = Direct       (pre-bake world matrices; default)
///   1 = SceneGraph   (skip world matrix pre-computation)
/// Unknown values fall back to Direct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_set_render_mode(context: *mut c_void, mode: u8) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_render_mode(crate::core::RenderMode::from_u8(mode));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_get_render_mode(context: *mut c_void) -> u8 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.render_mode() as u8;
    }
    crate::core::RenderMode::Direct as u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_set_coordinate_system(context: *mut c_void, coordinate_system: i32) {
    if let Some(ffi_context) = get_context(context) {
        let cs = match coordinate_system {
            1 => crate::core::SsCoordinateSystem::YDown,
            _ => crate::core::SsCoordinateSystem::YUp,
        };
        ffi_context.ctx.set_coordinate_system(cs);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_get_coordinate_system(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_coordinate_system() {
            crate::core::SsCoordinateSystem::YDown => 1,
            crate::core::SsCoordinateSystem::YUp => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_set_origin(context: *mut c_void, origin: i32) {
    if let Some(ffi_context) = get_context(context) {
        let o = match origin {
            1 => crate::core::SsOrigin::TopLeft,
            _ => crate::core::SsOrigin::Center,
        };
        ffi_context.ctx.set_origin(o);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_get_origin(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_origin() {
            crate::core::SsOrigin::TopLeft => 1,
            crate::core::SsOrigin::Center => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_set_unit(context: *mut c_void, unit: i32) {
    if let Some(ffi_context) = get_context(context) {
        let u = match unit {
            1 => crate::core::SsUnit::Normalized,
            _ => crate::core::SsUnit::Pixel,
        };
        ffi_context.ctx.set_unit(u);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_get_unit(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_unit() {
            crate::core::SsUnit::Normalized => 1,
            crate::core::SsUnit::Pixel => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_count(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_passed_event_indices().len() as i32;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_index(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(&event_idx) = ffi_context.ctx.get_passed_event_indices().get(index as usize)
    {
        return event_idx;
    }
    -1
}

/// Which source fired the collected event at `index` — parallel to
/// `ss_runtime_get_passed_event_index`, and read with the same index.
///
/// With every source at the default event threshold (`1.0`) this is uniformly
/// `ss_runtime_get_track_main_source` of track 0, so a Player that does not use
/// animation mixing can ignore it. It diverges once a secondary source opts in
/// to firing events via `ss_runtime_set_source_event_threshold`, and that is
/// when a host needs it: without it, two blended animations' events arrive in
/// one undifferentiated list.
///
/// Returns `U32_MAX` for a negative or out-of-range `index`, which no live
/// handle ever equals.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_source_handle(context: *mut c_void, index: i32) -> u32 {
    if index >= 0
        && let Some(ffi_context) = get_context(context)
        && let Some(&handle) = ffi_context.ctx.get_passed_event_source_handles().get(index as usize)
    {
        return handle;
    }
    u32::MAX
}

/// Resolve the active `EventInstance` for `slot_index` on the parent's
/// currently selected animation, at the current frame. Returns SS6
/// default semantics (`is_synthetic_default = true`, `loop_num = 1`,
/// `speed = 1.0`) when no event is active, so callers don't need to
/// branch on the no-event case.
///
/// `start_label_hash` / `end_label_hash` are returned raw. They are
/// already resolved internally by `ss_instance_slot_step` for the
/// child runtime — most Players should consume the slot helper rather
/// than resolving labels themselves.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_active_event_instance(context: *mut c_void, slot_index: u32) -> ss_event_instance_info {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_active_event_instance(slot_index);
    }
    ss_event_instance_info::synthetic_default()
}

/// Resolve the active `EventEffect` for `slot_index` on the parent's
/// currently selected animation, at the current frame. When no real event
/// has fired yet, falls back to `InitialEvents.effect` for the slot (with
/// `is_synthetic_default = true`); if neither exists, returns full SS6
/// defaults (`start_time = 0`, `speed = 1.0`, `independent = false`).
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_active_effect_event(context: *mut c_void, slot_index: u32) -> ss_effect_event_info {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_active_effect_event(slot_index);
    }
    ss_effect_event_info::synthetic_default()
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid null-terminated string.
pub unsafe extern "C" fn ss_runtime_hash_string(s: *const c_char) -> u32 {
    if s.is_null() {
        return 0;
    }
    let c_str = unsafe { core::ffi::CStr::from_ptr(s) };
    if let Ok(rust_str) = c_str.to_str() { crate::fnv1a_hash_str(rust_str) } else { 0 }
}

// --- Override Layer API (Phase 2) ---

/// Resolve a part name to its `part_index` in the current primary animation.
/// Returns `-1` on null context / null or invalid name / not found.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ss_runtime_find_part_index(context: *mut c_void, part_name: *const c_char) -> i32 {
    let Some(ffi_context) = get_context(context) else {
        return -1;
    };
    if part_name.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(part_name) };
    let Ok(name) = c_str.to_str() else {
        return -1;
    };
    ffi_context.ctx.find_part_index(name)
}

/// Set or clear a per-part visibility override.
/// `force`: `0` = clear/revert, `1` = force-hide, `2` = force-visible (reserved).
/// `cascade != 0` propagates a force-hide to descendants.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_visibility_override(context: *mut c_void, part: u32, force: c_uchar, cascade: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.set_part_visibility_override(part, force, cascade != 0) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_set_part_visibility_override", e);
            false
        }
    }
}

/// Clear a part's visibility override (revert to animation).
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_visibility_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.clear_part_visibility_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_clear_part_visibility_override", e);
            false
        }
    }
}

/// Set a per-part color override from a single packed RGBA (`0xRRGGBBAA`, fills
/// all four corners). `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`. `priority_mode`:
/// `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent`. Normal
/// parts only.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_color_override(context: *mut c_void, part: u32, blend_op: c_uchar, rgba: u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.set_part_color_override(part, blend_op, rgba, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_set_part_color_override", e);
            false
        }
    }
}

/// Set a per-part color override from four packed RGBA corners in ssruntime
/// order `lt, rt, lb, rb` (`rgba_ltrtlbrb` points to 4 `uint32` = `0xRRGGBBAA`).
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ss_runtime_set_part_color_override_corners(context: *mut c_void, part: u32, blend_op: c_uchar, rgba_ltrtlbrb: *const u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    if rgba_ltrtlbrb.is_null() {
        ffi_context.fail("ss_runtime_set_part_color_override_corners", crate::error::SSRuntimeError::InvalidParameter);
        return false;
    }
    let packed = unsafe { core::slice::from_raw_parts(rgba_ltrtlbrb, 4) };
    let corners = [packed[0].to_be_bytes(), packed[1].to_be_bytes(), packed[2].to_be_bytes(), packed[3].to_be_bytes()];
    match ffi_context.ctx.set_part_color_override_corners(part, blend_op, corners, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_set_part_color_override_corners", e);
            false
        }
    }
}

/// Clear a part's color override.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_color_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.clear_part_color_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_clear_part_color_override", e);
            false
        }
    }
}

/// Set a per-part cell override, identified by FNV-1a hashes of the cellmap name
/// (without `.ssce`) and the cell name. `(0, 0)` reverts to the animation.
/// `priority_mode`: `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=
/// Permanent`. Normal + Mask parts only. Fails if the cell cannot be resolved.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_cell_override(context: *mut c_void, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.set_part_cell_override(part, cellmap_name_hash, cell_name_hash, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_set_part_cell_override", e);
            false
        }
    }
}

/// Clear a part's cell override.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_cell_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    match ffi_context.ctx.clear_part_cell_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.fail("ss_runtime_clear_part_cell_override", e);
            false
        }
    }
}

/// Clear every per-part override on the context.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_all_part_overrides(context: *mut c_void) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = None;
    ffi_context.ctx.clear_all_part_overrides();
    true
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use core::ffi::c_void;
    use std::ffi::{CStr, CString};

    /// The code a context is holding, and the wording that code maps to.
    fn read_error(ctx: *mut c_void) -> (i32, std::string::String) {
        let code = ss_runtime_get_last_error(ctx);
        let text = unsafe { CStr::from_ptr(ss_runtime_error_message(code)) };
        (code, text.to_string_lossy().into_owned())
    }

    /// A valid FlatBuffer root whose `version` is not the one this runtime was
    /// built against — the failure a user hits by playing a pack from a
    /// different converter, and the reason the message has to carry numbers.
    fn ssab_with_wrong_version() -> std::vec::Vec<u8> {
        use ssab::ssab::ss::format as fmt;
        let mut b = flatbuffers::FlatBufferBuilder::new();
        let name = b.create_string("wrong_version");
        let parts = b.create_vector::<flatbuffers::ForwardsUOffset<fmt::PartData>>(&[]);
        let animations = b.create_vector::<flatbuffers::ForwardsUOffset<fmt::AnimationData>>(&[]);
        let keyframes = crate::core::keyframes::create_empty_keyframes(&mut b);
        let root = fmt::SsAnimeBinary::create(&mut b, &fmt::SsAnimeBinaryArgs { version: ssab::SSAB_SCHEMA_VERSION + 1, name: Some(name), parts: Some(parts), animations: Some(animations), keyframes: Some(keyframes), ..Default::default() });
        fmt::finish_ss_anime_binary_buffer(&mut b, root);
        b.finished_data().to_vec()
    }

    /// A failure is reported on both faces: the code for the caller that checks
    /// the return value, and a log line for the one that does not. The line
    /// names the entry point, because "SSAB data is not loaded" on its own does
    /// not say which call said so.
    #[test]
    fn a_failure_reaches_the_log_callback_tagged_with_its_entry_point() {
        static LINES: std::sync::Mutex<std::vec::Vec<(i32, std::string::String)>> = std::sync::Mutex::new(std::vec::Vec::new());

        extern "C" fn record(level: c_int, message: *const c_char) {
            let text = unsafe { CStr::from_ptr(message) }.to_string_lossy().into_owned();
            LINES.lock().unwrap().push((level, text));
        }
        ss_runtime_set_log_callback(record);

        let ctx = ss_runtime_create();
        let name = CString::new("walk").unwrap();
        // Nothing bound, so the runtime has no animation to set up.
        assert!(!unsafe { ss_runtime_setup_animation(ctx, name.as_ptr()) });
        ss_runtime_destroy(ctx);

        // The callback is a process-global the ABI gives no way to clear, so
        // other tests' failures land in here too -- look for this one rather
        // than counting.
        let lines = LINES.lock().unwrap();
        assert!(lines.iter().any(|(level, text)| *level == SS_RUNTIME_LOG_ERROR && text.starts_with("ss_runtime_setup_animation: ")), "the failure never reached the log callback: {lines:?}");
    }

    #[test]
    fn a_fresh_context_reports_no_error() {
        let ctx = ss_runtime_create();
        assert_eq!(read_error(ctx), (0, std::string::String::new()));
        ss_runtime_destroy(ctx);
    }

    #[test]
    fn a_failure_reports_a_code_and_the_matching_text() {
        let ctx = ss_runtime_create();
        let name = CString::new("walk").unwrap();
        // Nothing bound, so the runtime has no animation to set up.
        assert!(!unsafe { ss_runtime_setup_animation(ctx, name.as_ptr()) });
        let (code, text) = read_error(ctx);
        assert_eq!(code, crate::error::SSRuntimeError::NotLoaded.as_code());
        assert_eq!(text, "SSAB data is not loaded");
        ss_runtime_destroy(ctx);
    }

    #[test]
    fn the_reason_is_always_the_most_recent_one() {
        let ctx = ss_runtime_create();
        let name = CString::new("walk").unwrap();
        assert!(!unsafe { ss_runtime_setup_animation(ctx, name.as_ptr()) });
        assert_eq!(ss_runtime_get_last_error(ctx), crate::error::SSRuntimeError::NotLoaded.as_code());

        // A different failure replaces it rather than being masked by it.
        assert!(!unsafe { ss_runtime_load_ssab_borrow(ctx, core::ptr::null(), 0) });
        assert_eq!(ss_runtime_get_last_error(ctx), crate::error::SSRuntimeError::InvalidParameter.as_code());

        // And a success clears it: the field says "did the last call fail", not
        // "has anything ever failed".
        let ssab = ssab_with_wrong_version();
        let _ = unsafe { ss_runtime_load_ssab_borrow(ctx, ssab.as_ptr(), ssab.len()) };
        ss_runtime_reset(ctx);
        assert_eq!(read_error(ctx), (0, std::string::String::new()));
        ss_runtime_destroy(ctx);
    }

    /// The message no longer carries the two version numbers — `as_str` is
    /// payload-free and the numbers come from `ss_runtime_ssab_schema_version`.
    /// What this pins is the classification: a pack from another converter used
    /// to arrive as `MemoryError`, which sends the user looking at the wrong
    /// thing entirely.
    #[test]
    fn a_version_mismatch_is_not_reported_as_a_memory_error() {
        let ctx = ss_runtime_create();
        let ssab = ssab_with_wrong_version();
        assert!(!unsafe { ss_runtime_load_ssab_borrow(ctx, ssab.as_ptr(), ssab.len()) });

        let (code, text) = read_error(ctx);
        // Not MemoryError, which is what this path used to report for every
        // failed parse.
        assert_eq!(code, crate::error::SSRuntimeError::InvalidVersion { expected: 0, got: 0 }.as_code());
        assert_eq!(text, "Invalid SSAB version");
        ss_runtime_destroy(ctx);
    }

    /// The porting guide may only name C entry points that exist.
    ///
    /// This has gone wrong three times, and the expensive one was silent:
    /// `ss_runtime_get_last_error` was described in two places as the way to
    /// retrieve a failure's reason while no such symbol was ever exported, so
    /// every Player built on the C ABI reported "it failed" and nothing more.
    /// A reader has no way to tell a documented-but-absent API from one they
    /// merely cannot find, so the guide has to be checked against the exports
    /// rather than against memory.
    #[test]
    fn the_guide_only_names_entry_points_that_exist() {
        // Names the guide is allowed to mention without exporting them.
        const ALLOWED: &[(&str, &str)] = &[];

        // Both C ABIs the guide covers: the runtime's, and the converter's —
        // the editor-integration page documents `ss_converter_*`, and a check
        // that only knew about this crate would call every one of them a ghost.
        let mut exported = exported_c_symbols(&std::fs::read_to_string("src/ffi/mod.rs").expect("read ffi/mod.rs"));
        exported.extend(exported_c_symbols(&std::fs::read_to_string("../ssconverter/src/ffi/mod.rs").expect("read ssconverter's ffi/mod.rs")));
        assert!(exported.len() > 100, "the export scan found only {} symbols; the scan itself is broken", exported.len());
        let families: std::collections::BTreeSet<&str> = exported.iter().map(|s| family_of(s)).collect();

        let mut ghosts = std::vec::Vec::new();
        let mut pages = 0usize;
        for entry in walk_markdown(std::path::Path::new("docs")) {
            pages += 1;
            let text = std::fs::read_to_string(&entry).expect("read a guide page");
            for name in backticked_ss_names(&text) {
                // Only names in a family this ABI actually exports. Derived from
                // the exports rather than listed, so a new family is covered the
                // day it lands — and so that `ss_partcolor_blend` (a GLSL helper
                // in a shader example) and `ss_player_release` (prose about a
                // Player's own disposal call) are read as what they are rather
                // than as missing entry points.
                if !families.contains(family_of(&name)) || exported.contains(&name) || ALLOWED.iter().any(|(p, _)| name.starts_with(p)) {
                    continue;
                }
                ghosts.push(std::format!("{}: `{}`", entry.display(), name));
            }
        }
        assert!(pages > 20, "the page walk found only {pages} pages; the walk itself is broken");
        ghosts.sort();
        ghosts.dedup();
        assert!(ghosts.is_empty(), "the guide names C entry points that are not exported:\n  {}", ghosts.join("\n  "));
    }

    /// The first two segments of a C entry point's name — `ss_runtime_play` is
    /// in the `ss_runtime` family. What tells one of this ABI's names apart from
    /// any other identifier that happens to start with `ss_`.
    fn family_of(name: &str) -> &str {
        match name.match_indices('_').nth(1) {
            Some((i, _)) => &name[..i],
            None => name,
        }
    }

    /// The wasm face reports through `js_error`, never by formatting an error
    /// itself.
    ///
    /// `Debug` on an error prints the Rust variant name, and twenty-eight call
    /// sites were doing exactly that — handing a JS caller `AnimationNotFound`,
    /// an identifier from a language they are not writing, which then reaches
    /// their user through whatever the Player shows. It also drops what
    /// `InvalidVersion` knows. One closure copied twenty-eight times is how that
    /// happened, so what is pinned here is that there are none left.
    #[test]
    fn the_wasm_face_does_not_debug_format_an_error() {
        let web = std::fs::read_to_string("src/web/mod.rs").expect("read web/mod.rs");
        assert!(web.contains("fn js_error("), "js_error is gone; this test is pinning nothing");
        assert!(!web.contains("{:?}"), "src/web/mod.rs formats something with Debug. A fallible binding reports through `js_error`.");
    }

    /// The `ss_*` symbols this module actually exports, read from its own source
    /// rather than from the generated header — the header is a build artifact,
    /// and a test that needs one built first is a test that gets skipped.
    fn exported_c_symbols(source: &str) -> std::collections::BTreeSet<std::string::String> {
        let mut out = std::collections::BTreeSet::new();
        for line in source.lines() {
            let line = line.trim_start();
            let Some(rest) = line.strip_prefix("pub ") else { continue };
            let rest = rest.strip_prefix("unsafe ").unwrap_or(rest);
            let Some(rest) = rest.strip_prefix("extern \"C\" fn ") else { continue };
            let name: std::string::String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if name.starts_with("ss_") {
                out.insert(name);
            }
        }
        out
    }

    /// Every `ss_…`-looking identifier written in backticks. Backticks are the
    /// filter that matters: prose says "the `ss_resource_create_*` pair", and a
    /// trailing `*` or `_` is a wildcard rather than a name, so those are
    /// dropped instead of being reported as missing.
    fn backticked_ss_names(text: &str) -> std::vec::Vec<std::string::String> {
        let mut out = std::vec::Vec::new();
        for span in text.split('`').skip(1).step_by(2) {
            let span = span.trim();
            if !span.starts_with("ss_") {
                continue;
            }
            let name: std::string::String = span.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            // A wildcard stem, not a symbol.
            if span.len() > name.len() && span.as_bytes()[name.len()] == b'*' {
                continue;
            }
            if name.ends_with('_') {
                continue;
            }
            out.push(name);
        }
        out
    }

    fn walk_markdown(dir: &std::path::Path) -> std::vec::Vec<std::path::PathBuf> {
        let mut out = std::vec::Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else { return out };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk_markdown(&path));
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
        out
    }

    #[test]
    fn a_null_context_answers_without_dereferencing_it() {
        let null = core::ptr::null_mut();
        assert_eq!(ss_runtime_get_last_error(null), 0);
        // 0 is "no error", and the message for it is empty rather than null, so
        // a caller can log the result without checking either.
        let msg = ss_runtime_error_message(0);
        assert!(!msg.is_null());
        assert_eq!(unsafe { CStr::from_ptr(msg) }.to_bytes(), b"");
    }

    /// The `match code` in `ss_runtime_error_message` and the `match self` in
    /// `SSRuntimeError::as_str` are two tables for one mapping. Every drift this
    /// session came from exactly that shape, so pin them to each other.
    #[test]
    fn error_message_matches_every_variant() {
        use crate::error::SSRuntimeError::*;
        let every_variant = [EmptyData, InvalidVersion { expected: 1, got: 2 }, AnimationNotFound, NotLoaded, InvalidParameter, MemoryError, MalformedEffect, Unknown];
        for e in every_variant {
            let via_abi = unsafe { CStr::from_ptr(ss_runtime_error_message(e.as_code())) };
            assert_eq!(via_abi.to_str().unwrap(), e.as_str(), "code {} disagrees with the variant", e.as_code());
            assert!(!e.as_str().is_empty(), "{:?} has no wording", e);
        }
        // An unknown code is not a panic and not a null.
        assert_eq!(unsafe { CStr::from_ptr(ss_runtime_error_message(12345)) }.to_bytes(), b"");
    }

    /// The runtime says which `.ssab` version it reads, so a Player can report a
    /// mismatch in numbers instead of parsing them out of a sentence.
    #[test]
    fn the_runtime_publishes_the_schema_version_it_expects() {
        let expected = ss_runtime_ssab_schema_version();
        assert_ne!(expected, 0);
        // The pack built one version off is exactly what `InvalidVersion` is for.
        let ctx = ss_runtime_create();
        let ssab = ssab_with_wrong_version();
        assert!(!unsafe { ss_runtime_load_ssab_borrow(ctx, ssab.as_ptr(), ssab.len()) });
        assert_eq!(ss_runtime_get_last_error(ctx), crate::error::SSRuntimeError::InvalidVersion { expected: 0, got: 0 }.as_code());
        ss_runtime_destroy(ctx);
    }
}
