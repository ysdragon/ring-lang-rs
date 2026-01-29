use crate::ctypes::{c_char, c_double, c_int, c_uint, c_void};
use std::ffi::{CStr, CString};

use crate::ffi::{self, RingString};
use crate::{
    RING_CPOINTER_STATUS, RING_CPOINTERSTATUS_NOTASSIGNED, RING_OUTPUT_RETLISTBYREF,
    RING_OUTPUT_RETNEWREF, RingFunc, RingList, RingState,
};

#[inline]
pub fn ring_api_paracount(p: *mut c_void) -> c_int {
    unsafe { ffi::ring_vm_api_paracount(p) }
}

#[inline]
pub fn ring_api_isstring(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_isstring(p, n) != 0 }
}

#[inline]
pub fn ring_api_isnumber(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_isnumber(p, n) != 0 }
}

#[inline]
pub fn ring_api_ispointer(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_ispointer(p, n) != 0 }
}

#[inline]
pub fn ring_api_isptr(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_isptr(p, n) != 0 }
}

#[inline]
pub fn ring_api_iscpointer(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_iscpointer(p, n) != 0 }
}

#[inline]
pub fn ring_api_islist(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_islist(p, n) != 0 }
}

#[inline]
pub fn ring_api_islistornull(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_islistornull(p, n) != 0 }
}

#[inline]
pub fn ring_api_isobject(p: *mut c_void, n: c_int) -> bool {
    unsafe { ffi::ring_vm_api_isobject(p, n) != 0 }
}

#[inline]
pub fn ring_api_iscpointerlist(p: *mut c_void, list: RingList) -> bool {
    unsafe { ffi::ring_vm_api_iscpointerlist(p, list) != 0 }
}

#[inline]
pub fn ring_api_getstring(p: *mut c_void, n: c_int) -> *const c_char {
    unsafe { ffi::ring_vm_api_getstring(p, n) }
}

#[inline]
pub fn ring_api_getstring_str(p: *mut c_void, n: c_int) -> &'static str {
    unsafe {
        let ptr = ffi::ring_vm_api_getstring(p, n);
        if ptr.is_null() {
            ""
        } else {
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }
}

#[inline]
pub fn ring_api_getstring_bytes(p: *mut c_void, n: c_int) -> &'static [u8] {
    unsafe {
        let ptr = ffi::ring_vm_api_getstring(p, n);
        let size = ffi::ring_vm_api_getstringsize(p, n) as usize;
        if ptr.is_null() || size == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(ptr as *const u8, size)
        }
    }
}

#[inline]
pub fn ring_api_getstringsize(p: *mut c_void, n: c_int) -> c_uint {
    unsafe { ffi::ring_vm_api_getstringsize(p, n) }
}

#[inline]
pub fn ring_api_getstringraw(p: *mut c_void) -> RingString {
    unsafe { ffi::ring_vm_api_getstringraw(p) }
}

#[inline]
pub fn ring_api_getnumber(p: *mut c_void, n: c_int) -> c_double {
    unsafe { ffi::ring_vm_api_getnumber(p, n) }
}

#[inline]
pub fn ring_api_getpointer(p: *mut c_void, n: c_int) -> *mut c_void {
    unsafe { ffi::ring_vm_api_getpointer(p, n) }
}

#[inline]
pub fn ring_api_getpointertype(p: *mut c_void, n: c_int) -> c_int {
    unsafe { ffi::ring_vm_api_getpointertype(p, n) }
}

