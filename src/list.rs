use libc::{c_char, c_double, c_int, c_uint, c_void};
use std::ffi::{CStr, CString};

use crate::RingList;
use crate::ffi;

#[inline]
pub fn ring_list_new(size: c_uint) -> RingList {
    unsafe { ffi::ring_list_new(size) }
}

#[inline]
pub fn ring_list_delete(list: RingList) -> RingList {
    unsafe { ffi::ring_list_delete(list) }
}

#[inline]
pub fn ring_list_newlist(list: RingList) -> RingList {
    unsafe { ffi::ring_list_newlist(list) }
}

#[inline]
pub fn ring_list_getsize(list: RingList) -> c_uint {
    unsafe { ffi::ring_list_getsize(list) }
}

#[inline]
pub fn ring_list_gettype(list: RingList, index: c_uint) -> c_uint {
    unsafe { ffi::ring_list_gettype(list, index) }
}

#[inline]
pub fn ring_list_addint(list: RingList, n: c_int) {
    unsafe { ffi::ring_list_addint(list, n) }
}

#[inline]
pub fn ring_list_adddouble(list: RingList, n: c_double) {
    unsafe { ffi::ring_list_adddouble(list, n) }
}

#[inline]
pub fn ring_list_addstring(list: RingList, s: &[u8]) {
    unsafe { ffi::ring_list_addstring(list, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_list_addstring_str(list: RingList, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_list_addstring(list, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_list_addstring2(list: RingList, s: &[u8]) {
    unsafe { ffi::ring_list_addstring2(list, s.as_ptr() as *const c_char, s.len() as c_uint) }
}

#[inline]
pub fn ring_list_addpointer(list: RingList, ptr: *mut c_void) {
    unsafe { ffi::ring_list_addpointer(list, ptr) }
}

#[inline]
pub fn ring_list_addcpointer(list: RingList, ptr: *mut c_void, ctype: &[u8]) {
    unsafe { ffi::ring_list_addcpointer(list, ptr, ctype.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_list_getint(list: RingList, index: c_uint) -> c_int {
    unsafe { ffi::ring_list_getint(list, index) }
}

#[inline]
pub fn ring_list_getdouble(list: RingList, index: c_uint) -> c_double {
    unsafe { ffi::ring_list_getdouble(list, index) }
}

#[inline]
pub fn ring_list_getstring(list: RingList, index: c_uint) -> *const c_char {
    unsafe { ffi::ring_list_getstring(list, index) }
}

#[inline]
pub fn ring_list_getstring_str(list: RingList, index: c_uint) -> String {
    unsafe {
        let ptr = ffi::ring_list_getstring(list, index);
        if ptr.is_null() {
            String::new()
        } else {
            CStr::from_ptr(ptr).to_str().unwrap_or("").to_owned()
        }
    }
}

#[inline]
pub fn ring_list_getstringsize(list: RingList, index: c_uint) -> c_uint {
    unsafe { ffi::ring_list_getstringsize(list, index) }
}

#[inline]
pub fn ring_list_getpointer(list: RingList, index: c_uint) -> *mut c_void {
    unsafe { ffi::ring_list_getpointer(list, index) }
}

#[inline]
pub fn ring_list_getlist(list: RingList, index: c_uint) -> RingList {
    unsafe { ffi::ring_list_getlist(list, index) }
}

#[inline]
pub fn ring_list_setint(list: RingList, index: c_uint, n: c_int) {
    unsafe { ffi::ring_list_setint(list, index, n) }
}

#[inline]
pub fn ring_list_setdouble(list: RingList, index: c_uint, n: c_double) {
    unsafe { ffi::ring_list_setdouble(list, index, n) }
}

#[inline]
pub fn ring_list_setstring(list: RingList, index: c_uint, s: &[u8]) {
    unsafe { ffi::ring_list_setstring(list, index, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_list_setstring_str(list: RingList, index: c_uint, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_list_setstring(list, index, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_list_setstring2(list: RingList, index: c_uint, s: &[u8]) {
    unsafe {
        ffi::ring_list_setstring2(list, index, s.as_ptr() as *const c_char, s.len() as c_uint)
    }
}

#[inline]
pub fn ring_list_setpointer(list: RingList, index: c_uint, ptr: *mut c_void) {
    unsafe { ffi::ring_list_setpointer(list, index, ptr) }
}

#[inline]
pub fn ring_list_insertint(list: RingList, pos: c_uint, n: c_int) {
    unsafe { ffi::ring_list_insertint(list, pos, n) }
}

#[inline]
pub fn ring_list_insertdouble(list: RingList, pos: c_uint, n: c_double) {
    unsafe { ffi::ring_list_insertdouble(list, pos, n) }
}

#[inline]
pub fn ring_list_insertstring(list: RingList, pos: c_uint, s: &[u8]) {
    unsafe { ffi::ring_list_insertstring(list, pos, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_list_insertstring_str(list: RingList, pos: c_uint, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_list_insertstring(list, pos, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_list_insertstring2(list: RingList, pos: c_uint, s: &[u8]) {
    unsafe {
        ffi::ring_list_insertstring2(list, pos, s.as_ptr() as *const c_char, s.len() as c_uint)
    }
}

#[inline]
pub fn ring_list_insertpointer(list: RingList, pos: c_uint, ptr: *mut c_void) {
    unsafe { ffi::ring_list_insertpointer(list, pos, ptr) }
}

#[inline]
pub fn ring_list_insertlist(list: RingList, pos: c_uint) -> RingList {
    unsafe { ffi::ring_list_insertlist(list, pos) }
}

#[inline]
pub fn ring_list_deleteitem(list: RingList, index: c_uint) {
    unsafe { ffi::ring_list_deleteitem(list, index) }
}

#[inline]
pub fn ring_list_deleteallitems(list: RingList) {
    unsafe { ffi::ring_list_deleteallitems(list) }
}

#[inline]
pub fn ring_list_isnumber(list: RingList, index: c_uint) -> bool {
    unsafe { ffi::ring_list_isnumber(list, index) != 0 }
}

#[inline]
pub fn ring_list_isstring(list: RingList, index: c_uint) -> bool {
    unsafe { ffi::ring_list_isstring(list, index) != 0 }
}

#[inline]
pub fn ring_list_islist(list: RingList, index: c_uint) -> bool {
    unsafe { ffi::ring_list_islist(list, index) != 0 }
}

#[inline]
pub fn ring_list_ispointer(list: RingList, index: c_uint) -> bool {
    unsafe { ffi::ring_list_ispointer(list, index) != 0 }
}

#[inline]
pub fn ring_list_findstring(list: RingList, s: &[u8], column: c_uint) -> c_uint {
    unsafe { ffi::ring_list_findstring(list, s.as_ptr() as *const c_char, column) }
}

#[inline]
pub fn ring_list_findstring_str(list: RingList, s: &str, column: c_uint) -> c_uint {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_list_findstring(list, cstr.as_ptr(), column) }
    } else {
        0
    }
}

#[inline]
pub fn ring_list_finddouble(list: RingList, n: c_double, column: c_uint) -> c_uint {
    unsafe { ffi::ring_list_finddouble(list, n, column) }
}

#[inline]
pub fn ring_list_findpointer(list: RingList, ptr: *mut c_void) -> c_uint {
    unsafe { ffi::ring_list_findpointer(list, ptr) }
}

#[inline]
pub fn ring_list_copy(new_list: RingList, list: RingList) {
    unsafe { ffi::ring_list_copy(new_list, list) }
}

#[inline]
pub fn ring_list_swap(list: RingList, x: c_uint, y: c_uint) {
    unsafe { ffi::ring_list_swap(list, x, y) }
}

#[inline]
pub fn ring_list_swaptwolists(list1: RingList, list2: RingList) {
    unsafe { ffi::ring_list_swaptwolists(list1, list2) }
}

#[inline]
pub fn ring_list_genarray(list: RingList) {
    unsafe { ffi::ring_list_genarray(list) }
}

#[inline]
pub fn ring_list_deletearray(list: RingList) {
    unsafe { ffi::ring_list_deletearray(list) }
}

#[inline]
pub fn ring_list_genhashtable(list: RingList) {
    unsafe { ffi::ring_list_genhashtable(list) }
}

#[inline]
pub fn ring_list_genhashtable2(list: RingList) {
    unsafe { ffi::ring_list_genhashtable2(list) }
}

#[inline]
pub fn ring_list_print(list: RingList) {
    unsafe { ffi::ring_list_print(list) }
}

#[inline]
pub fn ring_list_print2(list: RingList, decimals: c_uint) {
    unsafe { ffi::ring_list_print2(list, decimals) }
}

#[inline]
pub fn ring_list_printobj(list: RingList, decimals: c_uint) {
    unsafe { ffi::ring_list_printobj(list, decimals) }
}

#[inline]
pub fn ring_list_sortstr(list: RingList, left: c_uint, right: c_uint, column: c_uint, attr: &[u8]) {
    unsafe { ffi::ring_list_sortstr(list, left, right, column, attr.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_list_sortstr_str(
    list: RingList,
    left: c_uint,
    right: c_uint,
    column: c_uint,
    attr: &str,
) {
    if let Ok(cattr) = CString::new(attr) {
        unsafe { ffi::ring_list_sortstr(list, left, right, column, cattr.as_ptr()) }
    }
}

#[inline]
pub fn ring_list_isobject(list: RingList) -> bool {
    unsafe { ffi::ring_list_isobject(list) != 0 }
}

#[inline]
pub fn ring_list_iscpointerlist(list: RingList) -> bool {
    unsafe { ffi::ring_list_iscpointerlist(list) != 0 }
}

#[inline]
pub fn ring_list_cpointercmp(list1: RingList, list2: RingList) -> bool {
    unsafe { ffi::ring_list_cpointercmp(list1, list2) != 0 }
}

#[inline]
pub fn ring_list_addringpointer(list: RingList, ptr: *mut c_void) {
    unsafe { ffi::ring_list_addringpointer(list, ptr) }
}

#[inline]
pub fn ring_list_newitem(list: RingList) {
    unsafe { ffi::ring_list_newitem(list) }
}

#[inline]
pub fn ring_list_insertitem(list: RingList, pos: c_uint) {
    unsafe { ffi::ring_list_insertitem(list, pos) }
}

#[inline]
pub fn ring_list_setfuncpointer(list: RingList, index: c_uint, func: extern "C" fn(*mut c_void)) {
    unsafe { ffi::ring_list_setfuncpointer(list, index, func) }
}

#[inline]
pub fn ring_list_addfuncpointer(list: RingList, func: extern "C" fn(*mut c_void)) {
    unsafe { ffi::ring_list_addfuncpointer(list, func) }
}

#[inline]
pub fn ring_list_isfuncpointer(list: RingList, index: c_uint) -> bool {
    unsafe { ffi::ring_list_isfuncpointer(list, index) != 0 }
}

#[inline]
pub fn ring_list_insertfuncpointer(list: RingList, pos: c_uint, func: extern "C" fn(*mut c_void)) {
    unsafe { ffi::ring_list_insertfuncpointer(list, pos, func) }
}

#[inline]
pub fn ring_list_getitem(list: RingList, index: c_uint) -> *mut ffi::Item {
    unsafe { ffi::ring_list_getitem(list, index) }
}

#[inline]
pub fn ring_list_setlist(list: RingList, index: c_uint) {
    unsafe { ffi::ring_list_setlist(list, index) }
}
