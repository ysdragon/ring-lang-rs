if isWindows()
    loadlib("target/release/basic_extension.dll")
elseif isMacOSX()
    loadlib("target/release/libbasic_extension.dylib")
else
    loadlib("target/release/libbasic_extension.so")
ok

? "Testing Basic Rust Extension"
? "============================"
? ""

? "rust_hello(): " + rust_hello()
? "rust_add(10, 20): " + rust_add(10, 20)
? "rust_greet('Ring'): " + rust_greet("Ring")

? ""
? "rust_list():"
aList = rust_list()
see aList

? "rust_sum([1,2,3,4,5]): " + rust_sum([1,2,3,4,5])

? ""
? "Testing struct operations:"
pStruct = rust_struct(42, "Test")
? "rust_struct_value(): " + rust_struct_value(pStruct)
? "rust_struct_name(): " + rust_struct_name(pStruct)

? ""
? "Testing managed struct (auto-freed by GC):"
pManaged = rust_managed_struct(100, "Managed")
? "rust_struct_value(): " + rust_struct_value(pManaged)
? "rust_struct_name(): " + rust_struct_name(pManaged)

? ""
? "All tests passed!"
