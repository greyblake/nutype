pub mod error;
pub mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Generics;

use self::error::gen_validation_error_type;
use super::{
    models::{FloatDeriveTrait, FloatGuard, FloatSanitizer, FloatType, FloatValidator},
    FloatNewtype,
};
use crate::{
    common::{
        gen::{
            tests::{
                gen_test_should_have_consistent_lower_and_upper_boundaries,
                gen_test_should_have_valid_default_value,
            },
            traits::GeneratedTraits,
            GenerateNewtype,
        },
        models::{ConstFn, ErrorTypePath, Guard, TypeName},
    },
    float::models::FloatInnerType,
};
use traits::gen_traits;

impl<T> GenerateNewtype for FloatNewtype<T>
where
    T: FloatType + ToTokens + PartialOrd,
{
    type Sanitizer = FloatSanitizer<T>;
    type Validator = FloatValidator<T>;
    type InnerType = FloatInnerType;
    type TypedTrait = FloatDeriveTrait;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        const_fn: ConstFn,
    ) -> TokenStream {
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
            #const_fn fn __sanitize__(mut value: #inner_type) -> #inner_type {
                #transformations
                value
            }
        )
    }

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
        const_fn: ConstFn,
    ) -> TokenStream {
        let validations: TokenStream = validators
            .iter()
            .map(|validator| match validator {
                FloatValidator::Less(exclusive_upper_bound) => {
                    quote!(
                        if val >= #exclusive_upper_bound {
                            return Err(#error_type_path::LessViolated);
                        }
                    )
                }
                FloatValidator::LessOrEqual(max) => {
                    quote!(
                        if val > #max {
                            return Err(#error_type_path::LessOrEqualViolated);
                        }
                    )
                }
                FloatValidator::Greater(exclusive_lower_bound) => {
                    quote!(
                        if val <= #exclusive_lower_bound {
                            return Err(#error_type_path::GreaterViolated);
                        }
                    )
                }
                FloatValidator::GreaterOrEqual(min) => {
                    quote!(
                        if val < #min {
                            return Err(#error_type_path::GreaterOrEqualViolated);
                        }
                    )
                }
                FloatValidator::Predicate(custom_is_valid_fn) => {
                    quote!(
                        if !(#custom_is_valid_fn)(&val) {
                            return Err(#error_type_path::PredicateViolated);
                        }
                    )
                }
                FloatValidator::Finite => {
                    quote!(
                        if !val.is_finite() {
                            return Err(#error_type_path::FiniteViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            #const_fn fn __validate__(val: &#inner_type) -> core::result::Result<(), #error_type_path> {
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
        guard: &FloatGuard<T>,
    ) -> Result<GeneratedTraits, syn::Error> {
        gen_traits(
            type_name,
            generics,
            inner_type,
            maybe_default_value,
            traits,
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

        quote! {
            #test_lower_vs_upper
            #test_valid_default_value
        }
    }
}
