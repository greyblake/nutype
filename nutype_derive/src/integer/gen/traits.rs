use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::gen::traits::{
        gen_impl_trait_as_ref, gen_impl_trait_borrow, gen_impl_trait_from, gen_impl_trait_into,
        gen_impl_trait_try_from,
    },
    integer::models::IntegerDeriveTrait,
};

// TODO: this can be shared structure among all the types
pub struct GeneratedTraits {
    pub derive_standard_traits: TokenStream,
    pub implement_traits: TokenStream,
}

pub fn gen_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<IntegerDeriveTrait>,
) -> GeneratedTraits {
    let (standard_traits, impl_traits) = split_traits(traits);

    let derive_standard_traits = quote! {
        #[derive(
            #(#standard_traits,)*
        )]
    };

    let implement_traits =
        gen_implemented_traits(type_name, inner_type, maybe_error_type_name, impl_traits);

    GeneratedTraits {
        derive_standard_traits,
        implement_traits,
    }
}

// TODO: this can be shared generic enum for all the types
enum Trait {
    Derived(DerivedTrait),
    Implemented(ImplementedTrait),
}

impl From<IntegerDeriveTrait> for Trait {
    fn from(derive_trait: IntegerDeriveTrait) -> Trait {
        match derive_trait {
            IntegerDeriveTrait::Debug => Trait::Derived(DerivedTrait::Debug),
            IntegerDeriveTrait::Clone => Trait::Derived(DerivedTrait::Clone),
            IntegerDeriveTrait::Copy => Trait::Derived(DerivedTrait::Copy),
            IntegerDeriveTrait::PartialEq => Trait::Derived(DerivedTrait::PartialEq),
            IntegerDeriveTrait::Eq => Trait::Derived(DerivedTrait::Eq),
            IntegerDeriveTrait::PartialOrd => Trait::Derived(DerivedTrait::PartialOrd),
            IntegerDeriveTrait::Ord => Trait::Derived(DerivedTrait::Ord),
            IntegerDeriveTrait::Hash => Trait::Derived(DerivedTrait::Hash),
            IntegerDeriveTrait::FromStr => Trait::Implemented(ImplementedTrait::FromStr),
            IntegerDeriveTrait::AsRef => Trait::Implemented(ImplementedTrait::AsRef),
            IntegerDeriveTrait::Into => Trait::Implemented(ImplementedTrait::Into),
            IntegerDeriveTrait::From => Trait::Implemented(ImplementedTrait::From),
            IntegerDeriveTrait::TryFrom => Trait::Implemented(ImplementedTrait::TryFrom),
            IntegerDeriveTrait::Borrow => Trait::Implemented(ImplementedTrait::Borrow),
        }
    }
}

/// A trait that can be automatically derived.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DerivedTrait {
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
enum ImplementedTrait {
    FromStr,
    AsRef,
    From,
    TryFrom,
    Borrow,
    Into,
}

impl ToTokens for DerivedTrait {
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

fn split_traits(
    input_traits: HashSet<IntegerDeriveTrait>,
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
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    impl_traits: Vec<ImplementedTrait>,
) -> TokenStream {
    impl_traits
        .iter()
        .map(|t| match t {
            ImplementedTrait::AsRef => gen_impl_trait_as_ref(type_name, inner_type),
            ImplementedTrait::FromStr => {
                gen_impl_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::From => gen_impl_trait_from(type_name, inner_type),
            ImplementedTrait::Into => gen_impl_trait_into(type_name, inner_type),
            ImplementedTrait::TryFrom => {
                let error_type_name = maybe_error_type_name
                    .as_ref()
                    .expect("TryFrom for integer is expected to have error_type_name");
                gen_impl_trait_try_from(type_name, inner_type, error_type_name)
            }
            ImplementedTrait::Borrow => gen_impl_trait_borrow(type_name, inner_type),
        })
        .collect()
}

fn gen_impl_from_str(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<&Ident>,
) -> TokenStream {
    if let Some(_error_type_name) = maybe_error_type_name {
        quote! {
            // TODO
            //
            // * Exact Parse error name generation into a separate reusable function
            // * Implement the Error trait for the parse error properly
            // * Consider using same type ParseTypeError for FromStr variant without validation
            // * Import the ParseTypeError properly
            //
            // Potential parse erro definition:
            //
            // enum Parse<#type_name>Error {
            //     Parse(<#inner_type as ::core::str::FromStr>::Err),
            //     Validate(#error_type_name),
            // }

            // impl core::str::FromStr for #type_name {
            //     type Err = #error_type_name;

            //     fn from_str(raw_string: &str) -> Result<Self, Self::Err> {
            //         #type_name::new(raw_string)
            //     }
            // }
        }
    } else {
        quote! {
            impl ::core::str::FromStr for #type_name {
                type Err = <#inner_type as ::core::str::FromStr>::Err;

                fn from_str(raw_string: &str) -> Result<Self, Self::Err> {
                    let value: #inner_type = raw_string.parse()?;
                    Ok(#type_name::new(value))
                }
            }
        }
    }
}
