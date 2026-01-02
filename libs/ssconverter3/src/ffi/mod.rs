use std::ffi::{c_char, c_uchar, c_void, CStr, OsString};
use std::path::{Path, PathBuf};
use crate::{ConvertConfig, SspjLoader};

#[repr(C)]
pub struct CConvertConfig {
    pub debug: bool,
    pub split: bool,
}
#[repr(C)]
pub struct CConverterConfig {
    convert_config: CConvertConfig,
    out_dir: Option<OsString>,
}
#[repr(C)]
pub struct CConverterResultItem {
    pub name: *mut c_char,
    pub data: *mut u8,
    pub data_len: usize,
}

#[repr(C)]
pub struct CConverterResult {
    pub items: *mut CConverterResultItem,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub fn ss_converter_convert_sspj_to_ssab_data(path: *mut c_char) -> *mut CConverterResult {
    let c_buf = unsafe { CStr::from_ptr(path) };
    let p = c_buf.to_string_lossy().into_owned();
    let path_buf = std::path::PathBuf::from(&p);
    let r = crate::converter::converter::convert_sspj_to_ssab_data(path_buf.as_path(), &ConvertConfig::default());
    let Ok(ssab_bin_map) = r else {
        return std::ptr::null_mut();
    };

    let mut vec = ssab_bin_map.into_iter().map(|(name, data)| {
        let c_name = std::ffi::CString::new(name).unwrap_or_default();
        let mut data_vec = data;
        let data_ptr = data_vec.as_mut_ptr();
        let data_len = data_vec.len();
        
        std::mem::forget(data_vec);

        CConverterResultItem { 
            name: c_name.into_raw(), 
            data: data_ptr, 
            data_len: data_len 
        }
    }).collect::<Vec<CConverterResultItem>>();

    let len = vec.len();
    let items_ptr = vec.as_mut_ptr();
    std::mem::forget(vec); // Vecのバッファが解放されないようにする

    let result = CConverterResult { items: items_ptr, len };
    Box::into_raw(Box::new(result))
}
