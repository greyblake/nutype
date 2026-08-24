mod arbitrary;

use alloc::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Generics;

use crate::{
    common::{
        generate::traits::{
            ConditionalTraits, GeneratableTrait, GeneratableTraits, GeneratedTraits,
            HasGeneratedParseError, gen_impl_trait_as_ref, gen_impl_trait_borrow,
            gen_impl_trait_default, gen_impl_trait_deref, gen_impl_trait_display,
            gen_impl_trait_from, gen_impl_trait_from_str, gen_impl_trait_into,
            gen_impl_trait_serde_deserialize, gen_impl_trait_serde_serialize,
            gen_impl_trait_try_from, process_conditional_derives, split_into_generatable_traits,
        },
        models::{ConditionalDeriveGroup, SerdeCustomization, SpannedDeriveUnsafeTrait, TypeName},
    },
    decimal::models::{DecimalDeriveTrait, DecimalGuard, DecimalInnerType},
};

type DecimalGeneratableTrait = GeneratableTrait<DecimalTransparentTrait, DecimalIrregularTrait>;

#[allow(clippy::too_many_arguments)]
pub fn gen_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &DecimalInnerType,
    traits: BTreeSet<DecimalDeriveTrait>,
    unsafe_traits: &[SpannedDeriveUnsafeTrait],
    maybe_default_value: Option<syn::Expr>,
    guard: &DecimalGuard<T>,
    conditional_derives: &[ConditionalDeriveGroup<DecimalDeriveTrait>],
    serde_customization: &SerdeCustomization,
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
        maybe_default_value.clone(),
        guard,
        serde_customization,
    )?;

    let ConditionalTraits {
        derive_transparent_traits: conditional_derive_transparent_traits,
        implement_traits: conditional_implement_traits,
        from_str_parse_errors: conditional_from_str_parse_errors,
    } = process_conditional_derives(conditional_derives, type_name, |irregular| {
        gen_implemented_traits(
            type_name,
            generics,
            inner_type,
            irregular,
            maybe_default_value.clone(),
            guard,
            serde_customization,
        )
    })?;

    Ok(GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
        conditional_derive_transparent_traits,
        conditional_implement_traits,
        conditional_from_str_parse_errors,
    })
}

impl From<DecimalDeriveTrait> for DecimalGeneratableTrait {
    fn from(derive_trait: DecimalDeriveTrait) -> DecimalGeneratableTrait {
        match derive_trait {
            DecimalDeriveTrait::Debug => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Debug)
            }
            DecimalDeriveTrait::Clone => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Clone)
            }
            DecimalDeriveTrait::Copy => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Copy)
            }
            DecimalDeriveTrait::PartialEq => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::PartialEq)
            }
            DecimalDeriveTrait::Eq => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Eq)
            }
            DecimalDeriveTrait::PartialOrd => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::PartialOrd)
            }
            DecimalDeriveTrait::Ord => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Ord)
            }
            DecimalDeriveTrait::Hash => {
                DecimalGeneratableTrait::Transparent(DecimalTransparentTrait::Hash)
            }
            DecimalDeriveTrait::FromStr => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::FromStr)
            }
            DecimalDeriveTrait::AsRef => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::AsRef)
            }
            DecimalDeriveTrait::Deref => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::Deref)
            }
            DecimalDeriveTrait::Into => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::Into)
            }
            DecimalDeriveTrait::From => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::From)
            }
            DecimalDeriveTrait::TryFrom => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::TryFrom)
            }
            DecimalDeriveTrait::Borrow => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::Borrow)
            }
            DecimalDeriveTrait::Display => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::Display)
            }
            DecimalDeriveTrait::Default => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::Default)
            }
            DecimalDeriveTrait::SerdeSerialize => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::SerdeSerialize)
            }
            DecimalDeriveTrait::SerdeDeserialize => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::SerdeDeserialize)
            }
            DecimalDeriveTrait::ArbitraryArbitrary => {
                DecimalGeneratableTrait::Irregular(DecimalIrregularTrait::ArbitraryArbitrary)
            }
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DecimalTransparentTrait {
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
enum DecimalIrregularTrait {
    FromStr,
    AsRef,
    Deref,
    From,
    TryFrom,
    Borrow,
    Into,
    Display,
    Default,
    SerdeSerialize,
    SerdeDeserialize,
    ArbitraryArbitrary,
}

/// Decimal's `FromStr` generates a `ParseError` type via `gen_impl_trait_from_str` ->
/// `gen_def_parse_error`, which needs module-level re-export in conditional derives.
impl HasGeneratedParseError for DecimalIrregularTrait {
    fn has_generated_parse_error(&self) -> bool {
        matches!(self, Self::FromStr)
    }
}

impl ToTokens for DecimalTransparentTrait {
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

#[allow(clippy::too_many_arguments)]
fn gen_implemented_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &DecimalInnerType,
    impl_traits: Vec<DecimalIrregularTrait>,
    maybe_default_value: Option<syn::Expr>,
    guard: &DecimalGuard<T>,
    serde_customization: &SerdeCustomization,
) -> Result<TokenStream, syn::Error> {
    let maybe_error_type_name = guard.maybe_error_type_path();
    impl_traits
        .iter()
        .map(|t| match t {
            DecimalIrregularTrait::AsRef => Ok(gen_impl_trait_as_ref(type_name, generics, inner_type)),
            DecimalIrregularTrait::Deref => Ok(gen_impl_trait_deref(type_name, generics, inner_type)),
            DecimalIrregularTrait::FromStr => {
                Ok(gen_impl_trait_from_str(type_name, generics, inner_type, maybe_error_type_name))
            }
            DecimalIrregularTrait::From => Ok(gen_impl_trait_from(type_name, generics, inner_type)),
            DecimalIrregularTrait::Into => Ok(gen_impl_trait_into(type_name, generics, inner_type)),
            DecimalIrregularTrait::TryFrom => {
                Ok(gen_impl_trait_try_from(type_name, generics, inner_type, maybe_error_type_name))
            }
            DecimalIrregularTrait::Borrow => Ok(gen_impl_trait_borrow(type_name, generics, inner_type)),
            DecimalIrregularTrait::Display => Ok(gen_impl_trait_display(type_name, generics)),
            DecimalIrregularTrait::Default => {
                match maybe_default_value {
                    Some(ref default_value) => {
                        let has_validation = maybe_error_type_name.is_some();
                        Ok(gen_impl_trait_default(type_name, generics, default_value, has_validation))
                    },
                    None => {
                        let span = proc_macro2::Span::call_site();
                        let msg = format!("Trait `Default` is derived for type {type_name}, but `default = ` parameter is missing in #[nutype] macro");
                        Err(syn::Error::new(span, msg))
                    }
                }
            }
            DecimalIrregularTrait::SerdeSerialize => Ok(gen_impl_trait_serde_serialize(
                type_name,
                generics,
                inner_type,
                serde_customization,
            )),
            DecimalIrregularTrait::SerdeDeserialize => Ok(gen_impl_trait_serde_deserialize(
                type_name,
                generics,
                inner_type,
                maybe_error_type_name,
                serde_customization,
            )),
            DecimalIrregularTrait::ArbitraryArbitrary => {
                arbitrary::gen_impl_trait_arbitrary(type_name, inner_type, guard)
            }
        })
        .collect()
}
