use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::common::models::{ErrorTypeName, TypeName};

/// Generate a default error type name if the error name is not specified explicitly by
/// the user in the attributes.
pub fn gen_error_type_name(type_name: &TypeName) -> ErrorTypeName {
    let ident = format_ident!("{type_name}Error");
    ErrorTypeName::new(ident)
}

// NOTE: There is no `::core::error::Error` yet in stable Rust.
// So for `no_std` we just don't implement `Error` trait.
#[allow(unused_variables)]
pub fn gen_impl_error_trait(error_type_name: &ErrorTypeName) -> TokenStream {
    cfg_if! {
        if #[cfg(feature = "std")] {
            quote! {
                impl ::std::error::Error for #error_type_name {
                    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                        None
                    }
                }
            }
        } else {
            quote!{}
        }
    }
}
