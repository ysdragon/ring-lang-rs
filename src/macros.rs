#[macro_export]
macro_rules! ring_func {
    ($name:ident, $body:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(p: *mut ::std::ffi::c_void) {
            $body(p)
        }
    };
}

#[macro_export]
macro_rules! ring_libinit {
    (@cfg $state:ident [ ] { $($name:literal => $func:ident),* $(,)? }) => {
        $( $crate::ring_register_function_str($state, concat!($name, "\0"), $func); )*
    };
    (@cfg $state:ident [ #[$attr:meta] $($rest:tt)* ] { $($body:tt)* }) => {
        #[$attr]
        { $crate::ring_libinit!(@cfg $state [ $($rest)* ] { $($body)* }); }
    };
    (@munch $state:ident) => {};
    (@munch $state:ident $(#[$attr:meta])+ { $($name:literal => $func:ident),* $(,)? } $(, $($rest:tt)*)?) => {
        $crate::ring_libinit!(@cfg $state [ $(#[$attr])* ] { $($name => $func),* });
        $( $crate::ring_libinit!(@munch $state $($rest)*); )?
    };
    (@munch $state:ident $name:literal => $func:ident $(, $($rest:tt)*)?) => {
        $crate::ring_register_function_str($state, concat!($name, "\0"), $func);
        $( $crate::ring_libinit!(@munch $state $($rest)*); )?
    };
    ($($name:literal => $func:ident),* $(,)?) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn ringlib_init(state: $crate::RingState) {
            $( $crate::ring_register_function_str(state, concat!($name, "\0"), $func); )*
        }
    };
    ($($tt:tt)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn ringlib_init(state: $crate::RingState) {
            $crate::ring_libinit!(@munch state $($tt)*);
        }
    };
}

#[macro_export]
macro_rules! ring_check_paracount {
    ($p:expr, $expected:expr) => {
        if $crate::ring_api_paracount($p) != $expected {
            $crate::ring_api_error($p, $crate::RING_API_BADPARACOUNT);
            return;
        }
    };
    ($p:expr, $expected:expr, $msg:expr) => {
        if $crate::ring_api_paracount($p) != $expected {
            $crate::ring_api_error($p, $msg);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_paracount_range {
    ($p:expr, $min:expr, $max:expr) => {
        let count = $crate::ring_api_paracount($p);
        if count < $min || count > $max {
            $crate::ring_api_error($p, $crate::RING_API_BADPARACOUNT);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_string {
    ($p:expr, $n:expr) => {
        if !$crate::ring_api_isstring($p, $n) {
            $crate::ring_api_error($p, $crate::RING_API_BADPARATYPE);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_number {
    ($p:expr, $n:expr) => {
        if !$crate::ring_api_isnumber($p, $n) {
            $crate::ring_api_error($p, $crate::RING_API_BADPARATYPE);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_pointer {
    ($p:expr, $n:expr) => {
        if !$crate::ring_api_ispointer($p, $n) {
            $crate::ring_api_error($p, $crate::RING_API_BADPARATYPE);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_cpointer {
    ($p:expr, $n:expr) => {
        if !$crate::ring_api_iscpointer($p, $n) {
            $crate::ring_api_error($p, $crate::RING_API_BADPARATYPE);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_check_list {
    ($p:expr, $n:expr) => {
        if !$crate::ring_api_islist($p, $n) {
            $crate::ring_api_error($p, $crate::RING_API_BADPARATYPE);
            return;
        }
    };
}

#[macro_export]
macro_rules! ring_get_string {
    ($p:expr, $n:expr) => {
        $crate::ring_api_getstring_str($p, $n)
    };
}

#[macro_export]
macro_rules! ring_get_number {
    ($p:expr, $n:expr) => {
        $crate::ring_api_getnumber($p, $n)
    };
}

#[macro_export]
macro_rules! ring_get_int {
    ($p:expr, $n:expr) => {
        $crate::ring_api_getnumber($p, $n) as i32
    };
}

#[macro_export]
macro_rules! ring_get_pointer {
    ($p:expr, $n:expr, $type:ty, $ctype:expr) => {{
        let ptr = $crate::ring_api_getcpointer($p, $n, $ctype);
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *(ptr as *mut $type) })
        }
    }};
}

#[macro_export]
macro_rules! ring_get_cpointer {
    ($p:expr, $n:expr, $ctype:expr) => {
        $crate::ring_api_getcpointer($p, $n, $ctype)
    };
}

#[macro_export]
macro_rules! ring_get_list {
    ($p:expr, $n:expr) => {
        $crate::ring_api_getlist($p, $n)
    };
}

#[macro_export]
macro_rules! ring_ret_number {
    ($p:expr, $n:expr) => {
        $crate::ring_api_retnumber($p, $n as f64)
    };
}

#[macro_export]
macro_rules! ring_ret_string {
    ($p:expr, $s:expr) => {
        $crate::ring_api_retstring_str($p, $s)
    };
}

#[macro_export]
macro_rules! ring_ret_cpointer {
    ($p:expr, $ptr:expr, $ctype:expr) => {
        $crate::ring_api_retcpointer($p, $ptr as *mut ::std::ffi::c_void, $ctype)
    };
}

#[macro_export]
macro_rules! ring_ret_managed_cpointer {
    ($p:expr, $ptr:expr, $ctype:expr, $free_func:expr) => {
        $crate::ring_api_retcpointer2(
            $p,
            $ptr as *mut ::std::ffi::c_void,
            $ctype,
            Some($free_func),
        )
    };
}

#[macro_export]
macro_rules! ring_ret_list {
    ($p:expr, $list:expr) => {
        $crate::ring_api_retlist($p, $list)
    };
}

#[macro_export]
macro_rules! ring_new_list {
    ($p:expr) => {
        $crate::ring_api_newlist($p)
    };
}

#[macro_export]
macro_rules! ring_error {
    ($p:expr, $msg:expr) => {
        $crate::ring_api_error_str($p, $msg)
    };
}
