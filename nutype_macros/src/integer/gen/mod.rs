pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Visibility;

use self::{error::gen_validation_error_type, traits::gen_traits};
use super::models::{IntegerDeriveTrait, IntegerSanitizer, IntegerValidator, NewtypeIntegerMeta};
use crate::{
    common::gen::{
        error::gen_error_type_name, gen_module_name_for_type, gen_reimports,
        parse_error::gen_parse_error_name, traits::GeneratedTraits, type_custom_closure,
    },
    models::IntegerType,
};

pub fn gen_nutype_for_integer<T>(
    doc_attrs: Vec<syn::Attribute>,
    vis: Visibility,
    number_type: IntegerType,
    type_name: &Ident,
    meta: NewtypeIntegerMeta<T>,
    traits: HashSet<IntegerDeriveTrait>,
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_implementation(type_name, number_type, &meta);
    let inner_type: TokenStream = quote!(#number_type);

    let maybe_error_type_name: Option<Ident> = match meta {
        NewtypeIntegerMeta::From { .. } => None,
        NewtypeIntegerMeta::TryFrom { .. } => Some(gen_error_type_name(type_name)),
    };

    let maybe_parse_error_type_name = if traits.contains(&IntegerDeriveTrait::FromStr) {
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
        derive_standard_traits,
        implement_traits,
    } = gen_traits(type_name, &inner_type, maybe_error_type_name, traits);

    quote!(
        mod #module_name {
            use super::*;

            #(#doc_attrs)*
            #derive_standard_traits
            pub struct #type_name(#inner_type);

            #implementation
            #implement_traits
        }
        #reimports
    )
}

pub fn gen_implementation<T>(
    type_name: &Ident,
    inner_type: IntegerType,
    meta: &NewtypeIntegerMeta<T>,
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let convert_implementation = match meta {
        NewtypeIntegerMeta::From { sanitizers } => {
            gen_new_without_validation(type_name, inner_type, sanitizers)
        }
        NewtypeIntegerMeta::TryFrom {
            sanitizers,
            validators,
        } => gen_new_with_validation(type_name, inner_type, sanitizers, validators),
    };
    let methods = gen_impl_methods(type_name, inner_type);

    quote! {
        #convert_implementation
        #methods
    }
}

fn gen_impl_methods(type_name: &Ident, inner_type: IntegerType) -> TokenStream {
    quote! {
        impl #type_name {
            pub fn into_inner(self) -> #inner_type {
                self.0
            }
        }
    }
}

fn gen_new_without_validation<T>(
    type_name: &Ident,
    inner_type: IntegerType,
    sanitizers: &[IntegerSanitizer<T>],
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let sanitize = gen_sanitize_fn(inner_type, sanitizers);

    quote!(
        impl #type_name {
            pub fn new(raw_value: #inner_type) -> Self {
                #sanitize
                Self(sanitize(raw_value))
            }
        }
    )
}

fn gen_new_with_validation<T>(
    type_name: &Ident,
    inner_type: IntegerType,
    sanitizers: &[IntegerSanitizer<T>],
    validators: &[IntegerValidator<T>],
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let sanitize = gen_sanitize_fn(inner_type, sanitizers);
    let validation_error = gen_validation_error_type(type_name, validators);
    let error_type_name = gen_error_type_name(type_name);
    let validate = gen_validate_fn(type_name, inner_type, validators);

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

fn gen_sanitize_fn<T>(inner_type: IntegerType, sanitizers: &[IntegerSanitizer<T>]) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let transformations: TokenStream = sanitizers
        .iter()
        .map(|san| match san {
            IntegerSanitizer::With(token_stream) => {
                let custom_sanitizer = type_custom_closure(token_stream, inner_type);
                quote!(
                    value = (#custom_sanitizer)(value);
                )
            }
            IntegerSanitizer::_Phantom(_) => {
                unreachable!("integer::gen: IntegerSanitizer::_Phantom must not be used")
            }
        })
        .collect();

    quote!(
        fn sanitize(mut value: #inner_type) -> #inner_type {
            #transformations
            value
        }
    )
}

fn gen_validate_fn<T>(
    type_name: &Ident,
    inner_type: IntegerType,
    validators: &[IntegerValidator<T>],
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let error_name = gen_error_type_name(type_name);

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            IntegerValidator::Max(max) => {
                quote!(
                    if val > #max {
                        return Err(#error_name::TooBig);
                    }
                )
            }
            IntegerValidator::Min(min) => {
                quote!(
                    if val < #min {
                        return Err(#error_name::TooSmall);
                    }
                )
            }
            IntegerValidator::With(is_valid_fn) => {
                let is_valid_fn = type_custom_closure(is_valid_fn, inner_type);
                quote!(
                    if !(#is_valid_fn)(&val) {
                        return Err(#error_name::Invalid);
                    }
                )
            }
        })
        .collect();

    quote!(
        fn validate(val: #inner_type) -> ::core::result::Result<(), #error_name> {
            #validations
            Ok(())
        }
    )
}
