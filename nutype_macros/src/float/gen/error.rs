use proc_macro2::TokenStream;
use quote::quote;

use crate::common::{
    gen::error::{gen_error_type_name, gen_impl_error_trait},
    models::{ErrorTypeName, TypeName},
};

use super::super::models::FloatValidator;

pub fn gen_validation_error_type<T>(
    type_name: &TypeName,
    validators: &[FloatValidator<T>],
) -> TokenStream {
    let error_type_name = gen_error_type_name(type_name);
    let definition = gen_definition(&error_type_name, validators);
    let impl_display_trait = gen_impl_display_trait(&error_type_name, validators);
    let impl_error_trait = gen_impl_error_trait(&error_type_name);

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #definition

        #impl_display_trait
        #impl_error_trait
    }
}

fn gen_definition<T>(
    error_type_name: &ErrorTypeName,
    validators: &[FloatValidator<T>],
) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            FloatValidator::Greater(_) => {
                quote!(GreaterViolated,)
            }
            FloatValidator::GreaterOrEqual(_) => {
                quote!(GreaterOrEqualViolated,)
            }
            FloatValidator::LessOrEqual(_) => {
                quote!(LessOrEqualViolated,)
            }
            FloatValidator::Less(_) => {
                quote!(LessViolated,)
            }
            FloatValidator::Predicate(_) => {
                quote!(PredicateViolated,)
            }
            FloatValidator::Finite => {
                quote!(FiniteViolated,)
            }
        })
        .collect();

    quote! {
        #[allow(clippy::enum_variant_names)]
        pub enum #error_type_name {
            #error_variants
        }
    }
}

fn gen_impl_display_trait<T>(
    error_type_name: &ErrorTypeName,
    validators: &[FloatValidator<T>],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        FloatValidator::Greater(_) => quote! {
             #error_type_name::GreaterViolated => write!(f, "too small")
        },
        FloatValidator::GreaterOrEqual(_) => quote! {
             #error_type_name::GreaterOrEqualViolated => write!(f, "too small")
        },
        FloatValidator::LessOrEqual(_) => quote! {
             #error_type_name::LessOrEqualViolated=> write!(f, "too big")
        },
        FloatValidator::Less(_) => quote! {
             #error_type_name::LessViolated=> write!(f, "too big")
        },
        FloatValidator::Predicate(_) => quote! {
             #error_type_name::PredicateViolated => write!(f, "invalid")
        },
        FloatValidator::Finite => quote! {
             #error_type_name::FiniteViolated => write!(f, "not finite")
        },
    });

    quote! {
        impl ::core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }
}
