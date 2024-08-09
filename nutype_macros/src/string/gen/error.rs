use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    common::{
        gen::error::gen_impl_error_trait,
        models::{ErrorTypeName, TypeName},
    },
    string::models::StringValidator,
};

pub fn gen_validation_error_type(
    type_name: &TypeName,
    error_type_name: &ErrorTypeName,
    validators: &[StringValidator],
) -> TokenStream {
    let definition = gen_definition(error_type_name, validators);
    let impl_display_trait = gen_impl_display_trait(type_name, error_type_name, validators);
    let impl_error_trait = gen_impl_error_trait(error_type_name);

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
        pub enum #error_type_name {
            #error_variants
        }
    }
}

fn gen_impl_display_trait(
    type_name: &TypeName,
    error_type_name: &ErrorTypeName,
    validators: &[StringValidator],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        StringValidator::LenCharMax(len_char_max) => quote! {
             #error_type_name::LenCharMaxViolated => write!(f, "{} is too long. The value length must be less than {:#?} character(s).", stringify!(#type_name), #len_char_max)
        },
        StringValidator::LenCharMin(len_char_min) => quote! {
             #error_type_name::LenCharMinViolated => write!(f, "{} is too short. The value length must be more than {:#?} character(s).", stringify!(#type_name), #len_char_min)
        },
        StringValidator::NotEmpty => quote! {
             #error_type_name::NotEmptyViolated => write!(f, "{} is empty.", stringify!(#type_name))
        },
        StringValidator::Predicate(_) => quote! {
             #error_type_name::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
        },
        StringValidator::Regex(_) => quote! {
             #error_type_name::RegexViolated => write!(f, "{} violated the regular expression.", stringify!(#type_name))
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
