use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

use crate::{
    any::models::AnyInnerType,
    common::{
        generate::{add_param, strip_trait_bounds_on_generics},
        models::TypeName,
    },
};

pub fn gen_impl_trait_into_iter(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &AnyInnerType,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let generics_with_iter_lifetime = add_param(generics, quote!('__nutype_iter));

    // In the comments below, we assume that IntoIterator is derived for the following type
    //
    //     struct Names<'a, T: Display>(Vec<&'a T>);
    //
    // NOTE: We deliberately do not generate an iterator over mutable references, because
    // this would allow the user to modify the elements of the collection, which may violate
    // the guarantees that nutype is supposed to provide.
    quote!(
        // Implement IntoIterator for the type.
        impl #generics ::core::iter::IntoIterator for #type_name #generics_without_bounds {    // impl<'a, T: Display> ::core::iter::IntoIterator for Names<'a, T> {
            type Item = <#inner_type as ::core::iter::IntoIterator>::Item;                     //     type Item = <Vec<&'a T> as ::core::iter::IntoIterator>::Item;
            type IntoIter = <#inner_type as ::core::iter::IntoIterator>::IntoIter;             //     type IntoIter = <Vec<&'a T> as ::core::iter::IntoIterator>::IntoIter;
                                                                                               //
            fn into_iter(self) -> Self::IntoIter {                                             //     fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()                                                             //         self.0.into_iter()
            }                                                                                  //     }
        }                                                                                      // }

        // IntoIterator for the reference to the type (so it can be iterated over references).
        impl #generics_with_iter_lifetime ::core::iter::IntoIterator                                  // impl<'a, '__nutype_iter, T: Display> ::core::iter::IntoIterator
        for &'__nutype_iter #type_name #generics_without_bounds {                                     // for &'__nutype_iter Names<'a, T> {
            type Item = <&'__nutype_iter #inner_type as ::core::iter::IntoIterator>::Item;            //     type Item = <&'__nutype_iter Vec<&'a T> as ::core::iter::IntoIterator>::Item;
            type IntoIter = <&'__nutype_iter #inner_type as ::core::iter::IntoIterator>::IntoIter;    //     type IntoIter = <&'__nutype_iter Vec<&'a T> as ::core::iter::IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {                                                    //     fn into_iter(self) -> Self::IntoIter {
                self.0.iter().into_iter()                                                             //         self.0.iter().into_iter()
            }                                                                                         //     }
        }
    )
}
