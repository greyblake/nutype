use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;

use crate::{
    any::models::AnyDeriveTrait,
    any::models::AnyInnerType,
    common::{
        gen::traits::{
            gen_impl_trait_as_ref, gen_impl_trait_dislpay, gen_impl_trait_from,
            gen_impl_trait_into, split_into_generatable_traits, GeneratableTrait,
            GeneratableTraits, GeneratedTraits,
        },
        models::{ErrorTypeName, TypeName},
    },
};

type AnyGeneratableTrait = GeneratableTrait<AnyTransparentTrait, AnyIrregularTrait>;

impl From<AnyDeriveTrait> for AnyGeneratableTrait {
    fn from(derive_trait: AnyDeriveTrait) -> AnyGeneratableTrait {
        match derive_trait {
            AnyDeriveTrait::Debug => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Debug),
            AnyDeriveTrait::Clone => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Clone),
            AnyDeriveTrait::Copy => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Copy),
            AnyDeriveTrait::PartialEq => {
                AnyGeneratableTrait::Transparent(AnyTransparentTrait::PartialEq)
            }
            AnyDeriveTrait::Eq => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Eq),
            AnyDeriveTrait::PartialOrd => {
                AnyGeneratableTrait::Transparent(AnyTransparentTrait::PartialOrd)
            }
            AnyDeriveTrait::Ord => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Ord),
            AnyDeriveTrait::AsRef => AnyGeneratableTrait::Irregular(AnyIrregularTrait::AsRef),
            AnyDeriveTrait::From => AnyGeneratableTrait::Irregular(AnyIrregularTrait::From),
            AnyDeriveTrait::Into => AnyGeneratableTrait::Irregular(AnyIrregularTrait::Into),
            AnyDeriveTrait::Display => AnyGeneratableTrait::Irregular(AnyIrregularTrait::Display),
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum AnyTransparentTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
}

impl ToTokens for AnyTransparentTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
            Self::Copy => quote!(Copy),
            Self::PartialEq => quote!(PartialEq),
            Self::Eq => quote!(Eq),
            Self::PartialOrd => quote!(PartialOrd),
            Self::Ord => quote!(Ord),
        };
        tokens.to_tokens(token_stream)
    }
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum AnyIrregularTrait {
    AsRef,
    From,
    Into,
    Display,
}

pub fn gen_traits(
    type_name: &TypeName,
    inner_type: &AnyInnerType,
    maybe_error_type_name: Option<ErrorTypeName>,
    traits: HashSet<AnyDeriveTrait>,
    maybe_default_value: Option<syn::Expr>,
) -> GeneratedTraits {
    let GeneratableTraits {
        transparent_traits,
        irregular_traits,
    } = split_into_generatable_traits(traits);

    let derive_transparent_traits = quote! {
        #[derive(
            #(#transparent_traits,)*
        )]
    };

    let implement_traits = gen_implemented_traits(
        type_name,
        inner_type,
        maybe_error_type_name,
        irregular_traits,
        maybe_default_value,
    );

    GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
    }
}

fn gen_implemented_traits(
    type_name: &TypeName,
    inner_type: &AnyInnerType,
    _maybe_error_type_name: Option<ErrorTypeName>,
    impl_traits: Vec<AnyIrregularTrait>,
    _maybe_default_value: Option<syn::Expr>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            AnyIrregularTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            AnyIrregularTrait::From => gen_impl_trait_from(type_name, inner_type),
            AnyIrregularTrait::Into => gen_impl_trait_into(type_name, inner_type.clone()),
            AnyIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
        })
        .collect()
}
