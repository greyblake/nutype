mod error;
mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

use crate::common::{
    gen::{error::gen_error_type_name, traits::GeneratedTraits, GenerateNewtype},
    models::{ErrorTypeName, TypeName},
};

use self::error::gen_validation_error_type;

use super::{
    models::{AnyDeriveTrait, AnyInnerType, AnySanitizer, AnyValidator},
    AnyNewtype,
};

use traits::gen_traits;

impl GenerateNewtype for AnyNewtype {
    type Sanitizer = AnySanitizer;
    type Validator = AnyValidator;
    type InnerType = AnyInnerType;
    type TypedTrait = AnyDeriveTrait;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
    ) -> TokenStream {
        let transformations: TokenStream = sanitizers
            .iter()
            .map(|san| match san {
                AnySanitizer::With(custom_sanitizer) => {
                    // TODO: convert into typed closure!
                    // let typed_custom_sanitized: TypedCustomFunction = custom_sanitizer.try_into_typed(
                    quote!(
                        value = (#custom_sanitizer)(value);
                    )
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

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream {
        let error_name = gen_error_type_name(type_name);

        let validations: TokenStream = validators
            .iter()
            .map(|validator| match validator {
                AnyValidator::Predicate(custom_is_valid_fn) => {
                    // TODO: convert into typed closure!
                    // let typed_custom: TypedCustomFunction = custom_is_valid_fn.try_into_typed(
                    quote!(
                        if !(#custom_is_valid_fn)(&val) {
                            return Err(#error_name::PredicateViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            fn validate(val: &#inner_type) -> ::core::result::Result<(), #error_name> {
                #validations
                Ok(())
            }
        )
    }

    fn gen_validation_error_type(
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream {
        gen_validation_error_type(type_name, validators)
    }

    fn gen_traits(
        type_name: &TypeName,
        inner_type: &Self::InnerType,
        maybe_error_type_name: Option<ErrorTypeName>,
        traits: HashSet<Self::TypedTrait>,
        maybe_default_value: Option<syn::Expr>,
    ) -> GeneratedTraits {
        gen_traits(
            type_name,
            inner_type,
            maybe_error_type_name,
            traits,
            maybe_default_value,
        )
    }
}
