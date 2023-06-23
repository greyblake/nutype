use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{
        gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_dislpay, gen_impl_trait_from,
        gen_impl_trait_from_str, gen_impl_trait_into, gen_impl_trait_serde_deserialize,
        gen_impl_trait_serde_serialize, gen_impl_trait_try_from, split_into_generatable_traits,
        GeneratableTrait, GeneratableTraits, GeneratedTraits,
    },
    common::models::{ErrorTypeName, FloatInnerType, TypeName},
    float::models::FloatDeriveTrait,
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
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FloatIrregularTrait {
    FromStr,
    AsRef,
    Into,
    From,
    Eq,
    Ord,
    TryFrom,
    Borrow,
    Display,
    SerdeSerialize,
    SerdeDeserialize,
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
            FloatDeriveTrait::SerdeSerialize => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::SerdeSerialize)
            }
            FloatDeriveTrait::SerdeDeserialize => {
                FloatGeneratableTrait::Irregular(FloatIrregularTrait::SerdeDeserialize)
            }
            FloatDeriveTrait::SchemarsJsonSchema => {
                FloatGeneratableTrait::Transparent(FloatTransparentTrait::SchemarsJsonSchema)
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
        };
        tokens.to_tokens(token_stream)
    }
}

pub fn gen_traits(
    type_name: &TypeName,
    inner_type: FloatInnerType,
    maybe_error_type_name: Option<ErrorTypeName>,
    traits: HashSet<FloatDeriveTrait>,
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
    );

    GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
    }
}

fn gen_implemented_traits(
    type_name: &TypeName,
    inner_type: FloatInnerType,
    maybe_error_type_name: Option<ErrorTypeName>,
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
                gen_impl_trait_try_from(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            FloatIrregularTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
            FloatIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
            FloatIrregularTrait::SerdeSerialize => gen_impl_trait_serde_serialize(type_name),
            FloatIrregularTrait::SerdeDeserialize => gen_impl_trait_serde_deserialize(
                type_name,
                inner_type,
                maybe_error_type_name.as_ref(),
            ),
            FloatIrregularTrait::Eq => gen_impl_trait_eq(type_name),
            FloatIrregularTrait::Ord => gen_impl_trait_ord(type_name),
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
// `NaN` values are supposed to be exluded by `finite` validation rule.
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
