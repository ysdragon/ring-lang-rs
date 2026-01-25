#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(ambiguous_glob_reexports)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_c_str_literals)]

pub mod api;
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

use std::ffi::c_void;

pub type RingState = *mut c_void;
pub type RingVM = *mut ffi::VM;
pub type RingList = *mut ffi::List;
pub type RingFunc = extern "C" fn(*mut c_void);

pub const RING_CPOINTER_STATUS: libc::c_uint = 3;
pub const RING_CPOINTERSTATUS_NOTASSIGNED: libc::c_int = 2;

pub const RING_OUTPUT_RETLIST: libc::c_int = 0;
pub const RING_OUTPUT_RETLISTBYREF: libc::c_int = 1;
pub const RING_OUTPUT_RETNEWREF: libc::c_int = 2;

pub const RING_VARVALUE_INT: libc::c_int = 1;
pub const RING_VARVALUE_FLOAT: libc::c_int = 2;

pub const RING_VAR_NAME: libc::c_uint = 1;
pub const RING_VAR_TYPE: libc::c_uint = 2;
pub const RING_VAR_VALUE: libc::c_uint = 3;
pub const RING_VAR_PVALUETYPE: libc::c_uint = 4;
pub const RING_VAR_PRIVATEFLAG: libc::c_uint = 5;

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
