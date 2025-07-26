use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    any::models::AnyValidator,
    common::{
        generate::error::gen_impl_error_trait,
        models::{ErrorTypePath, TypeName},
    },
};

pub fn gen_validation_error_type(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[AnyValidator],
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

fn gen_definition(error_type_path: &ErrorTypePath, validators: &[AnyValidator]) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            AnyValidator::Predicate(_) => {
                quote!(PredicateViolated,)
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

fn gen_impl_display_trait(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[AnyValidator],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        AnyValidator::Predicate(_) => quote! {
             #error_type_path::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
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
