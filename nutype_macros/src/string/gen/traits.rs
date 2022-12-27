use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::{
        gen::traits::{
            gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_dislpay,
            gen_impl_trait_from, gen_impl_trait_into, gen_impl_trait_serde_deserialize,
            gen_impl_trait_serde_serialize, gen_impl_trait_try_from, split_into_generatable_traits,
            GeneratableTrait, GeneratableTraits, GeneratedTraits,
        },
        models::TypeName,
    },
    string::models::StringDeriveTrait,
};

type StringGeneratableTrait = GeneratableTrait<StringStandardTrait, StringIrregularTrait>;

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum StringStandardTrait {
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
}

/// A trait that can not be automatically derived and we need to generate
/// an implementation for it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum StringIrregularTrait {
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Borrow,
    Display,
    SerdeSerialize,
    SerdeDeserialize,
}

impl From<StringDeriveTrait> for StringGeneratableTrait {
    fn from(derive_trait: StringDeriveTrait) -> StringGeneratableTrait {
        match derive_trait {
            StringDeriveTrait::Debug => {
                StringGeneratableTrait::Standard(StringStandardTrait::Debug)
            }
            StringDeriveTrait::Clone => {
                StringGeneratableTrait::Standard(StringStandardTrait::Clone)
            }
            StringDeriveTrait::PartialEq => {
                StringGeneratableTrait::Standard(StringStandardTrait::PartialEq)
            }
            StringDeriveTrait::Eq => StringGeneratableTrait::Standard(StringStandardTrait::Eq),
            StringDeriveTrait::PartialOrd => {
                StringGeneratableTrait::Standard(StringStandardTrait::PartialOrd)
            }
            StringDeriveTrait::Ord => StringGeneratableTrait::Standard(StringStandardTrait::Ord),
            StringDeriveTrait::Hash => StringGeneratableTrait::Standard(StringStandardTrait::Hash),
            StringDeriveTrait::FromStr => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::FromStr)
            }
            StringDeriveTrait::AsRef => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::AsRef)
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
            StringDeriveTrait::SerdeSerialize => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::SerdeSerialize)
            }
            StringDeriveTrait::SerdeDeserialize => {
                StringGeneratableTrait::Irregular(StringIrregularTrait::SerdeDeserialize)
            }
        }
    }
}

impl ToTokens for StringStandardTrait {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let tokens = match self {
            Self::Debug => quote!(Debug),
            Self::Clone => quote!(Clone),
            Self::PartialEq => quote!(PartialEq),
            Self::Eq => quote!(Eq),
            Self::PartialOrd => quote!(PartialOrd),
            Self::Ord => quote!(Ord),
            Self::Hash => quote!(Hash),
        };
        tokens.to_tokens(token_stream)
    }
}

pub fn gen_traits(
    type_name: &TypeName,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<StringDeriveTrait>,
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

    let implement_traits =
        gen_implemented_traits(type_name, maybe_error_type_name, irregular_traits);

    GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    }
}

fn gen_implemented_traits(
    type_name: &TypeName,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<StringIrregularTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            StringIrregularTrait::AsRef => gen_impl_trait_as_ref(type_name, quote!(str)),
            StringIrregularTrait::FromStr => {
                gen_impl_from_str(type_name, maybe_error_type_name.as_ref())
            }
            StringIrregularTrait::From => gen_impl_from_str_and_string(type_name),
            StringIrregularTrait::Into => gen_impl_trait_into(type_name, quote!(String)),
            StringIrregularTrait::TryFrom => {
                let error_type_name = maybe_error_type_name
                    .as_ref()
                    .expect("TryFrom for String is expected to have error_type_name");
                gen_impl_try_from(type_name, error_type_name)
            }
            StringIrregularTrait::Borrow => gen_impl_borrow_str_and_string(type_name),
            StringIrregularTrait::Display => gen_impl_trait_dislpay(type_name),
            StringIrregularTrait::SerdeSerialize => gen_impl_trait_serde_serialize(type_name),
            StringIrregularTrait::SerdeDeserialize => gen_impl_trait_serde_deserialize(
                type_name,
                quote!(String),
                maybe_error_type_name.as_ref(),
            ),
        })
        .collect()
}

fn gen_impl_from_str(type_name: &TypeName, maybe_error_type_name: Option<&Ident>) -> TokenStream {
    if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = #error_type_name;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    #type_name::new(raw_string)
                }
            }
        }
    } else {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = ::core::convert::Infallible;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    Ok(#type_name::new(raw_string))
                }
            }
        }
    }
}

fn gen_impl_from_str_and_string(type_name: &TypeName) -> TokenStream {
    let impl_from_string = gen_impl_trait_from(type_name, quote!(String));
    let impl_from_str = gen_impl_trait_from(type_name, quote!(&str));

    quote! {
        #impl_from_string
        #impl_from_str
    }
}

fn gen_impl_try_from(type_name: &TypeName, error_type_name: &Ident) -> TokenStream {
    let impl_try_from_string = gen_impl_trait_try_from(type_name, quote!(String), error_type_name);
    let impl_try_from_str = gen_impl_trait_try_from(type_name, quote!(&str), error_type_name);

    quote! {
        #impl_try_from_string
        #impl_try_from_str
    }
}

fn gen_impl_borrow_str_and_string(type_name: &TypeName) -> TokenStream {
    let impl_borrow_string = gen_impl_trait_borrow(type_name, quote!(String));
    let impl_borrow_str = gen_impl_trait_borrow(type_name, quote!(str));

    quote! {
        #impl_borrow_string
        #impl_borrow_str
    }
}
