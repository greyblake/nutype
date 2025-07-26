use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::common::models::{ErrorTypePath, TypeName};

/// Generate a default error type name if the error name is not specified explicitly by
/// the user in the attributes.
pub fn gen_error_type_name(type_name: &TypeName) -> ErrorTypePath {
    let ident = format_ident!("{type_name}Error");
    ErrorTypePath::new(ident)
}

// NOTE: `::core::error::Error` is stable only for rust >= 1.81.0.
#[allow(unused_variables)]
pub fn gen_impl_error_trait(error_type_path: &ErrorTypePath) -> TokenStream {
    cfg_if! {
        if #[cfg(any(ERROR_IN_CORE, feature = "std"))] {
            cfg_if! {
                if #[cfg(ERROR_IN_CORE)] {
                    let error = quote! { ::core::error::Error };
                } else {
                    let error = quote! { ::std::error::Error };
                }
            };

            quote! {
                impl #error for #error_type_path {
                    fn source(&self) -> Option<&(dyn #error + 'static)> {
                        None
                    }
                }
            }
        } else {
            quote!{}
        }
    }
}
