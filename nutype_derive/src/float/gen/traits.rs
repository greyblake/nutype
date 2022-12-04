use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{
        gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_from, gen_impl_trait_from_str,
        gen_impl_trait_into, gen_impl_trait_try_from,
    },
    float::models::FloatDeriveTrait,
};

// TODO: this can be a shared structure among all the types
pub struct GeneratedTraits {
    pub derive_standard_traits: TokenStream,
    pub implement_traits: TokenStream,
}

pub fn gen_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<FloatDeriveTrait>,
) -> GeneratedTraits {
    let (standard_traits, impl_traits) = split_traits(traits);

    let derive_standard_traits = quote! {
        #[derive(
            #(#standard_traits,)*
        )]
    };

    let implement_traits =
        gen_implemented_traits(type_name, inner_type, maybe_error_type_name, impl_traits);

    GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    }
}

// TODO: this can be shared generic enum for all the types
enum Trait {
    Derived(DerivedTrait),
    Implemented(ImplementedTrait),
}

impl From<FloatDeriveTrait> for Trait {
    fn from(derive_trait: FloatDeriveTrait) -> Trait {
        match derive_trait {
            FloatDeriveTrait::Debug => Trait::Derived(DerivedTrait::Debug),
            FloatDeriveTrait::Clone => Trait::Derived(DerivedTrait::Clone),
            FloatDeriveTrait::Copy => Trait::Derived(DerivedTrait::Copy),
            FloatDeriveTrait::PartialEq => Trait::Derived(DerivedTrait::PartialEq),
            FloatDeriveTrait::PartialOrd => Trait::Derived(DerivedTrait::PartialOrd),
            FloatDeriveTrait::FromStr => Trait::Implemented(ImplementedTrait::FromStr),
            FloatDeriveTrait::AsRef => Trait::Implemented(ImplementedTrait::AsRef),
            FloatDeriveTrait::From => Trait::Implemented(ImplementedTrait::From),
            FloatDeriveTrait::Into => Trait::Implemented(ImplementedTrait::Into),
            FloatDeriveTrait::TryFrom => Trait::Implemented(ImplementedTrait::TryFrom),
            FloatDeriveTrait::Borrow => Trait::Implemented(ImplementedTrait::Borrow),
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DerivedTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum ImplementedTrait {
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Borrow,
}

impl ToTokens for DerivedTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
            Self::Copy => quote!(Copy),
            Self::PartialEq => quote!(PartialEq),
            Self::PartialOrd => quote!(PartialOrd),
        };
        tokens.to_tokens(token_stream)
    }
}

fn split_traits(
    input_traits: HashSet<FloatDeriveTrait>,
) -> (Vec<DerivedTrait>, Vec<ImplementedTrait>) {
    let mut derive_traits: Vec<DerivedTrait> = Vec::with_capacity(24);
    let mut impl_traits: Vec<ImplementedTrait> = Vec::with_capacity(24);

    for input_trait in input_traits {
        match Trait::from(input_trait) {
            Trait::Derived(dt) => derive_traits.push(dt),
            Trait::Implemented(it) => impl_traits.push(it),
        };
    }

    (derive_traits, impl_traits)
}

fn gen_implemented_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<ImplementedTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            ImplementedTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            ImplementedTrait::FromStr => {
                gen_impl_trait_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::From => gen_impl_trait_from(type_name, inner_type),
            ImplementedTrait::Into => gen_impl_trait_into(type_name, inner_type),
            ImplementedTrait::TryFrom => {
                let error_type_name = maybe_error_type_name
                    .as_ref()
                    .expect("TryFrom for float is expected to have error_type_name");
                gen_impl_trait_try_from(type_name, inner_type, error_type_name)
            }
            ImplementedTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
        })
        .collect()
}
