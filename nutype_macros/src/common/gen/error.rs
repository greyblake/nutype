use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::common::models::{ErrorTypeName, TypeName};

pub fn gen_error_type_name(type_name: &TypeName) -> ErrorTypeName {
    let ident = format_ident!("{type_name}Error");
    ErrorTypeName::new(ident)
}

pub fn gen_impl_error_trait(error_type_name: &ErrorTypeName) -> TokenStream {
    quote! {
        impl ::std::error::Error for #error_type_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                None
            }
        }
    }
}
