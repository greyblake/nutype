mod arbitrary;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Generics;

use crate::{
    common::{
        generate::{
            parse_error::gen_parse_error_name,
            traits::{
                GeneratableTrait, GeneratableTraits, GeneratedTraits, gen_impl_trait_as_ref,
                gen_impl_trait_borrow, gen_impl_trait_default, gen_impl_trait_deref,
                gen_impl_trait_display, gen_impl_trait_from, gen_impl_trait_from_str,
                gen_impl_trait_into, gen_impl_trait_serde_deserialize,
                gen_impl_trait_serde_serialize, gen_impl_trait_try_from,
                split_into_generatable_traits,
            },
        },
        models::{ConditionalDeriveGroup, ParseErrorTypeName, SpannedDeriveUnsafeTrait, TypeName},
    },
    integer::models::{IntegerDeriveTrait, IntegerGuard, IntegerInnerType},
};

type IntegerGeneratableTrait = GeneratableTrait<IntegerTransparentTrait, IntegerIrregularTrait>;

#[allow(clippy::too_many_arguments)]
pub fn gen_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &IntegerInnerType,
    traits: HashSet<IntegerDeriveTrait>,
    unsafe_traits: &[SpannedDeriveUnsafeTrait],
    maybe_default_value: Option<syn::Expr>,
    guard: &IntegerGuard<T>,
    conditional_derives: &[ConditionalDeriveGroup<IntegerDeriveTrait>],
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
    )?;

    let mut conditional_derive_transparent_traits = TokenStream::new();
    let mut conditional_implement_traits = TokenStream::new();
    let mut conditional_from_str_parse_errors: Vec<(TokenStream, ParseErrorTypeName)> = vec![];

    for group in conditional_derives {
        let pred = &group.predicate;

        let cond_traits: HashSet<IntegerDeriveTrait> = group.typed_traits.iter().cloned().collect();
        let GeneratableTraits {
            transparent_traits: cond_transparent,
            irregular_traits: cond_irregular,
        } = split_into_generatable_traits(cond_traits);

        let cond_unchecked = &group.unchecked_traits;
        if !cond_transparent.is_empty() || !cond_unchecked.is_empty() {
            conditional_derive_transparent_traits.extend(quote! {
                #[cfg_attr(#pred, derive(
                    #(#cond_transparent,)*
                    #(#cond_unchecked,)*
                ))]
            });
        }

        if !cond_irregular.is_empty() {
            let has_from_str = cond_irregular
                .iter()
                .any(|t| matches!(t, IntegerIrregularTrait::FromStr));

            let impl_tokens = gen_implemented_traits(
                type_name,
                generics,
                inner_type,
                cond_irregular,
                maybe_default_value.clone(),
                guard,
            )?;

            if has_from_str {
                // When FromStr is conditional, use a module wrapper so ParseError
                // is accessible for re-export (not trapped inside const block).
                let fromstr_mod_name = quote::format_ident!("__fromstr_impl__");
                let parse_error_name = gen_parse_error_name(type_name);
                conditional_implement_traits.extend(quote! {
                    #[cfg(#pred)]
                    mod #fromstr_mod_name {
                        use super::*;
                        #impl_tokens
                    }
                    #[cfg(#pred)]
                    pub use #fromstr_mod_name::#parse_error_name;
                });
                conditional_from_str_parse_errors.push((pred.clone(), parse_error_name));
            } else {
                conditional_implement_traits.extend(quote! {
                    #[cfg(#pred)]
                    const _: () = {
                        #impl_tokens
                    };
                });
            }
        }
    }

    Ok(GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
        conditional_derive_transparent_traits,
        conditional_implement_traits,
        conditional_from_str_parse_errors,
    })
}

