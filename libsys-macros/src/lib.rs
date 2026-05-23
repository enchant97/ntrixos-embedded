//! Internal implementation to `libsys`.
//!
//! Do not use this crate directly.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType, parse_macro_input};

/// Entry point for app.
///
/// # Features
/// - Init memory
/// - Init ABI (global statics)
/// - handling clean app exit, returning `ExitCode::OK` exit code
///
/// # Example Usage
///
/// ```rs,editable
/// #![no_std]
/// #![no_main]
///
/// #[libsys::entrypoint]
/// fn main() {
///     let _ = libsys::core::get_abi_version();
/// }
/// ```
#[proc_macro_attribute]
pub fn entrypoint(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new_spanned(
            proc_macro2::TokenStream::from(args),
            "#[entrypoint] takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    let user_fn = parse_macro_input!(item as ItemFn);

    let returns_nothing = match &user_fn.sig.output {
        ReturnType::Default => true, // fn main()
        ReturnType::Type(_, ty) => {
            // fn main() -> ()
            matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
        }
    };

    if !returns_nothing {
        return Error::new_spanned(
            &user_fn.sig.output,
            "#[entrypoint] function must return `()` or have no return type",
        )
        .to_compile_error()
        .into();
    }

    if !user_fn.sig.inputs.is_empty() {
        return Error::new_spanned(
            &user_fn.sig.inputs,
            "#[entrypoint] function must take no arguments",
        )
        .to_compile_error()
        .into();
    }

    if user_fn.sig.asyncness.is_some() {
        return Error::new_spanned(
            &user_fn.sig.asyncness,
            "#[entrypoint] function must not be async \
             (core1 runs as raw-thread executor)",
        )
        .to_compile_error()
        .into();
    }

    let user_fn_ident = &user_fn.sig.ident;

    quote! {
        #user_fn
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text._start")]
        pub extern "C" fn _start(abi: *const ::libsys::sdk::KernelAbi) -> ::libsys::sdk::ExitCode {
            unsafe {
                ::libsys::mem::init_memory();
            }
            ::libsys::core::sys_init(abi);
            #user_fn_ident();
            ::libsys::sdk::ExitCode::Ok
        }
    }
    .into()
}
