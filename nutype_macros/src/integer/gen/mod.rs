pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Generics;

use self::{error::gen_validation_error_type, traits::gen_traits};
use super::{
    models::{
        IntegerDeriveTrait, IntegerGuard, IntegerInnerType, IntegerSanitizer, IntegerType,
        IntegerValidator,
    },
    IntegerNewtype,
};
use crate::common::{
    gen::{
        tests::{
            gen_associated_consts_should_be_valid,
            gen_test_should_have_consistent_lower_and_upper_boundaries,
            gen_test_should_have_valid_default_value,
        },
        traits::GeneratedTraits,
        GenerateNewtype,
    },
    models::{ConstAssign, ErrorTypePath, Guard, TypeName},
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
            fn __sanitize__(mut value: #inner_type) -> #inner_type {
                #transformations
                value
            }
        )
    }

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
    ) -> TokenStream {
        let validations: TokenStream = validators
            .iter()
            .map(|validator| match validator {
                IntegerValidator::Less(exclusive_upper_bound) => {
                    quote!(
                        if val >= #exclusive_upper_bound {
                            return Err(#error_type_path::LessViolated);
                        }
                    )
                }
                IntegerValidator::LessOrEqual(max) => {
                    quote!(
                        if val > #max {
                            return Err(#error_type_path::LessOrEqualViolated);
                        }
                    )
                }
                IntegerValidator::Greater(exclusive_lower_bound) => {
                    quote!(
                        if val <= #exclusive_lower_bound {
                            return Err(#error_type_path::GreaterViolated);
                        }
                    )
                }
                IntegerValidator::GreaterOrEqual(min) => {
                    quote!(
                        if val < #min {
                            return Err(#error_type_path::GreaterOrEqualViolated);
                        }
                    )
                }
                IntegerValidator::Predicate(custom_is_valid_fn) => {
                    quote!(
                        if !(#custom_is_valid_fn)(&val) {
                            return Err(#error_type_path::PredicateViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            fn __validate__(val: &#inner_type) -> ::core::result::Result<(), #error_type_path> {
                let val = *val;
                #validations
                Ok(())
            }
        )
    }

    fn gen_validation_error_type(
        type_name: &TypeName,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
    ) -> TokenStream {
        gen_validation_error_type(type_name, error_type_path, validators)
    }

    fn gen_traits(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        traits: HashSet<Self::TypedTrait>,
        maybe_default_value: Option<syn::Expr>,
        guard: &IntegerGuard<T>,
    ) -> Result<GeneratedTraits, syn::Error> {
        gen_traits(
            type_name,
            generics,
            inner_type,
            traits,
            maybe_default_value,
            guard,
        )
    }

    fn gen_tests(
        type_name: &TypeName,
        generics: &Generics,
        _inner_type: &Self::InnerType,
        maybe_default_value: &Option<syn::Expr>,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        _traits: &HashSet<Self::TypedTrait>,
        associated_consts: &[ConstAssign],
    ) -> TokenStream {
        let test_lower_vs_upper = guard.standard_validators().and_then(|validators| {
            gen_test_should_have_consistent_lower_and_upper_boundaries(type_name, validators)
        });

        let test_valid_default_value = gen_test_should_have_valid_default_value(
            type_name,
            generics,
            maybe_default_value,
            guard.has_validation(),
        );

        let test_associated_consts =
            gen_associated_consts_should_be_valid(type_name, associated_consts);

        quote! {
            #test_lower_vs_upper
            #test_valid_default_value
            #test_associated_consts
        }
    }
}
