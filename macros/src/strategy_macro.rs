use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, Expr, Ident, Token};

/// AST for the `strategy_nodes!` macro
struct StrategyImpl {
    /// Name of the strategy struct/type to implement
    ident: Ident,
    /// List of (condition, action_node) pairs
    rules: Vec<(Expr, Expr)>,
}

impl Parse for StrategyImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the struct/enum name
        let ident: Ident = input.parse()?;
        // Parse braces containing rule list
        let content;
        braced!(content in input);

        let mut rules = Vec::new();
        while !content.is_empty() {
            // condition expression
            let cond: Expr = content.parse()?;
            // => token
            content.parse::<Token![=>]>()?;
            // action node expression
            let node: Expr = content.parse()?;
            rules.push((cond, node));
            // optional trailing comma
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(StrategyImpl { ident, rules })
    }
}

/// Function-like proc-macro to generate `evaluate` impl for a strategy
#[proc_macro]
pub fn strategy_nodes(input: TokenStream) -> TokenStream {
    let StrategyImpl { ident, rules } = parse_macro_input!(input as StrategyImpl);

    // Generate code: a sequence of if checks in the evaluate method
    let arms = rules.iter().map(|(cond, node)| {
        quote! {
            if #cond.evaluate(data)? {
                return #node.evaluate(data)?;
            }
        }
    });

    let expanded = quote! {
        impl #ident {
            /// Auto-generated `evaluate` method from `strategy_nodes!` macro
            pub fn evaluate(&mut self, data: &mut crate::strategy::MarketData) -> crate::error::TaResult<crate::strategy::Action> {
                #(#arms)*
                Ok(crate::strategy::Action::Hold)
            }
        }
    };

    expanded.into()
}
