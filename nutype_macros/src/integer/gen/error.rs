use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::super::models::IntegerValidator;
use crate::common::{
    gen::error::{gen_error_type_name, gen_impl_error_trait},
    models::TypeName,
};

pub fn gen_validation_error_type<T>(
    type_name: &TypeName,
    validators: &[IntegerValidator<T>],
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

fn gen_definition<T>(error_type_name: &Ident, validators: &[IntegerValidator<T>]) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            IntegerValidator::Min(_) => {
                quote!(TooSmall,)
            }
            IntegerValidator::Max(_) => {
                quote!(TooBig,)
            }
            IntegerValidator::With(_) => {
                quote!(Invalid,)
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
    error_type_name: &Ident,
    validators: &[IntegerValidator<T>],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        IntegerValidator::Min(_) => quote! {
             #error_type_name::TooSmall => write!(f, "too small")
        },
        IntegerValidator::Max(_) => quote! {
             #error_type_name::TooBig=> write!(f, "too big")
        },
        IntegerValidator::With(_) => quote! {
             #error_type_name::Invalid => write!(f, "invalid")
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
