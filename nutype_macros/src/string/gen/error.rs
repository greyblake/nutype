use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    common::{
        gen::error::{gen_error_type_name, gen_impl_error_trait},
        models::{ErrorTypeName, TypeName},
    },
    string::models::StringValidator,
};

pub fn gen_validation_error_type(
    type_name: &TypeName,
    validators: &[StringValidator],
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

fn gen_definition(error_type_name: &ErrorTypeName, validators: &[StringValidator]) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::CharLenMax(_len) => {
                quote!(CharLenMaxViolated,)
            }
            StringValidator::CharLenMin(_len) => {
                quote!(CharLenMinViolated,)
            }
            StringValidator::NotEmpty => {
                quote!(NotEmptyViolated,)
            }
            StringValidator::Predicate(_) => {
                quote!(PredicateViolated,)
            }
            StringValidator::Regex(_) => {
                quote!(RegexViolated,)
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

fn gen_impl_display_trait(
    error_type_name: &ErrorTypeName,
    validators: &[StringValidator],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        StringValidator::CharLenMax(_len) => quote! {
             #error_type_name::CharLenMaxViolated => write!(f, "too long")
        },
        StringValidator::CharLenMin(_len) => quote! {
             #error_type_name::CharLenMinViolated => write!(f, "too short")
        },
        StringValidator::NotEmpty => quote! {
             #error_type_name::NotEmptyViolated => write!(f, "empty")
        },
        StringValidator::Predicate(_) => quote! {
             #error_type_name::PredicateViolated => write!(f, "invalid")
        },
        StringValidator::Regex(_) => quote! {
             #error_type_name::RegexViolated => write!(f, "regex violated")
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
