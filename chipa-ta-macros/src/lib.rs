use std::{
    collections::HashMap,
    fs::read_to_string,
    sync::{Arc, RwLock},
};

use darling::{
    ast::{Data, NestedMeta},
    util::Ignored,
    Error, FromDeriveInput, FromMeta, FromVariant,
};
use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, DeriveInput, Expr, ExprLit, FnArg, Ident, ItemTrait, Lit, Meta,
    Pat, Signature, TraitItemFn,
};
lazy_static! {
    static ref TRAIT_METHODS: Arc<RwLock<HashMap<String, Vec<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Derive macro options for auto-implementing traits
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(auto_implement))]
struct AutoImplOpts {
    /// The enum name
    ident: Ident,
    /// The enum data
    data: Data<EnumField, Ignored>,
    /// The traits to implement
    #[darling(multiple, rename = "trait")]
    traits: Vec<Ident>,
    /// The methods to map from trait to
    #[darling(multiple, rename = "method")]
    methods: Vec<MethodMapping>,
    #[darling()]
    path: Option<String>,
    #[darling()]
    context: Option<String>
}

#[derive(Debug, FromVariant)]
#[darling(attributes(auto_implement))]
struct EnumField {
    /// Variant name, e.g., "V1"
    ident: Ident,
}

/// Custom method mapping structure
#[derive(Debug, Clone)]
struct MethodMapping {
    trait_method: String,
    enum_method: String,
}

impl FromMeta for MethodMapping {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        // Handle method(key = "value") syntax
        if items.len() == 1 {
            if let NestedMeta::Meta(Meta::NameValue(nv)) = &items[0] {
                let trait_method = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| Error::custom("Expected identifier"))?
                    .to_string();

                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    let enum_method = lit_str.value();
                    return Ok(MethodMapping {
                        trait_method,
                        enum_method,
                    });
                }
            }
        }
        Err(Error::custom(
            "Expected method(trait_method = \"enum_method\")",
        ))
    }
}

impl EnumField {
    fn field_name(&self) -> Ident {
        Ident::new("inner", Span::call_site())
    }

    fn pattern(&self) -> TokenStream2 {
        let ident = &self.ident;
        let var_name = self.field_name();
        quote! {
            Self::#ident(#var_name)
        }
    }
}

impl AutoImplOpts {
    fn to_hashmap(&self) -> HashMap<Ident, Ident> {
        self.methods
            .iter()
            .map(|m| {
                let trait_method = Ident::new(&m.trait_method, Span::call_site());
                let enum_method = Ident::new(&m.enum_method, Span::call_site());
                (trait_method, enum_method)
            })
            .collect()
    }

