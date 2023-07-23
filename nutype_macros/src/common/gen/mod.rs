pub mod error;
pub mod new_unchecked;
pub mod parse_error;
pub mod traits;

use std::{collections::HashSet, hash::Hash};

use self::traits::GeneratedTraits;

use super::models::{ErrorTypeName, Guard, NewUnchecked, ParseErrorTypeName, TypeName, TypeTrait};
use crate::common::{
    gen::{error::gen_error_type_name, new_unchecked::gen_new_unchecked, parse_error::gen_parse_error_name},
    models::ModuleName,
};
use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
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

pub fn gen_module_name_for_type(type_name: &TypeName) -> ModuleName {
    let ident = format_ident!("__nutype_private_{type_name}__");
    ModuleName::new(ident)
}

pub fn gen_reimports(
    vis: Visibility,
    type_name: &TypeName,
    module_name: &ModuleName,
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

pub trait GenerateNewtype {
    type Sanitizer;
    type Validator;
    type InnerType: ToTokens;
    type TypedTrait: Hash + TypeTrait;

    fn gen_fn_sanitize(inner_type: &Self::InnerType, sanitizers: &[Self::Sanitizer])
        -> TokenStream;

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream;

    fn gen_validation_error_type(
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream;

    fn gen_traits(
        type_name: &TypeName,
        inner_type: &Self::InnerType,
        maybe_error_type_name: Option<ErrorTypeName>,
        traits: HashSet<Self::TypedTrait>,
        maybe_default_value: Option<syn::Expr>,
    ) -> GeneratedTraits;

    fn gen_new_with_validation(
        type_name: &TypeName,
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        validators: &[Self::Validator],
    ) -> TokenStream {
        let sanitize = Self::gen_fn_sanitize(inner_type, sanitizers);
        let validation_error = Self::gen_validation_error_type(type_name, validators);
        let error_type_name = gen_error_type_name(type_name);
        let validate = Self::gen_fn_validate(inner_type, type_name, validators);

        quote!(
            #validation_error

            impl #type_name {
                pub fn new(raw_value: #inner_type) -> ::core::result::Result<Self, #error_type_name> {
                    // Keep sanitize() and validate() within new() so they do not overlap with outer
                    // scope imported with `use super::*`.
                    #sanitize
                    #validate

                    let sanitized_value = sanitize(raw_value);
                    validate(sanitized_value)?;
                    Ok(#type_name(sanitized_value))
                }
            }
        )
    }

    fn gen_new_without_validation(
        type_name: &TypeName,
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
    ) -> TokenStream {
        let sanitize = Self::gen_fn_sanitize(inner_type, sanitizers);

        quote!(
            impl #type_name {
                pub fn new(raw_value: #inner_type) -> Self {
                    #sanitize
                    Self(sanitize(raw_value))
                }
            }
        )
    }

    fn gen_implementation(
        type_name: &TypeName,
        inner_type: &Self::InnerType,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        new_unchecked: NewUnchecked,
    ) -> TokenStream {
        let impl_new = match guard {
            Guard::WithoutValidation { sanitizers } => {
                Self::gen_new_without_validation(type_name, &inner_type, sanitizers)
            }
            Guard::WithValidation {
                sanitizers,
                validators,
            } => Self::gen_new_with_validation(type_name, &inner_type, sanitizers, validators),
        };
        let impl_into_inner = gen_impl_into_inner(type_name, inner_type);
        let impl_new_unchecked = gen_new_unchecked(type_name, inner_type, new_unchecked);

        quote! {
            #impl_new
            #impl_into_inner
            #impl_new_unchecked
        }
    }

    fn gen_nutype(
        doc_attrs: Vec<syn::Attribute>,
        vis: Visibility,
        inner_type: &Self::InnerType,
        type_name: &TypeName,
        guard: Guard<Self::Sanitizer, Self::Validator>,
        traits: HashSet<Self::TypedTrait>,
        new_unchecked: NewUnchecked,
        maybe_default_value: Option<syn::Expr>,
    ) -> TokenStream {
        let module_name = gen_module_name_for_type(type_name);
        let implementation =
            Self::gen_implementation(type_name, &inner_type, &guard, new_unchecked);

        let maybe_error_type_name: Option<ErrorTypeName> = match guard {
            Guard::WithoutValidation { .. } => None,
            Guard::WithValidation { .. } => Some(gen_error_type_name(type_name)),
        };

        let has_from_str_trait = traits.iter().any(|t| t.is_from_str());
        let maybe_parse_error_type_name = if has_from_str_trait {
            Some(gen_parse_error_name(type_name))
        } else {
            None
        };

        let reimports = gen_reimports(
            vis,
            type_name,
            &module_name,
            maybe_error_type_name.as_ref(),
            maybe_parse_error_type_name.as_ref(),
        );

        let GeneratedTraits {
            derive_transparent_traits,
            implement_traits,
        } = Self::gen_traits(
            type_name,
            &inner_type,
            maybe_error_type_name,
            traits,
            maybe_default_value,
        );

        quote!(
            #[doc(hidden)]
            mod #module_name {
                use super::*;

                #(#doc_attrs)*
                #derive_transparent_traits
                pub struct #type_name(#inner_type);

                #implementation
                #implement_traits
            }
            #reimports
        )
    }
}
