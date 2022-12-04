pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Attribute;

use crate::{
    common::gen::{gen_module_name_for_type, type_custom_sanitizier_closure},
    models::{StringSanitizer, StringValidator},
};

use self::{
    error::{gen_error_type_name, gen_validation_error_type},
    traits::{gen_traits, GeneratedTraits},
};

use super::models::{NewtypeStringMeta, StringDeriveTrait};

pub fn gen_nutype_for_string(
    doc_attrs: Vec<Attribute>,
    traits: HashSet<StringDeriveTrait>,
    vis: syn::Visibility,
    type_name: &Ident,
    meta: NewtypeStringMeta,
) -> TokenStream {
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_string_implementation(type_name, &meta);

    let maybe_error_type_name: Option<Ident> = match meta {
        NewtypeStringMeta::From { .. } => None,
        NewtypeStringMeta::TryFrom { .. } => Some(gen_error_type_name(type_name)),
    };

    let error_type_import = match maybe_error_type_name {
        None => quote!(),
        Some(ref error_type_name) => {
            quote! (
                #vis use #module_name::#error_type_name;
            )
        }
    };

    let GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    } = gen_traits(type_name, maybe_error_type_name, traits);

    quote!(
        mod #module_name {
            use super::*;

            #(#doc_attrs)*
            #derive_standard_traits
            pub struct #type_name(String);

            #implementation
            #implement_traits
        }
        #vis use #module_name::#type_name;
        #error_type_import
    )
}

pub fn gen_string_implementation(type_name: &Ident, meta: &NewtypeStringMeta) -> TokenStream {
    let methods = gen_impl_methods(type_name);
    let convert_implementation = match meta {
        NewtypeStringMeta::From { sanitizers } => gen_new_without_validation(type_name, sanitizers),
        NewtypeStringMeta::TryFrom {
            sanitizers,
            validators,
        } => gen_new_and_with_validation(type_name, sanitizers, validators),
    };

    quote! {
        #convert_implementation
        #methods
    }
}

fn gen_impl_methods(type_name: &Ident) -> TokenStream {
    quote! {
        impl #type_name {
            pub fn into_inner(self) -> String {
                self.0
            }
        }
    }
}

fn gen_new_without_validation(type_name: &Ident, sanitizers: &[StringSanitizer]) -> TokenStream {
    let sanitize = gen_string_sanitize_fn(sanitizers);

    quote!(
        impl #type_name {
            pub fn new(raw_value: impl Into<String>) -> Self {
                #sanitize
                #type_name(sanitize(raw_value.into()))
            }
        }
    )
}

fn gen_new_and_with_validation(
    type_name: &Ident,
    sanitizers: &[StringSanitizer],
    validators: &[StringValidator],
) -> TokenStream {
    let sanitize = gen_string_sanitize_fn(sanitizers);
    let validation_error = gen_validation_error_type(type_name, validators);
    let error_type_name = gen_error_type_name(type_name);
    let validate = gen_string_validate_fn(type_name, validators);

    quote!(
        #validation_error

        impl #type_name {
            pub fn new(raw_value: impl Into<String>) -> ::core::result::Result<Self, #error_type_name> {
                // Keep sanitize() and validate() within new() so they do not overlap with outer
                // scope imported with `use super::*`.
                #sanitize
                #validate

                let sanitized_value = sanitize(raw_value.into());
                validate(&sanitized_value)?;
                Ok(#type_name(sanitized_value))
            }
        }
    )
}

pub fn gen_string_sanitize_fn(sanitizers: &[StringSanitizer]) -> TokenStream {
    let transformations: TokenStream = sanitizers
        .iter()
        .map(|san| match san {
            StringSanitizer::Trim => {
                // TODO: consider optimizing sequences of [trim, lowercase] and [trim, uppercase] to avoid
                // unnecessary allocation with `to_string()`
                quote!(
                    let value: String = value.trim().to_string();
                )
            }
            StringSanitizer::Lowercase => {
                quote!(
                    let value: String = value.to_lowercase();
                )
            }
            StringSanitizer::Uppercase => {
                quote!(
                    let value: String = value.to_uppercase();
                )
            }
            StringSanitizer::With(custom_sanitizer_token_stream) => {
                let tp = Ident::new("String", Span::call_site());
                let tp = quote!(#tp);
                let custom_sanitizer =
                    type_custom_sanitizier_closure(custom_sanitizer_token_stream, tp);
                quote!(
                    let value: String = (#custom_sanitizer)(value);
                )
            }
        })
        .collect();

    quote!(
        fn sanitize(value: String) -> String {
            #transformations
            value
        }
    )
}

pub fn gen_string_validate_fn(type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::MaxLen(max_len) => {
                quote!(
                    if val.len() > #max_len {
                        return Err(#error_name::TooLong);
                    }
                )
            }
            StringValidator::MinLen(min_len) => {
                quote!(
                    if val.len() < #min_len {
                        return Err(#error_name::TooShort);
                    }
                )
            }
            StringValidator::Present => {
                quote!(
                    if val.is_empty() {
                        return Err(#error_name::Missing);
                    }
                )
            }
            StringValidator::With(is_valid_fn) => {
                let tp = quote!(&str);
                let is_valid_fn = type_custom_sanitizier_closure(is_valid_fn, tp);
                quote!(
                    if !(#is_valid_fn)(&val) {
                        return Err(#error_name::Invalid);
                    }
                )
            }
        })
        .collect();

    quote!(
        fn validate(val: &str) -> ::core::result::Result<(), #error_name> {
            #validations
            Ok(())
        }
    )
}
