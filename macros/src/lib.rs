//! # ring-lang-codegen
//!
//! Proc macros to generate [Ring](https://ring-lang.github.io/) programming language
//! extensions in Rust with zero configuration.
//!
//! ## Features
//!
//! - **Zero config** - Just use the `ring_extension!` macro, no separate config files
//! - **Auto-generated bindings** - Structs, impl blocks, and functions are automatically wrapped
//! - **Auto ring_libinit!** - Library registration is generated for you
//! - **Full IDE support** - Works with rust-analyzer, autocomplete, and type checking
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! ring-lang-rs = "0.1"
//! ring-lang-codegen = "0.1"
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ring_lang_codegen::ring_extension;
//! use ring_lang_rs::*;
//!
//! ring_extension! {
//!     prefix: "mylib";  // Optional prefix for all functions
//!
//!     // Standalone functions - generates mylib_add(a, b)
//!     pub fn add(a: i32, b: i32) -> i32 {
//!         a + b
//!     }
//!
//!     pub fn greet(name: &str) -> String {
//!         format!("Hello, {}!", name)
//!     }
//!
//!     // Structs with auto-generated accessors
//!     #[derive(Default)]
//!     pub struct Counter {
//!         pub value: i64,
//!         pub name: String,
//!     }
//!
//!     // Impl blocks with methods
//!     impl Counter {
//!         pub fn new(name: &str, initial: i64) -> Self {
//!             Counter { value: initial, name: name.to_string() }
//!         }
//!
//!         pub fn increment(&mut self) {
//!             self.value += 1;
//!         }
//!
//!         pub fn get_value(&self) -> i64 {
//!             self.value
//!         }
//!     }
//! }
//! ```
//!
//! ## What Gets Generated
//!
//! | Source | Generated Ring Functions |
//! |--------|--------------------------|
//! | `pub fn add(a, b)` | `mylib_add(a, b)` |
//! | `pub struct Counter` | `mylib_counter_new()`, `mylib_counter_delete(ptr)` |
//! | `pub value: i64` field | `mylib_counter_get_value(ptr)`, `mylib_counter_set_value(ptr, v)` |
//! | `impl Counter { pub fn new() }` | Replaces default `_new` with custom constructor |
//! | `pub fn increment(&mut self)` | `mylib_counter_increment(ptr)` |
//!
//! ## Ring Usage
//!
//! ```ring
//! loadlib("libmylib.so")  # or .dll / .dylib
//!
//! ? mylib_add(10, 20)        # 30
//! ? mylib_greet("World")     # Hello, World!
//!
//! obj = mylib_counter_new("test", 0)
//! mylib_counter_increment(obj)
//! ? mylib_counter_get_value(obj)  # 1
//! mylib_counter_delete(obj)
//! ```
//!
//! ## Supported Types
//!
//! ### Return Types
//!
//! | Rust Type | Ring Representation |
//! |-----------|---------------------|
//! | `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64` | Number |
//! | `bool` | Number (1 or 0) |
//! | `String`, `&str` | String |
//! | `Vec<T>` | List |
//! | `Vec<Vec<T>>` | Nested list (2D array) |
//! | `Option<T>` | Value or empty string for None |
//! | `Result<T, E>` | Value on Ok, Ring error on Err |
//! | `(A, B)`, `(A, B, C)` | List (tuple as list) |
//! | `Box<T>` | Unwrapped inner value |
//! | `HashMap<K, V>` | List of `[key, value]` pairs |
//! | Custom structs | C pointer |
//!
//! ### Parameter Types
//!
//! | Rust Type | Ring Input |
//! |-----------|------------|
//! | `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64` | Number |
//! | `bool` | Number (non-zero = true) |
//! | `&str`, `String` | String |
//! | `&T` (struct reference) | C pointer |
//! | `&mut T` (mutable struct reference) | C pointer |
//! | `Vec<T>` | List |
//! | `&[T]` (slice) | List |
//! | `Option<T>` | Value or empty string for None |
//! | Custom structs | C pointer |
//!
//! ### Field Types (Getters/Setters)
//!
//! | Rust Type | Get Returns | Set Accepts |
//! |-----------|-------------|-------------|
//! | Primitives | Number | Number |
//! | `String` | String | String |
//! | `Vec<T>` | List | List |
//! | `Option<T>` | Value or empty string | Value or empty string |
//! | Struct | C pointer | C pointer |

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{
    FnArg, Ident, ImplItem, ImplItemFn, Item, ItemFn, ItemImpl, ItemStruct, Pat, ReturnType, Token,
    Type, Visibility, parse_macro_input,
};

struct RingExtension {
    prefix: Option<String>,
    items: Vec<Item>,
}

impl Parse for RingExtension {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut prefix = None;
        let mut items = Vec::new();

        while !input.is_empty() {
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "prefix" {
                    let _: Token![:] = input.parse()?;
                    let lit: syn::LitStr = input.parse()?;
                    let _: Token![;] = input.parse()?;
                    prefix = Some(lit.value());
                    continue;
                } else {
                    return Err(syn::Error::new(ident.span(), "expected 'prefix' or item"));
                }
            }
            items.push(input.parse()?);
        }

        Ok(RingExtension { prefix, items })
    }
}

