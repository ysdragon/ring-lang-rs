use crate::RingState;
use crate::ffi;
use std::sync::Mutex;

type ExtensionInitFn = extern "C" fn(RingState);

static EXTENSION_INITS: Mutex<Vec<ExtensionInitFn>> = Mutex::new(Vec::new());

pub fn ring_register_extension(f: ExtensionInitFn) {
    if let Ok(mut inits) = EXTENSION_INITS.lock() {
        inits.push(f);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ring_vm_extension(state: RingState) {
    unsafe {
        #[cfg(feature = "ring-list")]
        ffi::ring_vm_list_loadfunctions(state);

        #[cfg(feature = "ring-math")]
        ffi::ring_vm_math_loadfunctions(state);

        #[cfg(all(feature = "ring-file", not(target_arch = "wasm32")))]
        ffi::ring_vm_file_loadfunctions(state);

        #[cfg(all(feature = "ring-os", not(target_arch = "wasm32")))]
        ffi::ring_vm_os_loadfunctions(state);

        #[cfg(all(
            feature = "ring-dll",
            not(target_os = "android"),
            not(target_os = "ios"),
            not(target_arch = "wasm32")
        ))]
        ffi::ring_vm_dll_loadfunctions(state);

        #[cfg(feature = "ring-refmeta")]
        ffi::ring_vm_refmeta_loadfunctions(state);

        #[cfg(feature = "ring-info")]
        ffi::ring_vm_info_loadfunctions(state);
    }

    if let Ok(inits) = EXTENSION_INITS.lock() {
        for init in inits.iter() {
            init(state);
        }
    }
}