    fn implement_trait(
        &self,
        trait_name: &Ident,
        methods: Vec<String>,
        maps: &HashMap<Ident, Ident>,
    ) -> syn::Result<TokenStream2> {
        let enum_name = &self.ident;
        // let data = TRAIT_METHODS.read().map_err(|e| syn::Error::new(Span::call_site(), format!("Failed to read trait methods: {}", e)))?;
        // let methods = data.get(&trait_name.to_string())
        //     .ok_or_else(|| syn::Error::new(Span::call_site(), format!("Trait '{}' not registered", trait_name)))?;
        let methods_signatures = methods
            .iter()
            .map(|s| syn::parse_str::<TraitItemFn>(s).map(|f| (f.sig, f.attrs)))
            .collect::<Result<Vec<(Signature, Vec<Attribute>)>, syn::Error>>()?;
        let methods_impl = methods_signatures
            .iter()
            .map(|(s, attrs)| self.implement_method(s, attrs.clone(), maps))
            .collect::<syn::Result<Vec<TokenStream2>>>()?;

        Ok(quote! {
            impl #trait_name for #enum_name {
                #(
                    #methods_impl
                )*
            }
        })
    }

    fn implement_method(
        &self,
        signature: &Signature,
        attrs: Vec<Attribute>,
        maps: &HashMap<Ident, Ident>,
    ) -> syn::Result<TokenStream2> {
        let fields = match &self.data {
            Data::Struct(_) => {
                return Err(syn::Error::new_spanned(
                    &signature.ident,
                    "AutoImpl can only be used on enums",
                ));
            }
            Data::Enum(fields) => fields,
        };

        let method_name = &signature.ident;
        let inputs = &signature.inputs;

        // Extract parameter names (excluding self)
        let param_names: Vec<TokenStream2> = inputs
            .iter()
            .filter_map(|input| {
                if let FnArg::Typed(pat_type) = input {
                    if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        let name = &pat_ident.ident;
                        return Some(quote! { #name });
                    }
                }
                None
            })
            .collect();

        if let Some(name) = maps.get(method_name) {
            // If the method is mapped, use the mapped name
            let mapped_name = name;
            return Ok(quote! {
                #(#attrs)*
                #signature {
                    Self::#mapped_name(#(#param_names),*)
                }
            });
        }

        // Generate match arms
        let match_arms: Vec<TokenStream2> = fields
            .iter()
            .map(|field| {
                let pattern = field.pattern();
                let field_name = field.field_name();
                quote! {
                    #pattern => #field_name.#method_name(#(#param_names),*)
                }
            })
            .collect();

        Ok(quote! {
            #(#attrs)*
            #signature {
                match self {
                    #(#match_arms,)*
                }
            }
        })
    }

    fn load_from_string(&self, string: &str) -> syn::Result<HashMap<String, Vec<String>>> {
        let mut methods: HashMap<String, Vec<String>> = HashMap::new();
        let file = syn::parse_file(string)?;
        for item in file.items {
            if let syn::Item::Trait(trait_item) = item {
                let trait_name = trait_item.ident.to_string();
                let trait_methods: Vec<String> = trait_item
                    .items
                    .iter()
                    .filter_map(|item| {
                        if let syn::TraitItem::Fn(method) = item {
                            Some(method.to_token_stream().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                methods.insert(trait_name, trait_methods);
            }
        }
        Ok(methods)
    }

    fn load_traits_from_file(&self) -> syn::Result<HashMap<String, Vec<String>>> {
        if let Some(path) = &self.path {
            let content = read_to_string(path).map_err(|e| {
                syn::Error::new(
                    Span::call_site(),
                    format!("Failed to read file '{path}': {e}"),
                )
            })?;
            self.load_from_string(&content)
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "Path to traits file is not specified",
            ))
        }
    }

    fn create_impls(&self) -> syn::Result<TokenStream2> {
        let maps = self.to_hashmap();
        let mut traits_and_methods: HashMap<String, Vec<String>> = TRAIT_METHODS.read().unwrap().clone();
        if let Some(_) = &self.path {
            traits_and_methods.extend(self.load_traits_from_file()?);
        }
        if let Some(context) = &self.context {
            traits_and_methods.extend(self.load_from_string(&context)?.into_iter());
        }

        let trait_impls = self
            .traits
            .iter()
            .map(|trait_name| {
                let methods = traits_and_methods
                    .get(&trait_name.to_string())
                    .ok_or_else(|| {
                        syn::Error::new(
                            Span::call_site(),
                            format!("Trait '{trait_name}' not registered"),
                        )
                    })?;

                self.implement_trait(trait_name, methods.clone(), &maps)
            })
            .collect::<Result<Vec<_>, syn::Error>>()?;
        Ok(quote! {
            #(#trait_impls)*
        })
    }
}

impl ToTokens for AutoImplOpts {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.create_impls() {
            Ok(impls) => {
                tokens.extend(impls);
            }
            Err(e) => {
                tokens.extend(e.to_compile_error());
            }
        }
    }
}

/// Derive macro for auto-implementing traits on enums
///
/// This macro automatically implements a trait for an enum by delegating
/// each method call to the corresponding method on the enum variants.
///
/// # Requirements
/// - The enum must have variants that are tuple structs with a single field
/// - All variant types must implement the specified trait
/// - The trait methods should be marked with `#[auto_method]` (optional but recommended)
///
/// # Usage
/// ```rust
/// #[derive(AutoImpl)]
/// #[auto_implement(trait_name = "MyTrait")]
/// pub enum MyEnum {
///     Variant1(Struct1),
///     Variant2(Struct2),
///     Variant3(Struct3),
/// }
/// ```
///
/// This will generate:
/// ```rust
/// impl MyTrait for MyEnum {
///     fn my_function(&self, param: i32) -> String {
///         match self {
///             Self::Variant1(inner) => inner.my_function(param),
///             Self::Variant2(inner) => inner.my_function(param),
///             Self::Variant3(inner) => inner.my_function(param),
///         }
///     }
/// }
/// ```
#[proc_macro_derive(AutoImpl, attributes(auto_implement))]
pub fn auto_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match AutoImplOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    quote! {
        #opts
    }
    .into()
}

/// Helper macro to generate trait method implementations for custom traits
///
/// This macro can be used to generate implementations for traits that aren't
/// hardcoded in the main macro. It takes a trait definition and generates
/// the corresponding enum implementation.
///
/// # Example
/// ```rust
/// register_trait! {
///     trait MyCustomTrait for MyEnum {
///         fn custom_method(&self, param: i32) -> String;
///         fn another_method(&mut self) -> Result<(), Error>;
///     }
/// }
/// ```
#[proc_macro]
pub fn register_trait(input: TokenStream) -> TokenStream {
    let mut state = match TRAIT_METHODS.write() {
        Ok(state) => state,
        Err(e) => {
            return syn::Error::new(Span::call_site(), e.to_string())
                .to_compile_error()
                .into()
        }
    };
    // Parse the input as a trait definition
    let input_dt = input.clone();
    let input_trait = parse_macro_input!(input as ItemTrait);

    // Extract trait information
    let trait_name = &input_trait.ident;
    let trait_name_str = trait_name.to_string();

    let trait_methods_str: Vec<String> = input_trait
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                Some(method.to_token_stream().to_string())
            } else {
                None
            }
        })
        .collect();

    state.insert(trait_name_str, trait_methods_str);

    input_dt
}
