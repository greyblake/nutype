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
            FloatValidator::Min(_) => {
                quote!(TooSmall,)
            }
            FloatValidator::Max(_) => {
                quote!(TooBig,)
            }
            FloatValidator::With(_) => {
                quote!(Invalid,)
            }
            FloatValidator::Finite => {
                quote!(NotFinite,)
            }
        })
        .collect();

    quote! {
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
        FloatValidator::Min(_) => quote! {
             #error_type_name::TooSmall => write!(f, "too small")
        },
        FloatValidator::Max(_) => quote! {
             #error_type_name::TooBig=> write!(f, "too big")
        },
        FloatValidator::With(_) => quote! {
             #error_type_name::Invalid => write!(f, "invalid")
        },
        FloatValidator::Finite => quote! {
             #error_type_name::NotFinite => write!(f, "not finite")
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
