pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Visibility;

use self::error::gen_validation_error_type;
use super::models::{FloatDeriveTrait, FloatGuard, FloatSanitizer, FloatValidator};
use crate::{
    common::{
        gen::{
            error::gen_error_type_name, gen_impl_into_inner, gen_module_name_for_type,
            gen_reimports, new_unchecked::gen_new_unchecked, parse_error::gen_parse_error_name,
            traits::GeneratedTraits,
        },
        models::{ErrorTypeName, NewUnchecked, TypeName},
    },
    float::models::FloatInnerType,
};
use traits::gen_traits;

// TODO: These are too many arguments indeed.
// Consider refactoring.
#[allow(clippy::too_many_arguments)]
pub fn gen_nutype_for_float<T>(
    doc_attrs: Vec<syn::Attribute>,
    vis: Visibility,
    inner_type: FloatInnerType,
    type_name: &TypeName,
    meta: FloatGuard<T>,
    traits: HashSet<FloatDeriveTrait>,
    new_unchecked: NewUnchecked,
    maybe_default_value: Option<syn::Expr>,
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_implementation(type_name, inner_type, &meta, new_unchecked);

    let maybe_error_type_name: Option<ErrorTypeName> = match meta {
        FloatGuard::WithoutValidation { .. } => None,
        FloatGuard::WithValidation { .. } => Some(gen_error_type_name(type_name)),
    };

    let maybe_parse_error_type_name = if traits.contains(&FloatDeriveTrait::FromStr) {
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
    } = gen_traits(
        type_name,
        inner_type,
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
            pub struct #type_name(#inner_type);

            #implementation
            #implement_traits
        }
        #reimports
    )
}

pub fn gen_implementation<T>(
    type_name: &TypeName,
    inner_type: FloatInnerType,
    meta: &FloatGuard<T>,
    new_unchecked: NewUnchecked,
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let impl_new = match meta {
        FloatGuard::WithoutValidation { sanitizers } => {
            gen_new_without_validation(type_name, inner_type, sanitizers)
        }
        FloatGuard::WithValidation {
            sanitizers,
            validators,
        } => gen_new_with_validation(type_name, inner_type, sanitizers, validators),
    };
    let impl_into_inner = gen_impl_into_inner(type_name, inner_type);
    let impl_new_unchecked = gen_new_unchecked(type_name, inner_type, new_unchecked);

    quote! {
        #impl_new
        #impl_into_inner
        #impl_new_unchecked
    }
}

fn gen_new_without_validation<T>(
    type_name: &TypeName,
    inner_type: FloatInnerType,
    sanitizers: &[FloatSanitizer<T>],
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
    type_name: &TypeName,
    inner_type: FloatInnerType,
    sanitizers: &[FloatSanitizer<T>],
    validators: &[FloatValidator<T>],
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

fn gen_sanitize_fn<T>(inner_type: FloatInnerType, sanitizers: &[FloatSanitizer<T>]) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let transformations: TokenStream = sanitizers
        .iter()
        .map(|san| match san {
            FloatSanitizer::With(custom_sanitizer) => {
                quote!(
                    value = (#custom_sanitizer)(value);
                )
            }
            FloatSanitizer::_Phantom(_) => {
                unreachable!("float::gen FloatSanitizer::_Phantom must not be used")
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
    type_name: &TypeName,
    inner_type: FloatInnerType,
    validators: &[FloatValidator<T>],
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let error_name = gen_error_type_name(type_name);

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            FloatValidator::Max(max) => {
                quote!(
                    if val > #max {
                        return Err(#error_name::TooBig);
                    }
                )
            }
            FloatValidator::Min(min) => {
                quote!(
                    if val < #min {
                        return Err(#error_name::TooSmall);
                    }
                )
            }
            FloatValidator::With(custom_is_valid_fn) => {
                quote!(
                    if !(#custom_is_valid_fn)(&val) {
                        return Err(#error_name::Invalid);
                    }
                )
            }
            FloatValidator::Finite => {
                quote!(
                    if !val.is_finite() {
                        return Err(#error_name::NotFinite);
                    }
                )
            }
        })
        .collect();

    quote!(
        fn validate(val: #inner_type) -> core::result::Result<(), #error_name> {
            #validations
            Ok(())
        }
    )
}
