use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common::{
        gen::traits::{
            gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_default,
            gen_impl_trait_deref, gen_impl_trait_dislpay, gen_impl_trait_from,
            gen_impl_trait_from_str, gen_impl_trait_into, gen_impl_trait_serde_deserialize,
            gen_impl_trait_serde_serialize, gen_impl_trait_try_from, split_into_generatable_traits,
            GeneratableTrait, GeneratableTraits, GeneratedTraits,
        },
        models::{ErrorTypeName, IntegerInnerType, TypeName},
    },
    integer::models::IntegerDeriveTrait,
};

type IntegerGeneratableTrait = GeneratableTrait<IntegerTransparentTrait, IntegerIrregularTrait>;

pub fn gen_traits(
    type_name: &TypeName,
    inner_type: IntegerInnerType,
    maybe_error_type_name: Option<ErrorTypeName>,
    traits: HashSet<IntegerDeriveTrait>,
    maybe_default_value: Option<TokenStream>,
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
        };
        tokens.to_tokens(token_stream)
    }
}

fn gen_implemented_traits(
    type_name: &TypeName,
    inner_type: IntegerInnerType,
    maybe_error_type_name: Option<ErrorTypeName>,
    impl_traits: Vec<IntegerIrregularTrait>,
    maybe_default_value: Option<TokenStream>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            IntegerIrregularTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            IntegerIrregularTrait::Deref => gen_impl_trait_deref(type_name, inner_type),
            IntegerIrregularTrait::FromStr => {
                gen_impl_trait_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            IntegerIrregularTrait::From => gen_impl_trait_from(type_name, inner_type),
            IntegerIrregularTrait::Into => gen_impl_trait_into(type_name, inner_type),
            IntegerIrregularTrait::TryFrom => {
                gen_impl_trait_try_from(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            IntegerIrregularTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
            IntegerIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
            IntegerIrregularTrait::Default => {
                match maybe_default_value {
                    Some(ref default_value) => {
                        let has_validation = maybe_error_type_name.is_some();
                        gen_impl_trait_default(type_name, default_value, has_validation)
                    },
                    None => {
                        panic!("Default trait is derived for type {type_name}, but `default = ` parameter is missing in #[nutype] macro");
                    }
                }
            }
            IntegerIrregularTrait::SerdeSerialize => gen_impl_trait_serde_serialize(type_name),
            IntegerIrregularTrait::SerdeDeserialize => gen_impl_trait_serde_deserialize(
                type_name,
                inner_type,
                maybe_error_type_name.as_ref(),
            ),
        })
        .collect()
}
