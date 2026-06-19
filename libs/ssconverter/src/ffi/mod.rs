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
        Self { error: None, error_c: None, handler: None, should_stop: Arc::new(AtomicBool::new(false)), scan: None, scan_results_c: Vec::new() }
    }
}

#[repr(i32)]
pub enum CConverterError {
    Success = 0,
    ErrorIo = -1,
    ErrorSspj = -2,
    ErrorConversion = -3,
    ErrorNotYetStarted = -4,
    ErrorAborted = -5,
    ErrorUnknown = -99,
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_version() -> *const c_char {
    let s = crate::utils::version_string();
    let s = CString::new(s).unwrap_or_else(|_| CString::new("unknown").unwrap());
    s.into_raw()
}

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

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_create() -> *mut Context {
    let c = Context::new();
    Box::into_raw(Box::new(c))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_destroy(context: *mut Context) {
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        let _ = Box::from_raw(context);
    }));
}

type LogCallback = Option<extern "C" fn(*const c_char)>;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_convert(ctx: *mut Context, sspj: *const c_char, output_dir: *const c_char, callback: LogCallback) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };
        context.error = None;
        context.error_c = None;

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_is_running(ctx: *mut Context) -> bool {
    let context = unsafe { &mut *ctx };
    if let Some(scan) = &context.scan {
        return !scan.is_finished();
    }
    context.handler.as_ref().is_some_and(|h| !h.is_finished())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_is_finished(context: *mut Context) -> bool {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };
        if let Some(scan) = &context.scan {
            return scan.is_finished();
        }
        context.handler.as_ref().map(|h| h.is_finished()).unwrap_or(false)
    }));
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_abort(context: *mut Context) {
    let context = unsafe { &mut *context };
    if let Some(scan) = &context.scan {
        scan.abort();
        return;
    }
    context.should_stop.store(true, std::sync::atomic::Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_get_result(context: *mut Context) -> CConverterError {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };

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
            CConverterError::ErrorNotYetStarted
        }
    }));
    result.unwrap_or(CConverterError::ErrorUnknown)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_get_error(context: *mut Context, out_data: *mut *const c_char, out_len: *mut usize) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_dir(ctx: *mut Context, dir: *const c_char, max_depth: u32, max_entries: usize, max_millis: u64) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_visited(ctx: *mut Context) -> usize {
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.visited()).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_found(ctx: *mut Context) -> usize {
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.found()).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_budget_exceeded(ctx: *mut Context) -> bool {
    let context = unsafe { &mut *ctx };
    context.scan.as_ref().map(|s| s.budget_exceeded()).unwrap_or(false)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_result_count(ctx: *mut Context) -> usize {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };
        let paths = context.scan.as_ref().map(|s| s.results()).unwrap_or_default();
        context.scan_results_c = paths.iter().filter_map(|p| CString::new(p.to_string_lossy().as_bytes()).ok()).collect();
        context.scan_results_c.len()
    }));
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_converter_scan_result_at(ctx: *mut Context, index: usize) -> *const c_char {
    let context = unsafe { &mut *ctx };
    context.scan_results_c.get(index).map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}
