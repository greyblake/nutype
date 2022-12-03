use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn gen_impl_trait_into(type_name: impl ToTokens, inner_type: impl ToTokens) -> TokenStream {
    // NOTE: We're getting blank implementation of
    //     Into<Inner> for Type
    // by implementing
    //     From<Type> for Inner
    quote! {
        impl ::core::convert::From<#type_name> for #inner_type {
            fn from(value: #type_name) -> Self {
                value.into_inner()
            }
        }
    }
}
