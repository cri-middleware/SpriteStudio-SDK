//! The converter's C ABI — what an editor plugin drives to turn a `.sspj` into
//! `.ssab` files. `cbindgen` generates `ssconverter.h` from this module and opens it
//! with this text, so it is written for the C or C++ integrator.
//!
//! # Calling convention
//!
//! Both jobs here are asynchronous: `ss_converter_convert` and
//! `ss_converter_scan_dir` return as soon as the worker is spawned, and the
//! caller polls `ss_converter_is_finished`. **A context runs one of them at a
//! time** — see `ss_converter_create`.
//!
//! Every entry point tolerates a null context and returns a safe default rather
//! than dereferencing it, and each one runs its work inside `catch_unwind`.
//! **That net only exists where the binary was built with unwinding.** This
//! repository's own profiles set `panic = "abort"`, so in the artifacts it ships
//! a panic ends the host process instead of being reported here. Reporting
//! malformed input as an error rather than panicking is therefore what the
//! conversion path is held to; the `catch_unwind` above it is a second line
//! for callers who build with unwinding, not the guarantee.
//!
//! # Strings
//!
//! `ss_converter_version` is the only one that **transfers ownership**; its
//! string must be released with `ss_converter_version_free`. Every other
//! string is **borrowed from the context**, must not be freed, and stays valid
//! only until the call named in its own documentation — usually the next job on
//! that context, or the context's destruction.

use crate::scan::SspjScan;
use crate::{ConverterConfig, ConverterError};
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;

// #[repr(C)]
pub struct Context {
    error: Option<String>,
    error_c: Option<CString>,

    handler: Option<JoinHandle<Result<(), crate::ConverterError>>>,
    should_stop: Arc<AtomicBool>,
    /// The outcome of the conversion that has already been joined. Kept so that
    /// `ss_converter_get_result` answers the same way every time it is asked:
    /// joining consumes `handler`, and without this a second call would report
    /// `ErrorNotYetStarted` about a conversion that had in fact finished.
    result: Option<CConverterError>,

    scan: Option<SspjScan>,
    scan_results_c: Vec<CString>,
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self { error: None, error_c: None, handler: None, should_stop: Arc::new(AtomicBool::new(false)), result: None, scan: None, scan_results_c: Vec::new() }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CConverterError {
    Success = 0,
    ErrorIo = -1,
    ErrorSspj = -2,
    ErrorConversion = -3,
    ErrorNotYetStarted = -4,
    ErrorAborted = -5,
    ErrorUnknown = -99,
}

/// The converter's version string.
///
/// **The caller owns the returned string** and must hand it to
/// `ss_converter_version_free`. It is the one function here that transfers
/// ownership; every other `*const c_char` this ABI returns is borrowed from a
/// context and must not be freed.
#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_version() -> *const c_char {
    let s = crate::utils::version_string();
    let s = CString::new(s).unwrap_or_else(|_| CString::new("unknown").unwrap());
    s.into_raw()
}

/// Release a string from `ss_converter_version`. Null is accepted and ignored.
/// Passing anything else — including a pointer from
/// `ss_converter_scan_result_at` or `ss_converter_get_error` — is undefined.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_version_free(s: *mut c_char) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if s.is_null() {
            return;
        }
        unsafe {
            let _ = CString::from_raw(s);
        }
    }));
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_ssab_schema_version() -> u32 {
    crate::ssab_schema_version()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_ssqb_schema_version() -> u32 {
    crate::ssqb_schema_version()
}

/// A conversion context, or null if the allocation failed.
///
/// **A context runs one job at a time** — a conversion
/// (`ss_converter_convert`) or a directory scan (`ss_converter_scan_dir`).
/// Starting a conversion drops any earlier scan, because `is_running`,
/// `is_finished` and `abort` all answer about whichever job is current. Use two
/// contexts to run a scan and a conversion at once.
///
/// Release it with `ss_converter_destroy`.
#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_create() -> *mut Context {
    let c = Context::new();
    Box::into_raw(Box::new(c))
}

/// Release a context from `ss_converter_create`. Null is accepted and ignored.
///
/// Every borrowed pointer taken from this context — the scan results, the error
/// text — dies with it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_destroy(context: *mut Context) {
    if context.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        let _ = Box::from_raw(context);
    }));
}

type LogCallback = Option<extern "C" fn(*const c_char)>;

