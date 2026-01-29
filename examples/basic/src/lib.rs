use libc::c_void;
use ring_lang_rs::*;

const MY_POINTER_TYPE: &[u8] = b"MyStruct\0";

struct MyStruct {
    value: i32,
    name: String,
}

ring_func!(ring_hello_world, |p| {
    ring_check_paracount!(p, 0);
    ring_ret_string!(p, "Hello from Rust!");
});

ring_func!(ring_add_numbers, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);

    let a = ring_get_number!(p, 1);
    let b = ring_get_number!(p, 2);

    ring_ret_number!(p, a + b);
});

ring_func!(ring_greet, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let name = ring_get_string!(p, 1);
    let greeting = format!("Hello, {}!", name);

    ring_ret_string!(p, &greeting);
});

ring_func!(ring_create_list, |p| {
    ring_check_paracount!(p, 0);

    let list = ring_new_list!(p);
    ring_list_addint(list, 1);
    ring_list_addint(list, 2);
    ring_list_addint(list, 3);
    ring_list_addstring(list, b"hello\0");
    ring_list_adddouble(list, 3.14);

    ring_ret_list!(p, list);
});

ring_func!(ring_create_struct, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_string!(p, 2);

    let value = ring_get_int!(p, 1);
    let name = ring_get_string!(p, 2).to_string();

    let my_struct = Box::new(MyStruct { value, name });
    let ptr = Box::into_raw(my_struct);

    ring_ret_cpointer!(p, ptr, MY_POINTER_TYPE);
});

extern "C" fn free_my_struct(_state: *mut c_void, ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut MyStruct);
        }
    }
}

ring_func!(ring_create_managed_struct, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_string!(p, 2);

    let value = ring_get_int!(p, 1);
    let name = ring_get_string!(p, 2).to_string();

    let my_struct = Box::new(MyStruct { value, name });
    let ptr = Box::into_raw(my_struct);

    ring_ret_managed_cpointer!(p, ptr, MY_POINTER_TYPE, free_my_struct);
});

ring_func!(ring_get_struct_value, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    if let Some(my_struct) = ring_get_pointer!(p, 1, MyStruct, MY_POINTER_TYPE) {
        ring_ret_number!(p, my_struct.value);
    } else {
        ring_error!(p, "Invalid pointer");
    }
});

ring_func!(ring_get_struct_name, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    if let Some(my_struct) = ring_get_pointer!(p, 1, MyStruct, MY_POINTER_TYPE) {
        ring_ret_string!(p, &my_struct.name);
    } else {
        ring_error!(p, "Invalid pointer");
    }
});

ring_func!(ring_sum_list, |p| {
    ring_check_paracount!(p, 1);
    ring_check_list!(p, 1);

    let list = ring_get_list!(p, 1);
    let size = ring_list_getsize(list);
    let mut sum = 0.0;

    for i in 1..=size {
        if ring_list_isnumber(list, i) {
            sum += ring_list_getdouble(list, i);
        }
    }

    ring_ret_number!(p, sum);
});

ring_libinit! {
    "rust_hello" => ring_hello_world,
    "rust_add" => ring_add_numbers,
    "rust_greet" => ring_greet,
    "rust_list" => ring_create_list,
    "rust_struct" => ring_create_struct,
    "rust_managed_struct" => ring_create_managed_struct,
    "rust_struct_value" => ring_get_struct_value,
    "rust_struct_name" => ring_get_struct_name,
    "rust_sum" => ring_sum_list,
}
