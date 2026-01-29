use crate::ctypes::{c_char, c_int, c_uint, c_void};
use std::ffi::CString;

use crate::ffi::{self, ITEM_NUMBERFLAG_DOUBLE, ITEMTYPE_STRING, RING_FALSE, RingVM};

#[inline]
pub fn ring_vm_callfunction(vm: RingVM, func_name: &[u8]) {
    unsafe { ffi::ring_vm_callfunction(vm, func_name.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_vm_callfunction_str(vm: RingVM, func_name: &str) {
    if let Ok(cstr) = CString::new(func_name) {
        unsafe { ffi::ring_vm_callfunction(vm, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_vm_mutexlock(vm: RingVM) {
    unsafe { ffi::ring_vm_mutexlock(vm) }
}

#[inline]
pub fn ring_vm_mutexunlock(vm: RingVM) {
    unsafe { ffi::ring_vm_mutexunlock(vm) }
}

#[inline]
pub fn ring_vm_mutexdestroy(vm: RingVM) {
    unsafe { ffi::ring_vm_mutexdestroy(vm) }
}

#[inline]
pub fn ring_vm_runcodefromthread(vm: RingVM, code: &[u8]) {
    unsafe { ffi::ring_vm_runcodefromthread(vm, code.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_vm_runcodefromthread_str(vm: RingVM, code: &str) {
    if let Ok(cstr) = CString::new(code) {
        unsafe { ffi::ring_vm_runcodefromthread(vm, cstr.as_ptr()) }
    }
}

pub struct VmMutexGuard {
    vm: RingVM,
}

impl VmMutexGuard {
    pub fn new(vm: RingVM) -> Self {
        unsafe { ffi::ring_vm_mutexlock(vm) };
        Self { vm }
    }
}

impl Drop for VmMutexGuard {
    fn drop(&mut self) {
        unsafe { ffi::ring_vm_mutexunlock(self.vm) };
    }
}

#[inline]
pub fn ring_vm_lock(vm: RingVM) -> VmMutexGuard {
    VmMutexGuard::new(vm)
}

#[inline]
pub fn ring_vm_loadfunc2(vm: RingVM, func_name: &[u8], performance: c_int) -> bool {
    unsafe { ffi::ring_vm_loadfunc2(vm, func_name.as_ptr() as *const c_char, performance) != 0 }
}

#[inline]
pub fn ring_vm_loadfunc2_str(vm: RingVM, func_name: &str) -> bool {
    if let Ok(cstr) = CString::new(func_name) {
        unsafe { ffi::ring_vm_loadfunc2(vm, cstr.as_ptr(), RING_FALSE) != 0 }
    } else {
        false
    }
}

#[inline]
pub fn ring_vm_call2(vm: RingVM) {
    unsafe { ffi::ring_vm_call2(vm) }
}

#[inline]
pub fn ring_vm_fetch(vm: RingVM) {
    unsafe { ffi::ring_vm_fetch(vm) }
}

#[inline]
pub fn ring_vm_fetch2(vm: RingVM) {
    unsafe { ffi::ring_vm_fetch2(vm) }
}

#[inline]
pub fn ring_vm_funccallscount(vm: RingVM) -> c_uint {
    unsafe { (*vm).nCurrentFuncCall }
}

#[inline]
pub fn ring_vm_get_sp(vm: RingVM) -> c_uint {
    unsafe { (*vm).nSP }
}

#[inline]
pub fn ring_vm_set_sp(vm: RingVM, sp: c_uint) {
    unsafe { (*vm).nSP = sp }
}

#[inline]
pub fn ring_vm_get_funcsp(vm: RingVM) -> c_uint {
    unsafe { (*vm).nFuncSP }
}

#[inline]
pub fn ring_vm_set_funcsp(vm: RingVM, funcsp: c_uint) {
    unsafe { (*vm).nFuncSP = funcsp }
}

#[inline]
pub unsafe fn ring_vm_stack_push_cvalue(vm: RingVM, s: &[u8]) {
    unsafe {
        let sp = (*vm).nSP as usize;
        (*vm).nSP += 1;
        let item = &mut (*vm).aStack[sp];
        item.flags = ITEMTYPE_STRING;
        ffi::ring_item_setstring2(item, s.as_ptr() as *const c_char, s.len() as c_uint);
    }
}

#[inline]
pub unsafe fn ring_vm_stack_push_number(vm: RingVM, num: f64) {
    unsafe {
        let sp = (*vm).nSP as usize;
        (*vm).nSP += 1;
        let item = &mut (*vm).aStack[sp];
        item.flags = (ITEM_NUMBERFLAG_DOUBLE << 3) | ffi::ITEMTYPE_NUMBER;
        item.data.dNumber = num;
    }
}

#[inline]
pub unsafe fn ring_vm_stack_push_int(vm: RingVM, num: c_int) {
    unsafe {
        let sp = (*vm).nSP as usize;
        (*vm).nSP += 1;
        let item = &mut (*vm).aStack[sp];
        item.flags = (ffi::ITEM_NUMBERFLAG_INT << 3) | ffi::ITEMTYPE_NUMBER;
        item.data.iNumber = num;
    }
}

// VM execution and utilities

#[inline]
pub fn ring_vm_runcode(vm: RingVM, code: &[u8]) {
    unsafe { ffi::ring_vm_runcode(vm, code.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_vm_runcode_str(vm: RingVM, code: &str) {
    if let Ok(cstr) = CString::new(code) {
        unsafe { ffi::ring_vm_runcode(vm, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_vm_numtostring(vm: RingVM, num: f64, buf: &mut [u8]) -> *mut c_char {
    unsafe { ffi::ring_vm_numtostring(vm, num, buf.as_mut_ptr() as *mut c_char) }
}

#[inline]
pub fn ring_vm_stringtonum(vm: RingVM, s: &[u8]) -> f64 {
    unsafe { ffi::ring_vm_stringtonum(vm, s.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_vm_stringtonum_str(vm: RingVM, s: &str) -> f64 {
    if let Ok(cstr) = CString::new(s) {
        unsafe { ffi::ring_vm_stringtonum(vm, cstr.as_ptr()) }
    } else {
        0.0
    }
}

#[inline]
pub fn ring_vm_aftercfunction(vm: RingVM, func_call: *mut c_void) {
    unsafe { ffi::ring_vm_aftercfunction(vm, func_call) }
}

#[inline]
pub fn ring_vm_callfuncwithouteval(vm: RingVM, func_name: &[u8], is_method: bool) {
    unsafe {
        ffi::ring_vm_callfuncwithouteval(
            vm,
            func_name.as_ptr() as *const c_char,
            is_method as c_uint,
        )
    }
}

#[inline]
pub fn ring_vm_callfuncwithouteval_str(vm: RingVM, func_name: &str, is_method: bool) {
    if let Ok(cstr) = CString::new(func_name) {
        unsafe { ffi::ring_vm_callfuncwithouteval(vm, cstr.as_ptr(), is_method as c_uint) }
    }
}

#[inline]
pub fn ring_vm_showbytecode(vm: RingVM) {
    unsafe { ffi::ring_vm_showbytecode(vm) }
}

#[inline]
pub fn ring_vm_showerrormessage(vm: RingVM, msg: &[u8]) {
    unsafe { ffi::ring_vm_showerrormessage(vm, msg.as_ptr() as *const c_char) }
}

#[inline]
pub fn ring_vm_showerrormessage_str(vm: RingVM, msg: &str) {
    if let Ok(cstr) = CString::new(msg) {
        unsafe { ffi::ring_vm_showerrormessage(vm, cstr.as_ptr()) }
    }
}

#[inline]
pub fn ring_vm_shutdown(vm: RingVM, exit_code: c_int) {
    unsafe { ffi::ring_vm_shutdown(vm, exit_code) }
}

// VM threading

#[inline]
pub fn ring_vm_createthreadstate(vm: RingVM) -> crate::RingState {
    unsafe { ffi::ring_vm_createthreadstate(vm) }
}

#[inline]
pub fn ring_vm_deletethreadstate(vm: RingVM, state: crate::RingState) {
    unsafe { ffi::ring_vm_deletethreadstate(vm, state) }
}

#[inline]
pub fn ring_vm_bytecodefornewthread(vm: RingVM, old_vm: RingVM) {
    unsafe { ffi::ring_vm_bytecodefornewthread(vm, old_vm) }
}

// Custom mutex functions

#[inline]
pub fn ring_vm_custmutexcreate(vm: RingVM) -> *mut c_void {
    unsafe { ffi::ring_vm_custmutexcreate(vm) }
}

#[inline]
pub fn ring_vm_custmutexdestroy(vm: RingVM, mutex: *mut c_void) {
    unsafe { ffi::ring_vm_custmutexdestroy(vm, mutex) }
}

#[inline]
pub fn ring_vm_custmutexlock(vm: RingVM, mutex: *mut c_void) {
    unsafe { ffi::ring_vm_custmutexlock(vm, mutex) }
}

#[inline]
pub fn ring_vm_custmutexunlock(vm: RingVM, mutex: *mut c_void) {
    unsafe { ffi::ring_vm_custmutexunlock(vm, mutex) }
}

#[inline]
pub fn ring_vm_statecustmutexlock(state: *mut c_void, mutex_index: c_uint) {
    unsafe { ffi::ring_vm_statecustmutexlock(state, mutex_index) }
}

#[inline]
pub fn ring_vm_statecustmutexunlock(state: *mut c_void, mutex_index: c_uint) {
    unsafe { ffi::ring_vm_statecustmutexunlock(state, mutex_index) }
}

#[inline]
pub fn ring_vm_mutexfunctions(
    vm: RingVM,
    create: Option<extern "C" fn() -> *mut c_void>,
    destroy: Option<extern "C" fn(*mut c_void)>,
    lock: Option<extern "C" fn(*mut c_void)>,
    unlock: Option<extern "C" fn(*mut c_void)>,
) {
    unsafe { ffi::ring_vm_mutexfunctions(vm, create, destroy, lock, unlock) }
}

/// RAII guard for custom mutex
pub struct CustMutexGuard {
    vm: RingVM,
    mutex: *mut c_void,
}

impl CustMutexGuard {
    pub fn new(vm: RingVM, mutex: *mut c_void) -> Self {
        unsafe { ffi::ring_vm_custmutexlock(vm, mutex) };
        Self { vm, mutex }
    }
}

impl Drop for CustMutexGuard {
    fn drop(&mut self) {
        unsafe { ffi::ring_vm_custmutexunlock(self.vm, self.mutex) };
    }
}

#[inline]
pub fn ring_vm_custmutex_lock(vm: RingVM, mutex: *mut c_void) -> CustMutexGuard {
    CustMutexGuard::new(vm, mutex)
}

// VM loading

#[inline]
pub fn ring_vm_loadcode(vm: RingVM) {
    unsafe { ffi::ring_vm_loadcode(vm) }
}
