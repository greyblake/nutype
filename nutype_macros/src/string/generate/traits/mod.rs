pub mod arbitrary;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Generics;

use crate::{
    common::{
        generate::traits::{
            GeneratableTrait, GeneratableTraits, GeneratedTraits, gen_impl_trait_as_ref,
            gen_impl_trait_borrow, gen_impl_trait_default, gen_impl_trait_deref,
            gen_impl_trait_display, gen_impl_trait_from, gen_impl_trait_into,
            gen_impl_trait_serde_deserialize, gen_impl_trait_serde_serialize,
            gen_impl_trait_try_from, split_into_generatable_traits,
        },
        models::{ErrorTypePath, TypeName},
    },
    string::models::{StringDeriveTrait, StringGuard, StringInnerType},
};

type StringGeneratableTrait = GeneratableTrait<StringTransparentTrait, StringIrregularTrait>;

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum StringTransparentTrait {
    Debug,
    Clone,
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
enum StringIrregularTrait {
    FromStr,
    AsRef,
    Deref,
    Into,
    From,
    TryFrom,
    Borrow,
    Display,
    Default,
    SerdeSerialize,
    SerdeDeserialize,
    ArbitraryArbitrary,
}

impl From<StringDeriveTrait> for StringGeneratableTrait {
    fn from(derive_trait: StringDeriveTrait) -> StringGeneratableTrait {
        match derive_trait {
            StringDeriveTrait::Debug => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::Debug)
            }
            StringDeriveTrait::Clone => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::Clone)
            }
            StringDeriveTrait::PartialEq => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::PartialEq)
            }
            StringDeriveTrait::Eq => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::Eq)
            }
            StringDeriveTrait::PartialOrd => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::PartialOrd)
            }
            StringDeriveTrait::Ord => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::Ord)
            }
            StringDeriveTrait::Hash => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::Hash)
            }
            StringDeriveTrait::FromStr => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::FromStr)
            }
            StringDeriveTrait::AsRef => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::AsRef)
            }
            StringDeriveTrait::Deref => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::Deref)
            }
            StringDeriveTrait::Into => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::Into)
            }
            StringDeriveTrait::From => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::From)
            }
            StringDeriveTrait::TryFrom => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::TryFrom)
            }
            StringDeriveTrait::Borrow => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::Borrow)
            }
            StringDeriveTrait::Display => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::Display)
            }
            StringDeriveTrait::Default => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::Default)
            }
            StringDeriveTrait::SerdeSerialize => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::SerdeSerialize)
            }
            StringDeriveTrait::SerdeDeserialize => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::SerdeDeserialize)
            }
            StringDeriveTrait::SchemarsJsonSchema => {
                StringGeneratableTrait::Transparent(StringTransparentTrait::SchemarsJsonSchema)
            }
            StringDeriveTrait::ArbitraryArbitrary => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::ArbitraryArbitrary)
            }
        }
    }
}

impl ToTokens for StringTransparentTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
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

pub fn gen_traits(
    type_name: &TypeName,
    generics: &Generics,
    traits: HashSet<StringDeriveTrait>,
    maybe_default_value: Option<syn::Expr>,
    guard: &StringGuard,
) -> Result<GeneratedTraits, syn::Error> {
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
        generics,
        maybe_default_value,
        irregular_traits,
        guard,
    )?;

    Ok(GeneratedTraits {
        derive_transparent_traits,
        implement_traits,
    })
}

fn gen_implemented_traits(
    type_name: &TypeName,
    generics: &Generics,
    maybe_default_value: Option<syn::Expr>,
    impl_traits: Vec<StringIrregularTrait>,
    guard: &StringGuard,
) -> Result<TokenStream, syn::Error> {
    let inner_type = StringInnerType;
    let maybe_error_type_name = guard.maybe_error_type_path();

    impl_traits
        .iter()
        .map(|t| match t {
            StringIrregularTrait::AsRef => Ok(gen_impl_trait_as_ref(type_name, generics, quote!(str))),
            StringIrregularTrait::Deref => Ok(gen_impl_trait_deref(type_name, generics, quote!(String))),
            StringIrregularTrait::FromStr => {
                Ok(gen_impl_from_str(type_name, maybe_error_type_name))
            }
            StringIrregularTrait::From => Ok(gen_impl_from_str_and_string(type_name)),
            StringIrregularTrait::Into => Ok(gen_impl_trait_into(type_name, &Generics::default(), inner_type)),
            StringIrregularTrait::TryFrom => {
                Ok(gen_impl_try_from(type_name, maybe_error_type_name))
            }
            StringIrregularTrait::Borrow => Ok(gen_impl_borrow_str_and_string(type_name)),
            StringIrregularTrait::Display => Ok(gen_impl_trait_display(type_name, &Generics::default())),
            StringIrregularTrait::Default => match maybe_default_value {
                Some(ref default_value) => {
                    let has_validation = maybe_error_type_name.is_some();
                    Ok(gen_impl_trait_default(
                        type_name,
                        generics,
                        default_value,
                        has_validation,
                    ))
                }
                None => {
                    let span = proc_macro2::Span::call_site();
                    let msg = format!("Trait `Default` is derived for type {type_name}, but `default = ` parameter is missing in #[nutype] macro");
                    Err(syn::Error::new(span, msg))
                }
            },
            StringIrregularTrait::SerdeSerialize => Ok(gen_impl_trait_serde_serialize(type_name, generics)),
            StringIrregularTrait::SerdeDeserialize => Ok(gen_impl_trait_serde_deserialize(
                type_name,
                generics,
                inner_type,
                maybe_error_type_name,
            )),
            StringIrregularTrait::ArbitraryArbitrary => {
                arbitrary::gen_impl_trait_arbitrary(type_name, guard)
            }
        })
        .collect()
}

fn gen_impl_from_str(
    type_name: &TypeName,
    maybe_error_type_name: Option<&ErrorTypePath>,
) -> TokenStream {
    if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = #error_type_name;

                #[inline]
                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    #type_name::try_new(raw_string)
                }
            }
        }
    } else {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = ::core::convert::Infallible;

                #[inline]
                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    Ok(#type_name::new(raw_string))
                }
            }
        }
    }
}

fn gen_impl_from_str_and_string(type_name: &TypeName) -> TokenStream {
    let generics = Generics::default();
    let impl_from_string = gen_impl_trait_from(type_name, &generics, quote!(String));
    let impl_from_str = gen_impl_trait_from(type_name, &generics, quote!(&str));

    quote! {
        #impl_from_string
        #impl_from_str
    }
}

fn gen_impl_try_from(
    type_name: &TypeName,
    maybe_error_type_name: Option<&ErrorTypePath>,
) -> TokenStream {
    let generics = Generics::default();
    let impl_try_from_string =
        gen_impl_trait_try_from(type_name, &generics, quote!(String), maybe_error_type_name);
    let impl_try_from_str =
        gen_impl_trait_try_from(type_name, &generics, quote!(&str), maybe_error_type_name);

    quote! {
        #impl_try_from_string
        #impl_try_from_str
    }
}

fn gen_impl_borrow_str_and_string(type_name: &TypeName) -> TokenStream {
    let generics = Generics::default();
    let impl_borrow_string = gen_impl_trait_borrow(type_name, &generics, quote!(String));
    let impl_borrow_str = gen_impl_trait_borrow(type_name, &generics, quote!(str));

    quote! {
        #impl_borrow_string
        #impl_borrow_str
    }
}
