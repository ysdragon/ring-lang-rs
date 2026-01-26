<div align="center">

# ring-lang-codegen

Proc macros to generate Ring extensions using Rust - zero configuration needed.

<a href="https://crates.io/crates/ring-lang-codegen"><img src="https://img.shields.io/crates/v/ring-lang-codegen.svg" alt="Crates.io"></a>
<a href="https://docs.rs/ring-lang-codegen"><img src="https://docs.rs/ring-lang-codegen/badge.svg" alt="Documentation"></a>
<a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>

</div>


## Usage

Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
ring-lang-rs = "0.1"
ring-lang-codegen = "0.1"
```

Then use `ring_extension!`:

```rust
use ring_lang_codegen::ring_extension;
use ring_lang_rs::*;

ring_extension! {
    prefix: "mylib";  // Optional prefix for all functions

    // Standalone functions
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }

    // Structs with auto-generated accessors
    #[derive(Default)]
    pub struct Counter {
        pub value: i64,
        pub name: String,
    }

    // Impl blocks with methods
    impl Counter {
        pub fn new(name: &str, initial: i64) -> Self {
            Counter { value: initial, name: name.to_string() }
        }

        pub fn increment(&mut self) {
            self.value += 1;
        }

        pub fn get_value(&self) -> i64 {
            self.value
        }
    }
}
```

That's it! No `.rf` file needed. Everything is auto-generated including `ring_libinit!`.

## What Gets Generated

| Source | Generated Ring Functions |
|--------|--------------------------|
| `pub fn add(a, b)` | `mylib_add(a, b)` |
| `pub struct Counter` | `mylib_counter_new()`, `mylib_counter_delete(ptr)` |
| `pub value: i64` field | `mylib_counter_get_value(ptr)`, `mylib_counter_set_value(ptr, v)` |
| `impl Counter { pub fn new() }` | Replaces default `_new` with custom constructor |
| `pub fn increment(&mut self)` | `mylib_counter_increment(ptr)` |

## Example: Hash Library

See `examples/hash-demo/` for a complete example wrapping `base64`, `sha2`, `md5` crates:

```rust
ring_extension! {
    prefix: "hash";

    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    #[derive(Default)]
    pub struct Hasher {
        pub algorithm: String,
    }

    impl Hasher {
        pub fn new(algorithm: &str) -> Self {
            Hasher { algorithm: algorithm.to_string() }
        }

        pub fn hash(&self, input: &str) -> String {
            // ...
        }
    }
}
```

Ring usage:
```ring
loadlib("libring_hash.so")

? hash_sha256("hello")  # Standalone function
? hash_base64_encode("hello")

h = hash_hasher_new("sha256")  # Struct with custom constructor
? hash_hasher_hash(h, "hello")
hash_hasher_delete(h)
```

## Examples

| Example | Crate | Description |
|---------|-------|-------------|
| `examples/hash-demo/` | base64, sha2, md5, hex | Hashing and encoding functions |
| `examples/json-demo/` | serde_json | JSON parsing, querying, modification |
| `examples/uuid-demo/` | uuid | UUID v4/v7 generation, parsing, validation |
| `examples/datetime-demo/` | chrono | Date/time operations, parsing, formatting |
| `examples/regex-demo/` | regex | Pattern matching, replacement, extraction |

## Comparison

| Feature | parsec.ring | ring_extension! |
|---------|------------------|--------------|
| Configuration | Manual `.rf` file | None - just annotate code |
| Sync with source | Manual | Automatic |
| `ring_libinit!` | Manual | Auto-generated |
| Type checking | String parsing | Full Rust types |
| IDE support | None | Full autocomplete |
| Build step | Run codegen script | Just `cargo build` |