/// Start converting one `.sspj` into `output_dir`, **on a worker thread**.
///
/// Returns immediately. Poll `ss_converter_is_finished`, then call
/// `ss_converter_get_result` for the outcome and `ss_converter_get_error`
/// for the text when it is not `Success`.
///
/// `callback` is invoked **on the worker thread** with a message that is only
/// valid for the duration of the call; copy anything you keep, and do not touch
/// a UI toolkit from it.
///
/// Starting a conversion clears this context's previous result, error text and
/// scan — see `ss_converter_create` on one job at a time.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_convert(ctx: *mut Context, sspj: *const c_char, output_dir: *const c_char, embedded: bool, callback: LogCallback) {
    if ctx.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };
        context.error = None;
        context.error_c = None;
        context.result = None;
        // A context does one job at a time. `scan` is what `is_running`,
        // `is_finished` and `abort` consult first, so a context that had scanned
        // earlier would keep answering about that finished scan — reporting the
        // conversion as already finished, and refusing to abort it.
        context.scan = None;
        context.scan_results_c.clear();

        if sspj.is_null() || output_dir.is_null() {
            return;
        }
        let sspj_buf = unsafe { CStr::from_ptr(sspj) };
        let sspj_str = sspj_buf.to_string_lossy().into_owned();
        let sspj_path = std::path::PathBuf::from(&sspj_str);

        let output_dir_buf = unsafe { CStr::from_ptr(output_dir) };
        let output_dir_str = Some(std::ffi::OsString::from(output_dir_buf.to_string_lossy().into_owned()));
        let mut config = ConverterConfig::default();
        config.output = output_dir_str;
        config.convert_config.embedded = embedded;

        if let Some(prev_handler) = context.handler.take() {
            // ignore previous handler result
            let _ = prev_handler.join();
        }

        context.should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_clone = context.should_stop.clone();

        let c = config.clone();
        let handler = std::thread::spawn(move || {
            let on_log = |msg: &str| {
                if let Some(cb) = callback
                    && let Ok(c_msg) = CString::new(msg)
                {
                    cb(c_msg.as_ptr());
                }
            };
            let result = catch_unwind(AssertUnwindSafe(|| crate::converter::converter::convert_sspj_to_ssab_files_cancellable(sspj_path.as_path(), &c, &on_log, &should_stop_clone)));
            match result {
                Ok(res) => res,
                Err(_) => Err(crate::ConverterError::ConversionFailed("Rust thread panicked".into())),
            }
        });
        context.handler = Some(handler);
    }));
}

/// Whether this context's current job is still going. The complement of
/// `ss_converter_is_finished` for a job that has started; both are `false` on
/// a context that has never been given one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_is_running(ctx: *mut Context) -> bool {
    if ctx.is_null() {
        return false;
    }
    let context = unsafe { &mut *ctx };
    if let Some(scan) = &context.scan {
        return !scan.is_finished();
    }
    context.handler.as_ref().is_some_and(|h| !h.is_finished())
}

/// Whether this context's current job has run to completion (or was aborted).
/// `false` on a context that has never been given one, so this is "done", not
/// "idle".
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_is_finished(context: *mut Context) -> bool {
    if context.is_null() {
        return false;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };
        if let Some(scan) = &context.scan {
            return scan.is_finished();
        }
        context.handler.as_ref().map(|h| h.is_finished()).unwrap_or(false)
    }));
    result.unwrap_or(false)
}

/// Ask this context's current job to stop early. Returns immediately — the
/// worker notices at its next checkpoint, so keep polling
/// `ss_converter_is_finished`. An aborted conversion reports `ErrorAborted`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_abort(context: *mut Context) {
    if context.is_null() {
        return;
    }
    let context = unsafe { &mut *context };
    if let Some(scan) = &context.scan {
        scan.abort();
        return;
    }
    context.should_stop.store(true, std::sync::atomic::Ordering::Relaxed)
}

/// The outcome of the conversion started on this context.
///
/// **Blocks until the worker finishes**, so call `ss_converter_is_finished`
/// first unless blocking the calling thread is acceptable — on an editor's UI
/// thread it is not.
///
/// Asking twice is safe and gives the same answer. `ErrorNotYetStarted` means
/// exactly that: no conversion was ever started on this context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_get_result(context: *mut Context) -> CConverterError {
    if context.is_null() {
        return CConverterError::ErrorUnknown;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };

        // Already joined once: the answer does not change, and the handler is
        // gone, so re-deriving it would produce `ErrorNotYetStarted`.
        if let Some(cached) = context.result {
            return cached;
        }

        if let Some(h) = context.handler.take() {
            match h.join() {
                Ok(result) => match result {
                    Ok(_) => CConverterError::Success,
                    Err(e) => {
                        let err_str = e.to_string();
                        context.error_c = CString::new(err_str.clone()).ok();
                        context.error = Some(err_str);
                        match e {
                            ConverterError::IoError(_) => CConverterError::ErrorIo,
                            ConverterError::SspjError(_) => CConverterError::ErrorSspj,
                            ConverterError::ConversionFailed(_) => CConverterError::ErrorConversion,
                            ConverterError::Aborted => CConverterError::ErrorAborted,
                        }
                    }
                },
                Err(_) => {
                    let err_str = "Converter thread panicked".to_string();
                    context.error_c = CString::new(err_str.clone()).ok();
                    context.error = Some(err_str);
                    CConverterError::ErrorUnknown
                }
            }
        } else {
            // Nothing was ever started on this context. Not an outcome, so the
            // caching below deliberately skips it.
            CConverterError::ErrorNotYetStarted
        }
    }));
    let outcome = result.unwrap_or(CConverterError::ErrorUnknown);
    if outcome != CConverterError::ErrorNotYetStarted && !context.is_null() {
        unsafe { &mut *context }.result = Some(outcome);
    }
    outcome
}

