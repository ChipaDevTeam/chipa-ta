use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse_quote, FnArg, ReturnType, parse_str, parse_macro_input, Ident};
use darling::{util::Override, FromDeriveInput};

#[derive(FromDeriveInput)]
#[darling(attributes(name))]
struct FunctionParser {
    ident: Ident,
    name: Override<String>
}

impl FunctionParser {
    fn print(&self) {
        
    }
}