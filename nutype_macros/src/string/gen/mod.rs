pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Attribute;

use crate::{
    common::{
        gen::{
            error::gen_error_type_name, gen_impl_into_inner, gen_module_name_for_type,
            gen_reimports, new_unchecked::gen_new_unchecked, traits::GeneratedTraits,
            type_custom_closure,
        },
        models::{ErrorTypeName, InnerType, NewUnchecked, TypeName},
    },
    string::models::{RegexDef, StringSanitizer, StringValidator},
};

use self::{error::gen_validation_error_type, traits::gen_traits};

use super::models::{StringDeriveTrait, StringGuard};

pub fn gen_nutype_for_string(
    doc_attrs: Vec<Attribute>,
    traits: HashSet<StringDeriveTrait>,
    vis: syn::Visibility,
    type_name: &TypeName,
    guard: StringGuard,
    new_unchecked: NewUnchecked,
    maybe_default_value: Option<TokenStream>,
) -> TokenStream {
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_string_implementation(type_name, &guard, new_unchecked);

    let maybe_error_type_name: Option<ErrorTypeName> = match guard {
        StringGuard::WithoutValidation { .. } => None,
        StringGuard::WithValidation { .. } => Some(gen_error_type_name(type_name)),
    };

    let reimports = gen_reimports(
        vis,
        type_name,
        &module_name,
        maybe_error_type_name.as_ref(),
        None,
    );

    let GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
    } = gen_traits(
        type_name,
        maybe_error_type_name,
        maybe_default_value,
        traits,
    );

    quote!(
        #[doc(hidden)]
        mod #module_name {
            use super::*;

            #(#doc_attrs)*
            #derive_transparent_traits
            pub struct #type_name(String);

            #implementation
            #implement_traits
        }
        #reimports
    )
}

pub fn gen_string_implementation(
    type_name: &TypeName,
    meta: &StringGuard,
    new_unchecked: NewUnchecked,
) -> TokenStream {
    let impl_new = match meta {
        StringGuard::WithoutValidation { sanitizers } => {
            gen_new_without_validation(type_name, sanitizers)
        }
        StringGuard::WithValidation {
            sanitizers,
            validators,
        } => gen_new_and_with_validation(type_name, sanitizers, validators),
    };
    let inner_type = InnerType::String;
    let impl_into_inner = gen_impl_into_inner(type_name, inner_type);
    let impl_new_unchecked = gen_new_unchecked(type_name, inner_type, new_unchecked);

    quote! {
        #impl_new
        #impl_into_inner
        #impl_new_unchecked
    }
}

fn gen_new_without_validation(type_name: &TypeName, sanitizers: &[StringSanitizer]) -> TokenStream {
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
    type_name: &TypeName,
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
                let custom_sanitizer = type_custom_closure(custom_sanitizer_token_stream, tp);
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

pub fn gen_string_validate_fn(type_name: &TypeName, validators: &[StringValidator]) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    // Indicates that `chars_count` variable needs to be set, which is used within
    // min_len and max_len validations.
    let mut requires_chars_count = false;

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::MaxLen(max_len) => {
                requires_chars_count = true;
                quote!(
                    if chars_count > #max_len {
                        return Err(#error_name::TooLong);
                    }
                )
            }
            StringValidator::MinLen(min_len) => {
                requires_chars_count = true;
                quote!(
                    if chars_count < #min_len {
                        return Err(#error_name::TooShort);
                    }
                )
            }
            StringValidator::NotEmpty => {
                quote!(
                    if val.is_empty() {
                        return Err(#error_name::Empty);
                    }
                )
            }
            StringValidator::With(is_valid_fn) => {
                let tp = quote!(&str);
                let is_valid_fn = type_custom_closure(is_valid_fn, tp);
                quote!(
                    if !(#is_valid_fn)(&val) {
                        return Err(#error_name::Invalid);
                    }
                )
            }
            StringValidator::Regex(regex_def) => {
                match regex_def {
                    RegexDef::StringLiteral(regex_str_lit) => {
                        quote!(
                            lazy_static::lazy_static! {
                                // Make up a sufficiently unique regex name to ensure that it does
                                // not clashes with anything import with `use super::*`.
                                static ref __NUTYPE_REGEX__: ::regex::Regex = ::regex::Regex::new(#regex_str_lit).expect("Nutype failed to a build a regex");
                            }
                            if !__NUTYPE_REGEX__.is_match(&val) {
                                return Err(#error_name::RegexMismatch);
                            }
                        )

                    }
                    RegexDef::Ident(regex_ident) => {
                        quote!(
                            if !#regex_ident.is_match(&val) {
                                return Err(#error_name::RegexMismatch);
                            }
                        )
                    }
                }
            }
        })
        .collect();

    let chars_count_if_required = if requires_chars_count {
        quote!(
            let chars_count = val.chars().count();
        )
    } else {
        quote!()
    };

    quote!(
        fn validate(val: &str) -> ::core::result::Result<(), #error_name> {
            #chars_count_if_required
            #validations
            Ok(())
        }
    )
}
