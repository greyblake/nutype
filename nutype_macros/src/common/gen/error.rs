use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::common::models::{ErrorTypeName, TypeName};

pub fn gen_error_type_name(type_name: &TypeName) -> ErrorTypeName {
    let error_name_str = format!("{type_name}Error");
    let ident = Ident::new(&error_name_str, Span::call_site());
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
