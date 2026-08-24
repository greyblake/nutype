mod error;
mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, parse_quote};

use crate::common::{
    generate::{
        GenerateNewtype, tests::gen_test_should_have_valid_default_value, traits::GeneratedTraits,
    },
    models::{
        ConditionalDeriveGroup, ConstFn, ErrorTypePath, Guard, SerdeCustomization,
        SpannedDeriveUnsafeTrait, TypeName, TypedCustomFunction,
    },
};

use self::error::gen_validation_error_type;

use super::{
    AnyNewtype,
    models::{AnyDeriveTrait, AnyGuard, AnyInnerType, AnySanitizer, AnyValidator},
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
        const_fn: ConstFn,
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
                AnyValidator::Predicate(predicate) => {
                    let inner_type_ref: syn::Type = parse_quote!(
                        &'nutype_a #inner_type
                    );
                    let typed_predicate: TypedCustomFunction = predicate
                        .clone()
                        .try_into_typed(&inner_type_ref)
                        .expect("Failed to convert predicate into a typed closure");
                    quote!(
                        if !(#typed_predicate)(val) {
                            return Err(#error_type_path::PredicateViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            // NOTE 1: we're using a unique lifetime name `nutype_a` in a hope that it will not clash
            // with any other lifetimes in the user's code.
            //
            // NOTE 2:
            // When inner type is Cow<'a, str>, the generated code will look like this (with 2
            // lifetimes):
            //
            //     fn __validate__<'nutype_a>(val: &'nutype_a Cow<'a, str>)
            //
            // Clippy does not like passing a reference to a Cow. So we need to ignore the `clippy::ptr_arg` warning.
            // Since this code is generic which is used for different inner types (not only Cow), we cannot easily fix it to make
            // clippy happy.
            #[allow(clippy::ptr_arg)]
            #const_fn fn __validate__<'nutype_a>(val: &'nutype_a #inner_type) -> ::core::result::Result<(), #error_type_path> {
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
        unsafe_traits: &[SpannedDeriveUnsafeTrait],
        maybe_default_value: Option<syn::Expr>,
        guard: &AnyGuard,
        conditional_derives: &[ConditionalDeriveGroup<Self::TypedTrait>],
        serde_customization: &SerdeCustomization,
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
            serde_customization,
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
        let test_valid_default_value = gen_test_should_have_valid_default_value(
            type_name,
            generics,
            maybe_default_value,
            guard.has_validation(),
        );

        quote! {
            #test_valid_default_value
        }
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    use crate::common::generate::gen_impl_into_inner;
    use crate::common::models::{ConstructorVisibility, ErrorTypePath, Validation};
    use quote::{format_ident, quote};
    use syn::parse::Parser;

    fn inner_type() -> AnyInnerType {
        let field = syn::Field::parse_unnamed
            .parse2(quote!(String))
            .expect("field should parse");
        AnyInnerType::new(field)
    }

    fn stripped(ts: TokenStream) -> String {
        ts.to_string().split_whitespace().collect()
    }

    #[test]
    fn new_is_marked_inline() {
        let rendered = stripped(AnyNewtype::gen_new(
            &TypeName::new(format_ident!("Foo")),
            &Generics::default(),
            &inner_type(),
            &[],
            ConstFn::NoConst,
            &ConstructorVisibility::Public,
        ));
        assert!(rendered.contains("#[inline]pubfnnew"), "{rendered}");
    }

    #[test]
    fn try_new_is_marked_inline() {
        let error_path: syn::Path = syn::parse_quote!(FooError);
        let validation = Validation::Standard {
            validators: Vec::<AnyValidator>::new(),
            error_type_path: ErrorTypePath::new(error_path),
        };
        let rendered = stripped(AnyNewtype::gen_try_new(
            &TypeName::new(format_ident!("Foo")),
            &Generics::default(),
            &inner_type(),
            &[],
            &validation,
            ConstFn::NoConst,
            &ConstructorVisibility::Public,
        ));
        assert!(rendered.contains("#[inline]pubfntry_new"), "{rendered}");
    }

    #[test]
    fn into_inner_stays_inline() {
        let rendered = stripped(gen_impl_into_inner(
            &TypeName::new(format_ident!("Foo")),
            &Generics::default(),
            inner_type(),
            ConstFn::NoConst,
        ));
        assert!(rendered.contains("#[inline]pubfninto_inner"), "{rendered}");
    }
}
