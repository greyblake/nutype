use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{
        gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_dislpay, gen_impl_trait_from,
        gen_impl_trait_from_str, gen_impl_trait_into, gen_impl_trait_try_from, GeneratableTrait,
        GeneratedTraits,
    },
    integer::models::IntegerDeriveTrait,
};

type IntegerGeneratableTrait = GeneratableTrait<IntegerStandardTrait, IntegerIrregularTrait>;

pub fn gen_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<IntegerDeriveTrait>,
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

impl From<IntegerDeriveTrait> for IntegerGeneratableTrait {
    fn from(derive_trait: IntegerDeriveTrait) -> IntegerGeneratableTrait {
        match derive_trait {
            IntegerDeriveTrait::Debug => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::Debug)
            }
            IntegerDeriveTrait::Clone => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::Clone)
            }
            IntegerDeriveTrait::Copy => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::Copy)
            }
            IntegerDeriveTrait::PartialEq => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::PartialEq)
            }
            IntegerDeriveTrait::Eq => IntegerGeneratableTrait::Standard(IntegerStandardTrait::Eq),
            IntegerDeriveTrait::PartialOrd => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::PartialOrd)
            }
            IntegerDeriveTrait::Ord => IntegerGeneratableTrait::Standard(IntegerStandardTrait::Ord),
            IntegerDeriveTrait::Hash => {
                IntegerGeneratableTrait::Standard(IntegerStandardTrait::Hash)
            }
            IntegerDeriveTrait::FromStr => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::FromStr)
            }
            IntegerDeriveTrait::AsRef => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::AsRef)
            }
            IntegerDeriveTrait::Into => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::Into)
            }
            IntegerDeriveTrait::From => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::From)
            }
            IntegerDeriveTrait::TryFrom => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::TryFrom)
            }
            IntegerDeriveTrait::Borrow => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::Borrow)
            }
            IntegerDeriveTrait::Display => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::Display)
            }
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum IntegerStandardTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum IntegerIrregularTrait {
    FromStr,
    AsRef,
    From,
    TryFrom,
    Borrow,
    Into,
    Display,
}

impl ToTokens for IntegerStandardTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
            Self::Copy => quote!(Copy),
            Self::PartialEq => quote!(PartialEq),
            Self::Eq => quote!(Eq),
            Self::PartialOrd => quote!(PartialOrd),
            Self::Ord => quote!(Ord),
            Self::Hash => quote!(Hash),
        };
        tokens.to_tokens(token_stream)
    }
}

// TODO: Reuse over different types
fn split_traits(
    input_traits: HashSet<IntegerDeriveTrait>,
) -> (Vec<IntegerStandardTrait>, Vec<IntegerIrregularTrait>) {
    let mut derive_traits: Vec<IntegerStandardTrait> = Vec::with_capacity(24);
    let mut impl_traits: Vec<IntegerIrregularTrait> = Vec::with_capacity(24);

    for input_trait in input_traits {
        match IntegerGeneratableTrait::from(input_trait) {
            IntegerGeneratableTrait::Standard(dt) => derive_traits.push(dt),
            IntegerGeneratableTrait::Irregular(it) => impl_traits.push(it),
        };
    }

    (derive_traits, impl_traits)
}

fn gen_implemented_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<IntegerIrregularTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            IntegerIrregularTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            IntegerIrregularTrait::FromStr => {
                gen_impl_trait_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            IntegerIrregularTrait::From => gen_impl_trait_from(type_name, inner_type),
            IntegerIrregularTrait::Into => gen_impl_trait_into(type_name, inner_type),
            IntegerIrregularTrait::TryFrom => {
                let error_type_name = maybe_error_type_name
                    .as_ref()
                    .expect("TryFrom for integer is expected to have error_type_name");
                gen_impl_trait_try_from(type_name, inner_type, error_type_name)
            }
            IntegerIrregularTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
            IntegerIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
        })
        .collect()
}
