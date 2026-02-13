pub mod arbitrary;
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
    float::models::{FloatDeriveTrait, FloatGuard, FloatInnerType},
};

type FloatGeneratableTrait = GeneratableTrait<FloatTransparentTrait, FloatIrregularTrait>;

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FloatTransparentTrait {
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    SchemarsJsonSchema,
    ValuableValuable,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FloatIrregularTrait {
    FromStr,
    AsRef,
    Deref,
    Into,
    From,
    Eq,
    Ord,
    TryFrom,
    Borrow,
    Display,
    Default,
    SerdeSerialize,
    SerdeDeserialize,
    ArbitraryArbitrary,
}

impl From<FloatDeriveTrait> for FloatGeneratableTrait {
    fn from(derive_trait: FloatDeriveTrait) -> FloatGeneratableTrait {
        match derive_trait {
            FloatDeriveTrait::Debug => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::Debug)
            }
            FloatDeriveTrait::Clone => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::Clone)
            }
            FloatDeriveTrait::Copy => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::Copy)
            }
            FloatDeriveTrait::PartialEq => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::PartialEq)
            }
            FloatDeriveTrait::Eq => FloatGeneratableTrait::Irregular(FloatIrregularTrait::Eq),
            FloatDeriveTrait::PartialOrd => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::PartialOrd)
            }
            FloatDeriveTrait::Ord => FloatGeneratableTrait::Irregular(FloatIrregularTrait::Ord),
            FloatDeriveTrait::FromStr => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::FromStr)
            }
            FloatDeriveTrait::AsRef => FloatGeneratableTrait::Irregular(FloatIrregularTrait::AsRef),
            FloatDeriveTrait::Deref => FloatGeneratableTrait::Irregular(FloatIrregularTrait::Deref),
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
            FloatDeriveTrait::Default => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::Default)
            }
            FloatDeriveTrait::SerdeSerialize => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::SerdeSerialize)
            }
            FloatDeriveTrait::SerdeDeserialize => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::SerdeDeserialize)
            }
            FloatDeriveTrait::ArbitraryArbitrary => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::ArbitraryArbitrary)
            }
            FloatDeriveTrait::SchemarsJsonSchema => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::SchemarsJsonSchema)
            }
            FloatDeriveTrait::ValuableValuable => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::ValuableValuable)
            }
        }
    }
}

