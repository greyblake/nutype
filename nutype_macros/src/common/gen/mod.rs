pub mod error;
pub mod new_unchecked;
pub mod parse_error;
pub mod traits;

use super::models::{ErrorTypeName, ParseErrorTypeName, TypeName};
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::Visibility;

/// Inject an inner type into a closure, so compiler does not complain if the token stream matchers
/// the expected closure pattern.
///
/// Input:
///   |s| s.trim().to_lowercase()
/// Output:
///   |s: String| s.trim().to_lowercase()
pub fn type_custom_closure(
    closure_or_func_path: &TokenStream,
    inner_type: impl ToTokens,
) -> TokenStream {
    let inner_type_tokens = quote!(#inner_type);
    let mut ts: Vec<TokenTree> = closure_or_func_path.clone().into_iter().collect();

    // Check if the tokens match `|s|` pattern
    // If so, inject the type, e.g. `|s: String|`
    if ts.len() >= 3 && is_pipe(&ts[0]) && is_ident(&ts[1]) && is_pipe(&ts[2]) {
        let colon = TokenTree::Punct(Punct::new(':', Spacing::Alone));
        ts.insert(2, colon);
        for (index, tok) in inner_type_tokens.into_iter().enumerate() {
            let pos = index + 3;
            ts.insert(pos, tok);
        }
    }

    ts.into_iter().collect()
}

fn is_pipe(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(ref punct) => punct.as_char() == '|',
        _ => false,
    }
}

fn is_ident(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Ident(_))
}

pub fn gen_module_name_for_type(type_name: &TypeName) -> Ident {
    let module_name = format!("__nutype_private_{type_name}__");
    Ident::new(&module_name, Span::call_site())
}

pub fn gen_reimports(
    vis: Visibility,
    type_name: &TypeName,
    module_name: &Ident,
    maybe_error_type_name: Option<&ErrorTypeName>,
    maybe_parse_error_type_name: Option<&ParseErrorTypeName>,
) -> TokenStream {
    let reimport_main_type = quote! {
        #vis use #module_name::#type_name;
    };

    let reimport_error_type_if_needed = match maybe_error_type_name {
        None => quote!(),
        Some(ref error_type_name) => {
            quote! (
                #vis use #module_name::#error_type_name;
            )
        }
    };

    let reimport_parse_error_type_if_needed = match maybe_parse_error_type_name {
        None => quote!(),
        Some(ref parse_error_type_name) => {
            quote! (
                #vis use #module_name::#parse_error_type_name;
            )
        }
    };

    quote! {
        #reimport_main_type
        #reimport_error_type_if_needed
        #reimport_parse_error_type_if_needed
    }
}

pub fn gen_impl_into_inner(type_name: &TypeName, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl #type_name {
            pub fn into_inner(self) -> #inner_type {
                self.0
            }
        }
    }
}