/// The failure text behind a non-`Success` `ss_converter_get_result`, as
/// UTF-8 with its length. Either out pointer may be null.
///
/// Writes a null pointer and a zero length when there is no error — including
/// before `ss_converter_get_result` has been called, which is what fills this
/// in. **The text is borrowed from the context**: do not free it, and do not
/// keep it past the next conversion on that context or the context's own
/// destruction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_get_error(context: *mut Context, out_data: *mut *const c_char, out_len: *mut usize) {
    if context.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };
        unsafe {
            if !out_data.is_null() {
                *out_data = context.error_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
            }
            if !out_len.is_null() {
                *out_len = context.error.as_ref().map(|s| s.len()).unwrap_or(0);
            }
        }
    }));
}

/// Start scanning `dir` for `.sspj` files, **on a worker thread**.
///
/// Returns immediately; poll `ss_converter_is_finished`. The three budgets
/// bound the walk — `max_depth` directories deep, `max_entries` entries, and
/// `max_millis` of wall clock; `ss_converter_scan_budget_exceeded` reports
/// whether one of them cut the walk short, in which case the results are a
/// prefix rather than the whole tree.
///
/// See `ss_converter_create` on one job at a time: a context scanning is not
/// a context converting.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_dir(ctx: *mut Context, dir: *const c_char, max_depth: u32, max_entries: usize, max_millis: u64) {
    if ctx.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };
        if dir.is_null() {
            return;
        }
        let dir_path = std::path::PathBuf::from(unsafe { CStr::from_ptr(dir) }.to_string_lossy().into_owned());
        context.scan_results_c.clear();
        context.scan = Some(SspjScan::start(dir_path, max_depth as usize, max_entries, max_millis));
    }));
}

/// Directory entries the scan has looked at so far — progress, not results.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_visited(ctx: *mut Context) -> usize {
    if ctx.is_null() {
        return 0;
    }
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.visited()).unwrap_or(0)
}

/// `.sspj` files the scan has found so far. Safe to poll while it runs, unlike
/// `ss_converter_scan_result_count`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_found(ctx: *mut Context) -> usize {
    if ctx.is_null() {
        return 0;
    }
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.found()).unwrap_or(0)
}

/// Whether one of `ss_converter_scan_dir`'s budgets stopped the walk before
/// it reached the end of the tree. The results are still usable; they are just
/// not everything.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_budget_exceeded(ctx: *mut Context) -> bool {
    if ctx.is_null() {
        return false;
    }
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.budget_exceeded()).unwrap_or(false)
}

/// How many `.sspj` paths the scan has, and **the call that publishes them**.
///
/// Despite the name this rebuilds the context's result strings from the
/// scanner, which is what makes them readable by
/// `ss_converter_scan_result_at` — so call it before the first read, and
/// again whenever you want newer results.
///
/// **It invalidates every pointer a previous `ss_converter_scan_result_at`
/// handed out.** Read the paths you want, or copy them, before calling this
/// again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_result_count(ctx: *mut Context) -> usize {
    if ctx.is_null() {
        return 0;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };
        let paths = context.scan.as_ref().map(|s| s.results()).unwrap_or_default();
        context.scan_results_c = paths.iter().filter_map(|p| CString::new(p.to_string_lossy().as_bytes()).ok()).collect();
        context.scan_results_c.len()
    }));
    result.unwrap_or(0)
}