/// Define a Ring module with auto-generated bindings and ring_libinit!
#[proc_macro]
pub fn ring_extension(input: TokenStream) -> TokenStream {
    let module = parse_macro_input!(input as RingExtension);

    let prefix = module.prefix.unwrap_or_default();
    let prefix_underscore = if prefix.is_empty() {
        String::new()
    } else {
        format!("{}_", prefix)
    };

    let mut structs_with_custom_new: HashSet<String> = HashSet::new();
    let mut impl_methods: HashSet<(String, String)> = HashSet::new();

    for item in &module.items {
        if let Item::Impl(i) = item {
            if let Type::Path(p) = &*i.self_ty {
                let struct_name = p.path.segments.last().unwrap().ident.to_string();
                for impl_item in &i.items {
                    if let ImplItem::Fn(method) = impl_item {
                        let method_name = method.sig.ident.to_string();
                        if method_name == "new" {
                            structs_with_custom_new.insert(struct_name.clone());
                        }
                        impl_methods.insert((struct_name.clone(), method_name));
                    }
                }
            }
        }
    }

    let mut original_items = Vec::new();
    let mut generated_code = Vec::new();
    let mut registrations: Vec<(String, syn::Ident)> = Vec::new();

    for item in module.items {
        match item {
            Item::Struct(s) => {
                let has_custom_new = structs_with_custom_new.contains(&s.ident.to_string());
                let (orig, generated, regs) =
                    process_struct(&s, &prefix_underscore, has_custom_new, &impl_methods);
                original_items.push(orig);
                generated_code.push(generated);
                registrations.extend(regs);
            }
            Item::Impl(i) => {
                let (orig, generated, regs) = process_impl(&i, &prefix_underscore);
                original_items.push(orig);
                generated_code.push(generated);
                registrations.extend(regs);
            }
            Item::Fn(f) => {
                let (orig, generated, regs) = process_function(&f, &prefix_underscore);
                original_items.push(orig);
                generated_code.push(generated);
                registrations.extend(regs);
            }
            other => {
                original_items.push(quote! { #other });
            }
        }
    }

    let libinit_entries: Vec<_> = registrations
        .iter()
        .map(|(name, fn_ident)| {
            quote! { #name => #fn_ident }
        })
        .collect();

    let expanded = quote! {
        #(#original_items)*
        #(#generated_code)*

        ring_libinit! {
            #(#libinit_entries),*
        }
    };

    expanded.into()
}

fn process_struct(
    s: &ItemStruct,
    prefix: &str,
    has_custom_new: bool,
    impl_methods: &HashSet<(String, String)>,
) -> (TokenStream2, TokenStream2, Vec<(String, syn::Ident)>) {
    let struct_name = &s.ident;
    let struct_name_lower = struct_name.to_string().to_lowercase();
    let type_const = format_ident!("{}_TYPE", struct_name.to_string().to_uppercase());
    let type_const_str = format!("{}\0", struct_name);

    let mut regs = Vec::new();

    let delete_fn_name = format_ident!("ring_{}{}_delete", prefix, struct_name_lower);
    let delete_ring_name = format!("{}{}_delete", prefix, struct_name_lower);
    regs.push((delete_ring_name, delete_fn_name.clone()));

    let new_code = if !has_custom_new {
        let new_fn_name = format_ident!("ring_{}{}_new", prefix, struct_name_lower);
        let new_ring_name = format!("{}{}_new", prefix, struct_name_lower);
        regs.push((new_ring_name, new_fn_name.clone()));

        quote! {
            ring_func!(#new_fn_name, |p| {
                ring_check_paracount!(p, 0);
                let obj = Box::new(#struct_name::default());
                ring_ret_cpointer!(p, Box::into_raw(obj), #type_const);
            });
        }
    } else {
        quote! {}
    };

    let mut accessors = Vec::new();
    let struct_name_str = struct_name.to_string();

    if let syn::Fields::Named(fields) = &s.fields {
        for field in &fields.named {
            if !matches!(field.vis, Visibility::Public(_)) {
                continue;
            }

            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let field_type = &field.ty;

            let getter_method = format!("get_{}", field_name_str);
            let setter_method = format!("set_{}", field_name_str);

            if !impl_methods.contains(&(struct_name_str.clone(), getter_method.clone()))
                && !impl_methods.contains(&(struct_name_str.clone(), field_name_str.clone()))
            {
                let getter_fn =
                    format_ident!("ring_{}{}_get_{}", prefix, struct_name_lower, field_name);
                let getter_name = format!("{}{}_get_{}", prefix, struct_name_lower, field_name);
                regs.push((getter_name, getter_fn.clone()));

                let getter_code = generate_field_getter(
                    &getter_fn,
                    struct_name,
                    &type_const,
                    field_name,
                    field_type,
                );
                accessors.push(getter_code);
            }

            if !impl_methods.contains(&(struct_name_str.clone(), setter_method)) {
                let setter_fn =
                    format_ident!("ring_{}{}_set_{}", prefix, struct_name_lower, field_name);
                let setter_name = format!("{}{}_set_{}", prefix, struct_name_lower, field_name);
                regs.push((setter_name, setter_fn.clone()));

                let setter_code = generate_field_setter(
                    &setter_fn,
                    struct_name,
                    &type_const,
                    field_name,
                    field_type,
                );
                accessors.push(setter_code);
            }
        }
    }

    let original = quote! { #s };

    let generated = quote! {
        const #type_const: &[u8] = #type_const_str.as_bytes();

        #new_code

        ring_func!(#delete_fn_name, |p| {
            ring_check_paracount!(p, 1);
            ring_check_cpointer!(p, 1);
            let ptr = ring_get_cpointer!(p, 1, #type_const);
            if !ptr.is_null() {
                unsafe { let _ = Box::from_raw(ptr as *mut #struct_name); }
            }
        });

        #(#accessors)*
    };

    (original, generated, regs)
}

fn process_impl(
    i: &ItemImpl,
    prefix: &str,
) -> (TokenStream2, TokenStream2, Vec<(String, syn::Ident)>) {
    let struct_name = match &*i.self_ty {
        Type::Path(p) => p.path.segments.last().unwrap().ident.clone(),
        _ => return (quote! { #i }, quote! {}, vec![]),
    };

    let struct_name_lower = struct_name.to_string().to_lowercase();
    let type_const = format_ident!("{}_TYPE", struct_name.to_string().to_uppercase());

    let mut regs = Vec::new();
    let mut method_wrappers = Vec::new();

    for item in &i.items {
        if let ImplItem::Fn(method) = item {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }

            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            if method_name_str == "new" {
                let (code, name, fn_ident) = generate_custom_new(
                    &struct_name,
                    &struct_name_lower,
                    &type_const,
                    method,
                    prefix,
                );
                method_wrappers.push(code);
                regs.push((name, fn_ident));
                continue;
            }

            let has_self = method
                .sig
                .inputs
                .iter()
                .any(|arg| matches!(arg, FnArg::Receiver(_)));

            if has_self {
                let (code, name, fn_ident) = generate_method(
                    &struct_name,
                    &struct_name_lower,
                    &type_const,
                    method,
                    prefix,
                );
                method_wrappers.push(code);
                regs.push((name, fn_ident));
            } else {
                let (code, name, fn_ident) = generate_static_method(
                    &struct_name,
                    &struct_name_lower,
                    &type_const,
                    method,
                    prefix,
                );
                method_wrappers.push(code);
                regs.push((name, fn_ident));
            }
        }
    }

    let original = quote! { #i };
    let generated = quote! { #(#method_wrappers)* };

    (original, generated, regs)
}

fn process_function(
    f: &ItemFn,
    prefix: &str,
) -> (TokenStream2, TokenStream2, Vec<(String, syn::Ident)>) {
    let fn_name = &f.sig.ident;
    let ring_fn_name = format_ident!("ring_{}{}", prefix, fn_name);
    let ring_name = format!("{}{}", prefix, fn_name);

    let params: Vec<_> = f
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                let name = if let Pat::Ident(ident) = &*pat.pat {
                    ident.ident.clone()
                } else {
                    return None;
                };
                Some((name, (*pat.ty).clone()))
            } else {
                None
            }
        })
        .collect();

    let param_count = params.len();
    let mut checks = Vec::new();
    let mut gets = Vec::new();
    let mut args = Vec::new();

    for (i, (name, ty)) in params.iter().enumerate() {
        let idx = (i + 1) as i32;
        let binding = generate_param_binding(name, ty, idx);
        checks.push(binding.check);
        gets.push(binding.get);
        args.push(binding.arg);
    }

    let param_count_i32 = param_count as i32;
    let return_code = generate_return_code(&f.sig.output, quote! { #fn_name(#(#args),*) });

    let original = quote! { #f };
    let generated = quote! {
        ring_func!(#ring_fn_name, |p| {
            ring_check_paracount!(p, #param_count_i32);
            #(#checks)*
            #(#gets)*
            #return_code
        });
    };

    (original, generated, vec![(ring_name, ring_fn_name)])
}

fn generate_field_getter(
    fn_name: &syn::Ident,
    struct_name: &syn::Ident,
    type_const: &syn::Ident,
    field_name: &syn::Ident,
    field_type: &Type,
) -> TokenStream2 {
    let type_str = quote!(#field_type).to_string();
    let return_expr = if is_number_type(&type_str) {
        quote! { ring_ret_number!(p, obj.#field_name as f64); }
    } else if is_string_type(&type_str) {
        quote! { ring_ret_string!(p, &obj.#field_name); }
    } else if type_str == "bool" {
        quote! { ring_ret_number!(p, if obj.#field_name { 1.0 } else { 0.0 }); }
    } else if is_vec_type(&type_str) {
        let inner = extract_vec_inner(&type_str).unwrap_or_default();
        if is_number_type(&inner) {
            quote! {
                let __list = ring_new_list!(p);
                for __item in &obj.#field_name {
                    ring_list_adddouble(__list, *__item as f64);
                }
                ring_ret_list!(p, __list);
            }
        } else if is_string_type(&inner) {
            quote! {
                let __list = ring_new_list!(p);
                for __item in &obj.#field_name {
                    ring_list_addstring_str(__list, &__item);
                }
                ring_ret_list!(p, __list);
            }
        } else if is_struct_type(&inner) {
            let inner_type_const = struct_type_const(&extract_struct_name(&inner));
            quote! {
                let __list = ring_new_list!(p);
                for __item in &obj.#field_name {
                    let __ptr = Box::into_raw(Box::new(__item.clone()));
                    ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #inner_type_const);
                }
                ring_ret_list!(p, __list);
            }
        } else {
            quote! {
                let __list = ring_new_list!(p);
                for __item in &obj.#field_name {
                    ring_list_adddouble(__list, *__item as f64);
                }
                ring_ret_list!(p, __list);
            }
        }
    } else if is_option_type(&type_str) {
        let inner = extract_option_inner(&type_str).unwrap_or_default();
        if is_number_type(&inner) {
            quote! {
                match &obj.#field_name {
                    Some(__val) => ring_ret_number!(p, *__val as f64),
                    None => {},
                }
            }
        } else if is_string_type(&inner) {
            quote! {
                match &obj.#field_name {
                    Some(__val) => ring_ret_string!(p, __val),
                    None => {},
                }
            }
        } else if is_struct_type(&inner) {
            let inner_type_const = struct_type_const(&extract_struct_name(&inner));
            quote! {
                match &obj.#field_name {
                    Some(__val) => {
                        ring_ret_cpointer!(p, Box::into_raw(Box::new(__val.clone())), #inner_type_const);
                    }
                    None => {},
                }
            }
        } else {
            quote! {
                match &obj.#field_name {
                    Some(__val) => ring_ret_number!(p, *__val as f64),
                    None => {},
                }
            }
        }
    } else if is_struct_type(&type_str) {
        let field_type_const = struct_type_const(&extract_struct_name(&type_str));
        quote! {
            ring_ret_cpointer!(p, Box::into_raw(Box::new(obj.#field_name.clone())), #field_type_const);
        }
    } else {
        quote! { ring_ret_number!(p, obj.#field_name as f64); }
    };

    quote! {
        ring_func!(#fn_name, |p| {
            ring_check_paracount!(p, 1);
            ring_check_cpointer!(p, 1);
            if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                #return_expr
            } else {
                ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
            }
        });
    }
}

fn generate_field_setter(
    fn_name: &syn::Ident,
    struct_name: &syn::Ident,
    type_const: &syn::Ident,
    field_name: &syn::Ident,
    field_type: &Type,
) -> TokenStream2 {
    let type_str = quote!(#field_type).to_string();

    if is_number_type(&type_str) {
        let cast = get_number_cast(&type_str);
        quote! {
            ring_func!(#fn_name, |p| {
                ring_check_paracount!(p, 2);
                ring_check_cpointer!(p, 1);
                ring_check_number!(p, 2);
                if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                    obj.#field_name = ring_get_number!(p, 2) as #cast;
                } else {
                    ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                }
            });
        }
    } else if is_string_type(&type_str) {
        quote! {
            ring_func!(#fn_name, |p| {
                ring_check_paracount!(p, 2);
                ring_check_cpointer!(p, 1);
                ring_check_string!(p, 2);
                if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                    obj.#field_name = ring_get_string!(p, 2).to_string();
                } else {
                    ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                }
            });
        }
    } else if type_str == "bool" {
        quote! {
            ring_func!(#fn_name, |p| {
                ring_check_paracount!(p, 2);
                ring_check_cpointer!(p, 1);
                ring_check_number!(p, 2);
                if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                    obj.#field_name = ring_get_number!(p, 2) != 0.0;
                } else {
                    ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                }
            });
        }
    } else if is_vec_type(&type_str) {
        let inner = extract_vec_inner(&type_str).unwrap_or_default();
        if is_number_type(&inner) {
            let cast = get_number_cast(&inner);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    ring_check_list!(p, 2);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        let __list = ring_get_list!(p, 2);
                        let __size = ring_list_getsize(__list);
                        let mut __vec = Vec::with_capacity(__size as usize);
                        for __i in 1..=__size {
                            if ring_list_isnumber(__list, __i) {
                                __vec.push(ring_list_getdouble(__list, __i) as #cast);
                            }
                        }
                        obj.#field_name = __vec;
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else if is_string_type(&inner) {
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    ring_check_list!(p, 2);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        let __list = ring_get_list!(p, 2);
                        let __size = ring_list_getsize(__list);
                        let mut __vec = Vec::with_capacity(__size as usize);
                        for __i in 1..=__size {
                            if ring_list_isstring(__list, __i) {
                                let __cstr = ring_list_getstring(__list, __i);
                                let __s = unsafe { std::ffi::CStr::from_ptr(__cstr).to_string_lossy().into_owned() };
                                __vec.push(__s);
                            }
                        }
                        obj.#field_name = __vec;
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else if is_struct_type(&inner) {
            let inner_struct_name = extract_struct_name(&inner);
            let inner_struct_ident = format_ident!("{}", inner_struct_name);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    ring_check_list!(p, 2);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        let __list = ring_get_list!(p, 2);
                        let __size = ring_list_getsize(__list);
                        let mut __vec = Vec::with_capacity(__size as usize);
                        for __i in 1..=__size {
                            let __ptr = if ring_list_ispointer(__list, __i) {
                                ring_list_getpointer(__list, __i)
                            } else if ring_list_islist(__list, __i) {
                                let __inner_list = ring_list_getlist(__list, __i);
                                if ring_list_ispointer(__inner_list, 1) {
                                    ring_list_getpointer(__inner_list, 1)
                                } else {
                                    std::ptr::null_mut()
                                }
                            } else {
                                std::ptr::null_mut()
                            };
                            if !__ptr.is_null() {
                                let __item = unsafe { &*(__ptr as *const #inner_struct_ident) };
                                __vec.push(__item.clone());
                            }
                        }
                        obj.#field_name = __vec;
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else {
            let cast = get_number_cast(&inner);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    ring_check_list!(p, 2);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        let __list = ring_get_list!(p, 2);
                        let __size = ring_list_getsize(__list);
                        let mut __vec = Vec::with_capacity(__size as usize);
                        for __i in 1..=__size {
                            if ring_list_isnumber(__list, __i) {
                                __vec.push(ring_list_getdouble(__list, __i) as #cast);
                            }
                        }
                        obj.#field_name = __vec;
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        }
    } else if is_option_type(&type_str) {
        let inner = extract_option_inner(&type_str).unwrap_or_default();
        if is_number_type(&inner) {
            let cast = get_number_cast(&inner);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        if ring_api_isstring(p, 2) {
                            let __s = ring_get_string!(p, 2);
                            if __s.is_empty() {
                                obj.#field_name = None;
                            } else {
                                ring_error!(p, "Expected number or empty string for Option");
                            }
                        } else if ring_api_isnumber(p, 2) {
                            obj.#field_name = Some(ring_get_number!(p, 2) as #cast);
                        } else {
                            ring_error!(p, "Expected number or empty string for Option");
                        }
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else if is_string_type(&inner) {
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    ring_check_string!(p, 2);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        let __s = ring_get_string!(p, 2);
                        if __s.is_empty() {
                            obj.#field_name = None;
                        } else {
                            obj.#field_name = Some(__s.to_string());
                        }
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else if is_struct_type(&inner) {
            let inner_struct_name = extract_struct_name(&inner);
            let inner_struct_ident = format_ident!("{}", inner_struct_name);
            let inner_type_const = struct_type_const(&inner_struct_name);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        if ring_api_isstring(p, 2) {
                            let __s = ring_get_string!(p, 2);
                            if __s.is_empty() {
                                obj.#field_name = None;
                            } else {
                                ring_error!(p, "Expected cpointer or empty string for Option<Struct>");
                            }
                        } else if ring_api_ispointer(p, 2) {
                            let __ptr = ring_get_cpointer!(p, 2, #inner_type_const);
                            if __ptr.is_null() {
                                obj.#field_name = None;
                            } else {
                                let __val = unsafe { &*(__ptr as *const #inner_struct_ident) };
                                obj.#field_name = Some(__val.clone());
                            }
                        } else {
                            ring_error!(p, "Expected cpointer or empty string for Option<Struct>");
                        }
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        } else {
            let cast = get_number_cast(&inner);
            quote! {
                ring_func!(#fn_name, |p| {
                    ring_check_paracount!(p, 2);
                    ring_check_cpointer!(p, 1);
                    if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                        if ring_api_isstring(p, 2) {
                            let __s = ring_get_string!(p, 2);
                            if __s.is_empty() {
                                obj.#field_name = None;
                            } else {
                                ring_error!(p, "Expected number or empty string for Option");
                            }
                        } else if ring_api_isnumber(p, 2) {
                            obj.#field_name = Some(ring_get_number!(p, 2) as #cast);
                        } else {
                            ring_error!(p, "Expected number or empty string for Option");
                        }
                    } else {
                        ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                    }
                });
            }
        }
    } else if is_struct_type(&type_str) {
        let field_struct_name = extract_struct_name(&type_str);
        let field_struct_ident = format_ident!("{}", field_struct_name);
        let field_type_const = struct_type_const(&field_struct_name);
        quote! {
            ring_func!(#fn_name, |p| {
                ring_check_paracount!(p, 2);
                ring_check_cpointer!(p, 1);
                ring_check_cpointer!(p, 2);
                if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                    if let Some(val) = ring_get_pointer!(p, 2, #field_struct_ident, #field_type_const) {
                        obj.#field_name = val.clone();
                    } else {
                        ring_error!(p, concat!("Invalid ", #field_struct_name, " pointer"));
                    }
                } else {
                    ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                }
            });
        }
    } else {
        quote! {
            ring_func!(#fn_name, |p| {
                ring_check_paracount!(p, 2);
                ring_check_cpointer!(p, 1);
                ring_check_number!(p, 2);
                if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                    obj.#field_name = ring_get_number!(p, 2) as _;
                } else {
                    ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
                }
            });
        }
    }
}

fn generate_custom_new(
    struct_name: &syn::Ident,
    struct_name_lower: &str,
    type_const: &syn::Ident,
    method: &ImplItemFn,
    prefix: &str,
) -> (TokenStream2, String, syn::Ident) {
    let fn_name = format_ident!("ring_{}{}_new", prefix, struct_name_lower);
    let ring_name = format!("{}{}_new", prefix, struct_name_lower);

    let params: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                let name = if let Pat::Ident(ident) = &*pat.pat {
                    ident.ident.clone()
                } else {
                    return None;
                };
                Some((name, (*pat.ty).clone()))
            } else {
                None
            }
        })
        .collect();

    let param_count = params.len() as i32;
    let mut checks = Vec::new();
    let mut gets = Vec::new();
    let mut args = Vec::new();

    for (i, (name, ty)) in params.iter().enumerate() {
        let idx = (i + 1) as i32;
        let binding = generate_param_binding(name, ty, idx);
        checks.push(binding.check);
        gets.push(binding.get);
        args.push(binding.arg);
    }

    let code = quote! {
        ring_func!(#fn_name, |p| {
            ring_check_paracount!(p, #param_count);
            #(#checks)*
            #(#gets)*
            let obj = Box::new(#struct_name::new(#(#args),*));
            ring_ret_cpointer!(p, Box::into_raw(obj), #type_const);
        });
    };

    (code, ring_name, fn_name)
}

fn generate_method(
    struct_name: &syn::Ident,
    struct_name_lower: &str,
    type_const: &syn::Ident,
    method: &ImplItemFn,
    prefix: &str,
) -> (TokenStream2, String, syn::Ident) {
    let method_name = &method.sig.ident;
    let fn_name = format_ident!("ring_{}{}_{}", prefix, struct_name_lower, method_name);
    let ring_name = format!("{}{}_{}", prefix, struct_name_lower, method_name);

    let params: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                let name = if let Pat::Ident(ident) = &*pat.pat {
                    ident.ident.clone()
                } else {
                    return None;
                };
                Some((name, (*pat.ty).clone()))
            } else {
                None
            }
        })
        .collect();

    let param_count = (params.len() + 1) as i32;
    let mut checks = Vec::new();
    let mut gets = Vec::new();
    let mut args = Vec::new();

    for (i, (name, ty)) in params.iter().enumerate() {
        let idx = (i + 2) as i32;
        let binding = generate_param_binding(name, ty, idx);
        checks.push(binding.check);
        gets.push(binding.get);
        args.push(binding.arg);
    }

    let return_code = generate_return_code_with_context(
        &method.sig.output,
        quote! { obj.#method_name(#(#args),*) },
        Some(struct_name),
    );

    let code = quote! {
        ring_func!(#fn_name, |p| {
            ring_check_paracount!(p, #param_count);
            ring_check_cpointer!(p, 1);
            #(#checks)*
            if let Some(obj) = ring_get_pointer!(p, 1, #struct_name, #type_const) {
                #(#gets)*
                #return_code
            } else {
                ring_error!(p, concat!("Invalid ", stringify!(#struct_name), " pointer"));
            }
        });
    };

    (code, ring_name, fn_name)
}

fn generate_static_method(
    struct_name: &syn::Ident,
    struct_name_lower: &str,
    _type_const: &syn::Ident,
    method: &ImplItemFn,
    prefix: &str,
) -> (TokenStream2, String, syn::Ident) {
    let method_name = &method.sig.ident;
    let fn_name = format_ident!("ring_{}{}_{}", prefix, struct_name_lower, method_name);
    let ring_name = format!("{}{}_{}", prefix, struct_name_lower, method_name);

    let params: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                let name = if let Pat::Ident(ident) = &*pat.pat {
                    ident.ident.clone()
                } else {
                    return None;
                };
                Some((name, (*pat.ty).clone()))
            } else {
                None
            }
        })
        .collect();

    let param_count = params.len() as i32;
    let mut checks = Vec::new();
    let mut gets = Vec::new();
    let mut args = Vec::new();

    for (i, (name, ty)) in params.iter().enumerate() {
        let idx = (i + 1) as i32;
        let binding = generate_param_binding(name, ty, idx);
        checks.push(binding.check);
        gets.push(binding.get);
        args.push(binding.arg);
    }

    let return_code = generate_return_code_with_context(
        &method.sig.output,
        quote! { #struct_name::#method_name(#(#args),*) },
        Some(struct_name),
    );

    let code = quote! {
        ring_func!(#fn_name, |p| {
            ring_check_paracount!(p, #param_count);
            #(#checks)*
            #(#gets)*
            #return_code
        });
    };

    (code, ring_name, fn_name)
}

fn generate_return_code(output: &ReturnType, call: TokenStream2) -> TokenStream2 {
    generate_return_code_with_context(output, call, None)
}

fn generate_return_code_with_context(
    output: &ReturnType,
    call: TokenStream2,
    self_struct: Option<&syn::Ident>,
) -> TokenStream2 {
    match output {
        ReturnType::Default => quote! { #call; },
        ReturnType::Type(_, ty) => {
            let type_str = quote!(#ty).to_string();
            generate_return_for_type(&type_str, call, self_struct)
        }
    }
}

fn generate_return_for_type(
    type_str: &str,
    call: TokenStream2,
    self_struct: Option<&syn::Ident>,
) -> TokenStream2 {
    let type_str = type_str.trim();

    if is_number_type(type_str) {
        quote! {
            let __result = #call;
            ring_ret_number!(p, __result as f64);
        }
    } else if is_string_type(type_str) {
        quote! {
            let __result = #call;
            ring_ret_string!(p, &__result);
        }
    } else if type_str == "bool" {
        quote! {
            let __result = #call;
            ring_ret_number!(p, if __result { 1.0 } else { 0.0 });
        }
    } else if is_vec_type(type_str) {
        generate_vec_return(type_str, call)
    } else if is_option_type(type_str) {
        generate_option_return(type_str, call)
    } else if is_result_type(type_str) {
        generate_result_return(type_str, call)
    } else if is_tuple_type(type_str) {
        generate_tuple_return(type_str, call)
    } else if is_box_type(type_str) {
        generate_box_return(type_str, call)
    } else if is_hashmap_type(type_str) {
        generate_hashmap_return(type_str, call)
    } else if type_str == "Self" {
        if let Some(struct_name) = self_struct {
            let type_const = struct_type_const(&struct_name.to_string());
            quote! {
                let __result = #call;
                ring_ret_cpointer!(p, Box::into_raw(Box::new(__result)), #type_const);
            }
        } else {
            quote! {
                let __result = #call;
                ring_ret_number!(p, __result as f64);
            }
        }
    } else if is_struct_type(type_str) {
        let struct_name = extract_struct_name(type_str);
        let type_const = struct_type_const(&struct_name);
        quote! {
            let __result = #call;
            ring_ret_cpointer!(p, Box::into_raw(Box::new(__result)), #type_const);
        }
    } else {
        quote! {
            let __result = #call;
            ring_ret_number!(p, __result as f64);
        }
    }
}

fn generate_vec_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let inner = extract_vec_inner(type_str).unwrap_or_default();

    if is_number_type(&inner) {
        quote! {
            let __result = #call;
            let __list = ring_new_list!(p);
            for __item in __result {
                ring_list_adddouble(__list, __item as f64);
            }
            ring_ret_list!(p, __list);
        }
    } else if is_string_type(&inner) {
        quote! {
            let __result = #call;
            let __list = ring_new_list!(p);
            for __item in __result {
                ring_list_addstring_str(__list, &__item);
            }
            ring_ret_list!(p, __list);
        }
    } else if is_option_type(&inner) {
        let opt_inner = extract_option_inner(&inner).unwrap_or_default();
        if is_number_type(&opt_inner) {
            quote! {
                let __result = #call;
                let __list = ring_new_list!(p);
                for __item in __result {
                    match __item {
                        Some(__val) => ring_list_adddouble(__list, __val as f64),
                        None => ring_list_newitem(__list),
                    }
                }
                ring_ret_list!(p, __list);
            }
        } else if is_string_type(&opt_inner) {
            quote! {
                let __result = #call;
                let __list = ring_new_list!(p);
                for __item in __result {
                    match __item {
                        Some(__val) => ring_list_addstring_str(__list, &__val),
                        None => ring_list_newitem(__list),
                    }
                }
                ring_ret_list!(p, __list);
            }
        } else if is_struct_type(&opt_inner) {
            let type_const = struct_type_const(&extract_struct_name(&opt_inner));
            quote! {
                let __result = #call;
                let __list = ring_new_list!(p);
                for __item in __result {
                    match __item {
                        Some(__val) => {
                            let __ptr = Box::into_raw(Box::new(__val));
                            ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #type_const);
                        }
                        None => ring_list_newitem(__list),
                    }
                }
                ring_ret_list!(p, __list);
            }
        } else {
            quote! {
                let __result = #call;
                let __list = ring_new_list!(p);
                for __item in __result {
                    match __item {
                        Some(__val) => ring_list_adddouble(__list, __val as f64),
                        None => ring_list_newitem(__list),
                    }
                }
                ring_ret_list!(p, __list);
            }
        }
    } else if is_vec_type(&inner) {
        let inner_inner = extract_vec_inner(&inner).unwrap_or_default();
        let add_inner_item = if is_number_type(&inner_inner) {
            quote! { ring_list_adddouble(__inner_list, __inner_item as f64); }
        } else if is_string_type(&inner_inner) {
            quote! { ring_list_addstring_str(__inner_list, &__inner_item); }
        } else if is_struct_type(&inner_inner) {
            let type_const = struct_type_const(&extract_struct_name(&inner_inner));
            quote! {
                let __ptr = Box::into_raw(Box::new(__inner_item));
                ring_list_addcpointer(__inner_list, __ptr as *mut std::ffi::c_void, #type_const);
            }
        } else {
            quote! { ring_list_adddouble(__inner_list, __inner_item as f64); }
        };

        quote! {
            let __result = #call;
            let __list = ring_new_list!(p);
            for __item in __result {
                let __inner_list = ring_list_newlist(__list);
                for __inner_item in __item {
                    #add_inner_item
                }
            }
            ring_ret_list!(p, __list);
        }
    } else if is_struct_type(&inner) {
        let type_const = struct_type_const(&extract_struct_name(&inner));
        quote! {
            let __result = #call;
            let __list = ring_new_list!(p);
            for __item in __result {
                let __ptr = Box::into_raw(Box::new(__item));
                ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #type_const);
            }
            ring_ret_list!(p, __list);
        }
    } else {
        quote! {
            let __result = #call;
            let __list = ring_new_list!(p);
            for __item in __result {
                ring_list_adddouble(__list, __item as f64);
            }
            ring_ret_list!(p, __list);
        }
    }
}

fn generate_option_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let inner = extract_option_inner(type_str).unwrap_or_default();

    if is_number_type(&inner) {
        quote! {
            let __result = #call;
            match __result {
                Some(__val) => ring_ret_number!(p, __val as f64),
                None => {},
            }
        }
    } else if is_string_type(&inner) {
        quote! {
            let __result = #call;
            match __result {
                Some(__val) => ring_ret_string!(p, &__val),
                None => {},
            }
        }
    } else if is_vec_type(&inner) {
        let vec_inner = extract_vec_inner(&inner).unwrap_or_default();
        if is_number_type(&vec_inner) {
            quote! {
                let __result = #call;
                match __result {
                    Some(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_adddouble(__list, __item as f64);
                        }
                        ring_ret_list!(p, __list);
                    }
                    None => {},
                }
            }
        } else if is_string_type(&vec_inner) {
            quote! {
                let __result = #call;
                match __result {
                    Some(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_addstring_str(__list, &__item);
                        }
                        ring_ret_list!(p, __list);
                    }
                    None => {},
                }
            }
        } else if is_struct_type(&vec_inner) {
            let type_const = struct_type_const(&extract_struct_name(&vec_inner));
            quote! {
                let __result = #call;
                match __result {
                    Some(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            let __ptr = Box::into_raw(Box::new(__item));
                            ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #type_const);
                        }
                        ring_ret_list!(p, __list);
                    }
                    None => {},
                }
            }
        } else {
            quote! {
                let __result = #call;
                match __result {
                    Some(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_adddouble(__list, __item as f64);
                        }
                        ring_ret_list!(p, __list);
                    }
                    None => {},
                }
            }
        }
    } else if is_struct_type(&inner) {
        let type_const = struct_type_const(&extract_struct_name(&inner));
        quote! {
            let __result = #call;
            match __result {
                Some(__val) => {
                    ring_ret_cpointer!(p, Box::into_raw(Box::new(__val)), #type_const);
                }
                None => {},
            }
        }
    } else {
        quote! {
            let __result = #call;
            match __result {
                Some(__val) => ring_ret_number!(p, __val as f64),
                None => {},
            }
        }
    }
}

fn generate_result_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let ok_type = extract_result_ok(type_str).unwrap_or_default();

    if ok_type == "()" || ok_type.is_empty() {
        quote! {
            let __result = #call;
            match __result {
                Ok(_) => {}
                Err(__e) => ring_error!(p, &format!("{}", __e)),
            }
        }
    } else if is_number_type(&ok_type) {
        quote! {
            let __result = #call;
            match __result {
                Ok(__val) => ring_ret_number!(p, __val as f64),
                Err(__e) => ring_error!(p, &format!("{}", __e)),
            }
        }
    } else if is_string_type(&ok_type) {
        quote! {
            let __result = #call;
            match __result {
                Ok(__val) => ring_ret_string!(p, &__val),
                Err(__e) => ring_error!(p, &format!("{}", __e)),
            }
        }
    } else if is_vec_type(&ok_type) {
        let vec_inner = extract_vec_inner(&ok_type).unwrap_or_default();
        if is_number_type(&vec_inner) {
            quote! {
                let __result = #call;
                match __result {
                    Ok(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_adddouble(__list, __item as f64);
                        }
                        ring_ret_list!(p, __list);
                    }
                    Err(__e) => ring_error!(p, &format!("{}", __e)),
                }
            }
        } else if is_string_type(&vec_inner) {
            quote! {
                let __result = #call;
                match __result {
                    Ok(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_addstring_str(__list, &__item);
                        }
                        ring_ret_list!(p, __list);
                    }
                    Err(__e) => ring_error!(p, &format!("{}", __e)),
                }
            }
        } else if is_struct_type(&vec_inner) {
            let type_const = struct_type_const(&extract_struct_name(&vec_inner));
            quote! {
                let __result = #call;
                match __result {
                    Ok(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            let __ptr = Box::into_raw(Box::new(__item));
                            ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #type_const);
                        }
                        ring_ret_list!(p, __list);
                    }
                    Err(__e) => ring_error!(p, &format!("{}", __e)),
                }
            }
        } else {
            quote! {
                let __result = #call;
                match __result {
                    Ok(__vec) => {
                        let __list = ring_new_list!(p);
                        for __item in __vec {
                            ring_list_adddouble(__list, __item as f64);
                        }
                        ring_ret_list!(p, __list);
                    }
                    Err(__e) => ring_error!(p, &format!("{}", __e)),
                }
            }
        }
    } else if is_struct_type(&ok_type) {
        let type_const = struct_type_const(&extract_struct_name(&ok_type));
        quote! {
            let __result = #call;
            match __result {
                Ok(__val) => {
                    ring_ret_cpointer!(p, Box::into_raw(Box::new(__val)), #type_const);
                }
                Err(__e) => ring_error!(p, &format!("{}", __e)),
            }
        }
    } else {
        quote! {
            let __result = #call;
            match __result {
                Ok(__val) => ring_ret_number!(p, __val as f64),
                Err(__e) => ring_error!(p, &format!("{}", __e)),
            }
        }
    }
}

fn generate_tuple_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let elements = extract_tuple_elements(type_str);

    if elements.is_empty() {
        return quote! { #call; };
    }

    let mut add_statements = Vec::new();

    for (i, elem_type) in elements.iter().enumerate() {
        let idx = syn::Index::from(i);

        let add_stmt = if is_number_type(elem_type) {
            quote! { ring_list_adddouble(__list, __result.#idx as f64); }
        } else if is_string_type(elem_type) {
            quote! { ring_list_addstring_str(__list, &__result.#idx); }
        } else if elem_type == "bool" {
            quote! { ring_list_adddouble(__list, if __result.#idx { 1.0 } else { 0.0 }); }
        } else if is_struct_type(elem_type) {
            let type_const = struct_type_const(&extract_struct_name(elem_type));
            quote! {
                let __ptr = Box::into_raw(Box::new(__result.#idx));
                ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, #type_const);
            }
        } else {
            quote! { ring_list_adddouble(__list, __result.#idx as f64); }
        };

        add_statements.push(add_stmt);
    }

    quote! {
        let __result = #call;
        let __list = ring_new_list!(p);
        #(#add_statements)*
        ring_ret_list!(p, __list);
    }
}

fn generate_box_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let inner = extract_box_inner(type_str).unwrap_or_default();

    if is_number_type(&inner) {
        quote! {
            let __result = #call;
            ring_ret_number!(p, *__result as f64);
        }
    } else if is_string_type(&inner) {
        quote! {
            let __result = #call;
            ring_ret_string!(p, &*__result);
        }
    } else if inner == "bool" {
        quote! {
            let __result = #call;
            ring_ret_number!(p, if *__result { 1.0 } else { 0.0 });
        }
    } else if is_struct_type(&inner) {
        let type_const = struct_type_const(&extract_struct_name(&inner));
        quote! {
            let __result = #call;
            ring_ret_cpointer!(p, Box::into_raw(__result), #type_const);
        }
    } else {
        quote! {
            let __result = #call;
            ring_ret_number!(p, *__result as f64);
        }
    }
}

fn generate_hashmap_return(type_str: &str, call: TokenStream2) -> TokenStream2 {
    let (key_type, value_type) = extract_hashmap_kv(type_str).unwrap_or_default();

    let add_key = if is_number_type(&key_type) {
        quote! { ring_list_adddouble(__pair, __k as f64); }
    } else if is_string_type(&key_type) {
        quote! { ring_list_addstring_str(__pair, &__k); }
    } else {
        quote! { ring_list_addstring_str(__pair, &format!("{:?}", __k)); }
    };

    let add_value = if is_number_type(&value_type) {
        quote! { ring_list_adddouble(__pair, __v as f64); }
    } else if is_string_type(&value_type) {
        quote! { ring_list_addstring_str(__pair, &__v); }
    } else if value_type == "bool" {
        quote! { ring_list_adddouble(__pair, if __v { 1.0 } else { 0.0 }); }
    } else if is_struct_type(&value_type) {
        let type_const = struct_type_const(&extract_struct_name(&value_type));
        quote! {
            let __ptr = Box::into_raw(Box::new(__v));
            ring_list_addcpointer(__pair, __ptr as *mut std::ffi::c_void, #type_const);
        }
    } else {
        quote! { ring_list_adddouble(__pair, __v as f64); }
    };

    quote! {
        let __result = #call;
        let __list = ring_new_list!(p);
        for (__k, __v) in __result {
            let __pair = ring_list_newlist(__list);
            #add_key
            #add_value
        }
        ring_ret_list!(p, __list);
    }
}

fn is_number_type(ty: &str) -> bool {
    let ty = ty.trim();
    matches!(
        ty,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
    )
}

fn is_string_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty == "String" || ty == "& str" || ty.contains("str")
}

fn get_number_cast(ty: &str) -> TokenStream2 {
    let ty = ty.trim();
    match ty {
        "i8" => quote!(i8),
        "i16" => quote!(i16),
        "i32" => quote!(i32),
        "i64" => quote!(i64),
        "i128" => quote!(i128),
        "isize" => quote!(isize),
        "u8" => quote!(u8),
        "u16" => quote!(u16),
        "u32" => quote!(u32),
        "u64" => quote!(u64),
        "u128" => quote!(u128),
        "usize" => quote!(usize),
        "f32" => quote!(f32),
        "f64" => quote!(f64),
        _ => quote!(f64),
    }
}

fn is_vec_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("Vec <") || ty.starts_with("Vec<")
}

fn is_option_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("Option <") || ty.starts_with("Option<")
}

fn is_result_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("Result <") || ty.starts_with("Result<")
}

fn is_tuple_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with('(') && ty.ends_with(')') && ty.contains(',')
}

fn is_box_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("Box <") || ty.starts_with("Box<")
}

fn is_hashmap_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("HashMap <")
        || ty.starts_with("HashMap<")
        || ty.starts_with("std :: collections :: HashMap")
        || ty.contains("HashMap <")
        || ty.contains("HashMap<")
}

fn is_slice_type(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("& [") || ty.starts_with("&[")
}

fn is_struct_type(ty: &str) -> bool {
    let ty = ty.trim();
    let ty = ty.trim_start_matches('&').trim_start_matches("mut ").trim();

    if is_vec_type(ty)
        || is_option_type(ty)
        || is_result_type(ty)
        || is_tuple_type(ty)
        || is_box_type(ty)
        || is_hashmap_type(ty)
    {
        return false;
    }

    if is_number_type(ty) || is_string_type(ty) || ty == "bool" || ty == "()" {
        return false;
    }

    ty.chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

fn is_struct_ref(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with('&') && is_struct_type(ty)
}

fn extract_vec_inner(ty: &str) -> Option<String> {
    let ty = ty.trim();
    if ty.starts_with("Vec <") {
        Some(ty[5..ty.len() - 1].trim().to_string())
    } else if ty.starts_with("Vec<") {
        Some(ty[4..ty.len() - 1].trim().to_string())
    } else {
        None
    }
}

fn extract_option_inner(ty: &str) -> Option<String> {
    let ty = ty.trim();
    if ty.starts_with("Option <") {
        Some(ty[8..ty.len() - 1].trim().to_string())
    } else if ty.starts_with("Option<") {
        Some(ty[7..ty.len() - 1].trim().to_string())
    } else {
        None
    }
}

fn extract_result_ok(ty: &str) -> Option<String> {
    let ty = ty.trim();
    let inner = if ty.starts_with("Result <") {
        &ty[8..ty.len() - 1]
    } else if ty.starts_with("Result<") {
        &ty[7..ty.len() - 1]
    } else {
        return None;
    };

    let mut depth = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                return Some(inner[..i].trim().to_string());
            }
            _ => {}
        }
    }
    None
}

fn extract_tuple_elements(ty: &str) -> Vec<String> {
    let ty = ty.trim();
    if !ty.starts_with('(') || !ty.ends_with(')') {
        return vec![];
    }

    let inner = &ty[1..ty.len() - 1];
    let mut elements = vec![];
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in inner.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                elements.push(inner[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }

    let last = inner[start..].trim();
    if !last.is_empty() {
        elements.push(last.to_string());
    }

    elements
}

fn extract_box_inner(ty: &str) -> Option<String> {
    let ty = ty.trim();
    if ty.starts_with("Box <") {
        Some(ty[5..ty.len() - 1].trim().to_string())
    } else if ty.starts_with("Box<") {
        Some(ty[4..ty.len() - 1].trim().to_string())
    } else {
        None
    }
}

fn extract_hashmap_kv(ty: &str) -> Option<(String, String)> {
    let ty = ty.trim();

    let hashmap_start = ty.find("HashMap")?;
    let rest = &ty[hashmap_start + 7..];

    let angle_start = rest.find('<')?;
    let angle_end = rest.rfind('>')?;
    let inner = &rest[angle_start + 1..angle_end];

    let mut depth = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let key = inner[..i].trim().to_string();
                let value = inner[i + 1..].trim().to_string();
                return Some((key, value));
            }
            _ => {}
        }
    }
    None
}

fn extract_slice_inner(ty: &str) -> Option<String> {
    let ty = ty.trim();
    if ty.starts_with("& [") {
        Some(ty[3..ty.len() - 1].trim().to_string())
    } else if ty.starts_with("&[") {
        Some(ty[2..ty.len() - 1].trim().to_string())
    } else {
        None
    }
}

fn extract_struct_name(ty: &str) -> String {
    ty.trim()
        .trim_start_matches('&')
        .trim()
        .trim_start_matches("mut ")
        .trim()
        .to_string()
}

fn struct_type_const(struct_name: &str) -> syn::Ident {
    format_ident!("{}_TYPE", struct_name.to_uppercase())
}

struct ParamBinding {
    check: TokenStream2,
    get: TokenStream2,
    arg: TokenStream2,
}

fn is_mut_struct_ref(ty: &str) -> bool {
    let ty = ty.trim();
    ty.starts_with("& mut ") && is_struct_type(&ty[6..])
}

fn generate_param_binding(name: &syn::Ident, ty: &Type, idx: i32) -> ParamBinding {
    let type_str = quote!(#ty).to_string();

    if is_number_type(&type_str) {
        let cast = get_number_cast(&type_str);
        ParamBinding {
            check: quote! { ring_check_number!(p, #idx); },
            get: quote! { let #name = ring_get_number!(p, #idx) as #cast; },
            arg: quote! { #name },
        }
    } else if is_string_type(&type_str) {
        ParamBinding {
            check: quote! { ring_check_string!(p, #idx); },
            get: quote! { let #name = ring_get_string!(p, #idx); },
            arg: quote! { #name },
        }
    } else if type_str == "bool" {
        ParamBinding {
            check: quote! { ring_check_number!(p, #idx); },
            get: quote! { let #name = ring_get_number!(p, #idx) != 0.0; },
            arg: quote! { #name },
        }
    } else if is_mut_struct_ref(&type_str) {
        let struct_name_str = extract_struct_name(&type_str);
        let struct_ident = format_ident!("{}", struct_name_str);
        let type_const = struct_type_const(&struct_name_str);
        let ptr_name = format_ident!("__ptr_{}", name);
        ParamBinding {
            check: quote! { ring_check_cpointer!(p, #idx); },
            get: quote! { let #ptr_name = ring_get_cpointer!(p, #idx, #type_const); },
            arg: quote! { unsafe { &mut *(#ptr_name as *mut #struct_ident) } },
        }
    } else if is_struct_ref(&type_str) {
        let struct_name_str = extract_struct_name(&type_str);
        let struct_ident = format_ident!("{}", struct_name_str);
        let type_const = struct_type_const(&struct_name_str);
        let ptr_name = format_ident!("__ptr_{}", name);
        ParamBinding {
            check: quote! { ring_check_cpointer!(p, #idx); },
            get: quote! { let #ptr_name = ring_get_cpointer!(p, #idx, #type_const); },
            arg: quote! { unsafe { &*(#ptr_name as *const #struct_ident) } },
        }
    } else if is_vec_type(&type_str) {
        generate_vec_param_binding(name, &type_str, idx)
    } else if is_option_type(&type_str) {
        generate_option_param_binding(name, &type_str, idx)
    } else if is_slice_type(&type_str) {
        generate_slice_param_binding(name, &type_str, idx)
    } else if is_struct_type(&type_str) {
        let struct_name_str = extract_struct_name(&type_str);
        let struct_ident = format_ident!("{}", struct_name_str);
        let type_const = struct_type_const(&struct_name_str);
        let ptr_name = format_ident!("__ptr_{}", name);
        ParamBinding {
            check: quote! { ring_check_cpointer!(p, #idx); },
            get: quote! {
                let #ptr_name = ring_get_cpointer!(p, #idx, #type_const);
                let #name = unsafe { (*( #ptr_name as *const #struct_ident)).clone() };
            },
            arg: quote! { #name },
        }
    } else {
        ParamBinding {
            check: quote! { ring_check_number!(p, #idx); },
            get: quote! { let #name = ring_get_number!(p, #idx) as _; },
            arg: quote! { #name },
        }
    }
}

fn generate_vec_param_binding(name: &syn::Ident, type_str: &str, idx: i32) -> ParamBinding {
    let inner = extract_vec_inner(type_str).unwrap_or_default();

    if is_number_type(&inner) {
        let cast = get_number_cast(&inner);
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #name = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isnumber(__list, __i) {
                        #name.push(ring_list_getdouble(__list, __i) as #cast);
                    }
                }
            },
            arg: quote! { #name },
        }
    } else if is_string_type(&inner) {
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #name = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isstring(__list, __i) {
                        let __cstr = ring_list_getstring(__list, __i);
                        let __s = unsafe { std::ffi::CStr::from_ptr(__cstr).to_string_lossy().into_owned() };
                        #name.push(__s);
                    }
                }
            },
            arg: quote! { #name },
        }
    } else if is_struct_type(&inner) {
        let inner_struct_name = extract_struct_name(&inner);
        let inner_struct_ident = format_ident!("{}", inner_struct_name);
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #name = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    let __ptr = if ring_list_ispointer(__list, __i) {
                        ring_list_getpointer(__list, __i)
                    } else if ring_list_islist(__list, __i) {
                        let __inner_list = ring_list_getlist(__list, __i);
                        if ring_list_ispointer(__inner_list, 1) {
                            ring_list_getpointer(__inner_list, 1)
                        } else {
                            std::ptr::null_mut()
                        }
                    } else {
                        std::ptr::null_mut()
                    };
                    if !__ptr.is_null() {
                        let __item = unsafe { &*(__ptr as *const #inner_struct_ident) };
                        #name.push(__item.clone());
                    }
                }
            },
            arg: quote! { #name },
        }
    } else {
        let cast = get_number_cast(&inner);
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #name = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isnumber(__list, __i) {
                        #name.push(ring_list_getdouble(__list, __i) as #cast);
                    }
                }
            },
            arg: quote! { #name },
        }
    }
}

fn generate_option_param_binding(name: &syn::Ident, type_str: &str, idx: i32) -> ParamBinding {
    let inner = extract_option_inner(type_str).unwrap_or_default();

    if is_number_type(&inner) {
        let cast = get_number_cast(&inner);
        ParamBinding {
            check: quote! {},
            get: quote! {
                let #name = if ring_api_isstring(p, #idx) {
                    let __s = ring_get_string!(p, #idx);
                    if __s.is_empty() { None } else { None }
                } else if ring_api_isnumber(p, #idx) {
                    Some(ring_get_number!(p, #idx) as #cast)
                } else {
                    None
                };
            },
            arg: quote! { #name },
        }
    } else if is_string_type(&inner) {
        ParamBinding {
            check: quote! {},
            get: quote! {
                let #name = if ring_api_isstring(p, #idx) {
                    let __s = ring_get_string!(p, #idx);
                    if __s.is_empty() { None } else { Some(__s.to_string()) }
                } else {
                    None
                };
            },
            arg: quote! { #name },
        }
    } else if is_struct_type(&inner) {
        let inner_struct_name = extract_struct_name(&inner);
        let inner_struct_ident = format_ident!("{}", inner_struct_name);
        let inner_type_const = struct_type_const(&inner_struct_name);
        ParamBinding {
            check: quote! {},
            get: quote! {
                let #name = if ring_api_isstring(p, #idx) {
                    None
                } else if ring_api_ispointer(p, #idx) {
                    let __ptr = ring_get_cpointer!(p, #idx, #inner_type_const);
                    if __ptr.is_null() {
                        None
                    } else {
                        Some(unsafe { (*(__ptr as *const #inner_struct_ident)).clone() })
                    }
                } else {
                    None
                };
            },
            arg: quote! { #name },
        }
    } else {
        let cast = get_number_cast(&inner);
        ParamBinding {
            check: quote! {},
            get: quote! {
                let #name = if ring_api_isstring(p, #idx) {
                    None
                } else if ring_api_isnumber(p, #idx) {
                    Some(ring_get_number!(p, #idx) as #cast)
                } else {
                    None
                };
            },
            arg: quote! { #name },
        }
    }
}

fn generate_slice_param_binding(name: &syn::Ident, type_str: &str, idx: i32) -> ParamBinding {
    let inner = extract_slice_inner(type_str).unwrap_or_default();
    let vec_name = format_ident!("__{}_vec", name);

    if is_number_type(&inner) {
        let cast = get_number_cast(&inner);
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #vec_name: Vec<#cast> = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isnumber(__list, __i) {
                        #vec_name.push(ring_list_getdouble(__list, __i) as #cast);
                    }
                }
            },
            arg: quote! { &#vec_name[..] },
        }
    } else if is_string_type(&inner) {
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #vec_name: Vec<String> = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isstring(__list, __i) {
                        let __cstr = ring_list_getstring(__list, __i);
                        let __s = unsafe { std::ffi::CStr::from_ptr(__cstr).to_string_lossy().into_owned() };
                        #vec_name.push(__s);
                    }
                }
            },
            arg: quote! { &#vec_name[..] },
        }
    } else if is_struct_type(&inner) {
        let inner_struct_name = extract_struct_name(&inner);
        let inner_struct_ident = format_ident!("{}", inner_struct_name);
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #vec_name: Vec<#inner_struct_ident> = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    let __ptr = if ring_list_ispointer(__list, __i) {
                        ring_list_getpointer(__list, __i)
                    } else {
                        std::ptr::null_mut()
                    };
                    if !__ptr.is_null() {
                        let __item = unsafe { &*(__ptr as *const #inner_struct_ident) };
                        #vec_name.push(__item.clone());
                    }
                }
            },
            arg: quote! { &#vec_name[..] },
        }
    } else {
        ParamBinding {
            check: quote! { ring_check_list!(p, #idx); },
            get: quote! {
                let __list = ring_get_list!(p, #idx);
                let __size = ring_list_getsize(__list);
                let mut #vec_name: Vec<f64> = Vec::with_capacity(__size as usize);
                for __i in 1..=__size {
                    if ring_list_isnumber(__list, __i) {
                        #vec_name.push(ring_list_getdouble(__list, __i));
                    }
                }
            },
            arg: quote! { &#vec_name[..] },
        }
    }
}
