pub mod arbitrary;
pub mod into_iter;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashSet;

use crate::{
    any::models::{AnyDeriveTrait, AnyGuard, AnyInnerType},
    common::{
        generate::traits::{
            GeneratableTrait, GeneratableTraits, GeneratedTraits, gen_impl_trait_as_ref,
            gen_impl_trait_borrow, gen_impl_trait_default, gen_impl_trait_deref,
            gen_impl_trait_display, gen_impl_trait_from, gen_impl_trait_from_str,
            gen_impl_trait_into, gen_impl_trait_serde_deserialize, gen_impl_trait_serde_serialize,
            gen_impl_trait_try_from, split_into_generatable_traits,
        },
        models::{SpannedDeriveUnsafeTrait, TypeName},
    },
};

type AnyGeneratableTrait = GeneratableTrait<AnyTransparentTrait, AnyIrregularTrait>;

impl From<AnyDeriveTrait> for AnyGeneratableTrait {
    fn from(derive_trait: AnyDeriveTrait) -> AnyGeneratableTrait {
        match derive_trait {
            AnyDeriveTrait::Debug => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Debug),
            AnyDeriveTrait::Clone => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Clone),
            AnyDeriveTrait::Copy => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Copy),
            AnyDeriveTrait::Hash => AnyGeneratableTrait::Transparent(AnyTransparentTrait::Hash),
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
            AnyDeriveTrait::Deref => AnyGeneratableTrait::Irregular(AnyIrregularTrait::Deref),
            AnyDeriveTrait::Borrow => AnyGeneratableTrait::Irregular(AnyIrregularTrait::Borrow),
            AnyDeriveTrait::FromStr => AnyGeneratableTrait::Irregular(AnyIrregularTrait::FromStr),
            AnyDeriveTrait::TryFrom => AnyGeneratableTrait::Irregular(AnyIrregularTrait::TryFrom),
            AnyDeriveTrait::Default => AnyGeneratableTrait::Irregular(AnyIrregularTrait::Default),
            AnyDeriveTrait::IntoIterator => {
                AnyGeneratableTrait::Irregular(AnyIrregularTrait::IntoIterator)
            }
            AnyDeriveTrait::SerdeSerialize => {
                AnyGeneratableTrait::Irregular(AnyIrregularTrait::SerdeSerialize)
            }
            AnyDeriveTrait::SerdeDeserialize => {
                AnyGeneratableTrait::Irregular(AnyIrregularTrait::SerdeDeserialize)
            }
            AnyDeriveTrait::ArbitraryArbitrary => {
                AnyGeneratableTrait::Irregular(AnyIrregularTrait::ArbitraryArbitrary)
            }
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
    Hash,
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
            Self::Hash => quote!(Hash),
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
    Deref,
    Borrow,
    FromStr,
    TryFrom,
    Default,
    IntoIterator,
    SerdeSerialize,
    SerdeDeserialize,
    ArbitraryArbitrary,
}

pub fn gen_traits(
    type_name: &TypeName,
    generics: &syn::Generics,
    inner_type: &AnyInnerType,
    traits: HashSet<AnyDeriveTrait>,
    unsafe_traits: &[SpannedDeriveUnsafeTrait],
    maybe_default_value: Option<syn::Expr>,
    guard: &AnyGuard,
) -> Result<GeneratedTraits, syn::Error> {
    let GeneratableTraits {
        transparent_traits,
        irregular_traits,
    } = split_into_generatable_traits(traits);

    let derive_transparent_traits = quote! {
        #[derive(
            #(#transparent_traits,)*
            #(#unsafe_traits,)*
        )]
    };

    let implement_traits = gen_implemented_traits(
        type_name,
        generics,
        inner_type,
        irregular_traits,
        maybe_default_value,
        guard,
    )?;

    Ok(GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
    })
}

fn gen_implemented_traits(
    type_name: &TypeName,
    generics: &syn::Generics,
    inner_type: &AnyInnerType,
    impl_traits: Vec<AnyIrregularTrait>,
    maybe_default_value: Option<syn::Expr>,
    guard: &AnyGuard,
) -> Result<TokenStream, syn::Error> {
    let maybe_error_type_name = guard.maybe_error_type_path();
    impl_traits
        .iter()
        .map(|t| match t {
            AnyIrregularTrait::AsRef => Ok(gen_impl_trait_as_ref(type_name, generics, inner_type)),
            AnyIrregularTrait::From => Ok(gen_impl_trait_from(type_name, generics, inner_type)),
            AnyIrregularTrait::Into => Ok(gen_impl_trait_into(type_name, generics, inner_type.clone())),
            AnyIrregularTrait::Display => Ok(gen_impl_trait_display(type_name, generics)),
            AnyIrregularTrait::Deref => Ok(gen_impl_trait_deref(type_name, generics, inner_type)),
            AnyIrregularTrait::Borrow => Ok(gen_impl_trait_borrow(type_name, generics, inner_type)),
            AnyIrregularTrait::FromStr => Ok(
                gen_impl_trait_from_str(type_name, generics, inner_type, maybe_error_type_name)
            ),
            AnyIrregularTrait::TryFrom => Ok(
                gen_impl_trait_try_from(type_name, generics, inner_type, maybe_error_type_name)
            ),
            AnyIrregularTrait::Default => match maybe_default_value {
                Some(ref default_value) => {
                    let has_validation = maybe_error_type_name.is_some();
                    Ok(gen_impl_trait_default(type_name, generics, default_value, has_validation))
                }
                None => {
                    let span = proc_macro2::Span::call_site();
                    let msg = format!("Trait `Default` is derived for type {type_name}, but `default = ` parameter is missing in #[nutype] macro");
                    Err(syn::Error::new(span, msg))
                }
            },
            AnyIrregularTrait::IntoIterator => {
                Ok(into_iter::gen_impl_trait_into_iter(type_name, generics, inner_type))
            }
            AnyIrregularTrait::SerdeSerialize => Ok(
                gen_impl_trait_serde_serialize(type_name, generics)
            ),
            AnyIrregularTrait::SerdeDeserialize => Ok(
                gen_impl_trait_serde_deserialize(type_name, generics, inner_type, maybe_error_type_name)
            ),
            AnyIrregularTrait::ArbitraryArbitrary => arbitrary::gen_impl_trait_arbitrary(type_name, generics, inner_type, guard),
        })
        .collect()
}
