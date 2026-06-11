pub mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Generics;

use self::traits::gen_traits;
use super::{
    DecimalNewtype,
    models::{
        DecimalDeriveTrait, DecimalGuard, DecimalInnerType, DecimalSanitizer, DecimalType,
        DecimalValidator,
    },
};
use crate::common::{
    generate::{
        GenerateNewtype,
        numeric::{
            gen_numeric_fn_sanitize, gen_numeric_fn_validate, gen_numeric_validation_error_type,
        },
        tests::{
            gen_test_should_have_consistent_lower_and_upper_boundaries,
            gen_test_should_have_valid_default_value,
        },
        traits::GeneratedTraits,
    },
    models::{
        ConditionalDeriveGroup, ConstFn, ErrorTypePath, Guard, SpannedDeriveUnsafeTrait, TypeName,
    },
};

impl<T> GenerateNewtype for DecimalNewtype<T>
where
    T: DecimalType + ToTokens + PartialOrd,
{
    type Sanitizer = DecimalSanitizer<T>;
    type Validator = DecimalValidator<T>;
    type InnerType = DecimalInnerType;
    type TypedTrait = DecimalDeriveTrait;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        const_fn: ConstFn,
    ) -> TokenStream {
        gen_numeric_fn_sanitize(inner_type, sanitizers, const_fn)
    }

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
        const_fn: ConstFn,
    ) -> TokenStream {
        gen_numeric_fn_validate(inner_type, error_type_path, validators, const_fn)
    }

    fn gen_validation_error_type(
        type_name: &TypeName,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
    ) -> TokenStream {
        gen_numeric_validation_error_type(type_name, error_type_path, validators)
    }

    fn gen_traits(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        traits: HashSet<Self::TypedTrait>,
        unsafe_traits: &[SpannedDeriveUnsafeTrait],
        maybe_default_value: Option<syn::Expr>,
        guard: &DecimalGuard<T>,
        conditional_derives: &[ConditionalDeriveGroup<Self::TypedTrait>],
    ) -> Result<GeneratedTraits, syn::Error> {
        gen_traits(
            type_name,
            generics,
            inner_type,
            traits,
            unsafe_traits,
            maybe_default_value,
            guard,
            conditional_derives,
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
