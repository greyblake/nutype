use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{
        gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_dislpay, gen_impl_trait_from,
        gen_impl_trait_from_str, gen_impl_trait_into, gen_impl_trait_try_from,
        split_into_generatable_traits, GeneratableTrait, GeneratableTraits, GeneratedTraits,
    },
    float::models::FloatDeriveTrait,
    models::FloatType,
};

type FloatGeneratableTrait = GeneratableTrait<FloatStandardTrait, FloatIrregularTrait>;

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FloatStandardTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FloatIrregularTrait {
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Borrow,
    Display,
}

impl From<FloatDeriveTrait> for FloatGeneratableTrait {
    fn from(derive_trait: FloatDeriveTrait) -> FloatGeneratableTrait {
        match derive_trait {
            FloatDeriveTrait::Debug => FloatGeneratableTrait::Standard(FloatStandardTrait::Debug),
            FloatDeriveTrait::Clone => FloatGeneratableTrait::Standard(FloatStandardTrait::Clone),
            FloatDeriveTrait::Copy => FloatGeneratableTrait::Standard(FloatStandardTrait::Copy),
            FloatDeriveTrait::PartialEq => {
                FloatGeneratableTrait::Standard(FloatStandardTrait::PartialEq)
            }
            FloatDeriveTrait::PartialOrd => {
                FloatGeneratableTrait::Standard(FloatStandardTrait::PartialOrd)
            }
            FloatDeriveTrait::FromStr => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::FromStr)
            }
            FloatDeriveTrait::AsRef => FloatGeneratableTrait::Irregular(FloatIrregularTrait::AsRef),
            FloatDeriveTrait::From => FloatGeneratableTrait::Irregular(FloatIrregularTrait::From),
            FloatDeriveTrait::Into => FloatGeneratableTrait::Irregular(FloatIrregularTrait::Into),
            FloatDeriveTrait::TryFrom => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::TryFrom)
            }
            FloatDeriveTrait::Borrow => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::Borrow)
            }
            FloatDeriveTrait::Display => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::Display)
            }
        }
    }
}

impl ToTokens for FloatStandardTrait {
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

pub fn gen_traits(
    type_name: &Ident,
    inner_type: FloatType,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<FloatDeriveTrait>,
) -> GeneratedTraits {
    let GeneratableTraits {
        standard_traits,
        irregular_traits,
    } = split_into_generatable_traits(traits);

    let derive_standard_traits = quote! {
        #[derive(
            #(#standard_traits,)*
        )]
    };

    let implement_traits = gen_implemented_traits(
        type_name,
        inner_type,
        maybe_error_type_name,
        irregular_traits,
    );

    GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    }
}

fn gen_implemented_traits(
    type_name: &Ident,
    inner_type: FloatType,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<FloatIrregularTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            FloatIrregularTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            FloatIrregularTrait::FromStr => {
                gen_impl_trait_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            FloatIrregularTrait::From => gen_impl_trait_from(type_name, inner_type),
            FloatIrregularTrait::Into => gen_impl_trait_into(type_name, inner_type),
            FloatIrregularTrait::TryFrom => {
                let error_type_name = maybe_error_type_name
                    .as_ref()
                    .expect("TryFrom for float is expected to have error_type_name");
                gen_impl_trait_try_from(type_name, inner_type, error_type_name)
            }
            FloatIrregularTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
            FloatIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
        })
        .collect()
}
