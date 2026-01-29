use crate::ffi_types::{CString, c_char, c_int, c_uint, c_void, size_t};

use crate::ffi;
use crate::{RingList, RingState};

#[inline]
pub fn ring_state_new() -> RingState {
    unsafe { ffi::ring_state_new() }
}

#[inline]
pub fn ring_state_init() -> RingState {
    unsafe { ffi::ring_state_init() }
}

#[inline]
pub fn ring_state_delete(state: RingState) -> RingState {
    unsafe { ffi::ring_state_delete(state) }
}

#[inline]
pub fn ring_state_runcode(state: RingState, code: &[u8]) {
    unsafe { ffi::ring_state_runcode(state, code.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_runcode`].
pub fn ring_state_runcode_str(state: RingState, code: &str) {
    if let Ok(cstr) = CString::new(code) {
        unsafe { ffi::ring_state_runcode(state, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_state_findvar(state: RingState, name: &[u8]) -> RingList {
    unsafe { ffi::ring_state_findvar(state, name.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_findvar`].
pub fn ring_state_findvar_str(state: RingState, name: &str) -> RingList {
    if let Ok(cstr) = CString::new(name) {
        unsafe { ffi::ring_state_findvar(state, cstr.as_ptr()) }
    } else {
        std::ptr::null_mut()
    }
}

#[inline]
pub fn ring_state_newvar(state: RingState, name: &[u8]) -> RingList {
    unsafe { ffi::ring_state_newvar(state, name.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_newvar`].
pub fn ring_state_newvar_str(state: RingState, name: &str) -> RingList {
    if let Ok(cstr) = CString::new(name) {
        unsafe { ffi::ring_state_newvar(state, cstr.as_ptr()) }
    } else {
        std::ptr::null_mut()
    }
}

#[inline]
pub fn ring_state_runfile(state: RingState, filename: &[u8]) -> c_int {
    unsafe { ffi::ring_state_runfile(state, filename.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_runfile`].
pub fn ring_state_runfile_str(state: RingState, filename: &str) -> c_int {
    if let Ok(cstr) = CString::new(filename) {
        unsafe { ffi::ring_state_runfile(state, cstr.as_ptr()) }
    } else {
        0
    }
}

#[inline]
pub fn ring_state_runstring(state: RingState, code: &[u8]) -> c_int {
    unsafe { ffi::ring_state_runstring(state, code.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_runstring`].
pub fn ring_state_runstring_str(state: RingState, code: &str) -> c_int {
    if let Ok(cstr) = CString::new(code) {
        unsafe { ffi::ring_state_runstring(state, cstr.as_ptr()) }
    } else {
        0
    }
}

#[inline]
pub fn ring_state_malloc(state: RingState, size: size_t) -> *mut c_void {
    unsafe { ffi::ring_state_malloc(state, size) }
}

#[inline]
pub fn ring_state_calloc(state: RingState, count: size_t, size: size_t) -> *mut c_void {
    unsafe { ffi::ring_state_calloc(state, count, size) }
}

#[inline]
pub fn ring_state_realloc(
    state: RingState,
    ptr: *mut c_void,
    old_size: size_t,
    new_size: size_t,
) -> *mut c_void {
    unsafe { ffi::ring_state_realloc(state, ptr, old_size, new_size) }
}

#[inline]
pub fn ring_state_free(state: RingState, ptr: *mut c_void) {
    unsafe { ffi::ring_state_free(state, ptr) }
}

#[inline]
pub fn ring_state_log(state: RingState, msg: &[u8]) {
    unsafe { ffi::ring_state_log(state, msg.as_ptr() as *const c_char) }
}

#[inline]
/// String variant of [`ring_state_log`].
pub fn ring_state_log_str(state: RingState, msg: &str) {
    if let Ok(cstr) = CString::new(msg) {
        unsafe { ffi::ring_state_log(state, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_state_exit(state: RingState, exit_code: c_int) {
    unsafe { ffi::ring_state_exit(state, exit_code) }
}

#[inline]
pub fn ring_state_runobjectfile(state: RingState, filename: &mut [u8]) {
    unsafe { ffi::ring_state_runobjectfile(state, filename.as_mut_ptr() as *mut c_char) }
}

#[inline]
pub fn ring_state_runobjectstring(state: RingState, code: &mut [u8], filename: &[u8]) {
    unsafe {
        ffi::ring_state_runobjectstring(
            state,
            code.as_mut_ptr() as *mut c_char,
            code.len() as c_uint,
            filename.as_ptr() as *const c_char,
        )
    }
}

#[inline]
pub fn ring_state_runprogram(state: RingState) {
    unsafe { ffi::ring_state_runprogram(state) }
}

#[inline]
pub fn ring_state_newbytecode(state: RingState, size: c_uint, literal: c_uint) {
    unsafe { ffi::ring_state_newbytecode(state, size, literal) }
}

#[inline]
pub fn ring_state_runbytecode(state: RingState) {
    unsafe { ffi::ring_state_runbytecode(state) }
}

#[inline]
pub fn ring_state_cgiheader(state: RingState) {
    unsafe { ffi::ring_state_cgiheader(state) }
}

#[inline]
pub fn ring_state_registerblock(state: RingState, start: *mut c_void, end: *mut c_void) {
    unsafe { ffi::ring_state_registerblock(state, start, end) }
}

#[inline]
pub fn ring_state_unregisterblock(state: RingState, start: *mut c_void) {
    unsafe { ffi::ring_state_unregisterblock(state, start) }
}

#[inline]
pub fn ring_state_willunregisterblock(state: RingState, start: *mut c_void) {
    unsafe { ffi::ring_state_willunregisterblock(state, start) }
}

#[inline]
pub fn ring_vm_loadcfunctions(state: RingState) {
    unsafe { ffi::ring_vm_loadcfunctions(state) }
}

#[inline]
pub fn ring_vm_generallib_loadfunctions(state: RingState) {
    unsafe { ffi::ring_vm_generallib_loadfunctions(state) }
}
