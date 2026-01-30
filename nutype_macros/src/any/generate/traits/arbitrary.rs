use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Generics;

use crate::{
    any::models::{AnyGuard, AnyInnerType},
    common::generate::generics::{SplitGenerics, add_bound_to_all_type_params, add_generic_param},
    common::models::TypeName,
};

pub fn gen_impl_trait_arbitrary(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &AnyInnerType,
    guard: &AnyGuard,
) -> Result<TokenStream, syn::Error> {
    // It's not possible to generate implementation of `Arbitrary` trait, because we don't know nor
    // type nor validation rules.
    if guard.has_validation() {
        let msg = format!(
            "Cannot derive trait `Arbitrary` for a custom type `{type_name}` which contains validation.\nYou have to implement `Arbitrary` trait manually to guarantee that it respects the validation rules.",
        );
        return Err(syn::Error::new(Span::call_site(), msg));
    }

    // Generate implementation of `Arbitrary` trait, assuming that inner type implements Arbitrary
    // too.
    //
    // We need to:
    // 1. Add a lifetime 'nu_arb
    // 2. Add Arbitrary<'nu_arb> bound to all type params
    let generics_with_lifetime = add_generic_param(generics, syn::parse_quote!('nu_arb));
    let generics_with_bounds = add_bound_to_all_type_params(
        &generics_with_lifetime,
        syn::parse_quote!(::arbitrary::Arbitrary<'nu_arb>),
    );

    let SplitGenerics {
        impl_generics,
        type_generics: _,
        where_clause,
    } = SplitGenerics::new(&generics_with_bounds);

    // Get type generics without the added lifetime
    let SplitGenerics { type_generics, .. } = SplitGenerics::new(generics);

    // Example for `struct Wrapper<T>(T) where T: Clone`:
    //
    // impl<'nu_arb, T: Arbitrary<'nu_arb>> Arbitrary<'nu_arb> for Wrapper<T> where T: Clone {
    //     fn arbitrary(u: &mut Unstructured<'nu_arb>) -> Result<Self> { ... }
    // }
    Ok(quote!(
        impl #impl_generics ::arbitrary::Arbitrary<'nu_arb> for #type_name #type_generics #where_clause {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'nu_arb>) -> ::arbitrary::Result<Self> {
                let inner_value: #inner_type = u.arbitrary()?;
                Ok(#type_name::new(inner_value))
            }

            #[inline]
            fn size_hint(_depth: usize) -> (usize, Option<usize>) {
                let n = ::core::mem::size_of::<#inner_type>();
                (n, Some(n))
            }
        }
    ))
}
