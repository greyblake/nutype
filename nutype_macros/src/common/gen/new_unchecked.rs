use crate::common::models::{ConstFn, NewUnchecked, TypeName};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn gen_new_unchecked(
    type_name: &TypeName,
    inner_type: impl ToTokens,
    new_unchecked: NewUnchecked,
    const_fn: ConstFn,
) -> TokenStream {
    match new_unchecked {
        NewUnchecked::Off => quote! {},
        NewUnchecked::On => quote! {
            impl #type_name {
                /// Creates a value of type skipping the sanitization and validation
                /// rules. Generally, you should avoid using `::new_unchecked()` without a real need.
                /// Use `::new()` instead when it's possible.
                pub #const_fn unsafe fn new_unchecked(inner_value: #inner_type) -> #type_name {
                    #type_name(inner_value)
                }
            }
        },
    }
}
