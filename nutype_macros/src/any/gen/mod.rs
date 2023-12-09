mod error;
mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::common::{
    gen::{
        error::gen_error_type_name, tests::gen_test_should_have_valid_default_value,
        traits::GeneratedTraits, GenerateNewtype,
    },
    models::{ErrorTypeName, Guard, TypeName, TypedCustomFunction},
};

use self::error::gen_validation_error_type;

use super::{
    models::{AnyDeriveTrait, AnyGuard, AnyInnerType, AnySanitizer, AnyValidator},
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
                    let inner_type_ref: syn::Type = parse_quote!(
                        #inner_type
                    );
                    let typed_sanitizer: TypedCustomFunction = custom_sanitizer
                        .clone()
                        .try_into_typed(&inner_type_ref)
                        .expect("Failed to convert `with` sanitizer into a typed closure");
                    quote!(
                        value = (#typed_sanitizer)(value);
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
                AnyValidator::Predicate(predicate) => {
                    let inner_type_ref: syn::Type = parse_quote!(
                        &'a #inner_type
                    );
                    let typed_predicate: TypedCustomFunction = predicate
                        .clone()
                        .try_into_typed(&inner_type_ref)
                        .expect("Failed to convert predicate into a typed closure");
                    quote!(
                        if !(#typed_predicate)(val) {
                            return Err(#error_name::PredicateViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            fn validate<'a>(val: &'a #inner_type) -> ::core::result::Result<(), #error_name> {
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
        _guard: &AnyGuard,
    ) -> Result<GeneratedTraits, syn::Error> {
        Ok(gen_traits(
            type_name,
            inner_type,
            maybe_error_type_name,
            traits,
            maybe_default_value,
        ))
    }

    fn gen_tests(
        type_name: &TypeName,
        _inner_type: &Self::InnerType,
        maybe_default_value: &Option<syn::Expr>,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        _traits: &HashSet<Self::TypedTrait>,
    ) -> TokenStream {
        let test_valid_default_value = gen_test_should_have_valid_default_value(
            type_name,
            maybe_default_value,
            guard.has_validation(),
        );

        quote! {
            #test_valid_default_value
        }
    }
}
