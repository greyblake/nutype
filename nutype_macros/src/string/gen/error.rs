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
            StringValidator::MaxLen(_len) => {
                quote!(TooLong,)
            }
            StringValidator::MinLen(_len) => {
                quote!(TooShort,)
            }
            StringValidator::NotEmpty => {
                quote!(Empty,)
            }
            StringValidator::With(_) => {
                quote!(Invalid,)
            }
            StringValidator::Regex(_) => {
                quote!(RegexMismatch,)
            }
        })
        .collect();

    quote! {
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
        StringValidator::MaxLen(_len) => quote! {
             #error_type_name::TooLong => write!(f, "too long")
        },
        StringValidator::MinLen(_len) => quote! {
             #error_type_name::TooShort => write!(f, "too short")
        },
        StringValidator::NotEmpty => quote! {
             #error_type_name::Empty => write!(f, "empty")
        },
        StringValidator::With(_) => quote! {
             #error_type_name::Invalid => write!(f, "invalid")
        },
        StringValidator::Regex(_) => quote! {
             #error_type_name::RegexMismatch => write!(f, "regex mismatch")
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
