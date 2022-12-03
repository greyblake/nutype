use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_into},
    string::models::StringDeriveTrait,
};

pub struct GeneratedTraits {
    pub derive_standard_traits: TokenStream,
    pub implement_traits: TokenStream,
}

pub fn gen_traits(
    type_name: &Ident,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<StringDeriveTrait>,
) -> GeneratedTraits {
    let (standard_traits, impl_traits) = split_traits(traits);

    let derive_standard_traits = quote! {
        #[derive(
            #(#standard_traits,)*
        )]
    };

    let implement_traits = gen_implemented_traits(type_name, maybe_error_type_name, impl_traits);

    GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    }
}

enum Trait {
    Derived(DerivedTrait),
    Implemented(ImplementedTrait),
}

impl From<StringDeriveTrait> for Trait {
    fn from(derive_trait: StringDeriveTrait) -> Trait {
        match derive_trait {
            StringDeriveTrait::Debug => Trait::Derived(DerivedTrait::Debug),
            StringDeriveTrait::Clone => Trait::Derived(DerivedTrait::Clone),
            StringDeriveTrait::PartialEq => Trait::Derived(DerivedTrait::PartialEq),
            StringDeriveTrait::Eq => Trait::Derived(DerivedTrait::Eq),
            StringDeriveTrait::PartialOrd => Trait::Derived(DerivedTrait::PartialOrd),
            StringDeriveTrait::Ord => Trait::Derived(DerivedTrait::Ord),
            StringDeriveTrait::Hash => Trait::Derived(DerivedTrait::Hash),
            StringDeriveTrait::FromStr => Trait::Implemented(ImplementedTrait::FromStr),
            StringDeriveTrait::AsRef => Trait::Implemented(ImplementedTrait::AsRef),
            StringDeriveTrait::Into => Trait::Implemented(ImplementedTrait::Into),
            StringDeriveTrait::From => Trait::Implemented(ImplementedTrait::From),
            StringDeriveTrait::TryFrom => Trait::Implemented(ImplementedTrait::TryFrom),
            StringDeriveTrait::Borrow => Trait::Implemented(ImplementedTrait::Borrow),
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DerivedTrait {
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
enum ImplementedTrait {
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Borrow,
}

impl ToTokens for DerivedTrait {
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

fn split_traits(
    input_traits: HashSet<StringDeriveTrait>,
) -> (Vec<DerivedTrait>, Vec<ImplementedTrait>) {
    let mut derive_traits: Vec<DerivedTrait> = Vec::with_capacity(24);
    let mut impl_traits: Vec<ImplementedTrait> = Vec::with_capacity(24);

    for input_trait in input_traits {
        match Trait::from(input_trait) {
            Trait::Derived(dt) => derive_traits.push(dt),
            Trait::Implemented(it) => impl_traits.push(it),
        };
    }

    (derive_traits, impl_traits)
}

fn gen_implemented_traits(
    type_name: &Ident,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<ImplementedTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            ImplementedTrait::AsRef => gen_impl_trait_as_ref(type_name, quote!(str)),
            ImplementedTrait::FromStr => {
                gen_impl_from_str(type_name, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::From => gen_impl_from(type_name),
            ImplementedTrait::Into => gen_impl_trait_into(type_name, quote!(String)),
            ImplementedTrait::TryFrom => {
                gen_impl_try_from(type_name, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::Borrow => gen_impl_borrow_str_and_string(type_name),
        })
        .collect()
}

fn gen_impl_from_str(type_name: &Ident, maybe_error_type_name: Option<&Ident>) -> TokenStream {
    if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = #error_type_name;

                fn from_str(raw_string: &str) -> Result<Self, Self::Err> {
                    #type_name::new(raw_string)
                }
            }
        }
    } else {
        quote! {
            impl core::str::FromStr for #type_name {
                type Err = ();

                fn from_str(raw_string: &str) -> Result<Self, Self::Err> {
                    Ok(#type_name::new(raw_string))
                }
            }
        }
    }
}

fn gen_impl_from(type_name: &Ident) -> TokenStream {
    quote! {
        impl ::core::convert::From<String> for #type_name {
            fn from(raw_value: String) -> Self {
                Self::new(raw_value)
            }
        }

        impl ::core::convert::From<&str> for #type_name {
            fn from(raw_value: &str) -> Self {
                Self::new(raw_value)
            }
        }
    }
}

fn gen_impl_try_from(type_name: &Ident, maybe_error_type_name: Option<&Ident>) -> TokenStream {
    let error_type_name = maybe_error_type_name
        .expect("gen_impl_try_from() for String is expected to have error_type_name");

    quote! {
        impl ::core::convert::TryFrom<String> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: String) -> Result<#type_name, Self::Error> {
                Self::new(raw_value)
            }
        }

        impl ::core::convert::TryFrom<&str> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: &str) -> Result<#type_name, Self::Error> {
                Self::new(raw_value)
            }
        }
    }
}

fn gen_impl_borrow_str_and_string(type_name: &Ident) -> TokenStream {
    let impl_borrow_str = gen_impl_trait_borrow(type_name, quote!(str));
    let impl_borrow_string = gen_impl_trait_borrow(type_name, quote!(String));

    quote! {
        #impl_borrow_str
        #impl_borrow_string
    }
}
