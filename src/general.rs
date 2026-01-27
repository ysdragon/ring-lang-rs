use crate::ctypes::{c_char, c_double, c_int, c_uint};
use std::ffi::{CStr, CString};

use crate::ffi;

#[inline]
pub fn ring_general_fexists(filename: &str) -> bool {
    if let Ok(cstr) = CString::new(filename) {
        unsafe { ffi::ring_general_fexists(cstr.as_ptr()) != 0 }
    } else {
        false
    }
}

#[inline]
pub fn ring_general_fexists_bytes(filename: &[u8]) -> bool {
    unsafe { ffi::ring_general_fexists(filename.as_ptr() as *const c_char) != 0 }
}

#[inline]
pub fn ring_general_currentdir() -> Option<String> {
    let mut buf = [0 as c_char; 4096];
    let result = unsafe { ffi::ring_general_currentdir(buf.as_mut_ptr()) };
    if result != 0 {
        unsafe {
            let cstr = CStr::from_ptr(buf.as_ptr());
            cstr.to_str().ok().map(|s| s.to_string())
        }
    } else {
        None
    }
}

#[inline]
pub fn ring_general_exefilename() -> Option<String> {
    let mut buf = [0 as c_char; 4096];
    let result = unsafe { ffi::ring_general_exefilename(buf.as_mut_ptr()) };
    if result != 0 {
        unsafe {
            let cstr = CStr::from_ptr(buf.as_ptr());
            cstr.to_str().ok().map(|s| s.to_string())
        }
    } else {
        None
    }
}

#[inline]
pub fn ring_general_exefolder() -> Option<String> {
    let mut buf = [0 as c_char; 4096];
    unsafe { ffi::ring_general_exefolder(buf.as_mut_ptr()) };
    unsafe {
        let cstr = CStr::from_ptr(buf.as_ptr());
        cstr.to_str().ok().map(|s| s.to_string())
    }
}

#[inline]
pub fn ring_general_chdir(dir: &str) -> bool {
    if let Ok(cstr) = CString::new(dir) {
        unsafe { ffi::ring_general_chdir(cstr.as_ptr()) != 0 }
    } else {
        false
    }
}

#[inline]
pub fn ring_general_chdir_bytes(dir: &[u8]) -> bool {
    unsafe { ffi::ring_general_chdir(dir.as_ptr() as *const c_char) != 0 }
}

#[inline]
pub fn ring_general_lower(s: &mut [u8]) {
    unsafe { ffi::ring_general_lower(s.as_mut_ptr() as *mut c_char) };
}

#[inline]
pub fn ring_general_upper(s: &mut [u8]) {
    unsafe { ffi::ring_general_upper(s.as_mut_ptr() as *mut c_char) };
}

#[inline]
pub fn ring_general_lower2(s: &mut [u8]) {
    unsafe { ffi::ring_general_lower2(s.as_mut_ptr() as *mut c_char, s.len() as c_uint) };
}

#[inline]
pub fn ring_general_upper2(s: &mut [u8]) {
    unsafe { ffi::ring_general_upper2(s.as_mut_ptr() as *mut c_char, s.len() as c_uint) };
}

#[inline]
pub fn ring_general_numtostring(num: c_double, decimals: c_int) -> Option<String> {
    let mut buf = [0 as c_char; 64];
    unsafe {
        ffi::ring_general_numtostring(num, buf.as_mut_ptr(), decimals);
        let cstr = CStr::from_ptr(buf.as_ptr());
        cstr.to_str().ok().map(|s| s.to_string())
    }
}

#[inline]
pub fn ring_general_find(haystack: &mut [u8], needle: &mut [u8]) -> Option<usize> {
    let result = unsafe {
        ffi::ring_general_find(
            haystack.as_mut_ptr() as *mut c_char,
            needle.as_mut_ptr() as *mut c_char,
        )
    };
    if result.is_null() {
        None
    } else {
        Some(result as usize - haystack.as_ptr() as usize)
    }
}

#[inline]
pub fn ring_general_find2(haystack: &mut [u8], needle: &mut [u8]) -> Option<usize> {
    let result = unsafe {
        ffi::ring_general_find2(
            haystack.as_mut_ptr() as *mut c_char,
            haystack.len() as c_uint,
            needle.as_mut_ptr() as *mut c_char,
            needle.len() as c_uint,
        )
    };
    if result.is_null() {
        None
    } else {
        Some(result as usize - haystack.as_ptr() as usize)
    }
}
