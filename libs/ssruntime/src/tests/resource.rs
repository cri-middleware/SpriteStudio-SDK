//! `SsabData` and `SsabResource` tests.

use super::*;

#[test]
fn test_ssab_data_and_resource_separation() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    assert_eq!(ssab_data.as_slice().len(), input_buf.len());

    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to create AnimationResource");

    let mut context = Context::new();
    context.bind_resource(&resource);

    let names = resource.get_animation_names();
    assert!(!names.is_empty());
    assert!(names.contains(&alloc::string::String::from("anime_1")));

    context.setup_animation("anime_1").expect("Failed to setup animation");
    assert_eq!(context.get_frame_no(), 0.0);
}

#[test]
fn test_ssab_resource_accessors() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let res = unsafe { SsabResource::new(ssab_data) };

    assert!(res.resource().is_some());

    assert_eq!(res.data().as_slice().len(), input_buf.len());

    let ptr = res.resource_static_ptr();
    assert!(!ptr.is_null());
    let safe_ref_addr = res.resource().unwrap() as *const _ as usize;
    assert_eq!(ptr as usize, safe_ref_addr);
}

#[test]
fn test_load_borrow_rejects_null_pointer() {
    // `load_borrow` must not construct an SsabData wrapping a null pointer,
    // because `as_slice` would then call `from_raw_parts(null, len)` (UB for len > 0).
    let result = unsafe { SsabData::load_borrow(::core::ptr::null(), 16) };
    match result {
        Err(error::SSRuntimeError::InvalidParameter) => {}
        _ => panic!("Expected InvalidParameter for null pointer"),
    }
}

#[test]
fn test_ssab_resource_arc_sharing_keeps_buffer_alive() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let res = unsafe { SsabResource::new(ssab_data) };

    // Take a raw pointer, then drop the original Arc handle while a clone keeps
    // the allocation alive. The pointer must remain valid through the clone.
    let ptr = res.resource_static_ptr();
    assert!(!ptr.is_null());
    let clone = res.clone();
    drop(res);

    // Safety: `clone` still owns the allocation.
    let names = unsafe { (*ptr).get_animation_names() };
    assert!(names.contains(&alloc::string::String::from("anime_1")));
    drop(clone);
}
