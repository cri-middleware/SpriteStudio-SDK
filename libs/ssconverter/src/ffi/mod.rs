use std::ffi::{c_char, CStr, OsString, CString};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use crate::{ConverterConfig, ConverterError};
use std::panic::{catch_unwind, AssertUnwindSafe};

// #[repr(C)]
pub struct Context {
    error: Option<String>,

    handler: Option<JoinHandle<Result<(), crate::ConverterError>>>,
    should_stop: Arc<AtomicBool>,
}
impl Context {
    pub fn new() -> Self {
        Self {
            error: None,
            handler: None,
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[repr(C)]
pub enum CConverterError {
    Success = 0,
    ErrorIo = -1,
    ErrorSspj = -2,
    ErrorConversion = -3,
    ErrorNotYetStarted = -4,
    ErrorUnknown = -99,
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_version() -> *const c_char {
    let s = crate::utils::version_string();
    let s = CString::new(s).unwrap_or_else(|_| CString::new("unknown").unwrap());
    s.into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_version_free(s: *mut c_char) {
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
pub extern "C" fn ss_converter_create() -> *mut Context {
    let c = Context::new();
    Box::into_raw(Box::new(c))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_destroy(context: *mut Context) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let _ = Box::from_raw(context);
        }
    }));
}

type LogCallback = Option<extern "C" fn(*const c_char)>;

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_convert(ctx: *mut Context, sspj: *const c_char, output_dir: *const c_char, callback: LogCallback) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *ctx };

        if sspj.is_null() || output_dir.is_null() { return; }
        let sspj_buf = unsafe { CStr::from_ptr(sspj) };
        let sspj_str = sspj_buf.to_string_lossy().into_owned();
        let sspj_path = std::path::PathBuf::from(&sspj_str);

        let output_dir_buf = unsafe { CStr::from_ptr(output_dir) };
        let output_dir_str = Some(OsString::from(output_dir_buf.to_string_lossy().into_owned()));
        let mut config = ConverterConfig::default();
        config.output = output_dir_str;

        if let Some(prev_handler)  = context.handler.take() {
            // ignore previous handler result
            let _ = prev_handler.join();
        }
        let c = config.clone();
        let handler = std::thread::spawn(move || {
            let on_log = |msg: &str| {
                if let Some(cb) = callback {
                    let c_msg = CString::new(msg).unwrap();
                    cb(c_msg.as_ptr());
                }
            };
            crate::converter::converter::convert_sspj_to_ssab_files(sspj_path.as_path(), &c, &on_log)
        });
        context.handler = Some(handler);
    }));
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_watch(ctx: *mut Context, sspj: *const c_char, output_dir: *const c_char) {
    let context = unsafe { &mut *ctx };

    let sspj_buf = unsafe { CStr::from_ptr(sspj) };
    let sspj_str = sspj_buf.to_string_lossy().into_owned();
    let sspj_path = std::path::PathBuf::from(&sspj_str);

    let output_dir_buf = unsafe { CStr::from_ptr(output_dir) };
    let output_dir_str = Some(OsString::from(output_dir_buf.to_string_lossy().into_owned()));
    let mut config = ConverterConfig::default();
    config.output = output_dir_str.clone();

    context.should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_clone = context.should_stop.clone();

    let c = config.clone();
    let handler = std::thread::spawn(move || {
        crate::watch::watch_sspj_project(sspj_path.as_path(), &c, should_stop_clone)
    });
    context.handler = Some(handler);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_is_running(ctx: *mut Context) -> bool {
    let context = unsafe { &mut *ctx };
    context.handler.as_ref().map_or(false, |h| !h.is_finished())
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_is_finished(context: *mut Context) -> bool {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };
        context.handler.as_ref().map(|h| h.is_finished()).unwrap_or(false)
    }));
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_abort(context: *mut Context) {
    let context = unsafe { &mut *context };
    context.should_stop.store(true, std::sync::atomic::Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_get_result(context: *mut Context) -> CConverterError {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };

        if let Some(h) = context.handler.take() {
            match h.join() {
                Ok(result) => {
                    match result {
                        Ok(_) => CConverterError::Success,
                        Err(e) => {
                            context.error = Some(e.to_string());
                            match e {
                                ConverterError::IoError(_) => CConverterError::ErrorIo,
                                ConverterError::SspjError(_) => CConverterError::ErrorSspj,
                                ConverterError::ConversionFailed(_) => CConverterError::ErrorConversion,
                            }
                        },
                    }
                }
                Err(_) => {
                    context.error = Some("Converter thread panicked".to_string());
                    CConverterError::Success
                }
            }
        } else {
            CConverterError::ErrorNotYetStarted
        }
    }));
    result.unwrap_or(CConverterError::ErrorUnknown)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_converter_get_error(context: *mut Context, out_data: *mut *mut c_char, out_len: *mut usize) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { &mut *context };
        unsafe {
            *out_data = context.error.as_ref().map(|s| s.as_ptr() as *mut c_char).unwrap_or(std::ptr::null_mut());
            *out_len = context.error.as_ref().map(|s| s.len()).unwrap_or(0)
        }
    }));
}
