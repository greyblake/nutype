use crate::common::models::{ConstFn, ConstructorVisibility, NewUnchecked, TypeName};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub fn gen_new_unchecked(
    type_name: &TypeName,
    inner_type: impl ToTokens,
    new_unchecked: NewUnchecked,
    const_fn: ConstFn,
    constructor_visibility: &ConstructorVisibility,
) -> TokenStream {
    match new_unchecked {
        NewUnchecked::Off => quote! {},
        NewUnchecked::On => quote! {
            impl #type_name {
                /// Creates a value of type skipping the sanitization and validation
                /// rules. Generally, you should avoid using `::new_unchecked()` without a real need.
                /// Use `::new()` instead when it's possible.
                #constructor_visibility #const_fn unsafe fn new_unchecked(inner_value: #inner_type) -> #type_name {
                    #type_name(inner_value)
                }
            }
        },
    }
}