impl ToTokens for FloatTransparentTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
            Self::Copy => quote!(Copy),
            Self::PartialEq => quote!(PartialEq),
            Self::PartialOrd => quote!(PartialOrd),
            Self::SchemarsJsonSchema => quote!(::schemars::JsonSchema),
            Self::ValuableValuable => quote!(::valuable::Valuable),
        };
        tokens.to_tokens(token_stream)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn gen_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &FloatInnerType,
    maybe_default_value: Option<syn::Expr>,
    traits: HashSet<FloatDeriveTrait>,
    unsafe_traits: &[SpannedDeriveUnsafeTrait],
    guard: &FloatGuard<T>,
    conditional_derives: &[ConditionalDeriveGroup<FloatDeriveTrait>],
) -> Result<GeneratedTraits, syn::Error> {
    let GeneratableTraits {
        transparent_traits,
        irregular_traits,
    } = split_into_generatable_traits(traits);

    // When PartialOrd is derived but Ord is manually implemented, clippy warns about it.
    // We need to allow this lint on the struct because nutype intentionally implements Ord
    // manually for floats with `finite` validation.
    let has_derived_partial_ord = transparent_traits
        .iter()
        .any(|t| matches!(t, FloatTransparentTrait::PartialOrd));
    let has_manual_ord = irregular_traits
        .iter()
        .any(|t| matches!(t, FloatIrregularTrait::Ord));
    let maybe_allow_ord_partial_ord_mismatch = if has_derived_partial_ord && has_manual_ord {
        quote! { #[allow(clippy::derive_ord_xor_partial_ord)] }
    } else {
        quote! {}
    };

    let derive_transparent_traits = quote! {
        #maybe_allow_ord_partial_ord_mismatch
        #[derive(
            #(#transparent_traits,)*
            #(#unsafe_traits,)*
        )]
    };

    let implement_traits = gen_implemented_traits(
        type_name,
        generics,
        inner_type,
        maybe_default_value.clone(),
        irregular_traits,
        guard,
    )?;

    let mut conditional_derive_transparent_traits = TokenStream::new();
    let mut conditional_implement_traits = TokenStream::new();
    let mut conditional_from_str_parse_errors: Vec<(TokenStream, ParseErrorTypeName)> = vec![];

    for group in conditional_derives {
        let pred = &group.predicate;

        let cond_traits: HashSet<FloatDeriveTrait> = group.typed_traits.iter().cloned().collect();
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
                .any(|t| matches!(t, FloatIrregularTrait::FromStr));

            let impl_tokens = gen_implemented_traits(
                type_name,
                generics,
                inner_type,
                maybe_default_value.clone(),
                cond_irregular,
                guard,
            )?;

            if has_from_str {
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

fn gen_implemented_traits<T: ToTokens>(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: &FloatInnerType,
    maybe_default_value: Option<syn::Expr>,
    impl_traits: Vec<FloatIrregularTrait>,
    guard: &FloatGuard<T>,
) -> Result<TokenStream, syn::Error> {
    let maybe_error_type_name = guard.maybe_error_type_path();
    impl_traits
        .iter()
        .map(|t| match t {
            FloatIrregularTrait::AsRef => Ok(gen_impl_trait_as_ref(type_name, generics, inner_type)),
            FloatIrregularTrait::Deref => Ok(gen_impl_trait_deref(type_name, generics, inner_type)),
            FloatIrregularTrait::FromStr => {
                Ok(gen_impl_trait_from_str(type_name, generics, inner_type, maybe_error_type_name))
            }
            FloatIrregularTrait::From => Ok(gen_impl_trait_from(type_name, generics, inner_type)),
            FloatIrregularTrait::Into => Ok(gen_impl_trait_into(type_name, generics, inner_type)),
            FloatIrregularTrait::TryFrom => {
                Ok(gen_impl_trait_try_from(type_name, generics, inner_type, maybe_error_type_name))
            }
            FloatIrregularTrait::Borrow => Ok(gen_impl_trait_borrow(type_name, generics, inner_type)),
            FloatIrregularTrait::Display => Ok(gen_impl_trait_display(type_name, generics)),
            FloatIrregularTrait::Default => match maybe_default_value {
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
            FloatIrregularTrait::SerdeSerialize => Ok(gen_impl_trait_serde_serialize(type_name, generics)),
            FloatIrregularTrait::SerdeDeserialize => Ok(gen_impl_trait_serde_deserialize(
                type_name,
                generics,
                inner_type,
                maybe_error_type_name,
            )),
            FloatIrregularTrait::Eq => Ok(gen_impl_trait_eq(type_name)),
            FloatIrregularTrait::Ord => Ok(gen_impl_trait_ord(type_name)),
            FloatIrregularTrait::ArbitraryArbitrary => {
                arbitrary::gen_impl_trait_arbitrary(type_name, inner_type, guard)
            }
        })
        .collect()
}

fn gen_impl_trait_eq(type_name: &TypeName) -> TokenStream {
    quote! {
        impl ::core::cmp::Eq for #type_name { }
    }
}

// The implementation below may panic.
// Function `partial_cmp` returns `None` only for `NaN` values, but
// `NaN` values are supposed to be excluded by `finite` validation rule.
// Without `finite` validation deriving `Ord` is not allowed.
fn gen_impl_trait_ord(type_name: &TypeName) -> TokenStream {
    let tp = type_name.to_string();
    quote! {
        // Make clippy ignore this manual implementation of Ord even when PartialOrd is derived.
        #[allow(clippy::derive_ord_xor_partial_ord)]
        impl ::core::cmp::Ord for #type_name {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                self.partial_cmp(other)
                    .unwrap_or_else(|| {
                        let tp = #tp;
                        panic!("{tp}::cmp() panicked, because partial_cmp() returned None. Could it be that you're using unsafe {tp}::new_unchecked() ?", tp=tp);
                    })
            }
        }
    }
}
