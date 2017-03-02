use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_char;
use super::build_query;

#[no_mangle]
pub extern "C" fn ext_build_query(query_str: *const c_char) -> *const c_char {
    let query_str = unsafe { CStr::from_ptr(query_str) };
    let query_str = query_str.to_string_lossy();
    let output    = build_query(query_str.as_ref());

    // pass ownership of string to caller
    let s = CString::new(&output[..]).unwrap();
    let p = s.into_raw();
    p as *mut _
}

#[no_mangle]
pub extern "C" fn ext_free_query(query: *mut c_char) {
    mem::drop(unsafe { CString::from_raw(query) });
}
