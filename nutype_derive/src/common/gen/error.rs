use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

pub fn gen_impl_error_trait(error_type_name: &Ident) -> TokenStream {
    quote! {
        impl ::std::error::Error for #error_type_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                None
            }
        }
    }
}
