use crate::ffi_types::{CString, c_char, c_double, c_int, c_uint, c_void};

use crate::ffi::{
    self, ITEMTYPE_FUNCPOINTER, ITEMTYPE_LIST, ITEMTYPE_NOTHING, ITEMTYPE_NUMBER, ITEMTYPE_POINTER,
    ITEMTYPE_STRING, Item, RingItem,
};

#[inline]
pub fn ring_item_new(item_type: c_uint) -> RingItem {
    unsafe { ffi::ring_item_new(item_type) }
}

#[inline]
pub fn ring_item_new_nothing() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_NOTHING) }
}

#[inline]
pub fn ring_item_new_string() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_STRING) }
}

#[inline]
pub fn ring_item_new_number() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_NUMBER) }
}

#[inline]
pub fn ring_item_new_pointer() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_POINTER) }
}

#[inline]
pub fn ring_item_new_list() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_LIST) }
}

#[inline]
pub fn ring_item_new_funcpointer() -> RingItem {
    unsafe { ffi::ring_item_new(ITEMTYPE_FUNCPOINTER) }
}

#[inline]
pub fn ring_item_delete(item: RingItem) -> RingItem {
    unsafe { ffi::ring_item_delete(item) }
}

#[inline]
pub fn ring_item_settype(item: RingItem, item_type: c_uint) {
    unsafe { ffi::ring_item_settype(item, item_type) }
}

#[inline]
pub fn ring_item_setstring(item: RingItem, s: &[u8]) {
    unsafe { ffi::ring_item_setstring(item, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_item_setstring_str(item: RingItem, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_item_setstring(item, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_item_setstring2(item: RingItem, s: &[u8]) {
    unsafe { ffi::ring_item_setstring2(item, s.as_ptr() as *const c_char, s.len() as c_uint) }
}

#[inline]
pub fn ring_item_setdouble(item: RingItem, x: c_double) {
    unsafe { ffi::ring_item_setdouble(item, x) }
}

#[inline]
pub fn ring_item_setint(item: RingItem, x: c_int) {
    unsafe { ffi::ring_item_setint(item, x) }
}

#[inline]
pub fn ring_item_setpointer(item: RingItem, ptr: *mut c_void) {
    unsafe { ffi::ring_item_setpointer(item, ptr) }
}

#[inline]
pub fn ring_item_getnumber(item: RingItem) -> c_double {
    unsafe { ffi::ring_item_getnumber(item) }
}

#[inline]
pub fn ring_item_print(item: RingItem) {
    unsafe { ffi::ring_item_print(item) }
}

#[inline]
pub fn ring_item_init(item: RingItem) {
    unsafe { ffi::ring_item_init(item) }
}

#[inline]
pub fn ring_item_deletecontent(item: RingItem) {
    unsafe { ffi::ring_item_deletecontent(item) }
}

#[inline]
pub fn ring_itemarray_setint(items: *mut Item, index: c_uint, value: c_int) {
    unsafe { ffi::ring_itemarray_setint(items, index, value) }
}

#[inline]
pub fn ring_itemarray_setdouble(items: *mut Item, index: c_uint, value: c_double) {
    unsafe { ffi::ring_itemarray_setdouble(items, index, value) }
}

#[inline]
pub fn ring_itemarray_setpointer(items: *mut Item, index: c_uint, ptr: *mut c_void) {
    unsafe { ffi::ring_itemarray_setpointer(items, index, ptr) }
}

#[inline]
pub fn ring_itemarray_setstring(items: *mut Item, index: c_uint, s: &[u8]) {
    unsafe { ffi::ring_itemarray_setstring(items, index, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_itemarray_setstring_str(items: *mut Item, index: c_uint, s: &str) {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_itemarray_setstring(items, index, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_itemarray_setstring2(items: *mut Item, index: c_uint, s: &[u8]) {
    unsafe {
        ffi::ring_itemarray_setstring2(items, index, s.as_ptr() as *const c_char, s.len() as c_uint)
    }
}
