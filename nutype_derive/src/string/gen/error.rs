use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::string::models::StringValidator;

pub fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

pub fn gen_validation_error_type(type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
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

fn gen_definition(error_type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::MaxLen(_len) => {
                quote!(TooLong,)
            }
            StringValidator::MinLen(_len) => {
                quote!(TooShort,)
            }
            StringValidator::Present => {
                quote!(Missing,)
            }
            StringValidator::With(_) => {
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

fn gen_impl_display_trait(error_type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        StringValidator::MaxLen(_len) => quote! {
             #error_type_name::TooLong => write!(f, "too long")
        },
        StringValidator::MinLen(_len) => quote! {
             #error_type_name::TooShort => write!(f, "too short")
        },
        StringValidator::Present => quote! {
             #error_type_name::Missing => write!(f, "missing")
        },
        StringValidator::With(_) => quote! {
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

fn gen_impl_error_trait(error_type_name: &Ident) -> TokenStream {
    quote! {
        impl ::std::error::Error for #error_type_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                None
            }
        }
    }
}