/// One `.sspj` path from the scan, as UTF-8, or null when `index` is past the
/// count that `ss_converter_scan_result_count` last reported.
///
/// **Borrowed from the context** — do not free it. It dies at the next
/// `ss_converter_scan_result_count`, the next `ss_converter_convert` on
/// this context, or the context's own destruction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_result_at(ctx: *mut Context, index: usize) -> *const c_char {
    if ctx.is_null() {
        return std::ptr::null();
    }
    let context = unsafe { &mut *ctx };
    context.scan_results_c.get(index).map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: every C-API entry point must tolerate a NULL context handle
    // (e.g. a caller that ignored a failed create) and return a safe default
    // instead of dereferencing NULL. These calls previously segfaulted, which
    // the surrounding catch_unwind cannot rescue.
    #[test]
    fn null_context_returns_safe_defaults() {
        let null: *mut Context = std::ptr::null_mut();
        unsafe {
            assert!(!ss_converter_is_running(null));
            assert!(!ss_converter_is_finished(null));
            assert!(matches!(ss_converter_get_result(null), CConverterError::ErrorUnknown));
            assert_eq!(ss_converter_scan_visited(null), 0);
            assert_eq!(ss_converter_scan_found(null), 0);
            assert!(!ss_converter_scan_budget_exceeded(null));
            assert_eq!(ss_converter_scan_result_count(null), 0);
            assert!(ss_converter_scan_result_at(null, 0).is_null());
            // Void-returning entry points must simply not crash on NULL.
            ss_converter_abort(null);
            ss_converter_destroy(null);
        }
    }

    /// `get_result` may be asked more than once.
    ///
    /// Joining consumes the handler, so the second call used to report
    /// `ErrorNotYetStarted` — a conversion that had in fact finished, described
    /// as one that never began. Any caller that logs the outcome and then
    /// branches on it reads the lie.
    #[test]
    fn get_result_answers_the_same_way_every_time() {
        unsafe {
            let ctx = ss_converter_create();
            assert!(!ctx.is_null());

            // Nothing started: that is not an outcome, and asking twice must not
            // turn it into one.
            assert_eq!(ss_converter_get_result(ctx), CConverterError::ErrorNotYetStarted);
            assert_eq!(ss_converter_get_result(ctx), CConverterError::ErrorNotYetStarted);

            // A conversion that fails (no such .sspj) still has an outcome, and
            // it has to survive being read.
            let sspj = CString::new("/nonexistent/does_not_exist.sspj").unwrap();
            let out = CString::new(std::env::temp_dir().to_string_lossy().as_ref()).unwrap();
            ss_converter_convert(ctx, sspj.as_ptr(), out.as_ptr(), false, None);
            let first = ss_converter_get_result(ctx);
            assert_ne!(first, CConverterError::ErrorNotYetStarted, "the conversion did run");
            assert_eq!(ss_converter_get_result(ctx), first, "the second read disagreed with the first");
            assert_eq!(ss_converter_get_result(ctx), first);

            ss_converter_destroy(ctx);
        }
    }

    /// Starting a conversion makes the conversion this context's job.
    ///
    /// `is_running` / `is_finished` / `abort` all consult `scan` first, and
    /// nothing used to clear it, so a context that had ever scanned reported
    /// that finished scan forever: the conversion looked done the moment it
    /// started, and `abort` returned without ever setting the stop flag.
    ///
    /// Asserted on the state rather than on timing — a conversion of a missing
    /// file finishes too fast to catch `is_running` in the act, but the scan's
    /// own progress counters are deterministic: non-zero after a scan that
    /// walked something, and gone once a conversion takes the context over.
    #[test]
    fn starting_a_conversion_takes_the_context_off_an_earlier_scan() {
        unsafe {
            let ctx = ss_converter_create();
            // The crate root: whatever else is true, it has entries to visit.
            let dir = CString::new(".").unwrap();
            ss_converter_scan_dir(ctx, dir.as_ptr(), 1, 64, 2_000);
            while ss_converter_is_running(ctx) {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert!(ss_converter_is_finished(ctx), "the scan should have finished");
            assert!(ss_converter_scan_visited(ctx) > 0, "the scan visited nothing; the fixture is wrong, not the code");

            let sspj = CString::new("/nonexistent/does_not_exist.sspj").unwrap();
            let out = CString::new(std::env::temp_dir().to_string_lossy().as_ref()).unwrap();
            ss_converter_convert(ctx, sspj.as_ptr(), out.as_ptr(), false, None);

            // The scan is no longer what this context reports on.
            assert_eq!(ss_converter_scan_visited(ctx), 0, "the finished scan is still answering for the conversion");
            assert_eq!(ss_converter_scan_found(ctx), 0);
            assert_eq!(ss_converter_scan_result_count(ctx), 0);

            assert_ne!(ss_converter_get_result(ctx), CConverterError::ErrorNotYetStarted);
            ss_converter_destroy(ctx);
        }
    }
}
