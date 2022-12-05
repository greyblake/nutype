use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::super::models::FloatValidator;

// TODO: Reuse
pub fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

pub fn gen_validation_error_type<T>(
    type_name: &Ident,
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

fn gen_definition<T>(error_type_name: &Ident, validators: &[FloatValidator<T>]) -> TokenStream {
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

// TODO: Reuse among the types
fn gen_impl_error_trait(error_type_name: &Ident) -> TokenStream {
    quote! {
        impl ::std::error::Error for #error_type_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                None
            }
        }
    }
}
