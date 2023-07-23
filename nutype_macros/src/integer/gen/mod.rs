pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use self::{error::gen_validation_error_type, traits::gen_traits};
use super::{
    models::{
        IntegerDeriveTrait, IntegerInnerType, IntegerSanitizer, IntegerType, IntegerValidator,
    },
    IntegerNewtype,
};
use crate::common::{
    gen::{error::gen_error_type_name, traits::GeneratedTraits, GenerateNewtype},
    models::{ErrorTypeName, TypeName},
};

impl<T> GenerateNewtype for IntegerNewtype<T>
where
    T: IntegerType + ToTokens + PartialOrd,
{
    type Sanitizer = IntegerSanitizer<T>;
    type Validator = IntegerValidator<T>;
    type InnerType = IntegerInnerType;
    type TypedTrait = IntegerDeriveTrait;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
    ) -> TokenStream {
        let transformations: TokenStream = sanitizers
            .iter()
            .map(|san| match san {
                IntegerSanitizer::With(custom_sanitizer) => {
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

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream {
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
                IntegerValidator::With(custom_is_valid_fn) => {
                    quote!(
                        if !(#custom_is_valid_fn)(&val) {
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
