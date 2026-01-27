use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let use_static = env::var("CARGO_FEATURE_STATIC").is_ok();
    let no_link = env::var("CARGO_FEATURE_NO_LINK").is_ok();
    let is_android = target_os == "android";
    let is_wasm = target_arch == "wasm32";

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RING");
    println!("cargo:rerun-if-env-changed=ring");

    if use_static || is_android || is_wasm {
        compile_ring_from_source(&target_os, is_wasm);
        return;
    }

    if no_link {
        return;
    }

    link_ring_dynamically(&target_os);
}

fn get_ring_home() -> PathBuf {
    env::var("RING")
        .or_else(|_| env::var("ring"))
        .map(PathBuf::from)
        .expect("RING environment variable must be set to Ring installation directory")
}

fn compile_ring_from_source(target_os: &str, is_wasm: bool) {
    let ring_home = get_ring_home();
    let src_dir = ring_home.join("language/src");
    let include_dir = ring_home.join("language/include");

    if !src_dir.exists() {
        panic!(
            "Ring source directory not found: {}\nSet RING env var to Ring installation path",
            src_dir.display()
        );
    }

    let is_android = target_os == "android";

    // Files to exclude from compilation
    let excluded_files: Vec<&str> = if is_android {
        vec!["ring.c", "ringw.c", "dll_e.c"]
    } else if is_wasm {
        vec!["ring.c", "ringw.c", "dll_e.c", "os_e.c", "file_e.c"]
    } else {
        vec!["ring.c", "ringw.c"]
    };

    let sources: Vec<PathBuf> = std::fs::read_dir(&src_dir)
        .expect("Failed to read Ring source directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |ext| ext == "c")
                && p.file_name().map_or(false, |name| {
                    let name_str = name.to_string_lossy();
                    !excluded_files.contains(&name_str.as_ref())
                })
        })
        .collect();

    let mut build = cc::Build::new();
    build
        .include(&include_dir)
        .files(&sources)
        .warnings(false)
        .pic(true); // Use cc's smart PIC handling

    match target_os {
        "android" => {
            build.define("RING_NODLL", "1");
        }
        "windows" => {
            // Handle static CRT when requested
            let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
            if target_features.split(',').any(|f| f == "crt-static") {
                build.static_crt(true);
            }
        }
        _ => {
            if is_wasm {
                let wasi_sysroot = env::var("WASI_SYSROOT")
                    .unwrap_or_else(|_| "/usr/include/wasm32-wasi".to_string());
                build.include(&wasi_sysroot);
                build.define("RING_NODLL", "1");
                build.define("__wasi__", "1");
                build.define("_WASI_EMULATED_SIGNAL", "1");
                build.define("_WASI_EMULATED_MMAN", "1");
                build.define("_WASI_EMULATED_PROCESS_CLOCKS", "1");
            }
        }
    }

    build.compile("ring");

    // Link system libraries (not needed for Windows/WASM)
    if target_os != "windows" && !is_wasm {
        println!("cargo:rustc-link-lib=m");
        if target_os != "android" {
            println!("cargo:rustc-link-lib=dl");
        }
    }

    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    for entry in std::fs::read_dir(&include_dir).expect("Failed to read include dir") {
        if let Ok(entry) = entry {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
}

fn link_ring_dynamically(target_os: &str) {
    if let Some(ring_home) = env::var("RING").ok().or_else(|| env::var("ring").ok()) {
        let lib_path = PathBuf::from(&ring_home).join("lib");
        println!("cargo:rustc-link-search=native={}", lib_path.display());

        // Use runtime target_os, not compile-time cfg!
        if target_os == "macos" {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
        }
    }

    println!("cargo:rustc-link-lib=dylib=ring");
}
