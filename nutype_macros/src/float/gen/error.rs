use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::common::{
    gen::error::gen_impl_error_trait,
    models::{ErrorTypePath, TypeName},
};

use super::super::models::FloatValidator;

pub fn gen_validation_error_type<T: ToTokens>(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[FloatValidator<T>],
) -> TokenStream {
    let definition = gen_definition(error_type_path, validators);
    let impl_display_trait = gen_impl_display_trait(type_name, error_type_path, validators);
    let impl_error_trait = gen_impl_error_trait(error_type_path);

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #definition

        #impl_display_trait
        #impl_error_trait
    }
}

fn gen_definition<T>(
    error_type_path: &ErrorTypePath,
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
        pub enum #error_type_path {
            #error_variants
        }
    }
}

fn gen_impl_display_trait<T: ToTokens>(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[FloatValidator<T>],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        FloatValidator::Greater(val) => quote! {
             #error_type_path::GreaterViolated => write!(f, "{} is too small. The value must be greater than {:#?}.", stringify!(#type_name), #val)
        },
        FloatValidator::GreaterOrEqual(val) => quote! {
             #error_type_path::GreaterOrEqualViolated => write!(f, "{} is too small. The value must be greater or equal to {:#?}.", stringify!(#type_name), #val)
        },
        FloatValidator::LessOrEqual(val) => quote! {
             #error_type_path::LessOrEqualViolated=> write!(f, "{} is too big. The value must be less than {:#?}.", stringify!(#type_name), #val)
        },
        FloatValidator::Less(val) => quote! {
             #error_type_path::LessViolated=> write!(f, "{} is too big. The value must be less or equal to {:#?}.", stringify!(#type_name), #val)
        },
        FloatValidator::Predicate(_) => quote! {
             #error_type_path::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
        },
        FloatValidator::Finite => quote! {
             #error_type_path::FiniteViolated => write!(f, "{} is not finite.", stringify!(#type_name))
        },
    });

    quote! {
        impl ::core::fmt::Display for #error_type_path {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }
}