#[inline]
pub fn ring_api_getcpointer(p: *mut c_void, n: c_int, ctype: &[u8]) -> *mut c_void {
    unsafe { ffi::ring_vm_api_getcpointer(p, n, ctype.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_getcpointer2pointer(p: *mut c_void, n: c_int, ctype: &[u8]) -> *mut c_void {
    unsafe { ffi::ring_vm_api_getcpointer2pointer(p, n, ctype.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_getlist(p: *mut c_void, n: c_int) -> RingList {
    unsafe { ffi::ring_vm_api_getlist(p, n) }
}

#[inline]
pub fn ring_api_getcpointerstatus(p: *mut c_void, n: c_int) -> c_int {
    let list = ring_api_getlist(p, n);
    unsafe { ffi::ring_list_getint(list, RING_CPOINTER_STATUS) }
}

#[inline]
pub fn ring_api_iscpointernotassigned(p: *mut c_void, n: c_int) -> bool {
    ring_api_getcpointerstatus(p, n) == RING_CPOINTERSTATUS_NOTASSIGNED
}

#[inline]
pub fn ring_api_setptr(p: *mut c_void, n: c_int, ptr: *mut c_void, ntype: c_int) {
    unsafe { ffi::ring_vm_api_setptr(p, n, ptr, ntype) }
}

#[inline]
pub fn ring_api_retnumber(p: *mut c_void, n: c_double) {
    unsafe { ffi::ring_vm_api_retnumber(p, n) }
}

#[inline]
pub fn ring_api_retstring(p: *mut c_void, s: &[u8]) {
    unsafe { ffi::ring_vm_api_retstring(p, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_retstring_str(p: *mut c_void, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_vm_api_retstring(p, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_api_retstring2(p: *mut c_void, s: &[u8]) {
    unsafe { ffi::ring_vm_api_retstring2(p, s.as_ptr() as *const c_char, s.len() as c_uint) }
}

#[inline]
pub fn ring_api_retstringsize(p: *mut c_void, size: c_uint) {
    unsafe { ffi::ring_vm_api_retstringsize(p, size) }
}

#[inline]
pub fn ring_api_retcpointer(p: *mut c_void, ptr: *mut c_void, ctype: &[u8]) {
    unsafe { ffi::ring_vm_api_retcpointer(p, ptr, ctype.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_retcpointer2(
    p: *mut c_void,
    ptr: *mut c_void,
    ctype: &[u8],
    free_func: Option<extern "C" fn(*mut c_void, *mut c_void)>,
) {
    unsafe { ffi::ring_vm_api_retcpointer2(p, ptr, ctype.as_ptr() as *const c_char, free_func) }
}

#[inline]
pub fn ring_api_retmanagedcpointer(
    p: *mut c_void,
    ptr: *mut c_void,
    ctype: &[u8],
    free_func: extern "C" fn(*mut c_void, *mut c_void),
) {
    unsafe {
        ffi::ring_vm_api_retcpointer2(p, ptr, ctype.as_ptr() as *const c_char, Some(free_func))
    }
}

#[inline]
pub fn ring_api_retlist(p: *mut c_void, list: RingList) {
    unsafe { ffi::ring_vm_api_retlist(p, list) }
}

#[inline]
pub fn ring_api_retlist2(p: *mut c_void, list: RingList, nref: c_int) {
    unsafe { ffi::ring_vm_api_retlist2(p, list, nref) }
}

#[inline]
pub fn ring_api_retlistbyref(p: *mut c_void, list: RingList) {
    unsafe { ffi::ring_vm_api_retlist2(p, list, RING_OUTPUT_RETLISTBYREF) }
}

#[inline]
pub fn ring_api_retnewref(p: *mut c_void, list: RingList) {
    unsafe { ffi::ring_vm_api_retlist2(p, list, RING_OUTPUT_RETNEWREF) }
}

#[inline]
pub fn ring_api_newlist(p: *mut c_void) -> RingList {
    unsafe { ffi::ring_vm_api_newlist(p) }
}

#[inline]
pub fn ring_api_newlistusingblocks(p: *mut c_void, nsize: c_uint, nsize2: c_uint) -> RingList {
    unsafe { ffi::ring_vm_api_newlistusingblocks(p, nsize, nsize2) }
}

#[inline]
pub fn ring_api_newlistusingblocks1d(p: *mut c_void, nsize: c_uint) -> RingList {
    unsafe { ffi::ring_vm_api_newlistusingblocks(p, nsize, c_uint::MAX) }
}

#[inline]
pub fn ring_api_newlistusingblocks2d(p: *mut c_void, rows: c_uint, cols: c_uint) -> RingList {
    unsafe { ffi::ring_vm_api_newlistusingblocks(p, rows, cols) }
}

#[inline]
pub fn ring_api_setnullpointer(p: *mut c_void, n: c_int) {
    unsafe { ffi::ring_vm_api_setcpointernull(p, n) }
}

#[inline]
pub fn ring_api_varpointer(p: *mut c_void, name: &[u8], ctype: &[u8]) -> *mut c_void {
    unsafe {
        ffi::ring_vm_api_varptr(
            p,
            name.as_ptr() as *const c_char,
            ctype.as_ptr() as *const c_char,
        )
    }
}

#[inline]
pub fn ring_api_varvalue(p: *mut c_void, name: &[u8], ntype: c_int) {
    unsafe { ffi::ring_vm_api_varvalue(p, name.as_ptr() as *const c_char, ntype) }
}

#[inline]
pub fn ring_api_intvalue(p: *mut c_void, name: &[u8]) {
    unsafe { ffi::ring_vm_api_intvalue(p, name.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_floatvalue(p: *mut c_void, name: &[u8]) {
    unsafe { ffi::ring_vm_api_floatvalue(p, name.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_getintpointer(p: *mut c_void, n: c_int) -> *mut c_int {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_varptr(p, name, b"int\0".as_ptr() as *const c_char) as *mut c_int }
}

#[inline]
pub fn ring_api_getfloatpointer(p: *mut c_void, n: c_int) -> *mut f32 {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_varptr(p, name, b"float\0".as_ptr() as *const c_char) as *mut f32 }
}

#[inline]
pub fn ring_api_getdoublepointer(p: *mut c_void, n: c_int) -> *mut f64 {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_varptr(p, name, b"double\0".as_ptr() as *const c_char) as *mut f64 }
}

#[inline]
pub fn ring_api_getcharpointer(p: *mut c_void, n: c_int) -> *mut c_char {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_varptr(p, name, b"char\0".as_ptr() as *const c_char) as *mut c_char }
}

#[inline]
pub fn ring_api_acceptintvalue(p: *mut c_void, n: c_int) {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_intvalue(p, name) }
}

#[inline]
pub fn ring_api_acceptfloatvalue(p: *mut c_void, n: c_int) {
    let name = ring_api_getstring(p, n);
    unsafe { ffi::ring_vm_api_floatvalue(p, name) }
}

#[inline]
pub fn ring_api_ignorecpointertype(p: *mut c_void) {
    unsafe { ffi::ring_vm_api_ignorecpointertypecheck(p) }
}

#[inline]
pub fn ring_api_callerscope(p: *mut c_void) -> RingList {
    unsafe { ffi::ring_vm_api_callerscope(p) }
}

#[inline]
pub fn ring_api_scopescount(p: *mut c_void) -> c_int {
    unsafe { ffi::ring_vm_api_scopescount(p) }
}

#[inline]
pub fn ring_api_cpointercmp(p: *mut c_void, list1: RingList, list2: RingList) -> bool {
    unsafe { ffi::ring_vm_api_cpointercmp(p, list1, list2) != 0 }
}

#[inline]
pub fn ring_api_error(p: *mut c_void, s: &[u8]) {
    unsafe { ffi::ring_vm_error(p, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_api_error_str(p: *mut c_void, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_vm_error(p, cstr.as_ptr()) }
    }
}

pub fn ring_register_function(state: RingState, name: &[u8], func: RingFunc) {
    unsafe {
        ffi::ring_vm_funcregister2(state, name.as_ptr() as *const c_char, func);
    }
}

pub fn ring_register_function_str(state: RingState, name: &str, func: RingFunc) {
    unsafe {
        ffi::ring_vm_funcregister2(state, name.as_ptr() as *const c_char, func);
    }
}
