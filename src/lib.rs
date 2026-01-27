//! # ring-lang-rs
//!
//! Rust bindings for the Ring programming language.
//!
//! ## Thread Safety
//!
//! Ring has **no GIL** (Global Interpreter Lock). Threading works as follows:
//! - **Separate states are independent**: Multiple `ring_state_new()` calls create isolated VMs
//! - **Shared state across threads**: Use `ring_vm_runcodefromthread()` which creates a per-thread
//!   VM that shares global scope with the main VM. Mutex protection is applied to globals,
//!   function hash table, variable hash table, and memory pool during thread creation/deletion.
//! - **Memory pool**: Thread-safe only after calling `ring_vm_mutexfunctions()` to register
//!   mutex callbacks, and only when using the threading API (`ring_vm_runcodefromthread`)
//!
//! ## Memory Safety
//!
//! Most functions in this crate are thin wrappers around Ring's C API. Callers must ensure:
//! - Pointers (`RingState`, `RingList`, `RingVM`) are valid and non-null
//! - Objects are not used after being deleted
//! - `ring_list_getstring_str()` returns an owned `String` (safe to store)
//! - `ring_api_getstring_str()` returns a reference valid only during the callback

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(ambiguous_glob_reexports)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_c_str_literals)]

pub mod api;
mod ctypes;
pub mod ffi;
pub mod general;
pub mod item;
pub mod list;
pub mod state;
pub mod string;
#[cfg(test)]
mod tests;
pub mod vm;
#[macro_use]
pub mod macros;

pub use api::*;
pub use general::*;
pub use item::*;
pub use list::*;
pub use state::*;
pub use string::*;
pub use vm::*;

use ctypes::c_void;

pub type RingState = *mut c_void;
pub type RingVM = *mut ffi::VM;
pub type RingList = *mut ffi::List;
pub type RingFunc = extern "C" fn(*mut c_void);

pub const RING_CPOINTER_STATUS: ctypes::c_uint = 3;
pub const RING_CPOINTERSTATUS_NOTASSIGNED: ctypes::c_int = 2;

pub const RING_OUTPUT_RETLIST: ctypes::c_int = 0;
pub const RING_OUTPUT_RETLISTBYREF: ctypes::c_int = 1;
pub const RING_OUTPUT_RETNEWREF: ctypes::c_int = 2;

pub const RING_VARVALUE_INT: ctypes::c_int = 1;
pub const RING_VARVALUE_FLOAT: ctypes::c_int = 2;

pub const RING_VAR_NAME: ctypes::c_uint = 1;
pub const RING_VAR_TYPE: ctypes::c_uint = 2;
pub const RING_VAR_VALUE: ctypes::c_uint = 3;
pub const RING_VAR_PVALUETYPE: ctypes::c_uint = 4;
pub const RING_VAR_PRIVATEFLAG: ctypes::c_uint = 5;

pub const RING_API_MISS1PARA: &[u8] = b"Bad parameters count, the function expect one parameter\0";
pub const RING_API_MISS2PARA: &[u8] = b"Bad parameters count, the function expect two parameters\0";
pub const RING_API_MISS3PARA: &[u8] =
    b"Bad parameters count, the function expect three parameters\0";
pub const RING_API_MISS4PARA: &[u8] =
    b"Bad parameters count, the function expect four parameters\0";
pub const RING_API_BADPARATYPE: &[u8] = b"Bad parameter type!\0";
pub const RING_API_BADPARACOUNT: &[u8] = b"Bad parameters count!\0";
pub const RING_API_BADPARARANGE: &[u8] = b"Bad parameters value, error in range!\0";
pub const RING_API_BADPARALENGTH: &[u8] = b"Bad parameters value, error in length!\0";
pub const RING_API_BADPARAVALUE: &[u8] = b"Bad parameter value!\0";
pub const RING_API_NOTPOINTER: &[u8] = b"Error in parameter, not pointer!\0";
pub const RING_API_NULLPOINTER: &[u8] = b"Error in parameter, NULL pointer!\0";
pub const RING_API_EMPTYLIST: &[u8] = b"Bad parameter, empty list!\0";
pub const RING_API_INTERNALFAILURE: &[u8] = b"Internal function call failed!\0";
pub const RING_API_RANGEEXCEEDED: &[u8] = b"Range Exceeded!\0";
