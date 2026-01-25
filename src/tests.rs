//! Unit tests for ring-lang-rs bindings

use std::mem::size_of;

use crate::ffi::{ByteCode, CFunction, FuncCall, Item, List, String as RingString, VM};

#[test]
fn test_struct_sizes() {
    assert_eq!(size_of::<VM>(), 217344, "VM struct size mismatch");
    assert_eq!(size_of::<List>(), 80, "List struct size mismatch");
    assert_eq!(size_of::<Item>(), 24, "Item struct size mismatch");
    assert_eq!(size_of::<RingString>(), 48, "String struct size mismatch");
    assert_eq!(size_of::<FuncCall>(), 104, "FuncCall struct size mismatch");
    assert_eq!(size_of::<ByteCode>(), 24, "ByteCode struct size mismatch");
    assert_eq!(size_of::<CFunction>(), 24, "CFunction struct size mismatch");
}

/// Verify stack size constant matches Ring's definition
#[test]
fn test_stack_size_constant() {
    assert_eq!(crate::ffi::RING_VM_STACK_SIZE, 1004);
}

/// Verify custom mutex count constant
#[test]
fn test_custommutex_count() {
    assert_eq!(crate::ffi::RING_VM_CUSTOMMUTEX_COUNT, 5);
}

/// Verify boolean constants
#[test]
fn test_boolean_constants() {
    assert_eq!(crate::ffi::RING_FALSE, 0);
    assert_eq!(crate::ffi::RING_TRUE, 1);
}

/// Verify API error message constants are defined
#[test]
fn test_api_error_constants() {
    // These should be valid null-terminated byte strings
    assert!(!crate::RING_API_BADPARATYPE.is_empty());
    assert!(!crate::RING_API_BADPARACOUNT.is_empty());
    assert!(crate::RING_API_BADPARATYPE.ends_with(&[0]));
    assert!(crate::RING_API_BADPARACOUNT.ends_with(&[0]));
}

/// Verify pointer type constants
#[test]
fn test_pointer_type_constants() {
    assert_eq!(crate::RING_CPOINTER_STATUS, 3);
    assert_eq!(crate::RING_CPOINTERSTATUS_NOTASSIGNED, 2);
}

/// Verify output constants
#[test]
fn test_output_constants() {
    // These are flags used by the Ring VM
    assert_eq!(crate::RING_OUTPUT_RETLISTBYREF, 1);
    assert_eq!(crate::RING_OUTPUT_RETNEWREF, 2);
}
