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

pub fn gen_impl_trait_as_ref(type_name: impl ToTokens, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::convert::AsRef<#inner_type> for #type_name {
            fn as_ref(&self) -> &#inner_type {
                &self.0
            }
        }
    }
}

pub fn gen_impl_trait_borrow(
    type_name: impl ToTokens,
    borrowed_type: impl ToTokens,
) -> TokenStream {
    quote! {
        impl ::core::borrow::Borrow<#borrowed_type> for #type_name {
            fn borrow(&self) -> &#borrowed_type {
                &self.0
            }
        }
    }
}

pub fn gen_impl_trait_from(type_name: impl ToTokens, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::convert::From<#inner_type> for #type_name {
            fn from(raw_value: #inner_type) -> Self {
                Self::new(raw_value)
            }
        }
    }
}
