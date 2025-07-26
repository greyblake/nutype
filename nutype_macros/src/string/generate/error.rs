use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    common::{
        generate::error::gen_impl_error_trait,
        models::{ErrorTypePath, TypeName},
    },
    string::models::StringValidator,
};

pub fn gen_validation_error_type(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[StringValidator],
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

fn gen_definition(error_type_path: &ErrorTypePath, validators: &[StringValidator]) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::LenCharMax(_len) => {
                quote!(LenCharMaxViolated,)
            }
            StringValidator::LenCharMin(_len) => {
                quote!(LenCharMinViolated,)
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
        pub enum #error_type_path {
            #error_variants
        }
    }
}

fn gen_impl_display_trait(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[StringValidator],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        StringValidator::LenCharMax(len_char_max) => quote! {
            #error_type_path::LenCharMaxViolated => write!(
                f,
                "{} is too long: the maximum valid length is {} character{}.",
                stringify!(#type_name),
                #len_char_max,
                if #len_char_max == 1 { "" } else { "s" }
            )
        },
        StringValidator::LenCharMin(len_char_min) => quote! {
            #error_type_path::LenCharMinViolated => write!(
                f,
                "{} is too short: the minimum valid length is {} character{}.",
                stringify!(#type_name),
                #len_char_min,
                if #len_char_min == 1 { "" } else { "s" }
            )
        },
        StringValidator::NotEmpty => quote! {
             #error_type_path::NotEmptyViolated => write!(f, "{} is empty.", stringify!(#type_name))
        },
        StringValidator::Predicate(_) => quote! {
             #error_type_path::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
        },
        StringValidator::Regex(_) => quote! {
             #error_type_path::RegexViolated => write!(f, "{} violated the regular expression.", stringify!(#type_name))
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
