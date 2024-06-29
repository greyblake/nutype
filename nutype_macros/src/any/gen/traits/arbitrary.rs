use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Generics;

use crate::{
    any::models::{AnyGuard, AnyInnerType},
    common::gen::{add_bound_to_all_type_params, add_param, strip_trait_bounds_on_generics},
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
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let generics_with_lifetime = add_param(&generics_without_bounds, quote!('nu_arb));
    let generics_with_bounds = add_bound_to_all_type_params(
        &generics_with_lifetime,
        quote!(::arbitrary::Arbitrary<'nu_arb>),
    );
    Ok(quote!(
        impl #generics_with_bounds ::arbitrary::Arbitrary<'nu_arb> for #type_name #generics_without_bounds {
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
