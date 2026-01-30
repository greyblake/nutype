use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

use crate::{
    any::models::AnyInnerType,
    common::{
        generate::generics::{SplitGenerics, add_generic_param},
        models::TypeName,
    },
};

pub fn gen_impl_trait_into_iter(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &AnyInnerType,
) -> TokenStream {
    let generics_with_iter_lifetime =
        add_generic_param(generics, syn::parse_quote!('__nutype_iter));

    let SplitGenerics {
        impl_generics,
        type_generics,
        where_clause,
    } = SplitGenerics::new(generics);

    let SplitGenerics {
        impl_generics: impl_generics_with_lifetime,
        type_generics: _,
        where_clause: where_clause_with_lifetime,
    } = SplitGenerics::new(&generics_with_iter_lifetime);

    // In the comments below, we assume that IntoIterator is derived for the following type
    //
    //     struct Names<'a, T: Display>(Vec<&'a T>) where T: Clone;
    //
    // NOTE: We deliberately do not generate an iterator over mutable references, because
    // this would allow the user to modify the elements of the collection, which may violate
    // the guarantees that nutype is supposed to provide.
    //
    // Example generated code:
    //
    // impl<'a, T: Display> IntoIterator for Names<'a, T> where T: Clone {
    //     type Item = <Vec<&'a T> as IntoIterator>::Item;
    //     type IntoIter = <Vec<&'a T> as IntoIterator>::IntoIter;
    //     fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
    // }
    //
    // impl<'a, '__nutype_iter, T: Display> IntoIterator for &'__nutype_iter Names<'a, T> where T: Clone {
    //     type Item = <&'__nutype_iter Vec<&'a T> as IntoIterator>::Item;
    //     ...
    // }
    quote!(
        // Implement IntoIterator for the type.
        impl #impl_generics ::core::iter::IntoIterator for #type_name #type_generics #where_clause {
            type Item = <#inner_type as ::core::iter::IntoIterator>::Item;
            type IntoIter = <#inner_type as ::core::iter::IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // IntoIterator for the reference to the type (so it can be iterated over references).
        impl #impl_generics_with_lifetime ::core::iter::IntoIterator
        for &'__nutype_iter #type_name #type_generics #where_clause_with_lifetime {
            type Item = <&'__nutype_iter #inner_type as ::core::iter::IntoIterator>::Item;
            type IntoIter = <&'__nutype_iter #inner_type as ::core::iter::IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter().into_iter()
            }
        }
    )
}
