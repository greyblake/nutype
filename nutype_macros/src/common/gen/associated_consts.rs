use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

use crate::common::{
    gen::strip_trait_bounds_on_generics,
    models::{ConstAssign, TypeName},
};

pub fn gen_associated_consts(
    type_name: &TypeName,
    generics: &Generics,
    associated_consts: &[ConstAssign],
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let consts = associated_consts.iter().map(
        |ConstAssign {
             const_name,
             const_value,
             ..
         }| {
            quote! {
                pub const #const_name: Self = Self(#const_value);
            }
        },
    );

    quote! {
        impl #generics #type_name #generics_without_bounds {
            #(#consts)*
        }
    }
}