impl From<IntegerDeriveTrait> for IntegerGeneratableTrait {
    fn from(derive_trait: IntegerDeriveTrait) -> IntegerGeneratableTrait {
        match derive_trait {
            IntegerDeriveTrait::Debug => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Debug)
            }
            IntegerDeriveTrait::Clone => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Clone)
            }
            IntegerDeriveTrait::Copy => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Copy)
            }
            IntegerDeriveTrait::PartialEq => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::PartialEq)
            }
            IntegerDeriveTrait::Eq => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Eq)
            }
            IntegerDeriveTrait::PartialOrd => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::PartialOrd)
            }
            IntegerDeriveTrait::Ord => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Ord)
            }
            IntegerDeriveTrait::Hash => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::Hash)
            }
            IntegerDeriveTrait::FromStr => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::FromStr)
            }
            IntegerDeriveTrait::AsRef => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::AsRef)
            }
            IntegerDeriveTrait::Deref => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::Deref)
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
            IntegerDeriveTrait::Default => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::Default)
            }
            IntegerDeriveTrait::SerdeSerialize => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::SerdeSerialize)
            }
            IntegerDeriveTrait::SerdeDeserialize => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::SerdeDeserialize)
            }
            IntegerDeriveTrait::SchemarsJsonSchema => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::SchemarsJsonSchema)
            }
            IntegerDeriveTrait::ArbitraryArbitrary => {
                IntegerGeneratableTrait::Irregular(IntegerIrregularTrait::ArbitraryArbitrary)
            }
            IntegerDeriveTrait::ValuableValuable => {
                IntegerGeneratableTrait::Transparent(IntegerTransparentTrait::ValuableValuable)
            }
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum IntegerTransparentTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    SchemarsJsonSchema,
    ValuableValuable,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum IntegerIrregularTrait {
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

impl ToTokens for IntegerTransparentTrait {
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
            Self::SchemarsJsonSchema => quote!(::schemars::JsonSchema),
            Self::ValuableValuable => quote!(::valuable::Valuable),
        };
        tokens.to_tokens(token_stream)
    }
}

fn gen_implemented_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &IntegerInnerType,
    impl_traits: Vec<IntegerIrregularTrait>,
    maybe_default_value: Option<syn::Expr>,
    guard: &IntegerGuard<T>,
) -> Result<TokenStream, syn::Error> {
    let maybe_error_type_name = guard.maybe_error_type_path();
    impl_traits
        .iter()
        .map(|t| match t {
            IntegerIrregularTrait::AsRef => Ok(gen_impl_trait_as_ref(type_name, generics, inner_type)),
            IntegerIrregularTrait::Deref => Ok(gen_impl_trait_deref(type_name, generics, inner_type)),
            IntegerIrregularTrait::FromStr => {
                Ok(gen_impl_trait_from_str(type_name, generics, inner_type, maybe_error_type_name))
            }
            IntegerIrregularTrait::From => Ok(gen_impl_trait_from(type_name, generics, inner_type)),
            IntegerIrregularTrait::Into => Ok(gen_impl_trait_into(type_name, generics, inner_type)),
            IntegerIrregularTrait::TryFrom => {
                Ok(gen_impl_trait_try_from(type_name, generics, inner_type, maybe_error_type_name))
            }
            IntegerIrregularTrait::Borrow => Ok(gen_impl_trait_borrow(type_name, generics, inner_type)),
            IntegerIrregularTrait::Display => Ok(gen_impl_trait_display(type_name, generics)),
            IntegerIrregularTrait::Default => {
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
            IntegerIrregularTrait::SerdeSerialize => Ok(gen_impl_trait_serde_serialize(type_name, generics)),
            IntegerIrregularTrait::SerdeDeserialize => Ok(gen_impl_trait_serde_deserialize(
                type_name,
                generics,
                inner_type,
                maybe_error_type_name,
            )),
            IntegerIrregularTrait::ArbitraryArbitrary => {
                arbitrary::gen_impl_trait_arbitrary(type_name, inner_type, guard)
            }
        })
        .collect()
}
