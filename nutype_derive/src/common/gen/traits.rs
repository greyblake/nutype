use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use super::parse_error::{gen_def_parse_error, gen_parse_error_name};

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

pub fn gen_impl_trait_dislpay(type_name: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::fmt::Display for #type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                use ::core::fmt::Display;
                self.0.fmt(f)
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

pub fn gen_impl_trait_try_from(
    type_name: impl ToTokens,
    inner_type: impl ToTokens,
    error_type_name: impl ToTokens,
) -> TokenStream {
    quote! {
        impl ::core::convert::TryFrom<#inner_type> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: #inner_type) -> Result<#type_name, Self::Error> {
                Self::new(raw_value)
            }
        }
    }
}

/// Generate implementation of FromStr trait for non-string types (e.g. integers or floats).
pub fn gen_impl_trait_from_str(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<&Ident>,
) -> TokenStream {
    let parse_error_type_name = gen_parse_error_name(type_name);
    let def_parse_error = gen_def_parse_error(
        inner_type,
        type_name,
        maybe_error_type_name,
        &parse_error_type_name,
    );

    if let Some(_error_type_name) = maybe_error_type_name {
        // The case with validation
        quote! {
            #def_parse_error

            impl ::core::str::FromStr for #type_name {
                type Err = #parse_error_type_name;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let raw_value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Self::new(raw_value).map_err(#parse_error_type_name::Validate)
                }
            }
        }
    } else {
        // The case without validation
        quote! {
            #def_parse_error

            impl ::core::str::FromStr for #type_name {
                type Err = #parse_error_type_name;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Ok(#type_name::new(value))
                }
            }
        }
    }
}
