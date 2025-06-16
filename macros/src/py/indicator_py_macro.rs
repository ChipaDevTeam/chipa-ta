//! Macro implementation for #[derive(IndicatorPy)]
//!
//! This macro parses an enum and generates a Python-compatible struct with pyo3 bindings.
//!
//! - For each variant, it generates a constructor method with the same arguments as the original enum's constructors.
//! - Implements `next`, `next_batched`, `next_candle`, and `next_candles` as Python-callable methods.
//! - Uses darling for attribute parsing, syn for syntax parsing, and quote for code generation.
//! - See the comments in this file for extension points and macro update instructions.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, DeriveInput, FnArg, Ident, ItemEnum, ItemFn, ItemImpl, Pat, ReturnType, Type,
};

/// Main macro: #[derive(IndicatorPy)]
#[proc_macro_derive(IndicatorPy)]
pub fn indicator_py_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_data = match &input.data {
        syn::Data::Enum(e) => e,
        _ => panic!("IndicatorPy can only be derived for enums"),
    };
    let enum_name = &input.ident;
    let py_struct_name = format_ident!("{}Py", enum_name);

    // Collect variant names for later use
    let variants: Vec<_> = enum_data.variants.iter().map(|v| &v.ident).collect();

    // TODO: Generate #[pyclass], #[gen_stub_pyclass], and wrapper struct
    // TODO: Generate #[pymethods] impl with next, next_batched, next_candle, next_candles, to_json, from_string
    // TODO: Generate #[staticmethod] constructors for each variant (to be filled by the second macro)

    let expanded = quote! {
        // #[pyclass]
        // #[gen_stub_pyclass]
        #[derive(Clone, Default)]
        pub struct #py_struct_name {
            pub inner: #enum_name,
        }
        // #[pymethods]
        impl #py_struct_name {
            // ... methods will be generated here ...
        }
    };
    expanded.into()
}

/// Second macro: #[indicator_py_methods]
/// Applied to the impl block of the enum, collects constructors and generates Python-callable wrappers.
#[proc_macro_attribute]
pub fn indicator_py_methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    // TODO: Parse each function, check if its name matches a variant (snake_case),
    // and if its return type is Self or Result<Self>. Store signature for wrapper generation.
    // For now, just return the original impl block.
    let expanded = quote! {
        #input
    };
    expanded.into()
}
