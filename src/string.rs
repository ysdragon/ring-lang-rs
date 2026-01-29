use crate::ffi_types::{CString, c_char, c_int, c_uint};

use crate::ffi::{self, RingString};

#[inline]
pub fn ring_string_new(s: &[u8]) -> RingString {
    unsafe { ffi::ring_string_new(s.as_ptr() as *const c_char) }
}

/// String variant of [`ring_string_new`] - accepts `&str` instead of `&[u8]`.
#[inline]
pub fn ring_string_new_str(s: &str) -> RingString {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_string_new(cstr.as_ptr()) }
    } else {
        unsafe { ffi::ring_string_new(b"\0".as_ptr() as *const c_char) }
    }
}

#[inline]
pub fn ring_string_new2(s: &[u8]) -> RingString {
    unsafe { ffi::ring_string_new2(s.as_ptr() as *const c_char, s.len() as c_uint) }
}

#[inline]
pub fn ring_string_delete(s: RingString) -> RingString {
    unsafe { ffi::ring_string_delete(s) }
}

#[inline]
pub fn ring_string_set(s: RingString, val: &[u8]) {
    unsafe { ffi::ring_string_set(s, val.as_ptr() as *const c_char) }
}

/// String variant of [`ring_string_set`].
#[inline]
pub fn ring_string_set_str(s: RingString, val: &str) {
    if let Ok(cstr) = CString::new(val) {
        unsafe { ffi::ring_string_set(s, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_string_set2(s: RingString, val: &[u8]) {
    unsafe { ffi::ring_string_set2(s, val.as_ptr() as *const c_char, val.len() as c_uint) }
}

#[inline]
pub fn ring_string_add(s: RingString, val: &[u8]) {
    unsafe { ffi::ring_string_add(s, val.as_ptr() as *const c_char) }
}

/// String variant of [`ring_string_add`].
#[inline]
pub fn ring_string_add_str(s: RingString, val: &str) {
    if let Ok(cstr) = CString::new(val) {
        unsafe { ffi::ring_string_add(s, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_string_add2(s: RingString, val: &[u8]) {
    unsafe { ffi::ring_string_add2(s, val.as_ptr() as *const c_char, val.len() as c_uint) }
}

#[inline]
pub fn ring_string_setfromint(s: RingString, x: c_int) {
    unsafe { ffi::ring_string_setfromint(s, x) }
}

#[inline]
pub fn ring_string_size(s: RingString) -> c_uint {
    unsafe { ffi::ring_string_size(s) }
}

#[inline]
pub fn ring_string_print(s: RingString) {
    unsafe { ffi::ring_string_print(s) }
}

#[inline]
pub fn ring_string_strdup(s: &[u8]) -> *mut c_char {
    unsafe { ffi::ring_string_strdup(s.as_ptr() as *const c_char) }
}

/// String variant of [`ring_string_strdup`].
#[inline]
pub fn ring_string_strdup_str(s: &str) -> *mut c_char {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_string_strdup(cstr.as_ptr()) }
    } else {
        std::ptr::null_mut()
    }
}
