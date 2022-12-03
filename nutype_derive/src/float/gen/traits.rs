use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{common::gen::traits::gen_impl_trait_into, float::models::FloatDeriveTrait};

// TODO: this can be a shared structure among all the types
pub struct GeneratedTraits {
    pub derive_standard_traits: TokenStream,
    pub implement_traits: TokenStream,
}

pub fn gen_traits(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<Ident>,
    traits: HashSet<FloatDeriveTrait>,
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

impl From<FloatDeriveTrait> for Trait {
    fn from(derive_trait: FloatDeriveTrait) -> Trait {
        match derive_trait {
            FloatDeriveTrait::Debug => Trait::Derived(DerivedTrait::Debug),
            FloatDeriveTrait::Clone => Trait::Derived(DerivedTrait::Clone),
            FloatDeriveTrait::Copy => Trait::Derived(DerivedTrait::Copy),
            FloatDeriveTrait::PartialEq => Trait::Derived(DerivedTrait::PartialEq),
            FloatDeriveTrait::PartialOrd => Trait::Derived(DerivedTrait::PartialOrd),
            FloatDeriveTrait::FromStr => Trait::Implemented(ImplementedTrait::FromStr),
            FloatDeriveTrait::AsRef => Trait::Implemented(ImplementedTrait::AsRef),
            FloatDeriveTrait::From => Trait::Implemented(ImplementedTrait::From),
            FloatDeriveTrait::Into => Trait::Implemented(ImplementedTrait::Into),
            FloatDeriveTrait::TryFrom => Trait::Implemented(ImplementedTrait::TryFrom),
            FloatDeriveTrait::Borrow => Trait::Implemented(ImplementedTrait::Borrow),
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
    PartialOrd,
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
            Self::Copy => quote!(Copy),
            Self::PartialEq => quote!(PartialEq),
            Self::PartialOrd => quote!(PartialOrd),
        };
        tokens.to_tokens(token_stream)
    }
}

fn split_traits(
    input_traits: HashSet<FloatDeriveTrait>,
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
            ImplementedTrait::AsRef => gen_impl_as_ref(type_name, inner_type),
            ImplementedTrait::FromStr => {
                gen_impl_from_str(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::From => gen_impl_from(type_name, inner_type),
            ImplementedTrait::Into => gen_impl_trait_into(type_name, inner_type),
            ImplementedTrait::TryFrom => {
                gen_impl_try_from(type_name, inner_type, maybe_error_type_name.as_ref())
            }
            ImplementedTrait::Borrow => gen_impl_borrow(type_name, inner_type),
        })
        .collect()
}

fn gen_impl_as_ref(type_name: &Ident, inner_type: &TokenStream) -> TokenStream {
    quote! {
        impl ::core::convert::AsRef<#inner_type> for #type_name {
            fn as_ref(&self) -> &#inner_type {
                &self.0
            }
        }
    }
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

fn gen_impl_from(type_name: &Ident, inner_type: &TokenStream) -> TokenStream {
    quote! {
        impl ::core::convert::From<#inner_type> for #type_name {
            fn from(raw_value: #inner_type) -> Self {
                Self::new(raw_value)
            }
        }

        impl ::core::convert::From<&#inner_type> for #type_name {
            fn from(raw_value: &#inner_type) -> Self {
                Self::new(*raw_value)
            }
        }
    }
}

fn gen_impl_try_from(
    type_name: &Ident,
    inner_type: &TokenStream,
    maybe_error_type_name: Option<&Ident>,
) -> TokenStream {
    let error_type_name = maybe_error_type_name
        .expect("gen_impl_try_from() for float is expected to have error_type_name");

    quote! {
        impl ::core::convert::TryFrom<#inner_type> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: #inner_type) -> Result<#type_name, Self::Error> {
                Self::new(raw_value)
            }
        }
    }
}

fn gen_impl_borrow(type_name: &Ident, inner_type: &TokenStream) -> TokenStream {
    quote! {
        impl ::core::borrow::Borrow<#inner_type> for #type_name {
            fn borrow(&self) -> &#inner_type {
                &self.0
            }
        }
    }
}
